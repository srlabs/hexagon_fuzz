# Building qemu-libafl-bridge for Hexagon

This guide provides instructions for building `qemu-libafl-bridge` to support Hexagon. 

## Steps
- Obtain [`quic/qemu`](https://github.com/quic/qemu) and checkout the commit where VERSION file is at 8.2.0
- Obtain [`qemu/qemu`](https://github.com/qemu/qemu) and checkout with VERSION file at 8.2.0
- Create a diff of both qemu and create a git patch file
- Apply the patch file to [`qemu-libafl-bridge`](https://github.com/AFLplusplus/qemu-libafl-bridge) v8.2.50

## Deugging steps
- Run a check on the patch before applying to `qemu-libafl-bridge`
```bash
cd qemu-libafl-bridge
git apply diff.patch --verbose --whitespace=fix --reject --check
```
- Apply the patch
```bash
git apply diff.patch --verbose --whitespace=fix --reject
```
- Fix the *.rej files manually and commit the code
- Compile the project
```bash
cd qemu-libafl-bridge
mkdir build
cd build
../configure --target-list=hexagon-softmmu --disable-dbus-display
make -j32
```


