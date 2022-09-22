/// Mips Reference impl. https://inst.eecs.berkeley.edu/~cs61c/resources/MIPS_help.html
module omo::mips {
    use Std::Vector::{empty};
    use std::bits;
    use std::bits::{Bits, se, data, left_shift, right_shift, slice, len};
    use std::i64;
    use trie::hash_value::HashValue;
    use trie::hash_value;

    public fun read_memory(state_hash: HashValue, addr: u64): u64 {
        0
    }

    public fun read_reg(state_hash: HashValue, reg_id: u64): u64 {
        0
    }

    public fun read_reg_bits(state_hash: HashValue, reg_id: u64): Bits {
        bits::from_u64(read_reg(state_hash, reg_id), 32)
    }

    public fun read_memory_bits(state_hash: HashValue, addr: u64): Bits {
        bits::from_u64(read_memory(state_hash, addr), 32)
    }

    public fun write_reg(state_hash: HashValue, reg_id: u64, v: u64): HashValue {
        hash_value::new(empty<u8>())
    }

    public fun write_reg_bits(state_hash: HashValue, reg_id: u64, bits: Bits): HashValue {
        hash_value::new(empty<u8>())
    }

    public fun write_memory(state_hash: HashValue, addr: u64, value: u64): HashValue {
        hash_value::new(empty<u8>())
    }

    const REG_OFFSET: u64 = 0xc0000000;
    const REG_ZERO: u64 = 0;
    const REG_LR: u64 = 0x1f;
    const REG_PC: u64 = 0x20;
    const REG_HI: u64 = 0x21;
    const REG_LO: u64 = 0x22;
    const REG_HEAP: u64 = 0x23;

    const EXIT_ADDRESS: u64 = 0xffffffff;

    fun reg_to_mem_addr(reg_id: u64): u64 {
        REG_OFFSET + reg_id * 4
    }

    public fun step(state_hash: HashValue): HashValue {
        let pc = read_memory(state_hash, reg_to_mem_addr(REG_PC));
        if (pc == EXIT_ADDRESS) {
            return state_hash
        };
        let new_state = step_pc(state_hash, pc, pc + 4);
        new_state
    }

