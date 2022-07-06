use crate::cc::{CallingConvention, CallingConventionCommon};
use crate::core::Core;
use crate::errors::EmulatorError;
use crate::memory::PointerSizeT;
use goblin::container::Endian;
use unicorn_engine::unicorn_const::{Arch, Mode};
use unicorn_engine::RegisterMIPS;

pub trait ArchT {
    fn endian(&self) -> Endian;
    fn pointer_size(&self) -> PointerSizeT;
    fn pc_reg_id(&self) -> i32;
    fn sp_reg_id(&self) -> i32;
    fn arch(&self) -> Arch;
    fn mode(&self) -> Mode;
}

#[derive(Copy, Eq, PartialEq, Debug, Clone)]
pub struct ArchInfo {
    pub endian: Endian,
    pub pointer_size: u8,
    pub mode: Mode,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct MipsProfile {
    mode32: bool,
    endian: Endian,
}
impl Default for MipsProfile {
    fn default() -> Self {
        Self {
            mode32: true,
            endian: Endian::Big,
        }
    }
}

impl MipsProfile {
    pub fn mode(&self) -> Mode {
        let mut mode = if self.mode32 {
            Mode::MODE_32
        } else {
            Mode::MODE_64
        };
        match self.endian {
            Endian::Little => {
                mode |= Mode::LITTLE_ENDIAN;
            }
            Endian::Big => {
                mode |= Mode::BIG_ENDIAN;
            }
        };
        mode
    }
    pub fn pointer_size(&self) -> u8 {
        if self.mode32 {
            4
        } else {
            8
        }
    }
}

#[derive(Debug)]
pub struct MIPS {
    pub(crate) arch_info: MipsProfile,
    pub(crate) cc: MipsCC,
}
impl ArchT for MIPS {
    fn endian(&self) -> Endian {
        self.arch_info.endian
    }
    fn pointer_size(&self) -> PointerSizeT {
        self.arch_info.pointer_size()
    }

    fn pc_reg_id(&self) -> i32 {
        RegisterMIPS::PC as i32
    }

    fn sp_reg_id(&self) -> i32 {
        RegisterMIPS::SP as i32
    }

    fn arch(&self) -> Arch {
        Arch::MIPS
    }

    fn mode(&self) -> Mode {
        self.arch_info.mode()
    }
}

impl MIPS {
    pub fn new(arch: MipsProfile) -> Self {
        Self {
            arch_info: arch,
            cc: MipsCC {
                inner: CallingConventionCommon::new(
                    MipsCC::RET_REG,
                    MipsCC::ARG_REGS.to_vec(),
                    MipsCC::ARG_ON_STACK,
                    MipsCC::SHADOW,
                    MipsCC::RET_ADDR_ON_STACK,
                    arch.pointer_size(),
                ),
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct MipsCC {
    inner: CallingConventionCommon,
}

impl MipsCC {
    const RET_REG: i32 = RegisterMIPS::V0 as i32;
    const ARG_REGS: [i32; 4] = [
        RegisterMIPS::A0 as i32,
        RegisterMIPS::A1 as i32,
        RegisterMIPS::A2 as i32,
        RegisterMIPS::A3 as i32,
    ];
    const ARG_ON_STACK: usize = 12;
    const SHADOW: usize = 4;
    const RET_ADDR_ON_STACK: bool = false;
}

impl<'a, A: ArchT> ArchT for Core<'a, A> {
    fn endian(&self) -> Endian {
        self.get_data().arch_info.endian()
    }

    fn pointer_size(&self) -> PointerSizeT {
        self.get_data().arch_info.pointer_size()
    }

    fn pc_reg_id(&self) -> i32 {
        self.get_data().arch_info.pc_reg_id()
    }

    fn sp_reg_id(&self) -> i32 {
        self.get_data().arch_info.sp_reg_id()
    }

    fn arch(&self) -> Arch {
        self.get_data().arch_info.arch()
    }

    fn mode(&self) -> Mode {
        self.get_data().arch_info.mode()
    }
}

impl<'a> CallingConvention for Core<'a, MIPS> {
    #[inline]
    fn get_num_slots(_argbits: u64) -> u64 {
        1
    }

    fn get_raw_param(&self, slot: u64, argbits: Option<u64>) -> crate::errors::Result<u64> {
        let inner = self.get_data().arch_info.cc.inner.clone();
        inner.get_ram_param(self, slot as usize, argbits)
    }

    fn set_raw_param(
        &mut self,
        slot: u64,
        value: u64,
        argbits: Option<u64>,
    ) -> crate::errors::Result<()> {
        let inner = self.get_data().arch_info.cc.inner.clone();
        inner.set_raw_param(self, slot as usize, value, argbits)
    }

    fn get_return_value(&self) -> crate::errors::Result<u64> {
        let inner = self.get_data().arch_info.cc.inner.clone();
        inner.get_return_value(self)
    }

    fn set_return_value(&mut self, val: u64) -> crate::errors::Result<()> {
        let inner = self.get_data().arch_info.cc.inner.clone();
        inner.set_return_value(self, val)
    }

    fn set_return_address(&mut self, _addr: u64) -> crate::errors::Result<()> {
        unreachable!()
    }

    fn reserve(&mut self, nslots: u64) -> crate::errors::Result<()> {
        let inner = self.get_data().arch_info.cc.inner.clone();
        inner.reserve(self, nslots as usize)
    }

    fn unwind(&mut self, _nslots: u64) -> Result<u64, EmulatorError> {
        // TODO: stack frame unwinding?
        Ok(self.reg_read(RegisterMIPS::RA)?)
    }
}
