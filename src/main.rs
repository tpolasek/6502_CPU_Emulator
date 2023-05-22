// Copyright Thomas Polasek
#![allow(non_snake_case)]
extern crate hex;
mod flag;

use std::time::Instant;
use minifb::{Key, ScaleMode, Window, WindowOptions};
use rand::Rng;

struct Bus {
    ram: [u8; 65536]
}

impl Bus {
    fn read_ram(&self, location : u16) -> u8 {
        if location == 0x00FE { //TODO remove
            return rand::thread_rng().gen_range(0..256) as u8;
        }
        return self.ram[location as usize];
    }

    fn write_ram(&mut self, location : u16, value : u8){
        self.ram[location as usize ] = value;
    }

    fn reset_ram(&mut self){
        for addr in 0..65535 {
            self.write_ram(addr, 0x00);
        }
    }

    fn print_ram(&self, start : u16, length : u16){
        println!("\nMemory: start=0x{:04x} length=0x{:04x}", start, length);
        let mut counter: u32 = 0;
        for addr in start..start+length+1 {
            if counter % 16 == 0 {
                print!("{:04x}: ", addr);
            }
            print!("{:02x} ",self.read_ram(addr));

            if counter % 16 == 15 {
                println!();
            }

            counter+=1;
        }
    }
}

struct Opcode <'a>{
    name : &'a str,
    addr_t :  fn(cpu : & mut Cpu) -> u8,
    operation : fn(cpu : & mut Cpu) -> u8,
    cycles : u8
}

impl Opcode <'_>{

    fn get_instruction_decoded(&self, cpu : & Cpu, pc_value : u16) -> String{
        let mut addr_u8 :u8 = 0xDD;
        let mut addr_u16 :u16 = 0xDEAD;

        if self.get_opcode_byte_size() == 2 {
            addr_u8 = cpu.bus.read_ram(pc_value + 1);
        }
        else if self.get_opcode_byte_size() == 3 {
            addr_u16 = (cpu.bus.read_ram(pc_value +1) as u16) | ((cpu.bus.read_ram(pc_value + 2) as u16) << 8);
        }

        if self.addr_t as usize == addr_ACC as usize {
            return format!("{:04x}: {} {}", pc_value, self.name, "A");
        }
        else if self.addr_t as usize == addr_ACC as usize {
            return format!("{:04x}: {}", pc_value, self.name);
        }
        else if self.addr_t as usize == addr_IMM as usize {
            return format!("{:04x}: {} #${:02x}", pc_value, self.name, addr_u8);
        }
        else if self.addr_t as usize == addr_ZPG as usize {
            return format!("{:04x}: {} ${:02x}", pc_value, self.name, addr_u8);
        }
        else if self.addr_t as usize == addr_ZPX as usize {
            return format!("{:04x}: {} ${:02x},X", pc_value, self.name, addr_u8);
        }
        else if self.addr_t as usize == addr_ZPY as usize {
            return format!("{:04x}: {} ${:02x},Y", pc_value, self.name, addr_u8);
        }
        else if self.addr_t as usize == addr_ABS as usize {
            return format!("{:04x}: {} ${:04x}", pc_value, self.name, addr_u8);
        }
        else if self.addr_t as usize == addr_ABX as usize {
            return format!("{:04x}: {} ${:04x},X", pc_value, self.name, addr_u16);
        }
        else if self.addr_t as usize == addr_ABY as usize {
            return format!("{:04x}: {} ${:04x},Y", pc_value, self.name, addr_u16);
        }
        else if self.addr_t as usize == addr_IND as usize {
            return format!("{:04x}: {} (${:04x})", pc_value, self.name, addr_u16);
        }
        else if self.addr_t as usize == addr_IDX as usize {
            return format!("{:04x}: {} (${:02x}, X)", pc_value, self.name, addr_u8);
        }
        else if self.addr_t as usize == addr_IDY as usize {
            return format!("{:04x}: {} (${:02x}), Y", pc_value, self.name, addr_u8);
        }
        else if self.addr_t as usize == addr_REL as usize {
            // +2 because jump is relative to the address at the end of the opcode
            return format!("{:04x}: {} (${:04x})", pc_value, self.name, (pc_value as i32) + (addr_u8 as i8) as i32 + 2 );
        }
        return String::from("???");
    }

    fn get_opcode_byte_size(&self) -> u16{
        let mut byte_count : u16 = 0;
        if self.addr_t as usize == addr_ACC as usize {
            byte_count = 1;
        }
        else if self.addr_t as usize == addr_ACC as usize {
            byte_count = 1;
        }
        else if self.addr_t as usize == addr_IMM as usize {
            byte_count = 2;
        }
        else if self.addr_t as usize == addr_ZPG as usize {
            byte_count = 2;
        }
        else if self.addr_t as usize == addr_ZPX as usize {
            byte_count = 2;
        }
        else if self.addr_t as usize == addr_ZPY as usize {
            byte_count = 2;
        }
        else if self.addr_t as usize == addr_ABS as usize {
            byte_count = 3;
        }
        else if self.addr_t as usize == addr_ABX as usize {
            byte_count = 3;
        }
        else if self.addr_t as usize == addr_ABY as usize {
            byte_count = 3;
        }
        else if self.addr_t as usize == addr_IND as usize {
            byte_count = 3;
        }
        else if self.addr_t as usize == addr_IDX as usize {
            byte_count = 2;
        }
        else if self.addr_t as usize == addr_IDY as usize {
            byte_count = 2;
        }
        else if self.addr_t as usize == addr_REL as usize {
            byte_count = 2;
        }

        assert!(byte_count != 0);

        return byte_count;
    }
}


// GOOD
fn addr_ACC(cpu : & mut Cpu) -> u8 {
    cpu.fetched = cpu.reg_a;
    return 0;
}

// GOOD
fn addr_IMM(cpu : & mut Cpu) -> u8 {
    cpu.abs_addr = cpu.pc;
    cpu.pc += 1;
    return 0;
}

// GOOD
fn addr_ZPG(cpu : & mut Cpu) -> u8 {
    cpu.abs_addr = cpu.bus.read_ram(cpu.pc) as u16;
    cpu.pc += 1;
    return 0;
}

// GOOD
fn addr_ZPX(cpu : & mut Cpu) -> u8 {
    cpu.abs_addr = cpu.bus.read_ram(cpu.pc) as u16 + cpu.reg_x as u16;
    cpu.abs_addr &= 0x00FF;
    cpu.pc += 1;
    return 0;
}

