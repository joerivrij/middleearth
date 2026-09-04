use std::{collections::HashMap, sync::Mutex};

use uuid::Uuid;

use crate::{
    application::{Hypervisor, HypervisorCreateError},
    domain::{CreateVmSpec, Disk, Image, Metadata, Network, Vm, VmId, VmResources, VmState},
};

pub struct InMemoryHypervisor {
    vms: Mutex<HashMap<VmId, Vm>>,
}

impl InMemoryHypervisor {
    pub fn new() -> Self {
        Self {
            vms: Mutex::new(HashMap::new()),
        }
    }
}

impl Hypervisor for InMemoryHypervisor {
    fn create_vm(&self, spec: CreateVmSpec) -> Result<Vm, HypervisorCreateError> {
        let CreateVmSpec {
            name,
            memory_mib,
            disk_gib,
            vcpus,
            image,
        } = spec;

        let mut vms = self.vms.lock().map_err(|_| {
            HypervisorCreateError::Internal("VM store lock is poisoned".to_string())
        })?;

        if vms.values().any(|vm| vm.name == name) {
            return Err(HypervisorCreateError::NameAlreadyExists);
        }

        let id = Uuid::new_v4();
        let vm = Vm {
            id,
            name,
            state: VmState::Running,
            resources: VmResources {
                memory_mib,
                vcpus,
                disk: Disk {
                    capacity_bytes: disk_gib * 1024 * 1024 * 1024,
                    allocated_bytes: 0,
                    format: Some("qcow2".to_owned()),
                },
            },
            image: Image {
                name: Some(image.unwrap_or_else(|| "ubuntu-24.04-arm64.qcow2".to_owned())),
            },
            network: Network {
                ip_address: None,
                mac_address: None,
            },
            metadata: Metadata {
                architecture: Some("aarch64".to_owned()),
                ssh_user: Some("ubuntu".to_owned()),
            },
        };

        vms.insert(id, vm.clone());

        Ok(vm)
    }

    fn list_vms(&self) -> Result<Vec<Vm>, String> {
        let vms = self
            .vms
            .lock()
            .map_err(|_| "VM store lock is poisoned".to_string())?;

        let mut result: Vec<Vm> = vms.values().cloned().collect();
        result.sort_by_key(|vm| vm.id);

        Ok(result)
    }

    fn get_vm(&self, id: VmId) -> Result<Option<Vm>, String> {
        let vms = self
            .vms
            .lock()
            .map_err(|_| "VM store lock is poisoned".to_string())?;

        Ok(vms.get(&id).cloned())
    }

    fn delete_vm(&self, id: VmId) -> Result<bool, String> {
        let mut vms = self
            .vms
            .lock()
            .map_err(|_| "VM store lock is poisoned".to_string())?;

        Ok(vms.remove(&id).is_some())
    }
}
