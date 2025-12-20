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

  . = ALIGN(16);
  __heap_start = .;
  __heap_end = 0x1000000; /* Up to 16MB */
}