// GOOD
fn addr_ZPY(cpu : & mut Cpu) -> u8 {
    cpu.abs_addr = cpu.bus.read_ram(cpu.pc) as u16 + cpu.reg_y as u16;
    cpu.abs_addr &= 0x00FF;
    cpu.pc += 1;
    return 0;
}

// GOOD
fn addr_ABS(cpu : & mut Cpu) -> u8 {
    let abs_addr_lo = cpu.bus.read_ram(cpu.pc) as u16;
    cpu.pc += 1;
    let abs_addr_hi = cpu.bus.read_ram(cpu.pc) as u16;
    cpu.pc += 1;
    cpu.abs_addr = abs_addr_hi << 8 | abs_addr_lo;
    return 0;
}

// GOOD
fn addr_ABX(cpu : & mut Cpu) -> u8 {
    let abs_addr_lo = cpu.bus.read_ram(cpu.pc) as u16;
    cpu.pc += 1;
    let abs_addr_hi = cpu.bus.read_ram(cpu.pc) as u16;
    cpu.pc += 1;

    let temp : u32 = (abs_addr_hi << 8 | abs_addr_lo) as u32 + cpu.reg_x as u32;
    cpu.abs_addr = (temp & 0xFFFF) as u16; // Assumed this is correct

    // changing page costs extra
    if cpu.abs_addr & 0xFF00 != abs_addr_hi << 8 {
        return 1;
    }
    return 0;
}

// GOOD
fn addr_ABY(cpu : & mut Cpu) -> u8 {
    let abs_addr_lo = cpu.bus.read_ram(cpu.pc) as u16;
    cpu.pc += 1;
    let abs_addr_hi = cpu.bus.read_ram(cpu.pc) as u16;
    cpu.pc += 1;

    let temp : u32 = (abs_addr_hi << 8 | abs_addr_lo) as u32 + cpu.reg_y as u32;
    cpu.abs_addr = (temp & 0xFFFF) as u16; // Assumed this is correct

    // changing page costs extra
    if cpu.abs_addr & 0xFF00 != abs_addr_hi << 8 {
        return 1;
    }
    return 0;
}

// MAYBE ? kind of complex
fn addr_IND(cpu : & mut Cpu) -> u8 {
    let  ptr_addr_lo = cpu.bus.read_ram(cpu.pc) as u16;
    cpu.pc += 1;
    let ptr_addr_hi = cpu.bus.read_ram(cpu.pc) as u16;
    cpu.pc += 1;

    let ptr : u16 = (ptr_addr_hi << 8 | ptr_addr_lo) as u16;
    let ptr2 : u16 = ((ptr & 0xFF00) | ((ptr + 1) & 0x00FF)) as u16; //replicate 6502 page-boundary wraparound bug

    cpu.abs_addr =  (cpu.bus.read_ram(ptr2) as u16) << 8  | cpu.bus.read_ram(ptr) as u16;

    return 0;
}

// GOOD
fn addr_IDX(cpu : & mut Cpu) -> u8 {
    let ptr_addr = (cpu.bus.read_ram(cpu.pc) as u16 + cpu.reg_x as u16) & 0xFF;
    cpu.pc += 1;

    let abs_addr_lo = cpu.bus.read_ram(ptr_addr) as u16;
    let abs_addr_hi = cpu.bus.read_ram(ptr_addr + 1) as u16;

    cpu.abs_addr = (abs_addr_hi << 8 | abs_addr_lo);

    return 0;
}

// GOOD
fn addr_IDY(cpu : & mut Cpu) -> u8 {
    let ptr_addr = cpu.bus.read_ram(cpu.pc) as u16;
    cpu.pc += 1;

    let abs_addr_lo = cpu.bus.read_ram(ptr_addr ) as u16;
    let abs_addr_hi = cpu.bus.read_ram(ptr_addr + 1) as u16;

    cpu.abs_addr = (abs_addr_hi << 8 | abs_addr_lo) + cpu.reg_y as u16;

    if cpu.abs_addr & 0xFF00 != abs_addr_hi << 8{
        return 1;
    }
    return 0;
}

fn addr_REL(cpu : & mut Cpu) -> u8 {
    cpu.relative_addr_offset = cpu.bus.read_ram(cpu.pc) as i8;
    cpu.pc += 1;
    return 0;
}


///////////////////////////////////////////////
fn push_stack_u16(cpu : & mut Cpu, value : u16) {
    cpu.bus.write_ram(0x100 + cpu.reg_sp as u16, ((value >> 8) & 0xFF) as u8);
    cpu.reg_sp -= 1;
    cpu.bus.write_ram(0x100 + cpu.reg_sp as u16, (value & 0xFF) as u8);
    cpu.reg_sp -= 1;
}

fn push_stack_u8(cpu : & mut Cpu, value : u8) {
    cpu.bus.write_ram(0x100 + cpu.reg_sp as u16, value);
    cpu.reg_sp -= 1;
}

fn pull_stack_u8(cpu : & mut Cpu) -> u8 {
    cpu.reg_sp += 1;
    return cpu.bus.read_ram(0x100 + cpu.reg_sp as u16);
}

fn pull_stack_u16(cpu : & mut Cpu) -> u16 {
    cpu.reg_sp += 1;
    let val_lo : u16 = cpu.bus.read_ram(0x100 + cpu.reg_sp as u16) as u16;
    cpu.reg_sp += 1;
    let val_hi : u16 = cpu.bus.read_ram(0x100 + cpu.reg_sp as u16) as u16;

    return val_lo | (val_hi << 8);
}


fn set_z_n_flags(cpu : & mut Cpu, val : u8){
    cpu.flag.set_flag_z(val == 0);
    cpu.flag.set_flag_n(val & 0x80 != 0);
}


// TODO verify this logic is correct
fn set_overflow_flag(cpu : & mut Cpu, result: u16, acc : u8, mem : u8){
    cpu.flag.set_flag_v((result ^ acc as u16) & (result ^ mem as u16) & 0x0080 != 0);
}

///////////////////////////////////////////////


fn op_NOP(_cpu : & mut Cpu) -> u8 {
    return 0;
}

fn op_ADC(cpu : & mut Cpu) -> u8 {
    cpu.fetch();

    let sum_u16 : u16 = cpu.reg_a as u16 + cpu.fetched as u16 + cpu.flag.flag_c as u16;
    let sum_u8 : u8 = (sum_u16 & 0xff) as u8;

    cpu.flag.set_flag_c(sum_u16 > 0xff);
    set_overflow_flag(cpu, sum_u16, cpu.reg_a, cpu.fetched);
    set_z_n_flags(cpu, sum_u8);

    cpu.reg_a = sum_u8;
    //println!("ADC = {:#x}", cpu.reg_a);
    return 0;
}

