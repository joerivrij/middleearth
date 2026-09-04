use serde::Serialize;
use uuid::Uuid;

pub type VmId = Uuid;

#[derive(Clone, Serialize)]
pub struct Vm {
    pub id: VmId,
    pub name: String,
    pub state: VmState,
    pub resources: VmResources,
    pub image: Image,
    pub network: Network,
    pub metadata: Metadata,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(not(feature = "libvirt"), allow(dead_code))]
pub enum VmState {
    NoState,
    Running,
    Blocked,
    Paused,
    ShuttingDown,
    ShutOff,
    Crashed,
    Suspended,
    Unknown,
}

#[derive(Clone, Serialize)]
pub struct Image {
    pub name: Option<String>,
}

#[derive(Clone, Serialize)]
pub struct VmResources {
    pub memory_mib: u64,
    pub vcpus: u32,
    pub disk: Disk,
}

#[derive(Clone, Serialize)]
pub struct Disk {
    pub capacity_bytes: u64,
    pub allocated_bytes: u64,
    pub format: Option<String>,
}

#[derive(Clone, Serialize)]
pub struct Network {
    pub ip_address: Option<String>,
    pub mac_address: Option<String>,
}

#[derive(Clone, Serialize)]
pub struct Metadata {
    pub architecture: Option<String>,
    pub ssh_user: Option<String>,
}

pub struct CreateVmSpec {
    pub name: String,
    pub memory_mib: u64,
    pub disk_gib: u64,
    pub vcpus: u32,
    pub image: Option<String>,
}
