TARGET = x86_64-unknown-none

.PHONY:
all: os.img

boot.bin: boot.asm
	nasm -f bin $< -o $@

stage2.o: stage2.asm
	nasm -f elf64 $< -o $@

rust.o: src/*.rs Cargo.toml Cargo.lock
	cargo +nightly rustc --release --target $(TARGET) \
		-Z build-std=core,compiler_builtins \
		-Z build-std-features=compiler-builtins-mem \
		-- -C relocation-model=static --emit=obj && \
	cp $$(ls -t target/x86_64-unknown-none/release/deps/qdw-*.o | head -1) $@

stage2.elf: stage2.o rust.o link.ld
	ld.lld -T link.ld -nostdlib -static -o $@ \
		stage2.o \
		rust.o

stage2.bin: stage2.elf
	llvm-objcopy -O binary $< $@

os.img: boot.bin stage2.bin
	dd if=/dev/zero  of=os.img bs=512 count=256
	dd if=boot.bin   of=os.img bs=512 seek=0 conv=notrunc
	dd if=stage2.bin of=os.img bs=512 seek=1 conv=notrunc

.PHONY:
clean:
	rm -rf *.bin *.elf *.o target os.img
