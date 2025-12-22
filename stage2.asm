BITS 16
section .text

extern kmain

global start16

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

    ; -------- Paging structures (identity map first 16 MiB using 2MiB pages) --------
    ; We place them at 0xA0000+
    mov dword [pml4+0], pdpt + 0x003         ; PML4[0] -> PDPT, P=1 RW=1
    mov dword [pml4+4], 0
    mov dword [pdpt+0], pd + 0x003           ; PDPT[0] -> PD,  P=1 RW=1
    mov dword [pdpt+4], 0

	; Map first 8 x 2MiB pages
	; Each entry: physical_addr | flags (0x83 = P=1 RW=1 PS=1)
	mov edi, pd
	mov eax, 0x00000083
	mov ecx, 8      						 ; 8 entries
.map_loop:
	mov [edi], eax
	mov dword [edi+4], 0 					 ; upper 32 bits = 0
	add eax, 0x00200000                      ; next 2MiB
	add edi, 8 								 ; next PD entry
	loop .map_loop

	; -------- Map MMIO (I/O APIC) --------

    ; PDPT entry 3 covers 0xC0000000 - 0xFFFFFFFF
    mov dword [pdpt+24], pd_high + 0x003  ; PDPT[3] -> pd_high
    mov dword [pdpt+28], 0

    ; Map page directory entry for 0xFEC00000
    ; 0xFEC00000 / 0x200000 = entry 502 in the PD
	mov dword [pd_high + 502*8], 0xFEC00083  ; Identity map I/O APIC
	mov dword [pd_high + 502*8 + 4], 0

	; -------- Transition to long mode --------

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

    call kmain

.hang:
    hlt
    jmp .hang

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
pml4:    times 512 dq 0
ALIGN 4096
pdpt:    times 512 dq 0
ALIGN 4096
pd:      times 512 dq 0
ALIGN 4096
pd_high: times 512 dq 0

