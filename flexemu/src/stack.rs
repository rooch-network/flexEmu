use unicorn_engine::unicorn_const::uc_error;

use crate::{
    arch::{ArchInfo, ArchT},
    engine::Engine,
    memory::Memory,
    registers::StackRegister,
    utils::align,
};

/// Stack operations
pub trait Stack {
    /// Push a value onto the stack.
    /// return the top stack address after pushing the value
    fn stack_push(&mut self, pointer: u64) -> Result<u64, uc_error>;

    /// Pop a value from stack.
    /// Returns: the value at the top.
    fn stack_pop(&mut self) -> Result<u64, uc_error>;

    /// Peek the architectural stack at a specified offset from its top, without affecting the top of the stack.
    /// Note that this operation violates the FIFO property of the stack and may be used cautiously.
    ///        Args:
    ///             offset: offset in bytes from the top of the stack, not necessarily aligned to the
    ///                     native stack item size. the offset may be either positive or negative, where
    ///                     a 0 value means retrieving the value at the top of the stack
    ///       Returns: the value at the specified address
    fn stack_read(&self, offset: i64) -> Result<u64, uc_error>;
    fn stack_write(&mut self, offset: i64, value: u64) -> Result<(), uc_error>;

    /// alignment default to pointer-size.
    fn aligned_push_bytes(
        &mut self,
        s: impl AsRef<[u8]>,
        alignment: Option<u32>,
    ) -> Result<u64, uc_error>;

    fn aligned_push_str(&mut self, s: &str) -> Result<u64, uc_error> {
        let mut b = s.as_bytes().to_vec();
        // add a 0x00 separator.
        b.push(0);
        self.aligned_push_bytes(&b, None)
    }
}

impl<'a, A: ArchT> Stack for Engine<'a, A> {
    fn stack_push(&mut self, pointer: u64) -> Result<u64, uc_error> {
        let ps = self.pointer_size();
        let new_sp = self.incr_sp(-(ps as i64))?;
        self.write_ptr(new_sp, pointer, Some(ps))?;
        Ok(new_sp)
    }

    fn stack_pop(&mut self) -> Result<u64, uc_error> {
        let ps = self.pointer_size();
        let v = self.read_ptr(self.sp()?, Some(ps))?;
        self.incr_sp(ps as i64)?;
        Ok(v)
    }

    fn stack_read(&self, offset: i64) -> Result<u64, uc_error> {
        let addr = self
            .sp()?
            .checked_add_signed(offset)
            .ok_or(uc_error::EXCEPTION)?;
        let v = self.read_ptr(addr, None)?;
        Ok(v)
    }
    fn stack_write(&mut self, offset: i64, value: u64) -> Result<(), uc_error> {
        let addr = self
            .sp()?
            .checked_add_signed(offset)
            .ok_or(uc_error::EXCEPTION)?;
        self.write_ptr(addr, value, None)?;
        Ok(())
    }

    /// alignment default to pointer-size.
    fn aligned_push_bytes(
        &mut self,
        s: impl AsRef<[u8]>,
        alignment: Option<u32>,
    ) -> Result<u64, uc_error> {
        let alignment = alignment.unwrap_or_else(|| self.pointer_size() as u32);
        let data = s.as_ref();
        let top = self.sp()?;
        // align by pointer size
        let top = align(top - data.len() as u64, alignment);
        Memory::write(self, top, data)?;
        self.set_sp(top)?;
        Ok(top)
    }
}
