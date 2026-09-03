mod application;
mod config;
mod domain;
mod http;
#[cfg(not(feature = "libvirt"))]
mod hypervisor;
#[cfg(feature = "libvirt")]
mod libvirt;

use std::sync::Arc;

use application::{Hypervisor, VmService};
#[cfg(not(feature = "libvirt"))]
use hypervisor::InMemoryHypervisor;
#[cfg(feature = "libvirt")]
use libvirt::LibvirtHypervisor;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = config::Config::from_env().expect("invalid config");

    #[cfg(feature = "libvirt")]
    let hypervisor: Arc<dyn Hypervisor> = Arc::new(LibvirtHypervisor::new(
        "qemu:///system",
        "celebrimbor",
        "/var/lib/libvirt/images/celebrimbor",
        "ubuntu-24.04-arm64.qcow2",
        "/home/pi/celebrimbor/authorized_key.pub",
    ));

    #[cfg(not(feature = "libvirt"))]
    let hypervisor: Arc<dyn Hypervisor> = Arc::new(InMemoryHypervisor::new());

    let vm_service = VmService::new(hypervisor);
    let app = http::router(vm_service);

    let address = format!("{}:{}", config.host, config.port);
    let listener = TcpListener::bind(&address)
        .await
        .expect("failed to bind TCP listener");

    tracing::info!(%address, "celebrimbor listening");

    axum::serve(listener, app).await.expect("failed to serve");
}
