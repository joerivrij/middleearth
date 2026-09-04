use super::{
    LibvirtHypervisor,
    xml::{VolumeXmlSpec, volume_xml},
};
use virt::{connect::Connect, storage_pool::StoragePool, storage_vol::StorageVol};

use crate::application::HypervisorCreateError;

impl LibvirtHypervisor {
    pub(super) fn create_volume(
        &self,
        connection: &Connect,
        name: &str,
        disk_gib: u64,
        image: Option<String>,
    ) -> Result<(StorageVol, String), HypervisorCreateError> {
        let image = image.unwrap_or_else(|| self.default_image.clone());
        let backing_path = self.storage_dir.join("src").join(image);
        if !backing_path.is_file() {
            return Err(HypervisorCreateError::ImageNotFound);
        }

        let pool = StoragePool::lookup_by_name(connection, &self.storage_pool)
            .map_err(|error| HypervisorCreateError::Internal(error.to_string()))?;

        let volume_name = format!("{name}.qcow2");

        let volume_xml = volume_xml(VolumeXmlSpec {
            volume_name: &volume_name,
            disk_gib,
            backing_path: &backing_path,
        });

        let volume = StorageVol::create_xml(&pool, &volume_xml, 0)
            .map_err(|error| HypervisorCreateError::Internal(error.to_string()))?;

        match volume.get_path() {
            Ok(path) => Ok((volume, path)),
            Err(error) => {
                Self::delete_volume(&volume);
                Err(HypervisorCreateError::Internal(error.to_string()))
            }
        }
    }

    pub(super) fn delete_volume(volume: &StorageVol) {
        let _ = volume.delete(0);
    }

    pub(super) fn delete_vm_volumes(&self, connection: &Connect, name: &str) -> Result<(), String> {
        let pool = StoragePool::lookup_by_name(connection, &self.storage_pool)
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

        Ok(())
    }
}
