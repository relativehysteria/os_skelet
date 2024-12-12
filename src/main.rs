#![no_std]
#![no_main]

use kernel::{ efi, serial, mm };

#[unsafe(no_mangle)]
fn efi_main(img_handle: efi::Handle,
            sys_table: *mut efi::SystemTable) -> efi::Status {
    // Initialize the serial driver
    serial::Serial::init();

    // Get the free memory map and exit the boot services.
    let memory = unsafe { efi::memory_map_exit(img_handle, sys_table) };

    // Initialize the memory manager
    // UEFI automatically sets up 1:1 paging, so each access is direct to
    // physical memory.
    mm::init(memory.expect("Couldn't acquire the free memory map."));

    // Your code here :)

    panic!("Reached end of execution.");
}
