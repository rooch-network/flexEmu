use goblin::elf::program_header::{PF_R, PF_W, PF_X};
use unicorn_engine::unicorn_const::Permission;

/// Align a value down to the specified alignment boundary. If `value` is already
/// aligned, the same value is returned. Commonly used to determine the base address
/// of the enclosing page.
///
/// Args:
/// value: a value to align
/// alignment: alignment boundary; must be a power of 2. if not specified value
/// will be aligned to page size
///
/// Returns: value aligned down to boundary
pub fn align(value: u32, alignment: u32) -> u32 {
    debug_assert_eq!(alignment & (alignment -1), 0);
    // round down to nearest alignment
    value & (!(alignment - 1))
}

pub fn align_up(value: u32, alignment: u32) -> u32 {

    debug_assert_eq!(alignment & (alignment -1), 0);
    // round up to nearest alignment
    (value + alignment - 1) & (!(alignment - 1))
}

/// Translate ELF segment perms to Unicorn protection constants.
pub fn seg_perm_to_uc_prot(perm: u32) -> Permission {
    let mut prot = Permission::NONE;
    if perm & PF_X {
        prot |= Permission::EXEC;
    }
    if perm & PF_W {
        prot |= Permission::WRITE;
    }
    if perm & PF_R {
        prot |= Permission::READ;
    }

    prot
}



#[cfg(test)]
mod test {
    use super::{align, align_up};

    #[test]
    pub fn test_align() {
        let pagesize = 0x1000;

        {
            assert_eq!(align(0x0111, pagesize), 0x0000);
            assert_eq!(align(0x1000, pagesize), 0x1000);
            assert_eq!(align(0x1001, pagesize), 0x1000);
            assert_eq!(align(0x1111, pagesize), 0x1000);
            assert_eq!(align(0x10000, pagesize), 0x10000);
        }

        {
            assert_eq!(align_up(0x0111, pagesize), 0x1000);
            assert_eq!(align_up(0x1000, pagesize), 0x1000);
            assert_eq!(align_up(0x1001, pagesize), 0x2000);
            assert_eq!(align_up(0x1111, pagesize), 0x2000);
            assert_eq!(align_up(0x2000, pagesize), 0x2000);
            assert_eq!(align_up(0x10000, pagesize), 0x10000);
        }
    }
}
