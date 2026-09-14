//! Call-boundary SM83 execution for the cartridge's battle object routines.
//! No host timing, guessed motion, or implicit object-state transitions.
#[allow(dead_code)]
#[path = "battle_anim_program/mod.rs"]
pub mod program;

const Z: u8 = 0x80;
const N: u8 = 0x40;
const H: u8 = 0x20;
const C: u8 = 0x10;
pub const OBJECT_SIZE: usize = 24;
pub const OBJECT_COUNT: usize = 10;

pub struct Machine {
    memory: Box<[u8; 65536]>,
    r: [u8; 8], // B C D E H L (memory) A
    f: u8,
    pc: u16,
    sp: u16,
    pub instructions: u64,
    pub frameset_reset: bool,
    pub obp0_write: Option<u8>,
}

impl Machine {
    pub fn new(player: bool) -> Self {
        let mut result = Self {
            memory: vec![0; 65536].into_boxed_slice().try_into().unwrap(),
            r: [0; 8],
            f: 0,
            pc: 0,
            sp: 0xdffe,
            instructions: 0,
            frameset_reset: false,
            obp0_write: None,
        };
        result.memory[0x4000..0x8000].copy_from_slice(program::BANK);
        result.write(program::H_BATTLE_TURN, u8::from(!player));
        result.write(0xff70, 5);
        result
    }
    pub fn read(&self, address: u16) -> u8 {
        self.memory[usize::from(address)]
    }
    pub fn write(&mut self, address: u16, value: u8) {
        assert!(
            !(0x4000..0x8000).contains(&address),
            "battle program writes ROM {address:04x}"
        );
        self.memory[usize::from(address)] = value;
        if address == program::W_O_B_P0 {
            self.obp0_write = Some(value);
        }
    }
    pub fn object(&self, slot: usize) -> &[u8] {
        assert!(slot < OBJECT_COUNT);
        let start = usize::from(program::W_ACTIVE_ANIM_OBJECTS) + slot * OBJECT_SIZE;
        &self.memory[start..start + OBJECT_SIZE]
    }
    pub fn object_mut(&mut self, slot: usize) -> &mut [u8] {
        assert!(slot < OBJECT_COUNT);
        let start = usize::from(program::W_ACTIVE_ANIM_OBJECTS) + slot * OBJECT_SIZE;
        &mut self.memory[start..start + OBJECT_SIZE]
    }
    pub fn initialize(
        &mut self,
        slot: usize,
        id: u8,
        definition: [u8; 6],
        x: u8,
        y: u8,
        param: u8,
    ) {
        let object = self.object_mut(slot);
        object.fill(0);
        object[0] = id;
        object[1..7].copy_from_slice(&definition);
        object[7] = x;
        object[8] = y;
        object[11] = param;
        object[13] = 0xff;
    }
    pub fn clear_objects(&mut self) {
        let start = usize::from(program::W_ACTIVE_ANIM_OBJECTS);
        self.memory[start..start + 0xa0].fill(0);
    }
    pub fn step_object(&mut self, slot: usize) -> Result<(), String> {
        self.frameset_reset = false;
        self.call(
            program::DO_BATTLE_ANIM_FRAME,
            program::W_ACTIVE_ANIM_OBJECTS + (slot * OBJECT_SIZE) as u16,
        )
        .map(|_| ())
    }
    pub fn call(&mut self, address: u16, object: u16) -> Result<u8, String> {
        self.pc = address;
        self.sp = 0xdffe;
        self.set_pair(0, object);
        self.push(0xffff);
        for _ in 0..100_000 {
            if self.pc == 0xffff {
                return Ok(self.r[7]);
            }
            if !(0x4000..0x8000).contains(&self.pc) {
                return Err(format!(
                    "battle program escaped its instruction image at {:04x}",
                    self.pc
                ));
            }
            self.step()?;
        }
        Err(format!("battle routine {address:04x} did not return"))
    }
    pub fn install_data(&mut self, address: u16, data: &[u8]) {
        let start = usize::from(address);
        self.memory[start..start + data.len()].copy_from_slice(data);
    }
    pub fn begin_oam(&mut self) {
        self.write(program::W_BATTLE_ANIM_O_A_M_POINTER_LO, 0);
        let start = usize::from(program::W_SHADOW_O_A_M);
        self.memory[start..start + 160].fill(0);
    }
    pub fn oam_update(&mut self, slot: usize) -> Result<bool, String> {
        self.call(
            program::BATTLE_ANIM_O_A_M_UPDATE,
            program::W_ACTIVE_ANIM_OBJECTS + (slot * OBJECT_SIZE) as u16,
        )?;
        Ok(self.f & C != 0)
    }
    pub fn oam(&self) -> &[u8] {
        let start = usize::from(program::W_SHADOW_O_A_M);
        &self.memory[start..start + 160]
    }
    fn byte(&mut self) -> u8 {
        let value = self.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        value
    }
    fn word(&mut self) -> u16 {
        let lo = self.byte();
        u16::from_le_bytes([lo, self.byte()])
    }
    fn pair(&self, index: u8) -> u16 {
        if index == 3 {
            self.sp
        } else {
            let i = usize::from(index) * 2;
            u16::from_be_bytes([self.r[i], self.r[i + 1]])
        }
    }
    fn set_pair(&mut self, index: u8, value: u16) {
        if index == 3 {
            self.sp = value;
        } else {
            let i = usize::from(index) * 2;
            let [hi, lo] = value.to_be_bytes();
            self.r[i] = hi;
            self.r[i + 1] = lo;
        }
    }
    fn reg(&self, index: u8) -> u8 {
        if index == 6 {
            self.read(self.pair(2))
        } else {
            self.r[usize::from(index)]
        }
    }
    fn set_reg(&mut self, index: u8, value: u8) {
        if index == 6 {
            self.write(self.pair(2), value);
        } else {
            self.r[usize::from(index)] = value;
        }
    }
    fn push(&mut self, value: u16) {
        let [lo, hi] = value.to_le_bytes();
        self.sp = self.sp.wrapping_sub(1);
        self.write(self.sp, hi);
        self.sp = self.sp.wrapping_sub(1);
        self.write(self.sp, lo);
    }
    fn pop(&mut self) -> u16 {
        let lo = self.read(self.sp);
        self.sp = self.sp.wrapping_add(1);
        let hi = self.read(self.sp);
        self.sp = self.sp.wrapping_add(1);
        u16::from_le_bytes([lo, hi])
    }
    fn condition(&self, code: u8) -> bool {
        match code {
            0 => self.f & Z == 0,
            1 => self.f & Z != 0,
            2 => self.f & C == 0,
            3 => self.f & C != 0,
            _ => unreachable!(),
        }
    }
    fn alu(&mut self, operation: u8, value: u8) {
        let a = self.r[7];
        let carry = u8::from(self.f & C != 0 && (operation == 1 || operation == 3));
        let (result, flags) = match operation {
            0 | 1 => {
                let sum = u16::from(a) + u16::from(value) + u16::from(carry);
                (
                    sum as u8,
                    if sum > 255 { C } else { 0 }
                        | if (a & 15) + (value & 15) + carry > 15 {
                            H
                        } else {
                            0
                        },
                )
            }
            2 | 3 | 7 => (
                a.wrapping_sub(value).wrapping_sub(carry),
                N | if u16::from(a) < u16::from(value) + u16::from(carry) {
                    C
                } else {
                    0
                } | if (a & 15) < (value & 15) + carry {
                    H
                } else {
                    0
                },
            ),
            4 => (a & value, H),
            5 => (a ^ value, 0),
            6 => (a | value, 0),
            _ => unreachable!(),
        };
        self.f = flags | if result == 0 { Z } else { 0 };
        if operation != 7 {
            self.r[7] = result;
        }
    }
    fn step(&mut self) -> Result<(), String> {
        let address = self.pc;
        if address == program::REINIT_BATTLE_ANIM_FRAMESET {
            self.frameset_reset = true;
        }
        let op = self.byte();
        self.instructions += 1;
        match op {
            0x00 => {}
            0x01 | 0x11 | 0x21 | 0x31 => {
                let value = self.word();
                self.set_pair(op >> 4, value);
            }
            0x02 | 0x12 => self.write(self.pair(op >> 4), self.r[7]),
            0x0a | 0x1a => self.r[7] = self.read(self.pair(op >> 4)),
            0x03 | 0x13 | 0x23 | 0x33 => {
                let i = op >> 4;
                self.set_pair(i, self.pair(i).wrapping_add(1));
            }
            0x0b | 0x1b | 0x2b | 0x3b => {
                let i = op >> 4;
                self.set_pair(i, self.pair(i).wrapping_sub(1));
            }
            0x09 | 0x19 | 0x29 | 0x39 => {
                let a = self.pair(2);
                let b = self.pair(op >> 4);
                let result = u32::from(a) + u32::from(b);
                self.f = (self.f & Z)
                    | if (a & 0xfff) + (b & 0xfff) > 0xfff {
                        H
                    } else {
                        0
                    }
                    | if result > 0xffff { C } else { 0 };
                self.set_pair(2, result as u16);
            }
            _ if op < 0x40 && op & 7 == 4 => {
                let index = (op >> 3) & 7;
                let old = self.reg(index);
                let value = old.wrapping_add(1);
                self.f = (self.f & C)
                    | if value == 0 { Z } else { 0 }
                    | if old & 15 == 15 { H } else { 0 };
                self.set_reg(index, value);
            }
            _ if op < 0x40 && op & 7 == 5 => {
                let index = (op >> 3) & 7;
                let old = self.reg(index);
                let value = old.wrapping_sub(1);
                self.f = (self.f & C)
                    | N
                    | if value == 0 { Z } else { 0 }
                    | if old & 15 == 0 { H } else { 0 };
                self.set_reg(index, value);
            }
            _ if op < 0x40 && op & 7 == 6 => {
                let value = self.byte();
                self.set_reg((op >> 3) & 7, value);
            }
            0x07 | 0x0f | 0x17 | 0x1f => {
                let a = self.r[7];
                let carry = u8::from(self.f & C != 0);
                let (value, next_carry) = match op {
                    0x07 => (a.rotate_left(1), a >> 7),
                    0x0f => (a.rotate_right(1), a & 1),
                    0x17 => ((a << 1) | carry, a >> 7),
                    _ => ((a >> 1) | (carry << 7), a & 1),
                };
                self.r[7] = value;
                self.f = if next_carry != 0 { C } else { 0 };
            }
            0x18 | 0x20 | 0x28 | 0x30 | 0x38 => {
                let offset = self.byte() as i8;
                if op == 0x18 || self.condition((op - 0x20) >> 3) {
                    self.pc = self.pc.wrapping_add_signed(i16::from(offset));
                }
            }
            0x22 | 0x32 => {
                let hl = self.pair(2);
                self.write(hl, self.r[7]);
                self.set_pair(2, hl.wrapping_add_signed(if op == 0x22 { 1 } else { -1 }));
            }
            0x2a | 0x3a => {
                let hl = self.pair(2);
                self.r[7] = self.read(hl);
                self.set_pair(2, hl.wrapping_add_signed(if op == 0x2a { 1 } else { -1 }));
            }
            0x2f => {
                self.r[7] = !self.r[7];
                self.f |= N | H;
            }
            0x37 => self.f = (self.f & Z) | C,
            0x3f => self.f = (self.f & (Z | C)) ^ C,
            0x40..=0x7f if op != 0x76 => {
                let value = self.reg(op & 7);
                self.set_reg((op >> 3) & 7, value);
            }
            0x80..=0xbf => {
                let value = self.reg(op & 7);
                self.alu((op >> 3) & 7, value);
            }
            0xc6 | 0xce | 0xd6 | 0xde | 0xe6 | 0xee | 0xf6 | 0xfe => {
                let value = self.byte();
                self.alu((op >> 3) & 7, value);
            }
            0xc0 | 0xc8 | 0xd0 | 0xd8 => {
                if self.condition((op >> 3) & 3) {
                    self.pc = self.pop();
                }
            }
            0xc9 => self.pc = self.pop(),
            0xc1 | 0xd1 | 0xe1 | 0xf1 => {
                let value = self.pop();
                if op == 0xf1 {
                    self.r[7] = (value >> 8) as u8;
                    self.f = value as u8 & 0xf0;
                } else {
                    self.set_pair((op >> 4) & 3, value);
                }
            }
            0xc5 | 0xd5 | 0xe5 | 0xf5 => {
                let value = if op == 0xf5 {
                    u16::from_be_bytes([self.r[7], self.f])
                } else {
                    self.pair((op >> 4) & 3)
                };
                self.push(value);
            }
            0xc2 | 0xca | 0xd2 | 0xda | 0xc3 => {
                let target = self.word();
                if op == 0xc3 || self.condition((op >> 3) & 3) {
                    self.pc = target;
                }
            }
            0xc4 | 0xcc | 0xd4 | 0xdc | 0xcd => {
                let target = self.word();
                if op == 0xcd || self.condition((op >> 3) & 3) {
                    self.push(self.pc);
                    self.pc = target;
                }
            }
            0xe0 => {
                let offset = self.byte();
                self.write(0xff00 | u16::from(offset), self.r[7]);
            }
            0xf0 => {
                let offset = self.byte();
                self.r[7] = self.read(0xff00 | u16::from(offset));
            }
            0xe2 => self.write(0xff00 | u16::from(self.r[1]), self.r[7]),
            0xf2 => self.r[7] = self.read(0xff00 | u16::from(self.r[1])),
            0xea => {
                let target = self.word();
                self.write(target, self.r[7]);
            }
            0xfa => {
                let target = self.word();
                self.r[7] = self.read(target);
            }
            0xe9 => self.pc = self.pair(2),
            0xcb => {
                let instruction = self.byte();
                let index = instruction & 7;
                let value = self.reg(index);
                let bit = (instruction >> 3) & 7;
                match instruction >> 6 {
                    0 => {
                        let carry = u8::from(self.f & C != 0);
                        let (result, next_carry) = match bit {
                            0 => (value.rotate_left(1), value >> 7),
                            1 => (value.rotate_right(1), value & 1),
                            2 => ((value << 1) | carry, value >> 7),
                            3 => ((value >> 1) | (carry << 7), value & 1),
                            4 => (value << 1, value >> 7),
                            5 => ((value >> 1) | (value & 0x80), value & 1),
                            6 => (value.rotate_left(4), 0),
                            7 => (value >> 1, value & 1),
                            _ => unreachable!(),
                        };
                        self.f =
                            if result == 0 { Z } else { 0 } | if next_carry != 0 { C } else { 0 };
                        self.set_reg(index, result);
                    }
                    1 => self.f = (self.f & C) | H | if value & (1 << bit) == 0 { Z } else { 0 },
                    2 => self.set_reg(index, value & !(1 << bit)),
                    3 => self.set_reg(index, value | (1 << bit)),
                    _ => unreachable!(),
                }
            }
            _ => {
                return Err(format!(
                    "unsupported battle instruction {op:02x} at {address:04x}"
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "battle_anim_program/oracle.rs"]
mod oracle;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_object_routine_matches_rom_register_state_on_both_sides() {
        let mut covered = [false; 80];
        for (case, &(function, object, x, y, param, player)) in oracle::CASES.iter().enumerate() {
            covered[usize::from(function)] = true;
            let mut machine = Machine::new(player);
            machine.write(program::W_CUR_ITEM, 5);
            let mut definition = [0, 128, 0, function, 0, 0];
            if object != 255 {
                for (index, value) in definition.iter_mut().enumerate().take(5) {
                    *value = machine
                        .read(program::BATTLE_ANIM_OBJECTS + u16::from(object) * 6 + index as u16);
                }
            }
            machine.initialize(0, 1, definition, x, y, param);
            for frame in 0..oracle::FRAMES {
                if machine.object(0)[0] != 0 {
                    machine.step_object(0).unwrap_or_else(|error| {
                        panic!(
                            "case {case} {} param {param} frame {frame}: {error}",
                            program::FUNCTIONS[function as usize]
                        )
                    });
                }
                let mut actual = machine.object(0)[..17].to_vec();
                actual.extend(
                    [
                        program::W_O_B_P0,
                        program::H_L_C_D_C_POINTER,
                        program::H_L_Y_OVERRIDE_START,
                        program::H_L_Y_OVERRIDE_END,
                    ]
                    .map(|address| machine.read(address)),
                );
                let offset = (case * oracle::FRAMES + frame) * 21;
                assert_eq!(
                    actual,
                    oracle::RECORDS[offset..offset + 21],
                    "case {case} {} param {param} player {player} frame {frame}",
                    program::FUNCTIONS[function as usize]
                );
            }
        }
        assert!(covered.into_iter().all(|covered| covered));
    }
}

#[cfg(test)]
#[path = "battle_anim_program/oracle_oam.rs"]
pub(crate) mod oracle_oam;

#[cfg(test)]
mod oam_tests {
    use super::*;
    #[test]
    fn every_object_frameset_and_oam_matches_rom_on_both_sides() {
        for (case, &(function, object, x, y, param, player)) in oracle_oam::CASES.iter().enumerate()
        {
            let mut machine = Machine::new(player);
            machine.write(program::W_CUR_ITEM, 5);
            let mut definition = [0, 128, 0, function, 0, 0];
            if object != 255 {
                for (index, value) in definition.iter_mut().enumerate().take(5) {
                    *value = machine
                        .read(program::BATTLE_ANIM_OBJECTS + u16::from(object) * 6 + index as u16);
                }
            }
            machine.initialize(0, 1, definition, x, y, param);
            for frame in 0..oracle_oam::FRAMES {
                machine.begin_oam();
                if machine.object(0)[0] != 0 {
                    machine.step_object(0).unwrap();
                    machine.oam_update(0).unwrap();
                }
                let mut actual = machine.object(0)[..17].to_vec();
                actual.extend(
                    [
                        program::W_O_B_P0,
                        program::H_L_C_D_C_POINTER,
                        program::H_L_Y_OVERRIDE_START,
                        program::H_L_Y_OVERRIDE_END,
                    ]
                    .map(|address| machine.read(address)),
                );
                actual.extend(machine.oam());
                actual.push(machine.read(program::W_BATTLE_ANIM_O_A_M_POINTER_LO));
                let hash = actual
                    .into_iter()
                    .fold(14695981039346656037_u64, |hash, byte| {
                        (hash ^ u64::from(byte)).wrapping_mul(1099511628211)
                    });
                let offset = (case * oracle_oam::FRAMES + frame) * 8;
                assert_eq!(
                    hash.to_le_bytes(),
                    oracle_oam::RECORDS[offset..offset + 8],
                    "case {case} {} param {param} player {player} frame {frame}",
                    program::FUNCTIONS[function as usize]
                );
            }
        }
    }
}
