//! EFI definitions

pub mod efi;
pub mod memory;
pub mod status;

pub use efi::*;
pub use memory::*;
pub use status::*;

/// Errors that can be possibly returned by memory routines
#[derive(Debug)]
pub enum Error {
    /// Memory map expected a larger array
    WrongMemoryMapSize(usize),

    /// Couldn't exit the boot services
    ExitBootSvcFailed,
}
