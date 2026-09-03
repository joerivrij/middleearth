use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use uuid::Uuid;
use virt::connect::Connect;
use virt::domain::Domain;
use virt::storage_pool::StoragePool;
use virt::storage_vol::StorageVol;

use crate::domain::CreateVmSpec;
use crate::{
    application::{Hypervisor, HypervisorCreateError},
    domain::{Disk, Image, Metadata, Network, Vm, VmId, VmResources, VmState},
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

    fn create_seed(&self, id: VmId, name: &str) -> Result<PathBuf, String> {
        let ssh_key = fs::read_to_string(&self.ssh_key_path)
            .map_err(|error| format!("failed to read SSH public key: {error}"))?;
        let ssh_key = ssh_key.trim();
        if !ssh_key.starts_with("ssh-") {
            return Err("SSH public key has an unsupported format".to_string());
        }

        let temporary_dir = std::env::temp_dir().join(format!("celebrimbor-{id}"));
        fs::create_dir(&temporary_dir)
            .map_err(|error| format!("failed to create cloud-init directory: {error}"))?;

        let user_data_path = temporary_dir.join("user-data");
        let meta_data_path = temporary_dir.join("meta-data");
        let seed_path = self.storage_dir.join(format!("{name}-seed.img"));

        let user_data = format!(
            "#cloud-config\nusers:\n  - name: ubuntu\n    groups: [adm, sudo]\n    shell: /bin/bash\n    sudo: ALL=(ALL) NOPASSWD:ALL\n    ssh_authorized_keys:\n      - {ssh_key}\nssh_pwauth: false\ndisable_root: true\n"
        );
        let meta_data = format!("instance-id: {id}\nlocal-hostname: {name}\n");

        let result = (|| {
            fs::write(&user_data_path, user_data)
                .map_err(|error| format!("failed to write cloud-init user-data: {error}"))?;
            fs::write(&meta_data_path, meta_data)
                .map_err(|error| format!("failed to write cloud-init meta-data: {error}"))?;

            let output = Command::new("cloud-localds")
                .arg(&seed_path)
                .arg(&user_data_path)
                .arg(&meta_data_path)
                .output()
                .map_err(|error| format!("failed to run cloud-localds: {error}"))?;

            if !output.status.success() {
                return Err(format!(
                    "cloud-localds failed: {}",
                    String::from_utf8_lossy(&output.stderr).trim()
                ));
            }

            Ok(seed_path.clone())
        })();

        let _ = fs::remove_dir_all(temporary_dir);
        result
    }

    fn remove_file(path: &Path) {
        let _ = fs::remove_file(path);
    }

    fn xml_attribute(xml: &str, element: &str, attribute: &str) -> Option<String> {
        let element = format!("<{element}");
        let element_start = xml.find(&element)?;
        let element_end = xml[element_start..].find('>')? + element_start;
        let element = &xml[element_start..element_end];
        let attribute = format!("{attribute}=");
        let value_start = element.find(&attribute)? + attribute.len();
        let quote = element[value_start..].chars().next()?;
        if quote != '\'' && quote != '"' {
            return None;
        }
        let value = &element[value_start + quote.len_utf8()..];
        let value_end = value.find(quote)?;
        Some(value[..value_end].to_owned())
    }

    fn backing_image(volume_xml: &str) -> Option<String> {
        let backing_store = volume_xml.find("<backingStore")?;
        let backing_store = &volume_xml[backing_store..];
        let path = Self::xml_attribute(backing_store, "source", "file")
            .or_else(|| Self::xml_attribute(backing_store, "path", "value"))
            .or_else(|| {
                let path_start = backing_store.find("<path>")? + "<path>".len();
                let path_end = backing_store[path_start..].find("</path>")? + path_start;
                Some(backing_store[path_start..path_end].to_owned())
            })?;

        Path::new(&path)
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
    }

    fn domain_to_vm(&self, connection: &Connect, domain: &Domain) -> Result<Vm, String> {
        let id = domain.get_uuid().map_err(|error| error.to_string())?;
        let name = domain.get_name().map_err(|error| error.to_string())?;
        let info = domain.get_info().map_err(|error| error.to_string())?;

        let state = match info.state {
            virt::sys::VIR_DOMAIN_NOSTATE => VmState::NoState,
            virt::sys::VIR_DOMAIN_RUNNING => VmState::Running,
            virt::sys::VIR_DOMAIN_BLOCKED => VmState::Blocked,
            virt::sys::VIR_DOMAIN_PAUSED => VmState::Paused,
            virt::sys::VIR_DOMAIN_SHUTDOWN => VmState::ShuttingDown,
            virt::sys::VIR_DOMAIN_SHUTOFF => VmState::ShutOff,
            virt::sys::VIR_DOMAIN_CRASHED => VmState::Crashed,
            virt::sys::VIR_DOMAIN_PMSUSPENDED => VmState::Suspended,
            _ => VmState::Unknown,
        };

        let ip_address = domain
            .interface_addresses(virt::sys::VIR_DOMAIN_INTERFACE_ADDRESSES_SRC_LEASE, 0)
            .ok()
            .into_iter()
            .flatten()
            .flat_map(|interface| interface.addrs)
            .find(|address| address.typed == i64::from(virt::sys::VIR_IP_ADDR_TYPE_IPV4))
            .map(|address| address.addr);

        let domain_xml = domain.get_xml_desc(0).map_err(|error| error.to_string())?;
        let mac_address = Self::xml_attribute(&domain_xml, "mac", "address");
        let architecture = Self::xml_attribute(&domain_xml, "type", "arch");

        let pool = StoragePool::lookup_by_name(connection, &self.storage_pool)
            .map_err(|error| error.to_string())?;
        let volume = StorageVol::lookup_by_name(&pool, &format!("{name}.qcow2"))
            .map_err(|error| error.to_string())?;
        let volume_info = volume.get_info().map_err(|error| error.to_string())?;
        let volume_xml = volume.get_xml_desc(0).map_err(|error| error.to_string())?;
        let image_name = Self::backing_image(&volume_xml);
        let disk_format = Self::xml_attribute(&volume_xml, "format", "type");

        Ok(Vm {
            id,
            name,
            state,
            resources: VmResources {
                memory_mib: info.max_mem / 1024,
                vcpus: info.nr_virt_cpu,
                disk: Disk {
                    capacity_bytes: volume_info.capacity,
                    allocated_bytes: volume_info.allocation,
                    format: disk_format,
                },
            },
            image: Image { name: image_name },
            network: Network {
                ip_address,
                mac_address,
            },
            metadata: Metadata {
                architecture,
                ssh_user: Some("ubuntu".to_owned()),
            },
        })
    }
}

