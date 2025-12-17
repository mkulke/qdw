ENTRY(start16)

SECTIONS
{
  . = 0x8000;

  .text : { *(.text .text.*) }
  .rodata : { *(.rodata .rodata.*) }
  .data : { *(.data .data.*) }

  .bss : {
    *(.bss .bss.*)
    *(COMMON)
  }
}