fn op_AND(cpu : & mut Cpu) -> u8 {
    cpu.fetch();

    cpu.reg_a &= cpu.fetched;

    set_z_n_flags(cpu, cpu.reg_a);
    return 0;
}

fn op_BIT(cpu : & mut Cpu) -> u8 {
    cpu.fetch();

    cpu.flag.set_flag_z(cpu.reg_a & cpu.fetched == 0x00);
    cpu.flag.set_flag_v(cpu.fetched & (1 << 6) != 0);
    cpu.flag.set_flag_n(cpu.fetched & (1 << 7) != 0);
    return 0;
}


fn op_LDA(cpu : & mut Cpu) -> u8 {
    cpu.fetch();

    cpu.reg_a = cpu.fetched;

    set_z_n_flags(cpu, cpu.reg_a);
    return 0;
}

fn op_LDX(cpu : & mut Cpu) -> u8 {
    cpu.fetch();

    cpu.reg_x = cpu.fetched;

    set_z_n_flags(cpu, cpu.reg_x);

    return 0;
}

fn op_LDY(cpu : & mut Cpu) -> u8 {
    cpu.fetch();

    cpu.reg_y = cpu.fetched;

    set_z_n_flags(cpu, cpu.reg_y);

    return 0;
}

fn op_STA(cpu : & mut Cpu) -> u8 {
    cpu.bus.write_ram(cpu.abs_addr, cpu.reg_a);
    return 0;
}

fn op_STX(cpu : & mut Cpu) -> u8 {
    cpu.bus.write_ram(cpu.abs_addr, cpu.reg_x);
    return 0;
}

fn op_STY(cpu : & mut Cpu) -> u8 {
    cpu.bus.write_ram(cpu.abs_addr, cpu.reg_y);
    return 0;
}

fn op_TAX(cpu : & mut Cpu) -> u8 {
    cpu.reg_x = cpu.reg_a;
    set_z_n_flags(cpu, cpu.reg_x);
    return 0;
}

fn op_TAY(cpu : & mut Cpu) -> u8 {
    cpu.reg_y = cpu.reg_a;
    set_z_n_flags(cpu, cpu.reg_y);
    return 0;
}

fn op_TSX(cpu : & mut Cpu) -> u8 {
    cpu.reg_x = cpu.reg_sp;
    set_z_n_flags(cpu, cpu.reg_x);
    return 0;
}

fn op_TXA(cpu : & mut Cpu) -> u8 {
    cpu.reg_a = cpu.reg_x;
    set_z_n_flags(cpu, cpu.reg_a);
    return 0;
}

fn op_TXS(cpu : & mut Cpu) -> u8 {
    cpu.reg_sp = cpu.reg_x;
    return 0;
}

fn op_TYA(cpu : & mut Cpu) -> u8 {
    cpu.reg_a = cpu.reg_y;
    set_z_n_flags(cpu, cpu.reg_a);
    return 0;
}

fn op_CLC(cpu : & mut Cpu) -> u8 {
    cpu.flag.set_flag_c(false);
    return 0;
}

fn op_CLD(cpu : & mut Cpu) -> u8 {
    cpu.flag.set_flag_d(false);
    return 0;
}

fn op_CLI(cpu : & mut Cpu) -> u8 {
    cpu.flag.set_flag_i(false);
    return 0;
}

fn op_CLV(cpu : & mut Cpu) -> u8 {
    cpu.flag.set_flag_v(false);
    return 0;
}

fn op_DEC(cpu : & mut Cpu) -> u8 {
    cpu.fetch();

    let mut temp :u8  = 0;
    if cpu.fetched == 0x00{
        temp = 0xFF;
    }
    else{
        temp = cpu.fetched - 1;
    }

    cpu.bus.write_ram(cpu.abs_addr, temp);

    set_z_n_flags(cpu, temp);
    return 0;
}

fn op_DEX(cpu : & mut Cpu) -> u8 {
    if cpu.reg_x == 0x00{
        cpu.reg_x = 0xFF;
    }
    else{
        cpu.reg_x -= 1;
    }
    set_z_n_flags(cpu, cpu.reg_x);
    return 0;
}

fn op_DEY(cpu : & mut Cpu) -> u8 {
    if cpu.reg_y == 0x00{
        cpu.reg_y = 0xFF;
    }
    else{
        cpu.reg_y -= 1;
    }
    set_z_n_flags(cpu, cpu.reg_y);
    return 0;
}

fn op_EOR(cpu : & mut Cpu) -> u8 {
    cpu.fetch();
    cpu.reg_a ^= cpu.fetched;

    set_z_n_flags(cpu, cpu.reg_a);
    return 0;
}

fn op_INC(cpu : & mut Cpu) -> u8 {
    cpu.fetch();

    let mut temp :u8  = 0;
    if cpu.fetched == 0xFF{
        temp = 0x00;
    }
    else{
        temp = cpu.fetched + 1;
    }

    cpu.bus.write_ram(cpu.abs_addr, temp);

    set_z_n_flags(cpu, temp);
    return 0;
}

fn op_INX(cpu : & mut Cpu) -> u8 {
    if cpu.reg_x == 0xFF{
        cpu.reg_x = 0x00;
    }
    else{
        cpu.reg_x += 1;
    }
    set_z_n_flags(cpu, cpu.reg_x);
    return 0;
}

fn op_INY(cpu : & mut Cpu) -> u8 {
    if cpu.reg_y == 0xFF{
        cpu.reg_y = 0x00;
    }
    else{
        cpu.reg_y += 1;
    }
    set_z_n_flags(cpu, cpu.reg_y);
    return 0;
}

// Shared function for jumps
fn op_jump(cpu : &mut Cpu, do_jump_condition : bool) -> u8 {
    let mut cycle_cost : u8 = 0;

    if do_jump_condition {
        cycle_cost +=1;

        let updated_pc = (cpu.pc as i32 + cpu.relative_addr_offset as i32) as u16;

        if updated_pc & 0xFF00 != cpu.pc & 0xFF00 {
            cycle_cost += 1;
        }
        cpu.pc = updated_pc;
    }
    return cycle_cost;
}

// carry flag
fn op_BCS(cpu : & mut Cpu) -> u8 {
    return op_jump( cpu, cpu.flag.get_flag_c() );
}

fn op_BCC(cpu : & mut Cpu) -> u8 {
    return op_jump( cpu, !cpu.flag.get_flag_c() );
}

