# QEMU debug workload

## Build

```bash
make
```

## Run

```bash
./qemu-system-x86_64 \
    -cpu qemu64 \
    -nographic \
    -no-reboot \
    -drive format=raw,file=/tmp/qdw.img \
    -accel mshv \
    -smp cpus=1 \
    -m 128M \
    -device isa-debug-exit,iobase=0xf4,iosize=0x04
```
