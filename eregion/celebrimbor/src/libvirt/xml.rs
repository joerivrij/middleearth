use std::path::Path;

use uuid::Uuid;
pub(super) struct VolumeXmlSpec<'a> {
    pub volume_name: &'a str,
    pub disk_gib: u64,
    pub backing_path: &'a Path,
}

pub(super) struct DomainXmlSpec<'a> {
    pub name: &'a str,
    pub id: Uuid,
    pub memory_mib: u64,
    pub vcpus: u32,
    pub disk_path: &'a str,
    pub seed_path: &'a Path,
}

pub(super) fn volume_xml(spec: VolumeXmlSpec<'_>) -> String {
    let VolumeXmlSpec {
        volume_name,
        disk_gib,
        backing_path,
    } = spec;

    format!(
        r#"<volume>
<name>{volume_name}</name>
<capacity unit="GiB">{disk_gib}</capacity>
<target>
<format type="qcow2"/>
</target>
<backingStore>
<path>{}</path>
<format type="qcow2"/>
</backingStore>
</volume>"#,
        backing_path.display()
    )
}

pub(super) fn domain_xml(spec: DomainXmlSpec<'_>) -> String {
    let DomainXmlSpec {
        name,
        id,
        memory_mib,
        vcpus,
        disk_path,
        seed_path,
    } = spec;

    format!(
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
    )
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use uuid::Uuid;

    use super::{DomainXmlSpec, VolumeXmlSpec, domain_xml, volume_xml};

    #[test]
    fn creates_volume_xml_from_spec() {
        let xml = volume_xml(VolumeXmlSpec {
            volume_name: "annatar.qcow2",
            disk_gib: 8,
            backing_path: Path::new("/images/ubuntu.qcow2"),
        });

        assert!(xml.contains("<name>annatar.qcow2</name>"));
        assert!(xml.contains("<capacity unit=\"GiB\">8</capacity>"));
        assert!(xml.contains("<path>/images/ubuntu.qcow2</path>"));
    }

    #[test]
    fn creates_domain_xml_from_spec() {
        let id = Uuid::parse_str("d2eb1806-a4ca-4e1e-8615-69b9bfbc79e0").unwrap();
        let xml = domain_xml(DomainXmlSpec {
            name: "annatar",
            id,
            memory_mib: 1024,
            vcpus: 2,
            disk_path: "/images/annatar.qcow2",
            seed_path: Path::new("/images/annatar-seed.img"),
        });

        assert!(xml.contains("<name>annatar</name>"));
        assert!(xml.contains("<uuid>d2eb1806-a4ca-4e1e-8615-69b9bfbc79e0</uuid>"));
        assert!(xml.contains("<memory unit=\"MiB\">1024</memory>"));
        assert!(xml.contains("<vcpu placement=\"static\">2</vcpu>"));
        assert!(xml.contains("<source file=\"/images/annatar.qcow2\"/>"));
        assert!(xml.contains("<source file=\"/images/annatar-seed.img\"/>"));
    }
}
