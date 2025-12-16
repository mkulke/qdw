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
    -drive format=raw,file=/tmp/os.img \
    -serial file:/home/cloud/qemu/serial.log \
    -accel mshv \
    -smp cpus=1 \
    -m 128M
```
