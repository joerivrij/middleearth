use std::sync::Arc;

use crate::domain::{CreateVmSpec, Vm, VmId};

pub trait Hypervisor: Send + Sync {
    fn create_vm(&self, spec: CreateVmSpec) -> Result<Vm, HypervisorCreateError>;
    fn list_vms(&self) -> Result<Vec<Vm>, String>;
    fn get_vm(&self, id: VmId) -> Result<Option<Vm>, String>;
    fn delete_vm(&self, id: VmId) -> Result<bool, String>;
}

#[cfg_attr(not(feature = "libvirt"), allow(dead_code))]
pub enum HypervisorCreateError {
    NameAlreadyExists,
    ImageNotFound,
    Internal(String),
}

#[derive(Clone)]
pub struct VmService {
    hypervisor: Arc<dyn Hypervisor>,
}

impl VmService {
    pub fn new(hypervisor: Arc<dyn Hypervisor>) -> Self {
        Self { hypervisor }
    }

    pub fn create_vm(&self, mut spec: CreateVmSpec) -> Result<Vm, CreateVmError> {
        spec.name = spec.name.trim().to_owned();

        if spec.name.is_empty()
            || !spec
                .name
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || "-_.".contains(character))
        {
            return Err(CreateVmError::InvalidName);
        }

        if !(128..=2048).contains(&spec.memory_mib) {
            return Err(CreateVmError::InvalidMemory);
        }

        if !(1..=4).contains(&spec.vcpus) {
            return Err(CreateVmError::InvalidVcpus);
        }

        if !(4..=10).contains(&spec.disk_gib) {
            return Err(CreateVmError::InvalidStorage);
        }

        if spec.image.as_ref().is_some_and(|image| {
            image.is_empty()
                || !image
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric() || "-_.".contains(character))
        }) {
            return Err(CreateVmError::InvalidImage);
        }

        self.hypervisor
            .create_vm(spec)
            .map_err(|error| match error {
                HypervisorCreateError::NameAlreadyExists => CreateVmError::NameAlreadyExists,
                HypervisorCreateError::ImageNotFound => CreateVmError::ImageNotFound,
                HypervisorCreateError::Internal(message) => CreateVmError::Hypervisor(message),
            })
    }

    pub fn list_vms(&self) -> Result<Vec<Vm>, ListVmsError> {
        self.hypervisor.list_vms().map_err(ListVmsError::Hypervisor)
    }

    pub fn get_vm(&self, id: VmId) -> Result<Vm, GetVmError> {
        self.hypervisor
            .get_vm(id)
            .map_err(GetVmError::Hypervisor)?
            .ok_or(GetVmError::NotFound)
    }

    pub fn delete_vm(&self, id: VmId) -> Result<(), DeleteVmError> {
        let deleted = self
            .hypervisor
            .delete_vm(id)
            .map_err(DeleteVmError::Hypervisor)?;

        if !deleted {
            return Err(DeleteVmError::NotFound);
        }

        Ok(())
    }
}

pub enum CreateVmError {
    InvalidName,
    InvalidMemory,
    InvalidVcpus,
    InvalidStorage,
    InvalidImage,
    ImageNotFound,
    NameAlreadyExists,
    Hypervisor(String),
}

pub enum GetVmError {
    NotFound,
    Hypervisor(String),
}

pub enum ListVmsError {
    Hypervisor(String),
}

pub enum DeleteVmError {
    NotFound,
    Hypervisor(String),
}
