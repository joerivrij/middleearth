use std::path::Path;

use virt::{connect::Connect, domain::Domain, storage_pool::StoragePool, storage_vol::StorageVol};

use crate::domain::{Disk, Image, Metadata, Network, Vm, VmResources, VmState};

use super::LibvirtHypervisor;

impl LibvirtHypervisor {
    pub(super) fn domain_to_vm(&self, connection: &Connect, domain: &Domain) -> Result<Vm, String> {
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
