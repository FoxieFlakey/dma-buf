use std::{
    ffi::{CStr, c_char},
    io,
    os::fd::{AsRawFd, BorrowedFd, FromRawFd, OwnedFd},
};

use enumflags2::BitFlags;

// NOTE: If end not set then this is start of session
// there no Start because no bit set is invalid for flag
#[enumflags2::bitflags]
#[repr(u64)]
#[derive(Clone, Copy)]
pub enum SyncFlags {
    Read = 1 << 0,
    Write = 1 << 1,
    End = 1 << 2,
}

// Like the SyncFlags but only Read/Writer
// for use import/export
#[enumflags2::bitflags]
#[repr(u32)]
#[derive(Clone, Copy)]
pub enum ImportExportSyncFlags {
    Read = 1 << 0,
    Write = 1 << 1,
}

#[repr(C)]
struct DmaBufSync {
    flags: u64,
}

#[repr(C)]
struct DmaBufExportSyncFile {
    flags: u32,
    fd: i32,
}

#[repr(C)]
struct DmaBufImportSyncFile {
    flags: u32,
    fd: i32,
}

mod ioctls {
    use nix::{ioctl_readwrite, ioctl_write_ptr};
    use std::ffi::c_char;

    use crate::allocation::{DmaBufExportSyncFile, DmaBufImportSyncFile, DmaBufSync};

    const DMA_BUF_BASE: u8 = b'b';

    ioctl_write_ptr!(sync, DMA_BUF_BASE, 0, DmaBufSync);
    ioctl_write_ptr!(set_name, DMA_BUF_BASE, 1, c_char);
    ioctl_readwrite!(export_sync_file, DMA_BUF_BASE, 2, DmaBufExportSyncFile);
    ioctl_write_ptr!(import_sync_file, DMA_BUF_BASE, 3, DmaBufImportSyncFile);
}

pub fn set_name(buf_fd: BorrowedFd<'_>, new_name: &CStr) -> io::Result<()> {
    unsafe { ioctls::set_name(buf_fd.as_raw_fd(), new_name.as_ptr() as *const c_char) }?;
    Ok(())
}

pub fn sync(buf_fd: BorrowedFd<'_>, flags: BitFlags<SyncFlags>) -> io::Result<()> {
    let mut req = DmaBufSync {
        flags: flags.bits(),
    };

    unsafe { ioctls::sync(buf_fd.as_raw_fd(), &raw mut req) }?;
    Ok(())
}

pub fn export_sync_file(
    buf_fd: BorrowedFd<'_>,
    flags: BitFlags<ImportExportSyncFlags>,
) -> io::Result<OwnedFd> {
    let mut req = DmaBufExportSyncFile {
        flags: flags.bits(),
        fd: 0,
    };

    unsafe { ioctls::export_sync_file(buf_fd.as_raw_fd(), &raw mut req) }?;

    // SAFETY: Kernel gave the 'fd' :33333
    Ok(unsafe { OwnedFd::from_raw_fd(req.fd) })
}

pub fn import_sync_file(
    buf_fd: BorrowedFd<'_>,
    sync_file: BorrowedFd<'_>,
    flags: BitFlags<ImportExportSyncFlags>,
) -> io::Result<()> {
    let mut req = DmaBufImportSyncFile {
        flags: flags.bits(),
        fd: sync_file.as_raw_fd(),
    };

    unsafe { ioctls::import_sync_file(buf_fd.as_raw_fd(), &raw mut req) }?;
    Ok(())
}
