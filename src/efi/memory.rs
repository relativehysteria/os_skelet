//! Memory related definitions

use crate::efi::*;
use crate::rangeset::{ Range, RangeSet };

/// Errors possibly returned by EFI routines
#[derive(Debug)]
pub enum Error {
    /// Memory map expected a larger array
    WrongMemoryMapSize(usize),

    /// Couldn't exit the boot services
    ExitBootSvcFailed,

    /// Some calculation overflowed while creating the free memory map
    MemoryMapOverflow,
}

#[derive(Debug, Copy, Clone)]
#[repr(C, align(16))]
/// Descriptors returned by `get_memory_map()`
pub struct MemoryDescriptor {
    /// Type of the memory region
    pub mem_type: MemoryType,

    /// Physical address of the first byte in the memory region
    pub phys_addr: usize,

    /// Virtual address of the first byte in the memory region
    pub virt_addr: usize,

    /// Number of 4 KiB pages in the memory region
    pub n_pages: u64,

    /// Attributes of the memory region that describe the bit mask of
    /// capabilities for that memory region, and not necessarily the current
    /// settings for that memory region
    _attribute: u64,
}

impl MemoryDescriptor {
    /// Returns a memory descriptor whose byte map is filled with 0s.
    const fn empty() -> Self {
        MemoryDescriptor {
            mem_type: MemoryType::Reserved,
            phys_addr: 0,
            virt_addr: 0,
            n_pages: 0,
            _attribute: 0,
        }
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(u32)]
/// EFI memory types as defined by the spec
pub enum MemoryType {
    /// Not usable
    Reserved,

    /// Code portion of a loaded UEFI application
    LoaderCode,

    /// Data portion of a loaded UEFI application
    LoaderData,

    /// Code portion of a loaded boot service driver
    BootServicesCode,

    /// Data portion of a loaded boot service driver
    BootServicesData,

    /// Code portion of a loaded runtime driver
    RuntimeServicesCode,

    /// Data portion of a loaded runtime driver
    RuntimeServicesData,

    /// Free (unallocated) memory
    ConventionalMemory,

    /// Memory in which errors have been detected
    UnusableMemory,

    /// Memory holding ACPI tables
    ACPIReclaimMemory,

    /// Reserved for use by the firmware
    ACPIMemoryNVS,

    /// Used by system firmware to request that a memory-mapped IO region be
    /// mapped by the OS to a virtual address so it can be accessed by EFI
    /// runtime services
    MemoryMappedIO,

    /// System memory-mapped IO region that is used to translate memory cycles
    /// to IO cycles by the processor
    MemoryMappedIOPortSpace,

    /// Address space reserved by the firmware for code that is part of the
    /// processor
    PalCode,

    /// Like `ConventionalMemory`, but also supports byte-addressable
    /// non-volatility
    PersistentMemory,

    /// Memory type not supported by our system whatsoever
    Unsupported,
}

impl MemoryType {
    /// Returns whether this memory type is available for general use after
    /// `exit_boot_services()` has been called
    pub fn available_post_boot_svc_exit(&self) -> bool {
        match self {
            MemoryType::BootServicesCode   |
            MemoryType::BootServicesData   |
            MemoryType::PersistentMemory   |
            MemoryType::ConventionalMemory => true,
            ______________________________ => false,
        }
    }
}

impl From<u32> for MemoryType {
    fn from(val: u32) -> MemoryType {
        match val {
             0 => MemoryType::Reserved,
             1 => MemoryType::LoaderCode,
             2 => MemoryType::LoaderData,
             3 => MemoryType::BootServicesCode,
             4 => MemoryType::BootServicesData,
             5 => MemoryType::RuntimeServicesCode,
             6 => MemoryType::RuntimeServicesData,
             7 => MemoryType::ConventionalMemory,
             8 => MemoryType::UnusableMemory,
             9 => MemoryType::ACPIReclaimMemory,
            10 => MemoryType::ACPIMemoryNVS,
            11 => MemoryType::MemoryMappedIO,
            12 => MemoryType::MemoryMappedIOPortSpace,
            13 => MemoryType::PalCode,
            14 => MemoryType::PersistentMemory,
            _  => MemoryType::Unsupported,
        }
    }
}

/// Get a memory map of [`MemoryDescriptor`]s and exit the boot services
pub unsafe fn memory_map_exit(
    image_handle: Handle,
    sys_table: *mut SystemTable
) -> Result<RangeSet, Error> {
    // Get the pointer to `get_memory_map()` and `exit_boot_services()`
    let boot_svc = unsafe { &*((*sys_table).boot_svc) };
    let get_memory_map = boot_svc.get_memory_map;
    let exit_boot_services = boot_svc.exit_boot_services;

    // Create the arguments for the call. We expect at most `N_MEM_DESC`
    // `MemoryDescriptor`s.
    const N_MEM_DESC: usize = 2048;
    let mut memory_map = [MemoryDescriptor::empty(); N_MEM_DESC];

    let mut size = core::mem::size_of_val(&memory_map);
    let mut key = 0;
    let mut desc_size = 0;
    let mut desc_version = 0;

    // Populate the memory map
    let ret = unsafe {
        get_memory_map(&mut size, memory_map.as_mut_ptr() as *mut u8, &mut key,
                       &mut desc_size, &mut desc_version)
    };

    // Make sure we got the map
    if ret != Status::Success { return Err(Error::WrongMemoryMapSize(size)); }

    // Transmute the byte array to an array of descriptors
    let memory_map = unsafe {
        core::slice::from_raw_parts(
            memory_map.as_ptr(),
            size / core::mem::size_of::<MemoryDescriptor>())
    };

    // Exit the boot services
    let ret = unsafe { exit_boot_services(image_handle, key) };

    // Make sure we have exited successfully
    if ret != Status::Success { return Err(Error::ExitBootSvcFailed); }

    // Now, only retain the memory that we're free to use in a memory allocator
    let mut free_memory: RangeSet = RangeSet::new();
    for desc in memory_map.iter() {
        // Make sure we're free to use this memory
        if !desc.mem_type.available_post_boot_svc_exit() { continue; }

        // Calculate the end of this memory
        let offset = (desc.n_pages as usize).checked_mul(4096)
            .ok_or(Error::MemoryMapOverflow)?;
        let end = desc.phys_addr.checked_add(offset - 1)
            .ok_or(Error::MemoryMapOverflow)?;

        // Write the memory down. I make the assumption this shit will never
        // return errors because I'm just that cool B)
        free_memory.insert(Range::new(desc.phys_addr, end).unwrap()).unwrap();
    }

    // Make the null byte impossible to be allocated
    let _ = free_memory.remove(Range::new(0, 1).unwrap());

    // Return the free memory map
    Ok(free_memory)
}