// zero
fn op_BEQ(cpu : & mut Cpu) -> u8 {
    return op_jump( cpu, cpu.flag.get_flag_z() );
}
fn op_BNE(cpu : & mut Cpu) -> u8 {
    return op_jump( cpu, !cpu.flag.get_flag_z() );
}

// negative
fn op_BMI(cpu : & mut Cpu) -> u8 {
    return op_jump( cpu, cpu.flag.get_flag_n() );
}
fn op_BPL(cpu : & mut Cpu) -> u8 {
    return op_jump( cpu, !cpu.flag.get_flag_n());
}

// overflow
fn op_BVS(cpu : & mut Cpu) -> u8 {
    return op_jump( cpu, cpu.flag.get_flag_v());
}
fn op_BVC(cpu : & mut Cpu) -> u8 {
    return op_jump( cpu, !cpu.flag.get_flag_v());
}

fn op_JMP(cpu : & mut Cpu) -> u8 {
    cpu.pc = cpu.abs_addr;
    return 0;
}

fn op_JSR(cpu : & mut Cpu) -> u8 {
    push_stack_u16(cpu, cpu.pc - 1);
    cpu.pc = cpu.abs_addr;
    return 0;
}

fn op_LSR(cpu : & mut Cpu) -> u8 {
    cpu.fetch();

    cpu.flag.set_flag_c(cpu.fetched & 0x01 != 0);

    let value = cpu.fetched >> 1;

    set_z_n_flags(cpu, value);
    cpu.write_value(value);
    return 0;
}

fn op_ORA(cpu : & mut Cpu) -> u8 {
    cpu.fetch();
    cpu.reg_a |= cpu.fetched;
    set_z_n_flags(cpu, cpu.reg_a);
    return 0;
}

fn op_PHA(cpu : & mut Cpu) -> u8 {
    push_stack_u8(cpu, cpu.reg_a);
    return 0;
}

fn op_PHP(cpu : & mut Cpu) -> u8 {
    push_stack_u8(cpu, cpu.flag.get_sr() | 0x10); // FLAG BREAK
    return 0;
}

fn op_PLA(cpu : & mut Cpu) -> u8 {
    cpu.reg_a = pull_stack_u8(cpu);
    set_z_n_flags(cpu, cpu.reg_a);
    return 0;
}

fn op_PLP(cpu : & mut Cpu) -> u8 {
    let sr : u8 = pull_stack_u8(cpu);
    cpu.flag.set_sr(sr);
    return 0;
}

fn op_ROL(cpu : & mut Cpu) -> u8 {
    cpu.fetch();
    let value : u8 = (cpu.fetched << 1) | cpu.flag.flag_c;

    cpu.flag.set_flag_c(cpu.fetched & 0x80 != 0);
    set_z_n_flags(cpu, value);

    cpu.write_value(value);

    return 0;
}

fn op_ROR(cpu : & mut Cpu) -> u8 {
    cpu.fetch();
    let value : u8 = (cpu.flag.flag_c << 7) | (cpu.fetched >> 1);

    cpu.flag.set_flag_c(cpu.fetched & 0x01 != 0);
    set_z_n_flags(cpu, value);

    cpu.write_value(value);

    return 0;
}

fn op_ASL(cpu : & mut Cpu) -> u8 {
    cpu.fetch();
    let value : u8 = cpu.fetched << 1;

    cpu.flag.set_flag_c(cpu.fetched & 0x80 != 0);
    set_z_n_flags(cpu, value);

    cpu.write_value(value);

    return 0;
}

fn op_RTI(cpu: & mut Cpu) -> u8 {
    let sr : u8 = pull_stack_u8(cpu);
    let pc : u16 = pull_stack_u16(cpu);
    cpu.flag.set_sr(sr);
    cpu.pc = pc;
    return 0;
}

fn op_RTS(cpu: & mut Cpu) -> u8 {
    let pc : u16 = pull_stack_u16(cpu);
    cpu.pc = pc + 1;
    return 0;
}

fn op_SEC(cpu: & mut Cpu) -> u8 {
    cpu.flag.set_flag_c(true);
    return 0;
}

fn op_SED(cpu: & mut Cpu) -> u8 {
    cpu.flag.set_flag_d(true);
    return 0;
}

fn op_SEI(cpu: & mut Cpu) -> u8 {
    cpu.flag.set_flag_i(true);
    return 0;
}

fn op_BRK(cpu: & mut Cpu) -> u8 {
    cpu.flag.set_flag_i(true);

    push_stack_u16(cpu,cpu.pc + 1);
    push_stack_u8(cpu, cpu.flag.get_sr() | 0x10); // FLAG BREAK

    cpu.pc = cpu.bus.read_ram(0xFFFE) as u16 |  (cpu.bus.read_ram(0xFFFF) as u16) << 8;
    return 0;
}

fn op_CMP(cpu: & mut Cpu) -> u8 {
    cpu.fetch();


    let mut result : u8 = 0;
    if cpu.reg_a < cpu.fetched{
        result = 0xFF - (cpu.fetched - cpu.reg_a);
    }
    else{
        result = cpu.reg_a - cpu.fetched;
    }

    cpu.flag.set_flag_c(cpu.reg_a >= cpu.fetched);
    cpu.flag.set_flag_z(cpu.reg_a == cpu.fetched);
    cpu.flag.set_flag_n(result & 0x80 != 0);
    return 0;
}

fn op_CPX(cpu: & mut Cpu) -> u8 {
    cpu.fetch();

    let mut result : u8 = 0;
    if cpu.reg_x < cpu.fetched{
        result = 0xFF - (cpu.fetched - cpu.reg_x);
    }
    else{
        result = cpu.reg_x - cpu.fetched;
    }

    cpu.flag.set_flag_c(cpu.reg_x >= cpu.fetched);
    cpu.flag.set_flag_z(cpu.reg_x == cpu.fetched);
    cpu.flag.set_flag_n(result & 0x80 != 0);
    return 0;
}

fn op_CPY(cpu: & mut Cpu) -> u8 {
    cpu.fetch();

    let mut result : u8 = 0;
    if cpu.reg_y < cpu.fetched{
        result = 0xFF - (cpu.fetched - cpu.reg_y);
    }
    else{
        result = cpu.reg_y - cpu.fetched;
    }

    cpu.flag.set_flag_c(cpu.reg_y >= cpu.fetched);
    cpu.flag.set_flag_z(cpu.reg_y == cpu.fetched);
    cpu.flag.set_flag_n(result & 0x80 != 0);
    return 0;
}

