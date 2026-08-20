/* CH32V003 (QingKe V2A) — 16K flash, 2K SRAM.
   Flash is mapped at 0x08000000 and aliased at 0x00000000 when booting from
   user flash. Code is linked at 0x08000000 (the address wlink flashes to);
   it executes correctly from either alias since addressing is PC-relative. */

ENTRY(_start);

MEMORY
{
    FLASH (rx)  : ORIGIN = 0x08000000, LENGTH = 16K
    RAM   (rwx) : ORIGIN = 0x20000000, LENGTH = 2K
}

SECTIONS
{
    /* Startup code must be first: the core starts executing at the flash base. */
    .init : {
        KEEP(*(.init))
    } > FLASH

    .vector_table : {
        KEEP(*(.vector_table))
    } > FLASH

    .text : {
        *(.text .text.*)
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
        . = ALIGN(4);
        _etext = .;
    } > FLASH

    /* Initialized data: lives in RAM, load image stored in flash at _etext.
       Startup code copies _sdata.._edata from _etext. */
    .data : AT(_etext) {
        _sdata = .;
        *(.data .data.*)
        *(.sdata .sdata.*)
        . = ALIGN(4);
        _edata = .;
    } > RAM

    .bss (NOLOAD) : {
        _sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        *(COMMON)
        . = ALIGN(4);
        _ebss = .;
    } > RAM

    /DISCARD/ : {
        *(.eh_frame .eh_frame_hdr)
        *(.riscv.attributes)
    }

    _stack_top = ORIGIN(RAM) + LENGTH(RAM);
}
