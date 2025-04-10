# Baseband Fuzzing Project

Welcome to the **Baseband Fuzzing Project**! This project focuses on fuzzing baseband firmware, specifically targeting Qualcomm Hexagon processors, to uncover vulnerabilities and improve the security of mobile devices. Below, you'll find a guide to get started, set up your environment, and dive into the exciting world of baseband fuzzing.


## Quick Start
To directly kickstart with fuzzing, follow the steps in [readme.md](../readme.md)

## Getting the baseband firmware
The iPhone firmware can be downloaded freely, which contains the Qualcomm Hexagon baseband firmware. The [iOS Firmware and Extraction](ios_firmware_extraction.md) guide follows through the steps for downloading and extracting the baseband firmware.

## Getting the tools required for the project
The Qualcomm tools such as QXDM and HEXAGON specific tools are helpful in debugging the firmware, capturing logs. The guide
[Toolchains and Setup](toolchain_setup.md) goes through these steps along with some helpful manuals from Qualcomm.

## Getting started with fuzzing
To begin your journey into using the project, follow the steps in [Getting Started](./usage.md). The guide helps to setup the fuzzer and using tmux session.

## Reversing the baseband firmware
Ghidra is a freely available tool for reverse engineering and it also supports a plugin for hexagon firmware. Reversing can be helpful to identify functions that can be targeted for fuzzing and understanding the overall code flow. The guide [Reverse Engineering using Ghidra](reverse_engineering.md) explains the steps for setting up Ghidra.

## Emulating the baseband firmware
Another approach to communicate with the baseband firmware is to emulate using QEMU and debug the flow. The guide
[Emulation and Debugging](emulation.md)	walks through the steps required to setup the qemu-hexagon and running the emulation.