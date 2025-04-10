#!/bin/bash

# create tmux split windows with following ids:
# 	1
# 0
#	2
tmux new -s baseband -d
tmux split-window -h -t baseband
tmux split-window -v -t baseband

# Setup LLDB
tmux send-keys -t baseband:0.0 '/data/lucag/Qualcomm/HEXAGON_Tools/8.7.06/Tools/bin/hexagon-lldb' C-m
tmux send-keys -t baseband:0.0 'command script import /data/lucag/qemu-libafl-bridge/HexQEMU.py' C-m
tmux send-keys -t baseband:0.0 'target create /data/lucag/baseband_fuzz/qdsp6sw.mbn' C-m
tmux send-keys -t baseband:0.0 'gdb-remote localhost:1234'

# Fuzzer
tmux send-keys -t baseband:0.1 'cargo fuzzd'

# Attach to QEMU monitor
tmux send-keys -t baseband:0.2 'socat -,echo=0,icanon=0 unix-connect:qemu-monitor-socket' 

# Attach to our tmux
tmux attach -t baseband