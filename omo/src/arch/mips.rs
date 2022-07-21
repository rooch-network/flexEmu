use crate::{
    arch::ArchT,
    cc::{CallingConvention, CallingConventionCommon},
    engine::Mach,
    errors::EmulatorError,
    memory::PointerSizeT,
    registers::Registers,
};
use goblin::container::Endian;
use unicorn_engine::{
    unicorn_const::{Arch, Mode},
    RegisterMIPS,
};

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
    cc: MipsCC,
}
impl ArchT for MIPS {
    type CC = MipsCC;
    const T: Arch = Arch::MIPS;
    const PC: i32 = RegisterMIPS::PC as i32;
    const SP: i32 = RegisterMIPS::SP as i32;

    fn cc(&self) -> Self::CC {
        self.cc.clone()
    }
}

impl MIPS {
    pub fn new(pointer_size: PointerSizeT) -> Self {
        Self {
            cc: MipsCC {
                inner: CallingConventionCommon::new(
                    MipsCC::RET_REG,
                    MipsCC::ARG_REGS.to_vec(),
                    MipsCC::ARG_ON_STACK,
                    MipsCC::SHADOW,
                    MipsCC::RET_ADDR_ON_STACK,
                    pointer_size,
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

impl CallingConvention for MipsCC {
    #[inline]
    fn get_num_slots(_argbits: u64) -> u64 {
        1
    }

    fn get_raw_param(
        &self,
        mach: &mut impl Mach,
        slot: u64,
        argbits: Option<u64>,
    ) -> crate::errors::Result<u64> {
        self.inner.get_ram_param(mach, slot as usize, argbits)
    }

    fn set_raw_param(
        &self,
        mach: &mut impl Mach,
        slot: u64,
        value: u64,
        argbits: Option<u64>,
    ) -> crate::errors::Result<()> {
        self.inner
            .set_raw_param(mach, slot as usize, value, argbits)
    }

    fn get_return_value(&self, mach: &mut impl Mach) -> crate::errors::Result<u64> {
        self.inner.get_return_value(mach)
    }

    fn set_return_value(&self, mach: &mut impl Mach, val: u64) -> crate::errors::Result<()> {
        self.inner.set_return_value(mach, val)?;
        Registers::write(mach, RegisterMIPS::A3, 0)?;
        Ok(())
    }

    fn set_return_address(&self, _mach: &mut impl Mach, _addr: u64) -> crate::errors::Result<()> {
        unreachable!()
    }

    fn reserve(&self, mach: &mut impl Mach, nslots: u64) -> crate::errors::Result<()> {
        self.inner.reserve(mach, nslots as usize)
    }

    fn unwind(&self, mach: &mut impl Mach, _nslots: u64) -> Result<u64, EmulatorError> {
        // TODO: stack frame unwinding?
        Ok(Registers::read(mach, RegisterMIPS::RA)?)
    }
}
