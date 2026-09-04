mod cloud_init;
mod domain;
mod storage;
mod xml;

use std::{
    fs,
    path::{Path, PathBuf},
};

use self::xml::{DomainXmlSpec, domain_xml};

use uuid::Uuid;
use virt::{connect::Connect, domain::Domain};

use crate::{
    application::{Hypervisor, HypervisorCreateError},
    domain::{CreateVmSpec, Vm, VmId},
};

pub struct LibvirtHypervisor {
    uri: String,
    storage_pool: String,
    storage_dir: PathBuf,
    default_image: String,
    ssh_key_path: PathBuf,
}

impl LibvirtHypervisor {
    pub fn new(
        uri: impl Into<String>,
        storage_pool: impl Into<String>,
        storage_dir: impl Into<PathBuf>,
        default_image: impl Into<String>,
        ssh_key_path: impl Into<PathBuf>,
    ) -> Self {
        Self {
            uri: uri.into(),
            storage_pool: storage_pool.into(),
            storage_dir: storage_dir.into(),
            default_image: default_image.into(),
            ssh_key_path: ssh_key_path.into(),
        }
    }

    fn remove_file(path: &Path) {
        let _ = fs::remove_file(path);
    }
}

impl Hypervisor for LibvirtHypervisor {
    fn create_vm(&self, spec: CreateVmSpec) -> Result<Vm, HypervisorCreateError> {
        let CreateVmSpec {
            name,
            memory_mib,
            disk_gib,
            vcpus,
            image,
        } = spec;

        let connection = Connect::open(Some(&self.uri))
            .map_err(|error| HypervisorCreateError::Internal(error.to_string()))?;

        let name_exists = connection
            .list_all_domains(0)
            .map_err(|error| HypervisorCreateError::Internal(error.to_string()))?
            .into_iter()
            .any(|domain| {
                domain
                    .get_name()
                    .is_ok_and(|domain_name| domain_name == name)
            });

        if name_exists {
            return Err(HypervisorCreateError::NameAlreadyExists);
        }

        let (volume, disk_path) = self.create_volume(&connection, &name, disk_gib, image)?;

        let id = Uuid::new_v4();
        let seed_path = match self.create_seed(id, &name) {
            Ok(path) => path,
            Err(error) => {
                Self::delete_volume(&volume);
                return Err(HypervisorCreateError::Internal(error));
            }
        };

        let xml = domain_xml(DomainXmlSpec {
            name: &name,
            id,
            memory_mib,
            vcpus,
            disk_path: &disk_path,
            seed_path: &seed_path,
        });

        let domain = match Domain::define_xml(&connection, &xml) {
            Ok(domain) => domain,
            Err(error) => {
                Self::remove_file(&seed_path);
                Self::delete_volume(&volume);
                return Err(HypervisorCreateError::Internal(error.to_string()));
            }
        };

        if let Err(error) = domain.create() {
            let _ = domain.undefine();
            Self::remove_file(&seed_path);
            Self::delete_volume(&volume);
            return Err(HypervisorCreateError::Internal(error.to_string()));
        }

        self.domain_to_vm(&connection, &domain)
            .map_err(HypervisorCreateError::Internal)
    }

    fn list_vms(&self) -> Result<Vec<Vm>, String> {
        let connection = Connect::open(Some(&self.uri)).map_err(|error| error.to_string())?;
        let domains = connection
            .list_all_domains(0)
            .map_err(|error| error.to_string())?;

        let mut vms = domains
            .into_iter()
            .map(|domain| self.domain_to_vm(&connection, &domain))
            .collect::<Result<Vec<_>, String>>()?;

        vms.sort_by_key(|vm| vm.id);
        Ok(vms)
    }

    fn get_vm(&self, id: VmId) -> Result<Option<Vm>, String> {
        let connection = Connect::open(Some(&self.uri)).map_err(|error| error.to_string())?;
        let domains = connection
            .list_all_domains(0)
            .map_err(|error| error.to_string())?;

        for domain in domains {
            let domain_id = domain.get_uuid().map_err(|error| error.to_string())?;
            if domain_id == id {
                return self.domain_to_vm(&connection, &domain).map(Some);
            }
        }

        Ok(None)
    }

    fn delete_vm(&self, id: VmId) -> Result<bool, String> {
        let connection = Connect::open(Some(&self.uri)).map_err(|error| error.to_string())?;
        let domains = connection
            .list_all_domains(0)
            .map_err(|error| error.to_string())?;

        let mut found = None;
        for domain in domains {
            if domain.get_uuid().map_err(|error| error.to_string())? == id {
                found = Some(domain);
                break;
            }
        }

        let Some(domain) = found else {
            return Ok(false);
        };

        let name = domain.get_name().map_err(|error| error.to_string())?;
        if domain.is_active().map_err(|error| error.to_string())? {
            domain.destroy().map_err(|error| error.to_string())?;
        }
        domain
            .undefine_flags(virt::sys::VIR_DOMAIN_UNDEFINE_NVRAM)
            .map_err(|error| error.to_string())?;

        self.delete_vm_volumes(&connection, &name)?;

        Ok(true)
    }
}
