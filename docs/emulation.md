# Emulation and Debugging

This guide provides instructions for building and running `qemu-system-hexagon` to emulate Qualcomm Hexagon DSP firmware. It also covers debugging techniques using `hexagon-lldb`, including system register analysis, interrupt handling, and attaching LLDB to a running QEMU instance. 

## Building the qemu-system-hexagon
Build the qemu-libafl-bridge:
```bash
cd qemu-libafl-bridge
mkdir build
cd build
../configure --target-list=hexagon-softmmu --disable-dbus-display
make -j32
```
This will create the `qemu-system-hexagon` binary under `build`

A more specific configure command can also be used:
```bash
../configure --cxx=linker_interceptor++.py --cc=linker_interceptor.py --as-shared-lib --target-list=hexagon-softmmu --disable-slirp --enable-fdt=internal --audio-drv-list= --disable-alsa --disable-attr --disable-auth-pam --disable-dbus-display --disable-bochs --disable-bpf --disable-brlapi --disable-bsd-user --disable-bzip2 --disable-capstone --disable-cap-ng --disable-canokey --disable-cloop --disable-cocoa --disable-coreaudio --disable-curl --disable-curses --disable-dmg --disable-docs --disable-dsound --disable-fuse --disable-fuse-lseek --disable-gcrypt --disable-gettext --disable-gio --disable-glusterfs --disable-gnutls --disable-gtk --disable-guest-agent --disable-guest-agent-msi --disable-hvf --disable-iconv --disable-jack --disable-keyring --disable-kvm --disable-libdaxctl --disable-libiscsi --disable-libnfs --disable-libpmem --disable-libssh --disable-libudev --disable-libusb --disable-linux-aio --disable-linux-io-uring --disable-linux-user --disable-live-block-migration --disable-lzfse --disable-lzo --disable-l2tpv3 --disable-malloc-trim --disable-mpath --disable-multiprocess --disable-netmap --disable-nettle --disable-numa --disable-nvmm --disable-opengl --disable-oss --disable-pa --disable-parallels --disable-png --disable-pvrdma --disable-qcow1 --disable-qed --disable-qga-vss --disable-rbd --disable-rdma --disable-replication --disable-sdl --disable-sdl-image --disable-seccomp --disable-selinux --disable-slirp-smbd --disable-smartcard --disable-snappy --disable-sndio --disable-sparse --disable-spice --disable-spice-protocol --disable-tools --disable-tpm --disable-usb-redir --disable-user --disable-u2f --disable-vde --disable-vdi --disable-vduse-blk-export --disable-vhost-crypto --disable-vhost-kernel --disable-vhost-net --disable-vhost-user-blk-server --disable-vhost-vdpa --disable-virglrenderer --disable-virtfs --disable-vmnet --disable-vnc --disable-vnc-jpeg --disable-vnc-sasl --disable-vte --disable-vvfat --disable-whpx --disable-xen --disable-xen-pci-passthrough --disable-xkbcommon --disable-zstd
```

## Debugging with qemu-system-hexagon
Command to emulate the firmware:
```bash
./qemu-system-hexagon -kernel qdsp6sw.mbn  -monitor stdio -s
```

Some useful parameters are:
- `--kernel`: specify the firmware file to emulate
- `--monitor stdio`: specifies QEMU to use the standard input/output (stdio) for the monitor interface
- `-s`: short for `-gdb tcp::1234`, tells QEMU to start server on TCP port 1234
- `-cpu <model>`: to specify the CPU model to emulate
- `-machine`: defines the type of machine to emulate, could be a specific platform or board
- `-nographic`: disables the graphic display
- `-snapshot`: to ensure that any changes made to the firmware during the emulation session are not permanent. QEMU creates a temporary snapshot of the disk image and the changes are discarded once the emulation is stopped.
- `-snapshot -S`: auses QEMU to pause the execution as soon as it starts, allowing you to attach a debugger and inspect the system right from the start, before any code is executed.

## Using Hexagon LLDB
`hexagon-lldb` is a debugger specifically for Qualcomm Hexagon DSP architecture and can be used for debugging the baseband firmware. 
```bash
    ./hexagon-lldb
    (lldb) command script import qemu-libafl-bridge/HexQEMU.py
    (lldb) target create qdsp6sw.mbn
```
The script `HexQEMU.py` can be imported in to `hexagon-lldb` to support analyzing system register, interrupts, stepping in etc.

To attach the LLDB with qemu monitor:
```bash
(lldb) gdb-remote localhost:1234
```


