#!/usr/bin/env bash
export KERNEL="./qdsp6sw.mbn" 
./target/debug/baseband_fuzz -monitor unix:qemu-monitor-socket,server,nowait -kernel ./qdsp6sw.mbn -serial null -nographic -snapshot -S -s
