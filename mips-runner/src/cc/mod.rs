use crate::arch::Core;
use crate::registers::Registers;
use std::cell::RefCell;
use std::rc::Rc;
use unicorn_engine::unicorn_const::uc_error;

pub trait CallingConvention {
    /// Get the number of slots allocated for an argument of width `argbits`.
    fn get_num_slots(argbits: u32) -> u32;

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
    fn get_raw_param(&self, slot: u32, argbits: Option<u32>) -> u32;
    fn set_raw_param(&self, slot: u32, value: u32, argbits: Option<u32>);
    fn get_return_value(&self) -> u32;
    fn set_return_value(&self, val: u32);
    fn set_return_address(&self, addr: u32);
    fn reserve(&self, nslots: u32);
    ///Reserve slots for function arguments.
    //
    // 		This may be used to stage a new frame before executing a native function.
    //
    // 		Args:
    // 			nslots: number of arg slots to reserve
    fn unwind(&self, nslots: u32) -> u32;
}

pub struct CallingConventionCommon<'a> {
    retreg: i32,
    argregs: Vec<i32>,
    shadow: u32,
    retaddr_on_stack: bool,
    /// native address size in bytes
    address_size: u64,

    arch: Rc<RefCell<Core<'a>>>,
}

impl<'a> CallingConventionCommon<'a> {
    pub fn new(
        ret_reg: i32,
        arg_regs: Vec<i32>,
        shadow: u32,
        ret_addr_on_stack: bool,
        arch: Rc<RefCell<Core<'a>>>,
    ) -> Self {
        Self {
            retreg: ret_reg,
            argregs: arg_regs,
            shadow,
            retaddr_on_stack: ret_addr_on_stack,
            address_size: arch.borrow().pointersize(),
            arch,
        }
    }
    pub fn get_return_value(&self) -> Result<u64, uc_error> {
        self.arch.borrow().uc().read(self.retreg)
    }
    pub fn set_return_value(&mut self, val: u64) -> Result<(), uc_error> {
        self.arch.borrow_mut().uc_mut().write(self.retreg, val)
    }

    pub fn reserve(&mut self, nslots: usize) -> Result<(), uc_error> {
        assert!(nslots < self.argregs.len(), "too many slots");
        // count how many slots should be reserved on the stack
        let si = self.argregs[0..nslots]
            .iter()
            .filter(|s| s.is_negative())
            .count() as u32;
        let sp_change = -((self.shadow + si) as i64) * (self.address_size as i64);
        self.arch.borrow_mut().uc_mut().incr_sp(sp_change)
    }
}
