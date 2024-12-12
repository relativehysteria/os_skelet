a minimal x64 uefi system. no paging, no processes, no drivers, just main.  
mostly useful for measuring algo performance.

check out the [`qemu`](./qemu) script for qemu requirements. use a different
hypervisor if you wish.

if you wanna use this as your os skeleton:
* the serial driver uses hardcoded uart ports. if this doesn't work for you,
  either enumerate them yourself or use spcr.
* no boot services are used. if you expect to have a large memory map, you might
  want to dynamically allocate space to parse them instead of using whatever
  hardcoded space i use.
