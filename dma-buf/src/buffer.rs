use std::{
    ffi::{CString, NulError},
    io,
    os::fd::{AsFd, OwnedFd},
};

pub use dma_buf_sys::allocation::{ImportExportSyncFlags, SyncFlags};
use enumflags2::BitFlags;
use memmap2::{Mmap, MmapMut, MmapOptions, MmapRaw};
use thiserror::Error;

pub struct DmaBuffer {
    len: usize,
    fd: OwnedFd,
}

impl AsFd for DmaBuffer {
    fn as_fd(&self) -> std::os::unix::prelude::BorrowedFd<'_> {
        self.fd.as_fd()
    }
}

#[derive(Error, Debug)]
pub enum SetNameError {
    #[error("Name is invalid CStr: {0:?}")]
    NameContainIsNotValidCStr(String, #[source] NulError),
    #[error("Cant do ioctl")]
    CantIoctl(#[source] io::Error),
}

impl DmaBuffer {
    // # Safety
    // caller must ensure that fd is valid DMA BUF fd and len is correct size for it
    pub unsafe fn from_owned_fd_and_size(fd: OwnedFd, len: usize) -> DmaBuffer {
        DmaBuffer { len, fd }
    }

    pub fn sync(&self, flags: BitFlags<SyncFlags>) -> io::Result<()> {
        dma_buf_sys::allocation::sync(self.as_fd(), flags)
    }

    pub fn set_name<S>(&self, name: &S) -> Result<(), SetNameError>
    where
        S: AsRef<str> + ?Sized,
    {
        let name = name.as_ref();
        dma_buf_sys::allocation::set_name(
            self.as_fd(),
            &CString::new(name)
                .map_err(|e| SetNameError::NameContainIsNotValidCStr(name.to_string(), e))?,
        )
        .map_err(SetNameError::CantIoctl)
    }

    // NOTE: Caller is responsible for doing necessary dma buffer syncs
    //
    // # Safety
    // This is unsafe, due nature that DMA BUF are shared with others
    // it is caller's job to ensure safetty of each access
    pub unsafe fn map(&self) -> io::Result<Mmap> {
        unsafe { MmapOptions::new().len(self.len).map(&self.fd) }
    }

    pub fn get_length(&self) -> usize {
        self.len
    }

    // NOTE: Caller is responsible for doing necessary dma buffer syncs
    //
    // # Safety
    // This is unsafe, due nature that DMA BUF are shared with others
    // it is caller's job to ensure safetty of each access
    pub unsafe fn map_mut(&mut self) -> io::Result<MmapMut> {
        unsafe { MmapOptions::new().len(self.len).map_mut(&self.fd) }
    }

    // NOTE: Caller is responsible for doing necessary dma buffer syncs
    pub fn map_raw(&mut self) -> io::Result<MmapRaw> {
        MmapOptions::new().len(self.len).map_raw(&self.fd)
    }
}
