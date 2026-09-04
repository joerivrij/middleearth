use std::{fs, path::PathBuf, process::Command};

use crate::domain::VmId;

use super::LibvirtHypervisor;

impl LibvirtHypervisor {
    pub(super) fn create_seed(&self, id: VmId, name: &str) -> Result<PathBuf, String> {
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
}