fn op_SBC(cpu: & mut Cpu) -> u8 {
    cpu.fetch();

    let value: u8 = cpu.fetched ^ 0xFF;
    let sum_u16: u16 = cpu.reg_a as u16 + value as u16 + cpu.flag.flag_c as u16;
    let sum_u8: u8 = (sum_u16 & 0xFF) as u8;


    cpu.flag.set_flag_c(sum_u16 > 0xff);
    set_overflow_flag(cpu, sum_u16, cpu.reg_a, value);
    set_z_n_flags(cpu, sum_u8);

    cpu.reg_a = sum_u8;
    return 0;
}

const OPCODE_LOOKUP: [Opcode; 256] = [
    Opcode { name: "BRK", addr_t: addr_ACC, operation: op_BRK, cycles: 0 },Opcode { name: "ORA", addr_t: addr_IDX, operation: op_ORA, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "ORA", addr_t: addr_ZPG, operation: op_ORA, cycles: 0 },Opcode { name: "ASL", addr_t: addr_ZPG, operation: op_ASL, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "PHP", addr_t: addr_ACC, operation: op_PHP, cycles: 0 },Opcode { name: "ORA", addr_t: addr_IMM, operation: op_ORA, cycles: 0 },Opcode { name: "ASL", addr_t: addr_ACC, operation: op_ASL, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "ORA", addr_t: addr_ABS, operation: op_ORA, cycles: 0 },Opcode { name: "ASL", addr_t: addr_ABS, operation: op_ASL, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },
    Opcode { name: "BPL", addr_t: addr_REL, operation: op_BPL, cycles: 0 },Opcode { name: "ORA", addr_t: addr_IDY, operation: op_ORA, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "ORA", addr_t: addr_ZPX, operation: op_ORA, cycles: 0 },Opcode { name: "ASL", addr_t: addr_ZPX, operation: op_ASL, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "CLC", addr_t: addr_ACC, operation: op_CLC, cycles: 0 },Opcode { name: "ORA", addr_t: addr_ABY, operation: op_ORA, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "ORA", addr_t: addr_ABX, operation: op_ORA, cycles: 0 },Opcode { name: "ASL", addr_t: addr_ABX, operation: op_ASL, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },
    Opcode { name: "JSR", addr_t: addr_ABS, operation: op_JSR, cycles: 0 },Opcode { name: "AND", addr_t: addr_IDX, operation: op_AND, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "BIT", addr_t: addr_ZPG, operation: op_BIT, cycles: 0 },Opcode { name: "AND", addr_t: addr_ZPG, operation: op_AND, cycles: 0 },Opcode { name: "ROL", addr_t: addr_ZPG, operation: op_ROL, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "PLP", addr_t: addr_ACC, operation: op_PLP, cycles: 0 },Opcode { name: "AND", addr_t: addr_IMM, operation: op_AND, cycles: 0 },Opcode { name: "ROL", addr_t: addr_ACC, operation: op_ROL, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "BIT", addr_t: addr_ABS, operation: op_BIT, cycles: 0 },Opcode { name: "AND", addr_t: addr_ABS, operation: op_AND, cycles: 0 },Opcode { name: "ROL", addr_t: addr_ABS, operation: op_ROL, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },
    Opcode { name: "BMI", addr_t: addr_REL, operation: op_BMI, cycles: 0 },Opcode { name: "AND", addr_t: addr_IDY, operation: op_AND, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "AND", addr_t: addr_ZPX, operation: op_AND, cycles: 0 },Opcode { name: "ROL", addr_t: addr_ZPX, operation: op_ROL, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "SEC", addr_t: addr_ACC, operation: op_SEC, cycles: 0 },Opcode { name: "AND", addr_t: addr_ABY, operation: op_AND, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "AND", addr_t: addr_ABX, operation: op_AND, cycles: 0 },Opcode { name: "ROL", addr_t: addr_ABX, operation: op_ROL, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },
    Opcode { name: "RTI", addr_t: addr_ACC, operation: op_RTI, cycles: 0 },Opcode { name: "XOR", addr_t: addr_IDX, operation: op_EOR, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "XOR", addr_t: addr_ZPG, operation: op_EOR, cycles: 0 },Opcode { name: "LSR", addr_t: addr_ZPG, operation: op_LSR, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "PHA", addr_t: addr_ACC, operation: op_PHA, cycles: 0 },Opcode { name: "XOR", addr_t: addr_IMM, operation: op_EOR, cycles: 0 },Opcode { name: "LSR", addr_t: addr_ACC, operation: op_LSR, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "JMP", addr_t: addr_ABS, operation: op_JMP, cycles: 0 },Opcode { name: "XOR", addr_t: addr_ABS, operation: op_EOR, cycles: 0 },Opcode { name: "LSR", addr_t: addr_ABS, operation: op_LSR, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },
    Opcode { name: "BVC", addr_t: addr_REL, operation: op_BVC, cycles: 0 },Opcode { name: "XOR", addr_t: addr_IDY, operation: op_EOR, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "XOR", addr_t: addr_ZPX, operation: op_EOR, cycles: 0 },Opcode { name: "LSR", addr_t: addr_ZPX, operation: op_LSR, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "CLI", addr_t: addr_ACC, operation: op_CLI, cycles: 0 },Opcode { name: "XOR", addr_t: addr_ABY, operation: op_EOR, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "XOR", addr_t: addr_ABX, operation: op_EOR, cycles: 0 },Opcode { name: "LSR", addr_t: addr_ABX, operation: op_LSR, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },
    Opcode { name: "RTS", addr_t: addr_ACC, operation: op_RTS, cycles: 0 },Opcode { name: "ADC", addr_t: addr_IDX, operation: op_ADC, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "ADC", addr_t: addr_ZPG, operation: op_ADC, cycles: 0 },Opcode { name: "ROR", addr_t: addr_ZPG, operation: op_ROR, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "PLA", addr_t: addr_ACC, operation: op_PLA, cycles: 0 },Opcode { name: "ADC", addr_t: addr_IMM, operation: op_ADC, cycles: 0 },Opcode { name: "ROR", addr_t: addr_ACC, operation: op_ROR, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "JMP", addr_t: addr_IND, operation: op_JMP, cycles: 0 },Opcode { name: "ADC", addr_t: addr_ABS, operation: op_ADC, cycles: 0 },Opcode { name: "ROR", addr_t: addr_ABS, operation: op_ROR, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },
    Opcode { name: "BVS", addr_t: addr_REL, operation: op_BVS, cycles: 0 },Opcode { name: "ADC", addr_t: addr_IDY, operation: op_ADC, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "ADC", addr_t: addr_ZPX, operation: op_ADC, cycles: 0 },Opcode { name: "ROR", addr_t: addr_ZPX, operation: op_ROR, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "SEI", addr_t: addr_ACC, operation: op_SEI, cycles: 0 },Opcode { name: "ADC", addr_t: addr_ABY, operation: op_ADC, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "ADC", addr_t: addr_ABX, operation: op_ADC, cycles: 0 },Opcode { name: "ROR", addr_t: addr_ABX, operation: op_ROR, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },
    Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "STA", addr_t: addr_IDX, operation: op_STA, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "STX", addr_t: addr_ZPG, operation: op_STY, cycles: 0 },Opcode { name: "STA", addr_t: addr_ZPG, operation: op_STA, cycles: 0 },Opcode { name: "STX", addr_t: addr_ZPG, operation: op_STX, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "DEY", addr_t: addr_ACC, operation: op_DEY, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "TXA", addr_t: addr_ACC, operation: op_TXA, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "STY", addr_t: addr_ABS, operation: op_STY, cycles: 0 },Opcode { name: "STA", addr_t: addr_ABS, operation: op_STA, cycles: 0 },Opcode { name: "STX", addr_t: addr_ABS, operation: op_STX, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },
    Opcode { name: "BCC", addr_t: addr_REL, operation: op_BCC, cycles: 0 },Opcode { name: "STA", addr_t: addr_IDY, operation: op_STA, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "STX", addr_t: addr_ZPX, operation: op_STY, cycles: 0 },Opcode { name: "STA", addr_t: addr_ZPX, operation: op_STA, cycles: 0 },Opcode { name: "STA", addr_t: addr_ZPY, operation: op_STA, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "TYA", addr_t: addr_ACC, operation: op_TYA, cycles: 0 },Opcode { name: "STA", addr_t: addr_ABY, operation: op_STA, cycles: 0 },Opcode { name: "TXS", addr_t: addr_ACC, operation: op_TXS, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "STA", addr_t: addr_ABX, operation: op_STA, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },
    Opcode { name: "LDY", addr_t: addr_IMM, operation: op_LDY, cycles: 0 },Opcode { name: "LDA", addr_t: addr_IDX, operation: op_LDA, cycles: 0 },Opcode { name: "LDX", addr_t: addr_IMM, operation: op_LDX, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "LDY", addr_t: addr_ZPG, operation: op_LDY, cycles: 0 },Opcode { name: "LDA", addr_t: addr_ZPG, operation: op_LDA, cycles: 0 },Opcode { name: "LDX", addr_t: addr_ZPG, operation: op_LDX, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "TAY", addr_t: addr_ACC, operation: op_TAY, cycles: 0 },Opcode { name: "LDA", addr_t: addr_IMM, operation: op_LDA, cycles: 0 },Opcode { name: "TAX", addr_t: addr_ACC, operation: op_TAX, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "LDY", addr_t: addr_ABS, operation: op_LDY, cycles: 0 },Opcode { name: "LDA", addr_t: addr_ABS, operation: op_LDA, cycles: 0 },Opcode { name: "LDX", addr_t: addr_ABS, operation: op_LDX, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },
    Opcode { name: "BCS", addr_t: addr_REL, operation: op_BCS, cycles: 0 },Opcode { name: "LDA", addr_t: addr_IDY, operation: op_LDA, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "LDY", addr_t: addr_ZPX, operation: op_LDY, cycles: 0 },Opcode { name: "LDA", addr_t: addr_ZPX, operation: op_LDA, cycles: 0 },Opcode { name: "LDX", addr_t: addr_ZPY, operation: op_LDX, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "CLV", addr_t: addr_ACC, operation: op_CLV, cycles: 0 },Opcode { name: "LDA", addr_t: addr_ABY, operation: op_LDA, cycles: 0 },Opcode { name: "TSX", addr_t: addr_ACC, operation: op_TSX, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "LDY", addr_t: addr_ABX, operation: op_LDY, cycles: 0 },Opcode { name: "LDA", addr_t: addr_ABX, operation: op_LDA, cycles: 0 },Opcode { name: "LDX", addr_t: addr_ABY, operation: op_LDX, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },
    Opcode { name: "CPY", addr_t: addr_IMM, operation: op_CPY, cycles: 0 },Opcode { name: "CMP", addr_t: addr_IDX, operation: op_CMP, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "CPY", addr_t: addr_ZPG, operation: op_CPY, cycles: 0 },Opcode { name: "CMP", addr_t: addr_ZPG, operation: op_CMP, cycles: 0 },Opcode { name: "DEC", addr_t: addr_ZPG, operation: op_DEC, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "INY", addr_t: addr_ACC, operation: op_INY, cycles: 0 },Opcode { name: "CMP", addr_t: addr_IMM, operation: op_CMP, cycles: 0 },Opcode { name: "DEX", addr_t: addr_ACC, operation: op_DEX, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "CPY", addr_t: addr_ABS, operation: op_CPY, cycles: 0 },Opcode { name: "CMP", addr_t: addr_ABS, operation: op_CMP, cycles: 0 },Opcode { name: "DEC", addr_t: addr_ABS, operation: op_DEC, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },
    Opcode { name: "BNE", addr_t: addr_REL, operation: op_BNE, cycles: 0 },Opcode { name: "CMP", addr_t: addr_IDY, operation: op_CMP, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "CMP", addr_t: addr_ZPX, operation: op_CMP, cycles: 0 },Opcode { name: "DEC", addr_t: addr_ZPX, operation: op_DEC, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "CLD", addr_t: addr_ACC, operation: op_CLD, cycles: 0 },Opcode { name: "CMP", addr_t: addr_ABY, operation: op_CMP, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "CMP", addr_t: addr_ABX, operation: op_CMP, cycles: 0 },Opcode { name: "DEC", addr_t: addr_ABX, operation: op_DEC, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },
    Opcode { name: "CPX", addr_t: addr_IMM, operation: op_CPX, cycles: 0 },Opcode { name: "SBC", addr_t: addr_ZPG, operation: op_SBC, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "CPX", addr_t: addr_ZPG, operation: op_CPX, cycles: 0 },Opcode { name: "SBC", addr_t: addr_ZPX, operation: op_SBC, cycles: 0 },Opcode { name: "INC", addr_t: addr_ZPG, operation: op_INC, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "INX", addr_t: addr_ACC, operation: op_INX, cycles: 0 },Opcode { name: "SBC", addr_t: addr_IMM, operation: op_SBC, cycles: 0 },Opcode { name: "NOP", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "CPX", addr_t: addr_ABS, operation: op_CPX, cycles: 0 },Opcode { name: "SBC", addr_t: addr_ABS, operation: op_SBC, cycles: 0 },Opcode { name: "INC", addr_t: addr_ABS, operation: op_INC, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },
    Opcode { name: "BEQ", addr_t: addr_REL, operation: op_BEQ, cycles: 0 },Opcode { name: "SBC", addr_t: addr_ABX, operation: op_SBC, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "SBC", addr_t: addr_ABY, operation: op_SBC, cycles: 0 },Opcode { name: "INC", addr_t: addr_ZPX, operation: op_INC, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "SED", addr_t: addr_ACC, operation: op_SED, cycles: 0 },Opcode { name: "SBC", addr_t: addr_IDX, operation: op_SBC, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 },Opcode { name: "SBC", addr_t: addr_IDY, operation: op_SBC, cycles: 0 },Opcode { name: "INC", addr_t: addr_ABX, operation: op_INC, cycles: 0 },Opcode { name: "NUL", addr_t: addr_ACC, operation: op_NOP, cycles: 0 }
];

