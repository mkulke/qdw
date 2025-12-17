; boot.asm — 512-byte boot sector that loads stage2 (next 16 sectors) to 0x8000
BITS 16
ORG 0x7C00

start:
    cli
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7C00

    ; Enable A20 (fast A20 gate)
    in  al, 0x92
    or  al, 0x02
    out 0x92, al

    ; Load stage2 from disk using BIOS INT 13h, AH=02h (CHS)
    ; Assumes QEMU boots from this image as drive 0x80.
    mov bx, 0x8000        ; ES:BX = 0000:8000
    mov ah, 0x02          ; read sectors
    mov al, 0x80          ; number of sectors to read
    mov ch, 0x00          ; cylinder
    mov cl, 0x02          ; sector (starts at 1), so sector 2 = immediately after boot sector
    mov dh, 0x00          ; head
    mov dl, 0x80          ; drive
    int 0x13
    jc  disk_error

    jmp 0x0000:0x8000     ; jump to stage2

disk_error:
    hlt
    jmp disk_error

times 510-($-$$) db 0
dw 0xAA55
