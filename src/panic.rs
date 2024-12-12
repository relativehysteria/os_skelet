use core::panic::PanicInfo;
use crate::cpu;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Print the location info
    if let Some(loc) = info.location() {
       print_shatter!("!!! PANIC !!! {} {}:{} ----",
            loc.file(), loc.column(), loc.line());
    }

    // Print the message
    print_shatter!(" {} ----\n", info.message());

    // And halt
    unsafe { cpu::halt() };
}
