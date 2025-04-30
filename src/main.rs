//! A fuzzer using qemu in systemmode for binary-only coverage of kernels

extern crate lazy_static;

mod breakpoints;
mod config;
mod unwinder;
use breakpoints::handle_breakpoint;
use config::{parse_config, Config, CONFIG_PATH};

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
    AsSlice,
};
use libafl_qemu::{
    edges::{edges_map_mut_slice, QemuEdgeCoverageHelper, MAX_EDGES_NUM},
    emu::Emulator,
    QemuExecutor, QemuHooks, Regs,
};
use std::{env, path::PathBuf, process::{self, exit}};

pub static mut MAX_INPUT_SIZE: usize = 50;

pub fn main() {
    env_logger::init();
    let config = parse_config(CONFIG_PATH).unwrap();
    println!("{config:?}");

    // Initialize QEMU
    let env: Vec<(String, String)> = env::vars().collect();
    let emu = Emulator::new(&config.qemu_args, &env).unwrap();

    let devices = emu.list_devices();
    println!("Devices = {devices:?}");

    let mut snap = None;
    let mut fuzz_target_found = false;
    let fuzz_target_return_address = config.fuzz_target_return_address;
    
    // boot
    unsafe {
        breakpoints::set_breakpoints(&emu, config.clone());
        if config.fuzz {
            emu.set_breakpoint(config.fuzz_target_address);
        }
        println!("Breakpoints set");

        let _ = emu.run();
        loop {
            let current_pc: u32 = emu.current_cpu().unwrap().read_reg(Regs::Pc).unwrap();
            let breakpoint_name = handle_breakpoint(&emu, config.clone()).unwrap();
            
            if config.fuzz && current_pc == config.fuzz_target_address {
                fuzz_target_found = true;
                println!("reached fuzz target during normal boot");
                emu.remove_breakpoint(config.fuzz_target_address);
                
                snap = Some(emu.create_fast_snapshot(true));
                println!("Snapshot created for the fuzz target");
                break;
            }
           
            if breakpoint_name == "app_init_done" {
                snap = Some(emu.create_fast_snapshot(true));
                break;
            }
            let _ = emu.run();
        }
    }

    // Boot execution till adventures
    if !config.fuzz {
        let current_pc: u32 = emu.current_cpu().unwrap().read_reg(Regs::Pc).unwrap();
        println!("app init done {current_pc:#x}");

        println!("lets go for adventures");
        unsafe {
            let _ = emu.run();
        }
        loop {
            let breakpoint_name = handle_breakpoint(&emu, config.clone()).unwrap();
            println!("handled breakpoint {breakpoint_name}");

            unsafe {
                let _ = emu.run();
            }
        }

        /*
        emu.current_cpu()
            .unwrap()
            .write_reg(Regs::Pc, 0xfe000000u32)
            .unwrap();

        let current_pc: u32 = emu.current_cpu().unwrap().read_reg(Regs::Pc).unwrap();
        println!("jumping to a random address {current_pc:#x}");

        emu.restore_fast_snapshot(snap.unwrap());
        let current_pc: u32 = emu.current_cpu().unwrap().read_reg(Regs::Pc).unwrap();
        println!("restored snapshot {current_pc:#x}");
        unsafe{
            emu.run();
        }
        */
    }
    // Fuzz a target function
    else {
        let timeout = Duration::from_secs(config.timeout_seconds.try_into().unwrap());
        let broker_port = config.broker_port.try_into().unwrap();
        let mut cores = Cores::all().unwrap();
        cores.trim(config.cores.try_into().unwrap()).unwrap();
        let corpus_dirs = [PathBuf::from("./corpus")];
        let objective_dir = PathBuf::from("./crashes");
        
        if fuzz_target_found == false {
            println!("Target function was not reached during normal boot. jumping there !!!");
            emu.current_cpu()
                .unwrap()
                .write_reg(Regs::Pc, config.fuzz_target_address)
                .unwrap();
            // snap = Some(emu.create_fast_snapshot(true));
            // println!("Snapshot created for the fuzz target");
        }

        let mut run_client = |state: Option<_>, mut mgr, _core_id| {
            // The wrapped fuzz target function, calling out to the LLVM-style harness
            let mut harness = |input: &BytesInput| {
                let target = input.target_bytes();
                let mut buf = target.as_slice();
                let len = buf.len();
                unsafe {
                    if len > MAX_INPUT_SIZE {
                        buf = &buf[0..MAX_INPUT_SIZE];
                    }

                    if len < 24 {
                        return ExitKind::Ok;
                    }

                    println!("Setting fuzzer inputs");
                    // this will work for target functions with max. 6 input parameters
                    let params: Vec<u32> = buf.chunks(4)
                        .take(6)
                        .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
                        .collect();
                    let [param1, param2, param3, param4, param5, param6] = params[..6].try_into().unwrap();

                    // Provide the fuzzer input to the target function
                    let cpu = emu.current_cpu().unwrap();
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

                    // Set breakpoint to the fuzz target's return address
                    emu.set_breakpoint(fuzz_target_return_address);
                    println!("Running the fuzzer on the target function");
                    emu.run().unwrap();

                    let pc2: u32 = emu
                        .current_cpu()
                        .unwrap()
                        .read_reg(Regs::Pc)
                        .expect("Failed to get pc");
                    if pc2 == fuzz_target_return_address {
                        println!("Fuzz target return");
                        // emu.restore_fast_snapshot(snap.unwrap());
                    }
                }
                ExitKind::Ok
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

            let mut hooks =
                QemuHooks::new(emu.clone(), tuple_list!(QemuEdgeCoverageHelper::default()));

            // Create a QEMU in-process executor
            let mut executor = QemuExecutor::new(
                &mut hooks,
                &mut harness,
                tuple_list!(edges_observer, time_observer),
                &mut fuzzer,
                &mut state,
                &mut mgr,
                timeout,
            )
            .expect("Failed to create QemuExecutor");

            // Instead of calling the timeout handler and restart the process, trigger a breakpoint ASAP
            executor.break_on_timeout();

            if state.must_load_initial_inputs() {
                state
                    .load_initial_inputs_forced(&mut fuzzer, &mut executor, &mut mgr, &corpus_dirs)
                    .unwrap_or_else(|_| {
                        println!("Failed to load initial corpus at {:?}", &corpus_dirs);
                        process::exit(0);
                    });
                println!("We imported {} inputs from disk.", state.corpus().count());
            }

            // Setup an havoc mutator with a mutational stage
            let mutator = StdScheduledMutator::new(havoc_mutations());
            let mut stages = tuple_list!(StdMutationalStage::new(mutator));

            fuzzer
                .fuzz_loop(&mut stages, &mut executor, &mut state, &mut mgr)
                .unwrap();
            Ok(())
        };

        // The shared memory allocator
        let shmem_provider = StdShMemProvider::new().expect("Failed to init shared memory");

        // The stats reporter for the broker
        let monitor = MultiMonitor::new(|s| println!("{s}"));

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
            Err(Error::ShuttingDown) => println!("Fuzzing stopped by user. Good bye."),
            Err(err) => panic!("Failed to run launcher: {err:?}"),
        }
    }
}