    fun step_pc(state_hash: HashValue, pc: u64, next_pc: u64): HashValue {
        pc = pc & 0xffffffff;
        next_pc = next_pc & 0xffffffff;

        let insn = read_memory(state_hash, pc);
        let insn = bits::from_u64(insn, 32);
        let opcode = bits::slice(insn, 31, 26); // first 6-bits
        let opcode = bits::data(&opcode);

        // r-type
        if (opcode == 0) {
            let funct = bits::data(&bits::slice(insn, 5, 0));
            let shamt = (bits::data(&bits::slice(insn, 10, 6)) as u8);
            let rd = bits::data(&bits::slice(insn, 15, 11));
            let rt = bits::data(&bits::slice(insn, 20, 16));
            let rs = bits::data(&bits::slice(insn, 25, 21));
            let rt_value = read_reg_bits(state_hash, rt);
            let rs_value = read_reg_bits(state_hash, rs);

            state_hash = if (funct == 0) {
                //sll
                let value = read_reg(state_hash, rt) << shamt;
                write_reg(state_hash, rd, value)
            } else if (funct == 2) {
                //srl
                let value = read_reg(state_hash, rt) >> shamt;
                write_reg(state_hash, rd, value)
            } else if (funct == 3) {
                //sra
                let value = bits::se(bits::from_u64(read_reg(state_hash, rt) >> shamt, 32 - shamt), 32);

                write_reg(state_hash, rd, bits::data(&value))
            } else if (funct == 4) {
                // sllv
                let rt_value = read_reg(state_hash, rt);
                let rs_value = bits::slice(read_reg_bits(state_hash, rs), 4, 0); // lower 5-bits
                write_reg(state_hash, rd, rt_value << (data(&rs_value) as u8))
            } else if (funct == 6) {
                // srlv
                let rt_value = read_reg(state_hash, rt);
                let rs_value = bits::slice(read_reg_bits(state_hash, rs), 4, 0); // lower 5-bits
                write_reg(state_hash, rd, rt_value >> (data(&rs_value) as u8))
            } else if (funct == 7) {
                // srav
                let rt_value = read_reg(state_hash, rt);
                let rs_value = (bits::data(&bits::slice(read_reg_bits(state_hash, rs), 4, 0)) as u8); // lower 5-bits
                write_reg(state_hash, rd, bits::data(&se(bits::from_u64(rt_value >> rs_value, 32 - rs_value), 32)))
            } else if (funct == 8) {
                // jr
                let rs_value = read_reg_bits(state_hash, rs);
                step_pc(state_hash, next_pc, bits::data(&rs_value))
            } else if (funct == 9) {
                // jalr
                let rs_value = read_reg_bits(state_hash, rs);
                let state_hash = step_pc(state_hash, next_pc, bits::data(&rs_value));
                write_reg(state_hash, REG_LR, pc + 8)
            } else if (funct == 12) {
                // syscall
                // TODO: handle syscall
                hash_value::new(empty<u8>())
            } else if (funct == 16) {
                // mfhi
                let val = read_reg_bits(state_hash, REG_HI);
                write_reg(state_hash, rd, bits::data(&val))
            } else if (funct == 17) {
                // mthi
                let val = data(&rs_value);
                write_reg(state_hash, REG_HI, val)
            } else if (funct == 18) {
                // mflo
                let val = read_reg_bits(state_hash, REG_LO);
                write_reg(state_hash, rd, bits::data(&val))
            } else if (funct == 19) {
                // mtlo
                let val = data(&rs_value);
                write_reg(state_hash, REG_LO, val)
            } else if (funct == 24) {
                // mult
                let val = i64::mul(
                    i64::from_bits(rs_value),
                    i64::from_bits(rt_value),
                );
                let val = bits::data(&i64::to_bits(val));
                let hi = val >> 32;
                let lo = val & 0xffffffff;
                write_reg(write_reg(state_hash, REG_HI, hi), REG_LO, lo)
            } else if (funct == 25) {
                // multu
                let v = data(&rs_value) * data(&rt_value);
                let hi = v >> 32;
                let lo = (v << 32) >> 32;
                write_reg(write_reg(state_hash, REG_HI, hi), REG_LO, lo)
            } else if (funct == 26) {
                // div
                let rs_value = i64::from_bits(rs_value);
                let rt_value = i64::from_bits(rt_value);
                let lo = bits::slice(i64::to_bits(i64::div(rs_value, rt_value)), 31, 0);
                let hi = bits::slice(i64::to_bits(i64::rem(rs_value, rt_value)), 31, 0);
                write_reg_bits(write_reg_bits(state_hash, REG_HI, hi), REG_LO, lo)
            } else if (funct == 27) {
                // divu
                let rs_value = data(&rs_value);
                let rt_value = data(&rt_value);
                let hi = rs_value % rt_value;
                let lo = rs_value / rt_value;
                write_reg(write_reg(state_hash, REG_HI, hi), REG_LO, lo)
            } else if (funct == 32) { // add
                addi(state_hash, rd, rs_value, rt_value)
            }else if (funct == 33) {
                //addu
                addu(state_hash, rd, rs_value, rt_value)
            }else if (funct == 34) {
                //sub
                let temp = i64::sub(i64::from_bits(rs_value), i64::from_bits(rt_value));
                let temp = i64::to_bits(temp);
                let bit_32 = bits::bit(&temp, 32);
                let bit_31 = bits::bit(&temp, 31);
                if (bit_31 != bit_32) {
                    abort 1000
                };
                write_reg_bits(state_hash, rd, bits::slice(temp, 31, 0))
            }else if (funct == 35) {
                //subu
                let temp = data(&rs_value) - data(&rt_value);
                write_reg_bits(state_hash, rd, bits::from_u64(temp, 32))
            }else if (funct == 36) {
                //and
                let temp = data(&rs_value) & data(&rt_value);
                write_reg_bits(state_hash, rd, bits::from_u64(temp, 32))
            }else if (funct == 37) {
                //or
                let temp = data(&rs_value) | data(&rt_value);
                write_reg_bits(state_hash, rd, bits::from_u64(temp, 32))
            }else if (funct == 38) {
                //xor
                let temp = data(&rs_value) ^ data(&rt_value);
                write_reg_bits(state_hash, rd, bits::from_u64(temp, 32))
            }else if (funct == 39) {
                // nor
                let temp = data(&rs_value) | data(&rt_value);
                temp = temp ^ 0xffffffff;  // not
                write_reg_bits(state_hash, rd, bits::from_u64(temp, 32))
            }else if (funct == 42) {
                // slt
                let temp = i64::less_than(
                    i64::from_bits(rs_value), i64::from_bits(rt_value));
                let temp = if (temp) { 1 } else { 0 };
                write_reg_bits(state_hash, rd, bits::from_u64(temp, 32))
            }else if (funct == 43) {
                // sltu
                let temp = if (data(&rs_value) < data(&rt_value)) { 1 } else { 0 };
                write_reg_bits(state_hash, rd, bits::from_u64(temp, 32))
            } else {
                abort 502
            };
            return state_hash
        };

        //let func = insn & 0b111111; // last 6-bits
        // j-type j/jal
        if (opcode == 2 || opcode == 3) {
            if (opcode == 3) {
                state_hash = write_memory(state_hash, reg_to_mem_addr(REG_LR), pc + 8);
            };
            let jump_address = bits::slice(insn, 25, 0);
            let higher = bits::slice(bits::from_u64(pc + 4, 32), 31, 28);
            let new_pc = bits::concat(higher, bits::concat(jump_address, bits::repeat_bit(false, 2)));

            let state_hash = step_pc(state_hash, next_pc, bits::data(&new_pc));

            return state_hash
        };

        // i-types insts
        if (opcode >=4 && opcode <=43) {
            let rs = data(&bits::slice(insn, 25, 21));
            let rt = data(&bits::slice(insn, 20, 16));
            let rs_value = read_reg_bits(state_hash, rs);
            let rt_value = read_reg_bits(state_hash, rt);
            let imm = bits::slice(insn, 15, 0);

            // branch insr
            if (opcode >= 4 && opcode < 8) {
                // beq, bne, blez, bgtz

                let should_branch = if (opcode == 4) {
                    rs_value == rt_value
                } else if (opcode == 5) {
                    rs_value != rt_value
                } else if (opcode == 6) {
                    // rs <= 0
                    !i64::less_than(
                        i64::zero(),
                        i64::from_bits(rs_value)
                    )
                } else if (opcode == 7) {
                    // rs > 0
                    i64::less_than(
                        i64::zero(),
                        i64::from_bits(rs_value)
                    )
                } else {
                    false
                };
                if (should_branch) {
                    return step_pc(state_hash, next_pc, (pc + 4 + data(&se(bits::left_shift(imm, 2), 32))) & 0xffffffff)
                } else {
                    return step_pc(state_hash, next_pc, next_pc + 4)
                }
            };

            state_hash = if (opcode == 8) { // addi
                addi(state_hash, rt, rs_value, se(imm, 32))
            } else if (opcode == 9) { // addiu
                addu(state_hash, rt, rs_value, se(imm, 32))
            } else if (opcode == 10) {
                //slti
                let temp = i64::less_than(i64::from_bits(rs_value), i64::from_bits(se(imm, 32)));
                write_reg(state_hash, rt, if(temp) {1} else {0})
            } else if (opcode == 11) {
                // sltiu
                let temp = data(&rs_value) < data(&se(imm, 32));
                write_reg(state_hash, rt, if(temp) {1} else {0})
            } else if (opcode == 12) { // andi
                let temp = data(&rs_value) & data(&imm);
                write_reg(state_hash, rt, temp & 0xffffffff)
            } else if (opcode == 13) {// ori
                let temp = data(&rs_value) | data(&imm);
                write_reg(state_hash, rt, temp & 0xffffffff)
            } else if (opcode == 14) {
                // xori
                let temp = data(&rs_value) ^ data(&imm);
                write_reg(state_hash, rt, temp & 0xffffffff)
            } else if (opcode == 15) {
                // lui
                write_reg_bits(state_hash, rt, left_shift(imm, 16))
            } else if (opcode == 32) {
                // lb
                let mem_addr = (data(&se(imm, 32)) + data(&rs_value)) & 0xffffffff;
                let memory_data_4b = bits::from_u64(read_memory(state_hash, mem_addr & 0xfffffffc),32);

                let mem_1b = slice(memory_data_4b, (31 - (mem_addr & 0x3) * 8 as u8), (32 - (mem_addr & 0x3) * 8 - 8 as u8));
                write_reg_bits(state_hash, rt, se(mem_1b, 32))
            } else if (opcode == 33) {
                // lh
                let mem_addr = (data(&se(imm, 32)) + data(&rs_value)) & 0xffffffff;
                // must be a mutiple 0f 2
                assert!(mem_addr & 0x1 == 0, 10000);
                let memory_data_4b = bits::from_u64(read_memory(state_hash, mem_addr & 0xfffffffc),32);
                let mem_2b = slice(memory_data_4b, (31 - (mem_addr & 0x3) * 8 as u8), (32 - (mem_addr & 0x3) * 8 - 16 as u8));
                write_reg_bits(state_hash, rt, se(mem_2b, 32))
            } else if (opcode == 34) {
                // lw
                let mem_addr = (data(&se(imm, 32)) + data(&rs_value)) & 0xffffffff;
                // must be a multiple of 4
                assert!(mem_addr & 0x3 == 0, 10000);
                let memory_data_4b = bits::from_u64(read_memory(state_hash, mem_addr & 0xfffffffc),32);
                write_reg_bits(state_hash, rt, memory_data_4b)
            } else if (opcode == 36) {
                // lbu
                let mem_addr = (data(&se(imm, 32)) + data(&rs_value)) & 0xffffffff;
                let memory_data_4b = bits::from_u64(read_memory(state_hash, mem_addr & 0xfffffffc),32);

                let mem_1b = slice(memory_data_4b, (31 - (mem_addr & 0x3) * 8 as u8), (32 - (mem_addr & 0x3) * 8 - 8 as u8));
                write_reg_bits(state_hash, rt, bits::ze(mem_1b, 32))
            } else if (opcode == 37) {
                // lhu
                let mem_addr = (data(&se(imm, 32)) + data(&rs_value)) & 0xffffffff;
                // must be a mutiple 0f 2
                assert!(mem_addr & 0x1 == 0, 10000);
                let memory_data_4b = bits::from_u64(read_memory(state_hash, mem_addr & 0xfffffffc),32);
                let mem_2b = slice(memory_data_4b, (31 - (mem_addr & 0x3) * 8 as u8), (32 - (mem_addr & 0x3) * 8 - 16 as u8));
                write_reg_bits(state_hash, rt, bits::ze(mem_2b, 32))
            } else if (opcode == 40){
                // sb
                let mem_addr = (data(&se(imm, 32)) + data(&rs_value)) & 0xffffffff;
                let read_addr = mem_addr & 0xfffffffc;
                let memory_data_4b = bits::from_u64(read_memory(state_hash, read_addr),32);

                let write_back = bits::write_range(
                    memory_data_4b,
                    (31 - (mem_addr & 0x3) * 8 as u8), (32 - (mem_addr & 0x3) * 8 - 8 as u8),
                    slice(rt_value, 7, 0)
                );
                write_memory(state_hash, read_addr, data(&write_back))
            } else if (opcode == 41) {
                // sh
                let mem_addr = (data(&se(imm, 32)) + data(&rs_value)) & 0xffffffff;
                assert!(mem_addr & 0x1 == 0, 10000);
                let read_addr = mem_addr & 0xfffffffc;
                let write_back = {
                    let memory_data_4b = bits::from_u64(read_memory(state_hash, read_addr),32);
                    bits::write_range(
                        memory_data_4b,
                        (31 - (mem_addr & 0x3) * 8 as u8), (32 - (mem_addr & 0x3) * 8 - 16 as u8),
                        slice(rt_value,15, 0)
                    )
                };
                write_memory(state_hash, read_addr, data(&write_back))
            } else if (opcode == 43) {
                // sw
                let mem_addr = (data(&se(imm, 32)) + data(&rs_value)) & 0xffffffff;
                assert!(mem_addr & 0x3 == 0, 10000);

                let read_addr = mem_addr & 0xfffffffc;
                let write_back = rt_value;
                write_memory(state_hash, read_addr, data(&write_back))
            } else {
                abort opcode
            };

            return state_hash
        };
        return hash_value::new(empty<u8>())
    }
    fun addi(state_hash: HashValue, store_reg: u64, a: Bits, b: Bits): HashValue {
        let val = i64::add(
            i64::from_bits(a),
            i64::from_bits(b),
        );
        let temp = i64::to_bits(val);
        let bit_32 = bits::bit(&temp, 32);
        let bit_31 = bits::bit(&temp, 31);
        if (bit_31 != bit_32) {
            abort 1000
        };
        write_reg_bits(state_hash, store_reg, bits::slice(temp, 31, 0))
    }
    fun addu(state_hash: HashValue, store_reg: u64, a: Bits, b: Bits): HashValue {
        let temp = data(&a) + data(&b);
        // only need last 32 bits
        let temp = temp & 0xffffffff;
        write_reg(state_hash, store_reg, temp)
    }
}

//module omo::utils {
//    /// SignExt_idx(dat)
//    public fun se(dat: u64, idx: u64): u64 {
//        0
//    }
//}