struct Cpu {
    bus: Bus,
    pc: u16,
    cycles: u8,
    reg_a: u8,
    reg_x: u8,
    reg_y: u8,
    reg_sp: u8,
    flag: flag::Flag,
    // other
    tick_count: u64,
    fetched: u8,
    abs_addr: u16,
    relative_addr_offset: i8,
    is_accumulator_opcode : bool
}

impl Cpu {

    pub fn new(bus: Bus) -> Self {
        let flag = flag::Flag {flag_n: 0, flag_v: 0, flag_b: 0, flag_d: 0, flag_i: 0, flag_z: 0, flag_c: 0};

        Self {
            bus,
            pc: 0x0600,
            cycles: 0,
            reg_a: 0,
            reg_x: 0,
            reg_y: 0,
            reg_sp: 0xFF,
            flag,
            tick_count : 0,
            fetched : 0,
            abs_addr: 0,
            relative_addr_offset: 0,
            is_accumulator_opcode: false
        }
    }

    fn write_value(& mut self, value : u8){
        if self.is_accumulator_opcode {
            self.reg_a = value;
        }
        else{
            self.bus.write_ram(self.abs_addr, value);
        }
    }

    fn fetch(&mut self){
        if !self.is_accumulator_opcode {
            self.fetched = self.bus.read_ram(self.abs_addr);
        }
    }

