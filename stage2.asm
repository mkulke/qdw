BITS 16
section .text

extern kmain

global start16
global serial_init
global serial_putc
global serial_print

start16:
    cli

    ; -------- GDT --------
    lgdt [gdt_desc]

    ; Enter protected mode (PE=1)
    mov eax, cr0
    or  eax, 1
    mov cr0, eax
    jmp CODE32_SEL:pm32_entry

; ---------------- 32-bit protected mode ----------------
BITS 32
pm32_entry:
    mov ax, DATA_SEL
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov esp, 0x90000

    ; -------- Paging structures (identity map first 2 MiB using 2MiB page) --------
    ; We place them at 0xA0000+
    mov dword [pml4+0], pdpt + 0x003         ; PML4[0] -> PDPT, P=1 RW=1
    mov dword [pml4+4], 0
    mov dword [pdpt+0], pd + 0x003           ; PDPT[0] -> PD,  P=1 RW=1
    mov dword [pdpt+4], 0
    mov dword [pd+0], 0x00000083             ; PD[0]  -> 2MiB page, P=1 RW=1 PS=1
    mov dword [pd+4], 0

    ; Load CR3 with PML4
    mov eax, pml4
    mov cr3, eax

    ; Enable PAE
    mov eax, cr4
    or  eax, (1 << 5)                         ; CR4.PAE
    mov cr4, eax

    ; Enable long mode in EFER (LME=1)
    mov ecx, 0xC0000080                        ; IA32_EFER
    rdmsr
    or  eax, (1 << 8)                          ; EFER.LME
    wrmsr

    ; Enable paging (PG=1) while in protected mode -> activates long mode when we jump to 64-bit code
    mov eax, cr0
    or  eax, (1 << 31)                         ; CR0.PG
    mov cr0, eax

    ; Far jump to 64-bit code segment
    jmp CODE64_SEL:lm64_entry

; ---------------- 64-bit long mode ----------------
BITS 64
lm64_entry:
    mov ax, DATA_SEL
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov rsp, 0x90000

    call serial_init
	call kmain

.hang:
    hlt
    jmp .hang

; -------- Serial (COM1 = 0x3F8) --------
serial_init:
    mov dx, 0x3F8 + 1      ; IER
    xor al, al
    out dx, al

    mov dx, 0x3F8 + 3      ; LCR
    mov al, 0x80           ; DLAB=1
    out dx, al

    mov dx, 0x3F8 + 0      ; DLL (divisor low) -> 115200/3 = 38400 baud
    mov al, 3
    out dx, al
    mov dx, 0x3F8 + 1      ; DLM
    xor al, al
    out dx, al

    mov dx, 0x3F8 + 3      ; LCR
    mov al, 0x03           ; 8N1, DLAB=0
    out dx, al

    mov dx, 0x3F8 + 2      ; FCR
    mov al, 0xC7           ; enable FIFO, clear, 14-byte threshold
    out dx, al

    mov dx, 0x3F8 + 4      ; MCR
    mov al, 0x0B           ; IRQs off, RTS/DSR set
    out dx, al
    ret

serial_putc:
    ; wait for THR empty
.wait:
    mov dx, 0x3F8 + 5      ; LSR
    in  al, dx
    test al, 0x20
    jz .wait
    mov dx, 0x3F8 + 0
    mov al, dil
    out dx, al
    ret

; -------- GDT (flat data, 32-bit code, 64-bit code) --------
ALIGN 8
gdt:
    dq 0x0000000000000000
    dq 0x00CF9A000000FFFF   ; 32-bit code
    dq 0x00CF92000000FFFF   ; data
    dq 0x00AF9A000000FFFF   ; 64-bit code (L=1)

gdt_desc:
    dw gdt_end - gdt - 1
    dd gdt
gdt_end:

CODE32_SEL  equ 0x08
DATA_SEL    equ 0x10
CODE64_SEL  equ 0x18

; -------- Page tables --------
ALIGN 4096
pml4: times 512 dq 0
ALIGN 4096
pdpt: times 512 dq 0
ALIGN 4096
pd:   times 512 dq 0

