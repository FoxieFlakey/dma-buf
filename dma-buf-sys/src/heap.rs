use std::{
    io,
    os::fd::{AsRawFd, BorrowedFd, FromRawFd, OwnedFd},
};

use enumflags2::{BitFlags, bitflags};

#[repr(C)]
struct DmaHeapAllocationDataRaw {
    len: u64,
    fd: u32,
    fd_flags: u32,
    heap_flags: u64,
}

#[bitflags]
#[repr(u32)]
#[derive(Clone, Copy)]
pub enum FdFlags {
    Read,
    Write,
    CloseOnExec,
}

#[allow(deprecated)]
mod a {
    #[enumflags2::bitflags]
    #[repr(u64)]
    #[derive(Clone, Copy)]
    pub enum HeapFlags {
        // Currently there none defined yet, this crate
        // will panic if this sets
        #[doc(hidden)]
        #[deprecated(
            since = "0.0.0",
            note = "DMA-BUF has no heap flags yet but enumflags2 cant work with empty enum.Please dont use"
        )]
        None = 1 << 63,
    }
}
pub use a::HeapFlags;

mod ioctls {
    use nix::ioctl_readwrite;

    use crate::heap::DmaHeapAllocationDataRaw;
    const DMA_HEAP_IOC_MAGIC: u8 = b'H';
    ioctl_readwrite!(heap_alloc, DMA_HEAP_IOC_MAGIC, 0, DmaHeapAllocationDataRaw);
}

pub fn alloc(
    main_fd: BorrowedFd<'_>,
    fd_flags: BitFlags<FdFlags>,
    heap_flags: BitFlags<HeapFlags>,
    len: usize,
) -> io::Result<OwnedFd> {
    assert!(
        heap_flags.is_empty(),
        "DMA-BUF has no heap flags defined at the moment, DO NOT USE HeapFlags::None"
    );

    let mut req = DmaHeapAllocationDataRaw {
        len: u64::try_from(len).map_err(|_| io::ErrorKind::InvalidInput)?,
        fd: 0,
        fd_flags: fd_flags.bits(),
        heap_flags: heap_flags.bits(),
    };

    unsafe { ioctls::heap_alloc(main_fd.as_raw_fd(), &raw mut req) }?;

    // SAFETY: Kernel gave fd
    Ok(unsafe { OwnedFd::from_raw_fd(i32::try_from(req.fd).expect("Fd is too big than u32")) })
}