impl Hypervisor for LibvirtHypervisor {
    fn create_vm(&self, spec: CreateVmSpec) -> Result<Vm, HypervisorCreateError> {
        let CreateVmSpec {
            name,
            memory_mib,
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

        let image = image.unwrap_or_else(|| self.default_image.clone());
        let backing_path = self.storage_dir.join("src").join(image);
        if !backing_path.is_file() {
            return Err(HypervisorCreateError::ImageNotFound);
        }

        let pool = StoragePool::lookup_by_name(&connection, &self.storage_pool)
            .map_err(|error| HypervisorCreateError::Internal(error.to_string()))?;

        let volume_name = format!("{name}.qcow2");
        let volume_xml = format!(
            r#"<volume>
  <name>{volume_name}</name>
  <capacity unit="GiB">10</capacity>
  <target>
    <format type="qcow2"/>
  </target>
  <backingStore>
    <path>{}</path>
    <format type="qcow2"/>
  </backingStore>
</volume>"#,
            backing_path.display()
        );

        let volume = StorageVol::create_xml(&pool, &volume_xml, 0)
            .map_err(|error| HypervisorCreateError::Internal(error.to_string()))?;
        let disk_path = match volume.get_path() {
            Ok(path) => path,
            Err(error) => {
                let _ = volume.delete(0);
                return Err(HypervisorCreateError::Internal(error.to_string()));
            }
        };

        let id = Uuid::new_v4();
        let seed_path = match self.create_seed(id, &name) {
            Ok(path) => path,
            Err(error) => {
                let _ = volume.delete(0);
                return Err(HypervisorCreateError::Internal(error));
            }
        };
        let xml = format!(
            r#"<domain type="kvm">
  <name>{name}</name>
  <uuid>{id}</uuid>
  <memory unit="MiB">{memory_mib}</memory>
  <vcpu placement="static">{vcpus}</vcpu>
  <os>
    <type arch="aarch64" machine="virt">hvm</type>
    <loader readonly="yes" type="pflash">/usr/share/AAVMF/AAVMF_CODE.no-secboot.fd</loader>
    <nvram template="/usr/share/AAVMF/AAVMF_VARS.fd"/>
    <boot dev="hd"/>
  </os>
  <features>
    <acpi/>
  </features>
  <cpu mode="host-passthrough"/>
  <devices>
    <emulator>/usr/bin/qemu-system-aarch64</emulator>
    <disk type="file" device="disk">
      <driver name="qemu" type="qcow2"/>
      <source file="{disk_path}"/>
      <target dev="vda" bus="virtio"/>
    </disk>
    <disk type="file" device="cdrom">
      <driver name="qemu" type="raw"/>
      <source file="{}"/>
      <target dev="sda" bus="scsi"/>
      <readonly/>
    </disk>
    <controller type="scsi" model="virtio-scsi"/>
    <interface type="network">
      <source network="default"/>
      <model type="virtio"/>
    </interface>
    <serial type="pty"/>
    <console type="pty"/>
  </devices>
</domain>"#,
            seed_path.display()
        );

        let domain = match Domain::define_xml(&connection, &xml) {
            Ok(domain) => domain,
            Err(error) => {
                Self::remove_file(&seed_path);
                let _ = volume.delete(0);
                return Err(HypervisorCreateError::Internal(error.to_string()));
            }
        };

        if let Err(error) = domain.create() {
            let _ = domain.undefine();
            Self::remove_file(&seed_path);
            let _ = volume.delete(0);
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

        let pool = StoragePool::lookup_by_name(&connection, &self.storage_pool)
            .map_err(|error| error.to_string())?;
        let volume_names = pool.list_volumes().map_err(|error| error.to_string())?;

        for volume_name in [format!("{name}.qcow2"), format!("{name}-seed.img")] {
            if volume_names.contains(&volume_name) {
                StorageVol::lookup_by_name(&pool, &volume_name)
                    .map_err(|error| error.to_string())?
                    .delete(0)
                    .map_err(|error| error.to_string())?;
            }
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::LibvirtHypervisor;

    #[test]
    fn extracts_xml_attributes_with_either_quote_style() {
        assert_eq!(
            LibvirtHypervisor::xml_attribute(
                "<mac address='52:54:00:12:34:56'/>",
                "mac",
                "address"
            ),
            Some("52:54:00:12:34:56".to_owned())
        );
        assert_eq!(
            LibvirtHypervisor::xml_attribute("<type arch=\"aarch64\">hvm</type>", "type", "arch"),
            Some("aarch64".to_owned())
        );
    }

    #[test]
    fn extracts_backing_image_from_libvirt_volume_xml() {
        let xml = r#"<volume>
  <target><format type='qcow2'/></target>
  <backingStore>
    <format type='qcow2'/>
    <source file='/var/lib/libvirt/images/celebrimbor/src/ubuntu-24.04-arm64.qcow2'/>
  </backingStore>
</volume>"#;

        assert_eq!(
            LibvirtHypervisor::backing_image(xml),
            Some("ubuntu-24.04-arm64.qcow2".to_owned())
        );
    }
}
