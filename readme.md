# Setup

## Install dependencies on Ubuntu
```bash
sudo apt install python3 python3-pip python3-sphinx python3-sphinx-rtd-theme ninja-build libglib2.0-dev flex bison clang rustup
```

## Clone the repo and fetch the qemu submodule
```bash
git clone https://github.com/srlabs/baseband_fuzz.git
git submodule update --init
```

## Tmux script
- Set the `SDK_HOME` env variable to the path of Hexagon SDK
- Run the `scripts/tmux_bootstrap.sh` to start the emulation and attach a LLDB for debugging

## Steps for fuzzing 
- Set the `SDK_HOME` env variable to the path of Hexagon SDK
- Set `"fuzz": true` in the `firmware_config.json`
- Set the fuzz target start and return address in `firmware_config.json`
- Run the fuzzer:
```bash
cargo build --release
./target/release/baseband_fuzz
```

## Docker setup
- Build and run the docker image
```bash
docker build -t baseband_fuzz .
docker run -it baseband_fuzz
```
- Build and run the fuzzer inside the docker container
```bash
cargo build --release
./target/release/baseband_fuzz
```

