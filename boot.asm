; boot.asm — 512-byte boot sector that loads stage2 using INT 13h LBA extensions
BITS 16
ORG 0x7C00

STAGE2_LBA equ 1
STAGE2_SECTORS equ 128

start:
    cli
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7C00

    ; Preserve boot drive from DL (BIOS passes it in DL)
    mov [boot_drive], dl

    ; Enable A20 (fast A20 gate)
    in  al, 0x92
    or  al, 0x02
    out 0x92, al

    ; Check for INT 13h extensions (EDD)
    mov dl, [boot_drive]
    mov ah, 0x41
    mov bx, 0x55AA
    int 0x13
    jc  disk_error        ; Extensions not supported
    cmp bx, 0xAA55
    jne disk_error

    ; Load stage2 using LBA mode (INT 13h, AH=42h)
    mov dl, [boot_drive]
    mov ah, 0x42
    mov si, dap
    int 0x13
    jc  disk_error

    ; Jump to stage2 with boot drive in DL
    mov dl, [boot_drive]
    jmp 0x0000:0x8000

disk_error:
    hlt
    jmp disk_error

; Disk Address Packet (DAP) for INT 13h AH=42h
align 4
dap:
    db 0x10               ; size of DAP (16 bytes)
    db 0                  ; reserved (must be 0)
    dw STAGE2_SECTORS     ; number of sectors to read
    dw 0x8000             ; offset (0x8000)
    dw 0x0000             ; segment (0x0000)
    dq STAGE2_LBA         ; starting LBA (sector 1)

boot_drive: db 0

times 510-($-$$) db 0
dw 0xAA55
