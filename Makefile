.PHONY:
all: os.img

boot.bin: boot.asm
	nasm -f bin boot.asm -o boot.bin

stage2.bin: stage2.asm
	nasm -f bin stage2.asm -o stage2.bin

os.img: boot.bin stage2.bin
	dd if=/dev/zero  of=os.img bs=512 count=64
	dd if=boot.bin   of=os.img bs=512 seek=0 conv=notrunc
	dd if=stage2.bin of=os.img bs=512 seek=1 conv=notrunc

.PHONY:
clean:
	rm -rf *.bin os.img