    fn tick(&mut self, print: bool) {
        self.tick_count += 1;

        if self.cycles > 0 {
            self.cycles -= 1;
        }

        if self.cycles == 0 {

            let current_opcode :u8 = self.bus.read_ram(self.pc);
            self.pc += 1;

            let opcode : &Opcode = &OPCODE_LOOKUP[current_opcode as usize];
            self.is_accumulator_opcode = (opcode.addr_t as usize == addr_ACC as usize);

            //println!("Optcode byte: {:02x}", value);
            if print {
                self.print_cpu_state(opcode);
            }

            self.cycles += (opcode.addr_t as fn(cpu : & mut Cpu) -> u8) (self);
            self.cycles += (opcode.operation as fn(cpu : & mut Cpu) -> u8) (self);
            self.cycles += opcode.cycles;

        }
    }

    fn print_cpu_state(&self, opcode :  &Opcode) {
        println!("{:20} A={:02x} X={:02x} Y={:02x} SP={:02x} PC={:04x} {}",
                 opcode.get_instruction_decoded(self, self.pc - 1),
                 self.reg_a, self.reg_x, self.reg_y, self.reg_sp, self.pc,
                 self.flag.get_formatted_str());
    }

    fn get_reg_sr(&self) -> u8 {
        return self.flag.get_sr();
    }

    fn run_until_interrupt(&mut self, print : bool){
        loop {
            self.tick(print);
            if self.pc == 0x00 {
                break;
            }
        }
    }
}

/*
Program format:
XXXX: XX XX XX XX XX XX #somecomment \n
XXXX: XX XX XX #somecomment \n
*/
fn loadProgram(bus : & mut Bus, start_address : u16, program: &str){
    bus.reset_ram(); // resets the ram

    let lines = program.split("\n");

    let mut hexchars = String::with_capacity(60);

    for line in lines {
        let char_vec: Vec<char> = line.chars().collect();
        let mut foundColon : bool = false;
        for c in char_vec {
            if c == ' '{
                continue;
            }
            if c == '#' {
                break;
            }

            if c == '\n'{
                break;
            }
            if c == ':' {
                foundColon = true;
                continue;
            }
            if !foundColon {
                continue;
            }

            hexchars.push(c);

        }
    }

    let hex_bytes = hex::decode(hexchars).expect("Decoding failed");

    let mut address : u16 = start_address;
    for hex in hex_bytes {
        bus.write_ram(address, hex);
        address += 1;
    }
}

fn test_LDA(){
    let mut bus = Bus { ram:  [0; 65536]};
    /*
    LDA #$00
    LDA #$80
    */
    loadProgram(&mut bus, 0x0600, "0600: a9 00 a9 80" );
    let mut cpu = Cpu::new(bus);
    cpu.tick(true);
    assert!(cpu.flag.get_flag_z());
    assert!(!cpu.flag.get_flag_n());
    assert!(cpu.reg_a == 0);
    cpu.tick(true);
    cpu.tick(true);
    assert!(!cpu.flag.get_flag_z());
    assert!(cpu.flag.get_flag_n());
    assert!(cpu.reg_a == 0x80);
}

fn test_Stack(){
    let mut bus = Bus { ram:  [0; 65536]};
    loadProgram(&mut bus, 0x0600, "0600: a2 00 a0 00 8a 99 00 02 48 e8 c8 c0 10 d0 f5 68 99 00 02 c8 c0 20 d0 f7" );
    let mut cpu = Cpu::new(bus);
    cpu.run_until_interrupt(true);
    cpu.bus.print_ram(0x200, 0xff);
}

