use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use serde::Deserialize;

use crate::domain::CreateVmSpec;
use crate::{
    application::{CreateVmError, DeleteVmError, GetVmError, ListVmsError, VmService},
    domain::{Vm, VmId},
};

fn default_memory_mib() -> u64 {
    512
}

fn default_vcpus() -> u32 {
    1
}

fn default_disk_gib() -> u64 {
    8
}

pub fn router(vm_service: VmService) -> Router {
    Router::new()
        .route("/v1/vms", post(create_vm).get(list_vms))
        .route("/v1/vms/{id}", get(get_vm).delete(delete_vm))
        .with_state(vm_service)
}

#[derive(Deserialize)]
struct CreateVmRequest {
    name: String,
    image: Option<String>,

    #[serde(default = "default_memory_mib")]
    memory_mib: u64,

    #[serde(default = "default_vcpus")]
    vcpus: u32,

    #[serde(default = "default_disk_gib")]
    disk_gib: u64,
}

async fn create_vm(
    State(vm_service): State<VmService>,
    Json(request): Json<CreateVmRequest>,
) -> Result<(StatusCode, Json<Vm>), StatusCode> {
    let spec = CreateVmSpec {
        name: request.name,
        memory_mib: request.memory_mib,
        disk_gib: request.disk_gib,
        vcpus: request.vcpus,
        image: request.image,
    };

    let vm = vm_service.create_vm(spec).map_err(|error| match error {
        CreateVmError::InvalidName
        | CreateVmError::InvalidMemory
        | CreateVmError::InvalidVcpus
        | CreateVmError::InvalidImage
        | CreateVmError::InvalidStorage
        | CreateVmError::ImageNotFound => StatusCode::BAD_REQUEST,
        CreateVmError::NameAlreadyExists => StatusCode::CONFLICT,
        CreateVmError::Hypervisor(message) => {
            tracing::error!(%message, "failed to create VM");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

    Ok((StatusCode::CREATED, Json(vm)))
}

async fn list_vms(State(vm_service): State<VmService>) -> Result<Json<Vec<Vm>>, StatusCode> {
    let vms = vm_service.list_vms().map_err(|error| match error {
        ListVmsError::Hypervisor(message) => {
            tracing::error!(%message, "failed to list VMs");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

    Ok(Json(vms))
}

async fn get_vm(
    State(vm_service): State<VmService>,
    Path(id): Path<VmId>,
) -> Result<Json<Vm>, StatusCode> {
    let vm = vm_service.get_vm(id).map_err(|error| match error {
        GetVmError::NotFound => StatusCode::NOT_FOUND,
        GetVmError::Hypervisor(message) => {
            tracing::error!(%message, "failed to get VM");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

    Ok(Json(vm))
}

async fn delete_vm(
    State(vm_service): State<VmService>,
    Path(id): Path<VmId>,
) -> Result<StatusCode, StatusCode> {
    vm_service.delete_vm(id).map_err(|error| match error {
        DeleteVmError::NotFound => StatusCode::NOT_FOUND,
        DeleteVmError::Hypervisor(message) => {
            tracing::error!(%message, "failed to delete VM");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

    Ok(StatusCode::NO_CONTENT)
}
