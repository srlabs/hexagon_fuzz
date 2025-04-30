#!/bin/bash

# Check if the SDK_HOME environment variable is set
if [ -z "$SDK_HOME" ]; then
  echo "Error: SDK_HOME is not set."
  echo "Please set the SDK_HOME environment variable to the path of your SDK installation."
  exit 1
fi

# If SDK_HOME is set, continue with your script
echo "SDK_HOME is set to: $SDK_HOME"

# create tmux split windows with following ids:
# 	1
# 0
#	2
tmux new -s baseband -d -e "SDK_HOME=$SDK_HOME"
tmux setenv SDK_HOME $SDK_HOME
tmux split-window -h -t baseband
tmux split-window -v -t baseband


# Fuzzer
tmux send-keys -t baseband:0.1 './target/release/baseband_fuzz' C-m
sleep 1

# Setup LLDB
tmux send-keys -t baseband:0.0 '$SDK_HOME/bin/hexagon-lldb' C-m
tmux send-keys -t baseband:0.0 'command script import /data/lucag/qemu-libafl-bridge/HexQEMU.py' C-m
tmux send-keys -t baseband:0.0 'target create qdsp6sw.mbn' C-m
tmux send-keys -t baseband:0.0 'gdb-remote localhost:1234' C-m

# Attach to QEMU monitor
tmux send-keys -t baseband:0.2 'socat -,echo=0,icanon=0 unix-connect:qemu-monitor-socket' C-m

# Attach to our tmux
tmux attach -t baseband
