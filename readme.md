# Baseband Fuzz

A fuzzing framework for Hexagon baseband firmware using QEMU system emulation. This tool enables security researchers to fuzz baseband processors by emulating firmware in a controlled environment, supporting debugging and vulnerability discovery in cellular modem implementations.

## Features

- QEMU-based emulation of baseband firmware
- LibAFL-based
- Support for Hexagon (Qualcomm DSP) architecture
- Integrated LLDB debugging capabilities with the Qualcomm SDK
- Configurable fuzzing targets and parameters
- Docker containerization for easy deployment

## Setup

### Install dependencies on Ubuntu
```bash
sudo apt install python3 python3-pip python3-sphinx python3-sphinx-rtd-theme ninja-build libglib2.0-dev flex bison clang rustup tmux gdb gdbserver socat
```

### Clone the repo and fetch the qemu submodule
```bash
git clone https://github.com/srlabs/hexagon_fuzz.git
git submodule update --init
```

### Tmux script
- Set the `SDK_HOME` env variable to the path of Hexagon SDK
- Run the `scripts/tmux_bootstrap.sh` to start the emulation and attach a LLDB for debugging

## Steps for fuzzing
- Set the `SDK_HOME` env variable to the path of Hexagon SDK
- Set `"fuzz": true` in the `firmware_config.json`
- Set the fuzz target start and return address in `firmware_config.json`
- Run the fuzzer:
```bash
cargo build --release
./target/release/hexagon_fuzz
```

After building it once, you can skip re-building/re-configuring the QEMU submodule by setting the environment variables `CUSTOM_QEMU_NO_BUILD = "1"` and `CUSTOM_QEMU_NO_CONFIGURE = "1"`.
This can also be done in .cargo/config.toml if using cargo.

## Docker setup
- Build and run the docker image
```bash
docker build -t hexagon_fuzz .
docker run -it hexagon_fuzz
```
- Build and run the fuzzer inside the docker container
```bash
cargo build --release
./target/release/hexagon_fuzz
```

## Documentation
Some documentation around reversing, setting up the tooling, emulation and more can be found in the [docs directory](./docs/index.md)
