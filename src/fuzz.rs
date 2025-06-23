use crate::{
    config::Config,
    utils::{boot_firmware, init_qemu},
    MAX_INPUT_SIZE,
};

use core::{ptr::addr_of_mut, time::Duration};

use libafl::{
    corpus::{Corpus, InMemoryCorpus, OnDiskCorpus},
    events::{launcher::Launcher, EventConfig},
    executors::ExitKind,
    feedback_or, feedback_or_fast,
    feedbacks::{CrashFeedback, MaxMapFeedback, TimeFeedback, TimeoutFeedback},
    fuzzer::{Fuzzer, StdFuzzer},
    inputs::{BytesInput, HasTargetBytes},
    monitors::MultiMonitor,
    mutators::scheduled::{havoc_mutations, StdScheduledMutator},
    observers::{HitcountsMapObserver, TimeObserver, VariableMapObserver},
    schedulers::{IndexesLenTimeMinimizerScheduler, QueueScheduler},
    stages::StdMutationalStage,
    state::{HasCorpus, StdState},
    Error,
};
use libafl_bolts::{
    core_affinity::Cores,
    current_nanos,
    rands::StdRand,
    shmem::{ShMemProvider, StdShMemProvider},
    tuples::tuple_list,
    AsSlice, HasLen,
};
use libafl_qemu::{
    edges::{edges_map_mut_slice, QemuEdgeCoverageHelper, MAX_EDGES_NUM},
    emu::Emulator,
    FastSnapshot, QemuExecutor, QemuHooks, Regs,
};
use log::{debug, error, info};
use std::{
    path::PathBuf,
    process::{self},
};

pub fn run_fuzzer(config: Config) {
    info!("Starting fuzzer with config: timeout={}s, broker_port={}, fuzz_target=0x{:x}, return_addr=0x{:x}",
          config.timeout_seconds, config.broker_port, config.fuzz_target_address, config.fuzz_target_return_address);

    let timeout = Duration::from_secs(config.timeout_seconds.into());
    let broker_port = config.broker_port.try_into().unwrap();
    let mut cores = Cores::all().unwrap();
    cores.trim(config.cores.try_into().unwrap()).unwrap();
    debug!("Using {} CPU cores for fuzzing", cores.ids.len());

    let corpus_dirs = [PathBuf::from("./corpus")];
    let objective_dir = PathBuf::from("./crashes");
    debug!(
        "Corpus directory: {:?}, Crashes directory: {:?}",
        corpus_dirs, objective_dir
    );

    let mut run_client = |state: Option<_>, mut mgr, _core_id| {
        // Initialize QEMU
        let emu = init_qemu(&config);

        // boot
        let snap = boot_firmware(&config, &emu).unwrap_or_else(|| {
            error!("Could not snapshot firmware!");
            process::exit(1);
        });
        // The wrapped fuzz target function, calling out to the LLVM-style harness
        let mut wrapped_harness = |input: &BytesInput| {
            debug!("Harness called with input size: {} bytes", input.len());
            harness(&config, &emu, snap, input)
        };

        // Create an observation channel using the coverage map
        let edges_observer = unsafe {
            HitcountsMapObserver::new(VariableMapObserver::from_mut_slice(
                "edges",
                edges_map_mut_slice(),
                addr_of_mut!(MAX_EDGES_NUM),
            ))
        };

        // Create an observation channel to keep track of the execution time
        let time_observer = TimeObserver::new("time");

        // Feedback to rate the interestingness of an input
        // This one is composed by two Feedbacks in OR
        let mut feedback = feedback_or!(
            // New maximization map feedback linked to the edges observer and the feedback state
            MaxMapFeedback::tracking(&edges_observer, true, true),
            // Time feedback, this one does not need a feedback state
            TimeFeedback::with_observer(&time_observer)
        );

        // A feedback to choose if an input is a solution or not
        let mut objective = feedback_or_fast!(CrashFeedback::new(), TimeoutFeedback::new());

        // If not restarting, create a State from scratch
        let mut state = state.unwrap_or_else(|| {
            debug!("Creating new fuzzer state from scratch");
            StdState::new(
                // RNG
                StdRand::with_seed(current_nanos()),
                // Corpus that will be evolved, we keep it in memory for performance
                InMemoryCorpus::new(),
                // Corpus in which we store solutions (crashes in this example),
                // on disk so the user can get them after stopping the fuzzer
                OnDiskCorpus::new(objective_dir.clone()).unwrap(),
                // States of the feedbacks.
                // The feedbacks can report the data that should persist in the State.
                &mut feedback,
                // Same for objective feedbacks
                &mut objective,
            )
            .unwrap()
        });

        // A minimization+queue policy to get testcasess from the corpus
        let scheduler = IndexesLenTimeMinimizerScheduler::new(QueueScheduler::new());

        // A fuzzer with feedbacks and a corpus scheduler
        let mut fuzzer = StdFuzzer::new(scheduler, feedback, objective);

        let mut hooks = QemuHooks::new(emu.clone(), tuple_list!(QemuEdgeCoverageHelper::default()));

        // Create a QEMU in-process executor
        debug!("Creating QEMU executor with timeout: {:?}", timeout);
        let mut executor = QemuExecutor::new(
            &mut hooks,
            &mut wrapped_harness,
            tuple_list!(edges_observer, time_observer),
            &mut fuzzer,
            &mut state,
            &mut mgr,
            timeout,
        )
        .expect("Failed to create QemuExecutor");

        // Instead of calling the timeout handler and restart the process, trigger a breakpoint ASAP
        executor.break_on_timeout();
        debug!("QEMU executor created successfully with breakpoint on timeout");

        if state.must_load_initial_inputs() {
            debug!("Loading initial inputs from corpus directories");
            state
                .load_initial_inputs_forced(&mut fuzzer, &mut executor, &mut mgr, &corpus_dirs)
                .unwrap_or_else(|_| {
                    error!("Failed to load initial corpus at {:?}", &corpus_dirs);
                    process::exit(0);
                });
            info!("Imported {} inputs from disk.", state.corpus().count());
        } else {
            debug!("Skipping initial input loading - state already has inputs");
        }

        // Setup an havoc mutator with a mutational stage
        debug!("Setting up havoc mutator and mutational stage");
        let mutator = StdScheduledMutator::new(havoc_mutations());
        let mut stages = tuple_list!(StdMutationalStage::new(mutator));

        info!("Starting main fuzzing loop");
        fuzzer
            .fuzz_loop(&mut stages, &mut executor, &mut state, &mut mgr)
            .unwrap();
        Ok(())
    };

    // The shared memory allocator
    let shmem_provider = StdShMemProvider::new().expect("Failed to init shared memory");

    // The stats reporter for the broker
    let monitor = MultiMonitor::new(|s| info!("{s}"));

    // Build and run a Launcher
    let ret = Launcher::builder()
        .shmem_provider(shmem_provider)
        .broker_port(broker_port)
        .configuration(EventConfig::from_build_id())
        .monitor(monitor)
        .run_client(&mut run_client)
        .cores(&cores)
        // .stdout_file(Some("/dev/null"))
        .build()
        .launch();

    match ret {
        Ok(()) => (),
        Err(Error::ShuttingDown) => info!("Fuzzing stopped by user. Good bye."),
        Err(err) => panic!("Failed to run launcher: {err:?}"),
    }
}

