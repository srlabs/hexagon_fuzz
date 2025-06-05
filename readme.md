# Setup

## Clone and fetch the qemu submodule
```bash
git clone ssh://git@ssh.gitlab.srlabs.de:45983/research/baseband_fuzz.git
git submodule update --init
```

## Run input or fuzz

If you want to fuzz, use `cargo fuzz`.
If you want to run a specific input, remore the "fuzzing" feature and use `cargo emu <input_path>` 


## Docker setup
- Build and run the docker image
```bash
docker build -t baseband_fuzz .
docker run -it baseband_fuzz
```
- Build and run the fuzzer
```bash
cargo build --release
./target/release/baseband_fuzz
```

