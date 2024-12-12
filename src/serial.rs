//! A serial port driver
//!
//! This driver doesn't enumerate any tables but rather uses the good old
//! hardcoded ports. Chances of this not working on your hardware are small but
//! if you don't see any output, you might wanna fiddle with this code.

use core::fmt::Write;
use crate::spinlock::SpinLock;
use crate::cpu::{ in8, out8 };

/// The global serial driver.
///
/// This driver has to be initialized by `Serial::init()` and is a global,
/// because the print macro doesn't have access to any arguments.
pub static SERIAL_DRIVER: SpinLock<Option<Serial>> = SpinLock::new(None);

/// Addreses of the serial ports that are to be used by this serial driver
pub const PORT_ADDRESSES: [*const u16; 2] = [
    0x2F8 as *const u16,
    0x3F8 as *const u16,
];

/// A more-or-less dummy struct that implements `Write` such that `print!()` can
/// be used on it
pub struct Serial;

impl Serial {
    /// Initialize the serial ports at addresses [`PORT_ADDRESSES`] on the
    /// system to 28800n1.
    ///
    /// Panics if the serial driver is already initialized
    pub fn init() {
        // Get the lock to the global driver
        let mut driver = SERIAL_DRIVER.lock();

        // Make sure we're not re-initializing
        if driver.is_some() { return; }

        // Go through each COM port and initialize it
        for port in PORT_ADDRESSES.iter() {
            unsafe {
                // Disable all interrupts
                out8(port.offset(1), 0x00);

                // Enable DLAB (set baud divisor)
                out8(port.offset(3), 0x80);

                // Divisor = 115200 / this; low byte and high byte, respectively
                out8(port.offset(0), 0x04);
                out8(port.offset(1), 0x00);

                // 8 bits, no parity, one stop bit
                out8(port.offset(3), 0x03);

                // IRQs disabled, RTS/DSR set
                out8(port.offset(4), 0x03);
            }
        }

        // Save the initialized driver
        *driver = Some(Self);
    }

    /// Read a byte from the first COM port that has a byte available
    pub fn read_byte(&mut self) -> Option<u8> {
        // Iterate through the devices
        for &port in PORT_ADDRESSES.iter() {
            unsafe {
                // Check if there is a byte available.
                // If yes, read and return it
                if (in8(port.offset(5)) & 1) != 0 {
                    return Some(in8(port));
                }
            }
        }

        // No bytes to read
        None
    }

    /// Write a byte to a COM port
    fn write_byte(&mut self, port: *const u16, byte: u8) {
        // Check if this port exists
        if let Some(&port) = PORT_ADDRESSES.iter().find(|&&x| x == port) {
            unsafe {
                // Wait for the transmit to be empty
                while in8(port.offset(5)) & 0x20 == 0 {};

                // Write the byte
                out8(port, byte);
            }
        }
    }

    /// Write bytes to all mapped serial devices
    pub fn write(&mut self, bytes: &[u8]) {
        // Iterate through the bytes
        for &byte in bytes {
            // Write the byte to all mapped serial devices
            for &port in PORT_ADDRESSES.iter() {
                // Handle newlines correctly
                if byte == b'\n' { self.write_byte(port, b'\r'); }

                // Write the byte
                self.write_byte(port, byte);
            }
        }
    }
}

impl Write for Serial {
    fn write_str(&mut self, string: &str) -> core::fmt::Result {
        let mut serial = SERIAL_DRIVER.lock();
        if let Some(serial) = &mut *serial {
            serial.write(string.as_bytes());
        }
        Ok(())
    }
}

/// Serial `print!()` support
#[macro_export] macro_rules! print {
    ($($arg:tt)*) => {
        let _ = <$crate::serial::Serial as core::fmt::Write>::write_fmt(
            &mut $crate::serial::Serial, format_args!($($arg)*));
    }
}

/// Dummy type to implement the `Write` trait on -- used for serial shattering
pub struct SerialShatter;

impl Write for SerialShatter {
    fn write_str(&mut self, string: &str) -> core::fmt::Result {
        unsafe {
            let serial = SERIAL_DRIVER.shatter();
            if let Some(serial) = &mut *serial {
                serial.write(string.as_bytes());
            }
        }
        Ok(())
    }
}

/// Serial `print!()` that shatters the serial driver lock on print and
/// as such is unsafe. Meant to be used in panics
#[macro_export] macro_rules! print_shatter {
    ($($arg:tt)*) => {
        let _ = <$crate::serial::Serial as core::fmt::Write>::write_fmt(
            &mut $crate::serial::Serial, format_args!($($arg)*));
    }
}
