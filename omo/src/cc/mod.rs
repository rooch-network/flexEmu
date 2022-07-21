use crate::{
    data::Mach,
    errors::Result,
    memory::PointerSizeT,
    registers::{Registers, StackRegister},
    stack::Stack,
};
use anyhow::anyhow;

pub trait CallingConvention {
    /// Get the number of slots allocated for an argument of width `argbits`.
    fn get_num_slots(argbits: u64) -> u64;

    // Read a value of native size from the specified argument slot.
    //
    // Note that argument slots and argument indexes are not the same. Though they often correlate
    // to each other, some implementations might use more than one slot to represent a sigle argument.
    //
    // Args:
    // slot: argument slot to access
    // argbits: argument size in bits (default: arch native size)
    //
    // Returns: raw value
    fn get_raw_param(&self, mach: &mut impl Mach, slot: u64, argbits: Option<u64>) -> Result<u64>;
    fn set_raw_param(
        &self,
        mach: &mut impl Mach,
        slot: u64,
        value: u64,
        argbits: Option<u64>,
    ) -> Result<()>;
    fn get_return_value(&self, mach: &mut impl Mach) -> Result<u64>;
    // TODO: handle negative value?
    fn set_return_value(&self, mach: &mut impl Mach, val: u64) -> Result<()>;
    fn set_return_address(&self, mach: &mut impl Mach, addr: u64) -> Result<()>;
    fn reserve(&self, mach: &mut impl Mach, nslots: u64) -> Result<()>;

    /// Reserve slots for function arguments.
    ///
    /// This may be used to stage a new frame before executing a native function.
    ///
    /// Args:
    /// nslots: number of arg slots to reserve
    fn unwind(&self, mach: &mut impl Mach, nslots: u64) -> Result<u64>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallingConventionCommon {
    retreg: i32,
    argregs: Vec<i32>,
    arg_on_stack_num: usize,
    shadow: usize,
    retaddr_on_stack: bool,
    /// native address size in bytes
    address_size: PointerSizeT,
}

impl CallingConventionCommon {
    pub fn new(
        ret_reg: i32,
        arg_regs: Vec<i32>,
        arg_on_stack: usize,
        shadow: usize,
        ret_addr_on_stack: bool,
        address_size: PointerSizeT,
    ) -> Self {
        Self {
            retreg: ret_reg,
            argregs: arg_regs,
            arg_on_stack_num: arg_on_stack,
            shadow,
            retaddr_on_stack: ret_addr_on_stack,
            address_size,
        }
    }
    pub fn get_return_value(&self, core: &impl Registers) -> Result<u64> {
        Ok(core.read(self.retreg)?)
    }
    pub fn set_return_value(&self, core: &mut impl Registers, val: u64) -> Result<()> {
        Ok(core.write(self.retreg, val)?)
    }

    pub fn reserve(&self, core: &mut impl StackRegister, nslots: usize) -> Result<()> {
        let si = self.arg_on_stack_num;
        assert!(nslots < self.argregs.len() + si, "too many slots");
        // count how many slots should be reserved on the stack

        let sp_change = ((self.shadow + si) * self.address_size as usize) as i64;
        core.incr_sp(-sp_change)?;
        Ok(())
    }

    fn get_param_access(&self, index: usize) -> std::result::Result<i32, u64> {
        if (index as usize) < self.argregs.len() {
            return Ok(self.argregs[index as usize]);
        }
        let si = index as usize - self.argregs.len();
        if si < self.arg_on_stack_num {
            return Err((self.retaddr_on_stack as usize + self.shadow + si) as u64
                * self.address_size as u64);
        }

        panic!("")
    }
    pub fn get_ram_param(
        &self,
        core: &(impl Registers + Stack),
        index: usize,
        argbits: Option<u64>,
    ) -> Result<u64> {
        if index >= self.arg_on_stack_num + self.argregs.len() {
            Err(anyhow!(
                "tried to access arg {}, but only {} args are supported",
                index,
                self.arg_on_stack_num + self.argregs.len()
            ))?;
        }

        let v = match self.get_param_access(index) {
            Ok(reg) => Registers::read(core, reg)?,
            Err(s) => Stack::stack_read(core, s as i64)?,
        };

        Ok(match argbits {
            None => v,
            Some(bits) => {
                let mask = (1 << bits) - 1;
                v & mask
            }
        })
    }

    pub fn set_raw_param(
        &self,
        core: &mut (impl Registers + Stack),
        index: usize,
        value: u64,
        argbits: Option<u64>,
    ) -> Result<()> {
        if index >= self.arg_on_stack_num + self.argregs.len() {
            Err(anyhow!(
                "tried to access arg {}, but only {} args are supported",
                index,
                self.arg_on_stack_num + self.argregs.len()
            ))?;
        }
        let v = match argbits {
            None => value,
            Some(bits) => {
                let mask = (1 << bits) - 1;
                value & mask
            }
        };
        match self.get_param_access(index) {
            Ok(reg) => {
                Registers::write(core, reg, v)?;
            }
            Err(s) => {
                Stack::stack_write(core, s as i64, v)?;
            }
        }
        Ok(())
    }
}