fn harness(config: &Config, emu: &Emulator, snap: FastSnapshot, input: &BytesInput) -> ExitKind {
    debug!(
        "Harness: restoring snapshot and preparing input (size: {})",
        input.len()
    );
    emu.restore_fast_snapshot(snap);
    let target = input.target_bytes();
    let mut buf = target.as_slice();
    let len = buf.len();
    unsafe {
        if len > MAX_INPUT_SIZE {
            buf = &buf[0..MAX_INPUT_SIZE];
        }

        if len < 24 {
            debug!(
                "Harness: input size {} too small (< 24 bytes), skipping",
                len
            );
            return ExitKind::Ok;
        }

        debug!("Setting fuzzer inputs from {} bytes", buf.len());
        // this will work for target functions with max. 6 input parameters
        let params: Vec<u32> = buf
            .chunks(4)
            .take(6)
            .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
            .collect();
        let [param1, param2, param3, param4, param5, param6] = params[..6].try_into().unwrap();
        debug!(
            "Harness: extracted parameters: 0x{:x}, 0x{:x}, 0x{:x}, 0x{:x}, 0x{:x}, 0x{:x}",
            param1, param2, param3, param4, param5, param6
        );

        // Provide the fuzzer input to the target function
        let cpu = emu.current_cpu().unwrap();

        cpu.write_reg(Regs::Pc, config.fuzz_target_address).unwrap();
        let pc2: u32 = emu
            .current_cpu()
            .unwrap()
            .read_reg(Regs::Pc)
            .expect("Failed to get pc");
        debug!(
            "Harness: set PC to 0x{:x}, verified PC is 0x{:x}",
            config.fuzz_target_address, pc2
        );
        for (reg, param) in [
            (Regs::R0, param1),
            (Regs::R1, param2),
            (Regs::R2, param3),
            (Regs::R3, param4),
            (Regs::R4, param5),
            (Regs::R5, param6),
        ] {
            cpu.write_reg(reg, param).unwrap();
        }
        debug!("Harness: set registers R0-R5 with parameter values");

        // Set breakpoint to the fuzz target's return address
        emu.set_breakpoint(config.fuzz_target_return_address);
        debug!(
            "Harness: set breakpoint at return address 0x{:x}",
            config.fuzz_target_return_address
        );
        debug!("Running the fuzzer on the target function");
        emu.run().unwrap();

        let pc2: u32 = emu
            .current_cpu()
            .unwrap()
            .read_reg(Regs::Pc)
            .expect("Failed to get pc");
        debug!("Harness: execution stopped at PC 0x{:x}", pc2);
        if pc2 == config.fuzz_target_return_address {
            debug!("Harness: successfully reached fuzz target return address");
        } else {
            debug!(
                "Harness: execution stopped at unexpected address 0x{:x} (expected 0x{:x})",
                pc2, config.fuzz_target_return_address
            );
        }
    }
    ExitKind::Ok
}
