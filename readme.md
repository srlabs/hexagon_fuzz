# Setup

## Clone and fetch the qemu submodule
```bash
git clone ssh://git@ssh.gitlab.srlabs.de:45983/research/baseband_fuzz.git
git submodule update --init
```

## Run input or fuzz

If you want to fuzz, use `cargo fuzz`.
If you want to run a specific input, remore the "fuzzing" feature and use `cargo emu <input_path>` 
