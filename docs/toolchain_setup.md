# Toolchains and Setup

## Some useful tools
### hexagon-lldb
Useful for debugging the hexagon firmware.
#### How to get
- Download Hexagon SDK from Qualcomm [software center](https://www.qualcomm.com/developer/software/list?query=hexagon+sdk). This requires a valid license.
- Extract the zip and `hexagon-lldb` is under the path `/tools/HEXAGON_Tools/8.7.03/Tools/bin/`
#### How to use
See [Using hexagon-lldb](./emulation.md#using-hexagon-lldb).

### QXDM (Qualcomm eXtensible Diagnostic Monitor)
A diagnostic tool used for debugging, monitoring, and analyzing Qualcomm-based devices
#### How to get
- Download QXDM from the Qualcomm [software center](Qualcomm® eXtensible Diagnostic Monitor 5). This requires a valid license.
#### How to use
1. Connect the Qualcomm based device to the PC (where QXDM is installed) via USB. The device needs to be in 'Diagnostig Mode' to communicate with QXDM
2. Select the communication port
3. Monitor live data or run commands
QXDM can be used to view logs, for real-time monitoring, collecting diagnostic data or sending AT commands.

## Manually fixing the qemu-libafl-bridge for hexagon code
If it is required to fix some code in [qemu-libafl-bridge](https://github.com/AFLplusplus/qemu-libafl-bridge) to get the hexagon emulation work, this needs to be done [manually](./qemu-hexagon.md) by picking the right code from [quic/qemu](https://github.com/quic/qemu). Search for the branch `hexagon_sysemu_*` and select the version that is closest to the branch in `qemu-libafl-bridge` and merge/cherry pick the fix.


## Available Qualcomm Documentation
Some useful qualcomm manuals are:
- [Hexagon QEMU user guide](qualcomm_manuals/80-N2040-52_AC_qualcomm_hexagon_QEMU_user_guide.pdf): Commands to emulate the firmware using QEMU
- [Hexagon ABI user guide](qualcomm_manuals/80-N2040-23_K_qualcomm_hexagon_ABI_user_guide.pdf) 
- [Hexagon Programmer reference manual](qualcomm_manuals/80-N2040-45_B_qualcomm_hexagon_v67_programmer_reference_manual.pdf): Reference manual for hexagon processors, contains overview of instruction set, registers, memory etc.
