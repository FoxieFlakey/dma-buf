use std::{
    fs::OpenOptions,
    io,
    os::fd::{AsFd, OwnedFd},
    path::Path,
};

use enumflags2::BitFlags;
use nix::sys::stat::{fstat, major};
use procfs::{Current, Devices, ProcError};
use thiserror::Error;

use crate::buffer::DmaBuffer;

pub use dma_buf_sys::heap::{FdFlags, HeapFlags};

pub const DEFAULT_SYSTEM_HEAP: &str = "/dev/dma_heap/system";

pub mod buffer;

pub use memmap2;

pub struct DmaHeap {
    fd: OwnedFd,
}

#[derive(Error, Debug)]
pub enum OpenError {
    #[error("Cannot open heap file")]
    OpenFile(#[source] io::Error),
    #[error("Cant read /proc/devices")]
    ReadProcDevices(#[source] ProcError),
    #[error("Cant find dma heap major number")]
    NoDmaHeapMajor,
    #[error("Cant find fstat the dma heap file")]
    FstatError(#[source] io::Error),
    #[error("File is not DMA heap")]
    NonDmaHeapFile,
}

#[derive(Error, Debug)]
pub enum AllocError {
    #[error("Cannot perform allocate ioctl")]
    CannotAlloc(#[source] io::Error),
}

impl DmaHeap {
    pub fn open<P>(path: &P) -> Result<Self, OpenError>
    where
        P: AsRef<Path> + ?Sized,
    {
        let fd = OwnedFd::from(
            OpenOptions::new()
                .write(true)
                .read(true)
                .open(path)
                .map_err(OpenError::OpenFile)?,
        );

        let devices = Devices::current().map_err(OpenError::ReadProcDevices)?;
        let dma_heap_major = devices
            .char_devices
            .iter()
            .filter(|x| x.name == "dma_heap")
            .map(|x| x.major)
            .next()
            .ok_or(OpenError::NoDmaHeapMajor)?;

        let stat = fstat(&fd).map_err(|x| OpenError::FstatError(x.into()))?;
        if major(stat.st_rdev) != u64::from(dma_heap_major) {
            return Err(OpenError::NonDmaHeapFile);
        }

        Ok(Self { fd })
    }

    pub fn alloc(
        &self,
        fd_flags: BitFlags<FdFlags>,
        heap_flags: BitFlags<HeapFlags>,
        len: usize,
    ) -> Result<DmaBuffer, AllocError> {
        dma_buf_sys::heap::alloc(self.fd.as_fd(), fd_flags, heap_flags, len)
            // SAFETY: This indeed dma buffer
            .map(|x| unsafe { DmaBuffer::from_owned_fd_and_size(x, len) })
            .map_err(AllocError::CannotAlloc)
    }
}
