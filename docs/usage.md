# Getting started

1. Clone the [repository](../readme.md#clone_and_fetch_the_qemu_submodule).

2. Set Up the Fuzzer:
    The environment variables used by the fuzzer are set in `.cargo/config.toml`. 
    Descriptions:

    - CUSTOM_QEMU_DIR: sets the path to the `qemu-libafl-bridge` directory
    - CUSTOM_QEMU_NO_BUILD: can be set to `1` to prevent qemu from rebuilding the `qemu-libafl-bridge`. For the first run, it is recommended to comment this out.
    - CUSTOM_QEMU_NO_CONFIGURE: can be set to `1` to prevent qemu from reconfiguring. For the first run, it is recommended to comment this out.
    - KERNEL: path to the baseband firmware
    - NUM_JOBS: number of jobs the fuzzer will run
    
    The fuzzer dependencies and features are set in `Cargo.toml`. 

    Run the fuzzer with the following command:
    ```bash
    cargo fuzz
    ```

    To run the project without fuzzing and just starting the emulation, run: ```cargo emu```

3. Killing the Fuzzer:
    **We probably wouldn\'t need this later on**

    If you need to stop the fuzzer, press Ctrl-Z to suspend it, then run:
    ```bash
    kill -9 %1
    ```

4. Using tmux:
    The script [tmux_bootsrap.sh](../scripts/tmux_bootstrap.sh) creates a tmux session with 3 windows - fuzzer, qemu monitor and lldb. The setup can directly be used for fuzzing or debugging the emulation.