fn test_Snake(){
    let mut bus = Bus { ram:  [0; 65536]};
    loadProgram(&mut bus, 0x0600, "0600: 20 06 06 20 38 06 20 0d 06 20 2a 06 60 a9 02 85
0610: 02 a9 04 85 03 a9 11 85 10 a9 10 85 12 a9 0f 85
0620: 14 a9 04 85 11 85 13 85 15 60 a5 fe 85 00 a5 fe
0630: 29 03 18 69 02 85 01 60 20 4d 06 20 8d 06 20 c3
0640: 06 20 19 07 20 20 07 20 2d 07 4c 38 06 a5 ff c9
0650: 77 f0 0d c9 64 f0 14 c9 73 f0 1b c9 61 f0 22 60
0660: a9 04 24 02 d0 26 a9 01 85 02 60 a9 08 24 02 d0
0670: 1b a9 02 85 02 60 a9 01 24 02 d0 10 a9 04 85 02
0680: 60 a9 02 24 02 d0 05 a9 08 85 02 60 60 20 94 06
0690: 20 a8 06 60 a5 00 c5 10 d0 0d a5 01 c5 11 d0 07
06a0: e6 03 e6 03 20 2a 06 60 a2 02 b5 10 c5 10 d0 06
06b0: b5 11 c5 11 f0 09 e8 e8 e4 03 f0 06 4c aa 06 4c
06c0: 35 07 60 a6 03 ca 8a b5 10 95 12 ca 10 f9 a5 02
06d0: 4a b0 09 4a b0 19 4a b0 1f 4a b0 2f a5 10 38 e9
06e0: 20 85 10 90 01 60 c6 11 a9 01 c5 11 f0 28 60 e6
06f0: 10 a9 1f 24 10 f0 1f 60 a5 10 18 69 20 85 10 b0
0700: 01 60 e6 11 a9 06 c5 11 f0 0c 60 c6 10 a5 10 29
0710: 1f c9 1f f0 01 60 4c 35 07 a0 00 a5 fe 91 00 60
0720: a6 03 a9 00 81 10 a2 00 a9 01 81 10 60 a2 00 ea
0730: ea ca d0 fb 60 " );
    let mut cpu = Cpu::new(bus);
    cpu.run_until_interrupt(false);
    cpu.bus.print_ram(0x00, 0xff);
}

fn test_loop(loop_count : u32){
    let mut bus = Bus { ram:  [0; 65536]};
    loadProgram(&mut bus, 0x0600, "0600: a2 00 a0 00 a9 00 e8 c8 69 01 18 90 f9" );
    let mut cpu = Cpu::new(bus);

    let start = Instant::now();
    for addr in 0..loop_count {
        cpu.tick(false);
    }

    let elapsed = start.elapsed();
    println!("Ms: {}ms", elapsed.as_millis());
    println!("Clock speed: {}mhz", ((loop_count as f32) / (elapsed.as_millis() as f32) / 1000 as f32));
}


const WIDTH: usize = 32*16;
const HEIGHT: usize =  32*16;

fn main() {
    //test_LDA();
    //test_Stack();
    //test_Snake();
    //test_loop(100_000_000);

    let mut bus = Bus { ram:  [0; 65536]};

    loadProgram(&mut bus, 0x0600, "0600: 20 06 06 20 38 06 20 0d 06 20 2a 06 60 a9 02 85
0610: 02 a9 04 85 03 a9 11 85 10 a9 10 85 12 a9 0f 85
0620: 14 a9 04 85 11 85 13 85 15 60 a5 fe 85 00 a5 fe
0630: 29 03 18 69 02 85 01 60 20 4d 06 20 8d 06 20 c3
0640: 06 20 19 07 20 20 07 20 2d 07 4c 38 06 a5 ff c9
0650: 77 f0 0d c9 64 f0 14 c9 73 f0 1b c9 61 f0 22 60
0660: a9 04 24 02 d0 26 a9 01 85 02 60 a9 08 24 02 d0
0670: 1b a9 02 85 02 60 a9 01 24 02 d0 10 a9 04 85 02
0680: 60 a9 02 24 02 d0 05 a9 08 85 02 60 60 20 94 06
0690: 20 a8 06 60 a5 00 c5 10 d0 0d a5 01 c5 11 d0 07
06a0: e6 03 e6 03 20 2a 06 60 a2 02 b5 10 c5 10 d0 06
06b0: b5 11 c5 11 f0 09 e8 e8 e4 03 f0 06 4c aa 06 4c
06c0: 35 07 60 a6 03 ca 8a b5 10 95 12 ca 10 f9 a5 02
06d0: 4a b0 09 4a b0 19 4a b0 1f 4a b0 2f a5 10 38 e9
06e0: 20 85 10 90 01 60 c6 11 a9 01 c5 11 f0 28 60 e6
06f0: 10 a9 1f 24 10 f0 1f 60 a5 10 18 69 20 85 10 b0
0700: 01 60 e6 11 a9 06 c5 11 f0 0c 60 c6 10 a5 10 29
0710: 1f c9 1f f0 01 60 4c 35 07 a0 00 a5 fe 91 00 60
0720: a6 03 a9 00 81 10 a2 00 a9 01 81 10 60 a2 00 ea
0730: ea ca d0 fb 60 " );

    let mut cpu = Cpu::new(bus);

    let mut window = Window::new(
        "Snake - Press ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions {
            resize: true,
            scale_mode: ScaleMode::UpperLeft,
            ..WindowOptions::default()
        },
    )
        .expect("Unable to create window");

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut buffer: Vec<u32> = Vec::with_capacity(WIDTH * HEIGHT);

    let mut size = (0, 0);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let new_size = (window.get_size().0, window.get_size().1);
        if new_size != size {
            size = new_size;
            buffer.resize(size.0 * size.1, 0);
        }

        for addr in 0..200 {
            if cpu.pc == 0x00 {
                break;
            }
            cpu.tick(false);
        }

        let mut index : u32 = 0;
        for i in buffer.iter_mut() {

            let y : u32 = index / (WIDTH as u32) / 16;
            let x : u32 = index % (WIDTH as u32) / 16;

            let dest : u16 = 0x0200 + (x + y*(WIDTH as u32)/16) as u16;

            if cpu.bus.read_ram(dest) != 0{
                *i = (0xFF0000);
            }
            else{
                *i = (0x00);
            }

            index+=1;
        }

        window.get_keys().map(|keys| {
            for t in keys {
                match t {
                    Key::W => cpu.bus.write_ram(0x00FF, 0x77),
                    Key::A => cpu.bus.write_ram(0x00FF, 0x61),
                    Key::S => cpu.bus.write_ram(0x00FF, 0x73),
                    Key::D => cpu.bus.write_ram(0x00FF, 0x64),
                    _ => (),
                }
            }
        });

        window.get_keys_released().map(|keys| {
            for t in keys {
                match t {
                    _ => (),
                }
            }
        });

        window
            .update_with_buffer(&buffer, new_size.0, new_size.1)
            .unwrap();
    }
}
