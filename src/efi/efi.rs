//! Generic EFI definitions to be used all over

use crate::efi::*;

/// Handle to any thing within the EFI spec
pub type Handle = *const usize;

#[derive(Debug)]
#[repr(C)]
/// Data structure that precedes all of the standard EFI table types
pub struct TableHeader {
    /// A 64-bit signature that identifies the type of table that follows
    /// Unique signatures have been generated for the EFI System Table, the EFI
    /// Boot Services Table, and the EFI Runtime Services Table
    pub signature: u64,

    /// The revision of the EFI Specification to which this table conforms. The
    /// upper 16 bits of this field contain the major revision value, and the
    /// lower 16 bits contain the minor revision value. The minor revision
    /// values are binary coded decimals and are limited to the range of 00..99
    ///
    /// A specification with the revision value ((2<<16) | (30)) would be
    /// referred as 2.3
    /// A specification with the revision value ((2<<16) | (31)) would be
    /// referred as 2.3.1
    pub revision: u32,

    /// The size, in bytes, of the entire table including the TableHeader
    pub header_size: u32,

    /// The 32-bit CRC for the entire table. This value is computed by setting
    /// this field to 0, and computing the 32-bit CRC for header_size bytes
    pub crc32: u32,

    /// Reserved; must be 0
    reserved: u32,
}

#[derive(Debug)]
#[repr(C)]
/// Struct containing pointers to the runtime and boot services tables
pub struct SystemTable {
    /// The table header for this struct
    pub hdr: TableHeader,

    /// Pointer to a null terminated string that identifies the vendor of the
    /// system firmware for the platform
    pub fw_vendor: *const u16,

    /// A firmware vendor specific revision of the system firmware
    pub fw_revision: u32,

    /// Pointers to unused structures
    _padding: [usize; 7],

    /// Pointer to the EFI Boot Services Table
    pub boot_svc: *const BootServices,

    /// Number of system configuration tables in the buffer `cfg_table`
    pub n_cfg_entries: usize,

    /// A pointer to the system configuration tables
    pub cfg_tables: *const ConfigTable,
}

/// Contains a set of GUID/pointer pairs compromised of the `cfg_table` field in
/// the [`SystemTable`]
pub struct ConfigTable {
    /// GUID identifying the configuration table
    pub guid: Guid,

    /// Pointer to the table associated with this GUID
    pub table: *const usize,
}

#[derive(Debug)]
#[repr(C)]
/// Struct containing pointers to `get_memory_map()` and `exit_boot_services()`,
/// padded to be aligned as defined by EFI_BOOT_SERVICES.
pub struct BootServices {
    /// The table header for this struct
    pub hdr: TableHeader,

    /// Pointers to unused functions
    _padding1: [usize; 4],

    /// Returns the current boot services memory map and memory map key
    pub get_memory_map: unsafe fn(memory_map_size:    &mut usize,
                                  memory_map:         *mut u8,
                                  map_key:            &mut usize,
                                  descriptor_size:    &mut usize,
                                  descriptor_version: &mut u32) -> Status,


    /// Pointers to unused functions
    _padding2: [usize; 21],

    /// Terminates boot services
    pub exit_boot_services: unsafe fn(image_handle: Handle,
                                      map_key:      usize) -> Status,

    // Other services omitted
}

#[derive(Debug, PartialEq)]
#[repr(C, packed)]
#[allow(missing_docs)]
/// UEFI defined global unique ID
pub struct Guid {
    pub d1: u32,
    pub d2: u16,
    pub d3: u16,
    pub d4: [u8; 8],
}

impl Guid {
    /// Returns a new Guid
    pub const fn new(d1: u32, d2: u16, d3: u16, d4: [u8; 8]) -> Self {
        Self { d1, d2, d3, d4 }
    }
}
