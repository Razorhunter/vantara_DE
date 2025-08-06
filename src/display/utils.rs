use std::fs;

const DEFAULT_GPU_DRIVE_PATH: &str = "/dev/dri";

pub fn find_drm_card() -> Option<String> {
    let entries = fs::read_dir(DEFAULT_GPU_DRIVE_PATH).ok()?;
    for entry in entries {
        let path = entry.ok()?.path();
        if path.file_name()?.to_string_lossy().starts_with("card") {
            return Some(path.to_string_lossy().to_string());
        }
    }
    None
}
