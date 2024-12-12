//! Arch specific routines that interface with the CPU directly

use core::arch::asm;
use core::arch::x86_64::_rdtsc;

/// SMR for the active GS base
pub const IA32_GS_BASE: u32 = 0xC0000101;

/// Write a byte to I/O port `addr
#[inline]
pub unsafe fn out8(addr: *const u16, byte: u8) {
    unsafe { asm!("out dx, al", in("dx") addr, in("al") byte) };
}

/// Read a byte from I/O port `addr`
#[inline]
pub unsafe fn in8(addr: *const u16) -> u8 {
    let mut byte: u8;
    unsafe { asm!("in al, dx", in("dx") addr, out("al") byte) };
    byte
}

/// Output a 32-bit `val` to I/O port `addr`
#[inline]
pub unsafe fn out32(addr: *const u16, byte: u32) {
    unsafe { asm!("out dx, eax", in("dx") addr, in("eax") byte) };
}

/// Read an 32-bit value from I/O port `addr`
#[inline]
pub unsafe fn in32(addr: *const u16) -> u32 {
    let mut byte: u32;
    unsafe { asm!("in eax, dx", in("dx") addr, out("eax") byte) };
    byte
}

/// Read the value from the Model-Specific Register `msr`
#[inline]
pub unsafe fn rdmsr(msr: u32) -> u64 {
    let high: u32;
    let low: u32;
    unsafe { asm!("rdmsr", in("ecx") msr, out("edx") high, out("eax") low) };
    ((high as u64) << 32) | (low as u64)
}

/// Write the 64-bit `val` to the Model-Specific Register `msr`
#[inline]
pub unsafe fn wrmsr(msr: u32, val: u64) {
    let high = (val >> 32) as u32;
    let low = val as u32;
    unsafe { asm!("wrmsr", in("ecx") msr, in("edx") high, in("eax") low) };
}

/// Calls RDTSC
#[inline]
pub unsafe fn rdtsc() -> usize {
    unsafe { _rdtsc() as usize }
}

/// Clears interrupts and halts the core
#[inline]
pub unsafe fn halt() -> ! {
    unsafe { asm!("cli", "hlt") };
    loop { core::hint::spin_loop(); }
}
