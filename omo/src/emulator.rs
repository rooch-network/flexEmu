//use crate::arch::Core;

use crate::{
    arch::{ArchInfo, ArchT},
    config::OmoConfig,
    engine::{Engine, Machine, MemoryState},
    errors::EmulatorError,
    loader::{ElfLoader, LoadInfo},
    os::Runner,
    registers::{RegisterState, Registers},
};
use log::{info, trace};
use num_traits::Zero;
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    ops::Deref,
    rc::Rc,
};
use unicorn_engine::unicorn_const::{HookType, MemType, Mode};

pub struct Emulator<'a, A, Os> {
    config: OmoConfig,
    core: Engine<'a, A>,
    os: Os,
}

impl<'a, A, O> Emulator<'a, A, O> {
    pub fn engine(&self) -> &Engine<'a, A> {
        &self.core
    }
    pub fn runner(&self) -> &O {
        &self.os
    }
}

impl<'a, A: ArchT, O: Runner> Emulator<'a, A, O> {
    pub fn new(conf: OmoConfig, arch: A, mode: Mode, os: O) -> Result<Self, EmulatorError> {
        let mut machine = Machine::create(arch, mode);
        // let binary = binary.as_ref();
        // let load_result = ElfLoader::load(&config.os, binary, argv, &mut machine)?;
        // os.on_load(&mut machine, load_result.clone())?;
        machine.add_mem_hook(
            HookType::MEM_WRITE | HookType::MEM_READ_AFTER,
            0,
            //align_up((conf.os.stack_address + conf.os.stack_size) as u32, 32) as u64,
            u32::MAX as u64,
            {
                |uc, mem_type, addr, size, value| {
                    trace!("{:?} -> ({},{}), v: {}", mem_type, addr, size, value);
                    match mem_type {
                        MemType::WRITE => {
                            debug_assert_eq!(
                                uc.mem_read_as_vec(addr, size).unwrap(),
                                uc.get_data().state.memory.read_bytes(addr, size)
                            );
                            uc.get_data_mut()
                                .state
                                .memory
                                .write_value(addr, size, value);
                        }
                        MemType::READ_AFTER => {
                            debug_assert_eq!(
                                &(value as u32).to_be_bytes().as_slice()[(4 - size)..],
                                uc.get_data().state.memory.read_bytes(addr, size)
                            );
                        }
                        _ => {}
                    }
                    true
                }
            },
        )?;

        // machine.add_code_hook(0, u32::MAX as u64, {
        //     |uc, addr, size| {
        //         info!("code hook, {} {}, pc {}", addr, size, uc.pc_read().unwrap());
        //         uc.get_data_mut().state.steps += 1;
        //     }
        // })?;

        // machine.add_block_hook(|uc, addr, size| {
        //     info!("block hook, {} {}", addr, size);
        // })?;

        Ok(Self {
            config: conf,
            core: machine,
            os,
        })
    }

    pub fn load(
        &mut self,
        binary: impl AsRef<[u8]>,
        argv: Vec<String>,
        env: Vec<(String, String)>,
    ) -> Result<LoadInfo, EmulatorError> {
        let binary = binary.as_ref();
        let load_result = ElfLoader::load(
            &self.config.os,
            binary,
            argv,
            env.into_iter().collect::<BTreeMap<_, _>>(),
            &mut self.core,
        )?;
        self.os.on_load(&mut self.core, load_result)?;
        Ok(load_result)
    }

    pub fn run(
        &mut self,
        entrypoint: u64,
        exitpoint: Option<u64>,
        timeout: Option<u64>,
        count: Option<usize>,
    ) -> Result<(), EmulatorError> {
        let exitpoint = exitpoint.unwrap_or_else(|| default_exitpoint(self.core.pointer_size()));
        self.core.emu_start(
            entrypoint,
            exitpoint,
            timeout.unwrap_or_default(),
            count.unwrap_or_default(),
        )?;
        Ok(())
    }

    pub fn run_until(
        &mut self,
        entrypoint: u64,
        exitpoint: Option<u64>,
        timeout: Option<u64>,
        count: usize,
    ) -> Result<StateChange, EmulatorError> {
        let exitpoint = exitpoint.unwrap_or_else(|| default_exitpoint(self.core.pointer_size()));

        info!("pc: {}", self.core.pc()?);

        let state_before = if count.is_zero() {
            self.save()?
        } else {
            self.core
                .emu_start(entrypoint, exitpoint, timeout.unwrap_or_default(), count)?;
            self.save()?
        };

        let mem_access_sequence = Rc::new(RefCell::new(vec![]));
        let handle = self.core.add_mem_hook(
            HookType::MEM_READ_AFTER | HookType::MEM_WRITE,
            0,
            u32::MAX as u64,
            {
                let mem_access = mem_access_sequence.clone();
                move |_uc, mem_type, addr, size, value| {
                    match mem_type {
                        MemType::WRITE => {
                            mem_access.borrow_mut().push(MemAccess {
                                write: true,
                                addr,
                                size,
                                value,
                            });
                        }
                        MemType::READ_AFTER => {
                            mem_access.borrow_mut().push(MemAccess {
                                write: false,
                                addr,
                                size,
                                value,
                            });
                        }
                        _ => {}
                    }
                    true
                }
            },
        )?;
        self.core
            .emu_start(self.core.pc()?, exitpoint, timeout.unwrap_or_default(), 1)?;
        self.core.remove_hook(handle)?;
        let state_after = self.save()?;
        Ok(StateChange {
            state_after,
            state_before,
            step: (count + 1) as u64,
            access: {
                let x = mem_access_sequence.borrow();
                x.to_vec()
            },
        })
    }

    pub fn save(&self) -> Result<EmulatorState, EmulatorError> {
        let register_vals = self.core.save_registers()?;
        let memory = self.core.get_data().state.snapshot().memory;
        Ok(EmulatorState {
            regs: register_vals,
            memories: memory,
        })
    }
}

pub fn default_exitpoint(point_size: u8) -> u64 {
    match point_size {
        2 => 0xfffff, // 20bit address lane
        4 => 0x8fffffff,
        8 => 0xffffffffffffffff,
        _ => unreachable!(),
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EmulatorState {
    pub regs: RegisterState,
    pub memories: MemoryState,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StateChange {
    pub state_before: EmulatorState,
    pub state_after: EmulatorState,
    pub step: u64,
    pub access: Vec<MemAccess>,
}
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct MemAccess {
    /// read or write
    pub write: bool,
    pub addr: u64,
    pub size: usize,
    pub value: i64,
}
