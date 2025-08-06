use std::fs::File;
use std::os::fd::{AsFd, BorrowedFd};
use std::os::unix::io::{FromRawFd, IntoRawFd};

use drm::Device as BasicDevice;
use drm::control::Device as ControlDevice;
use nix::fcntl::{open as nix_open, OFlag};
use nix::sys::stat::Mode;

pub struct DrmDeviceWrapper(pub File);

impl AsFd for DrmDeviceWrapper {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}
impl BasicDevice for DrmDeviceWrapper {}
impl ControlDevice for DrmDeviceWrapper {}

impl DrmDeviceWrapper {
    pub fn open(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let fd = nix_open(std::path::Path::new(path), OFlag::O_RDWR, Mode::empty())?;
        let file = unsafe { File::from_raw_fd(fd.into_raw_fd()) };
        Ok(DrmDeviceWrapper(file))
    }
}
