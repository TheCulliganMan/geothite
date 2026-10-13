//! SM83 instruction execution and CGB timer/bus model for the Rust intro oracle.
//! Game bytes stay in the external source workspace. This helper is not a
//! runtime emulator and must not certify complete intro timing by itself.
pub(super) struct Bus {
    flat: bool,
    rom: std::sync::Arc<Vec<u8>>,
    ram: Vec<u8>,
    wram: Vec<u8>,
    vram: Vec<u8>,
    bank: usize,
    sram: usize,
    ticks: u64,
    dots: u64,
    double: bool,
    reload_delay: u8,
    reload_hold: u8,
}
impl Bus {
    pub(super) fn new(rom: impl Into<std::sync::Arc<Vec<u8>>>) -> Self {
        let mut b = Self {
            flat: false,
            rom: rom.into(),
            ram: vec![0; 65536],
            wram: vec![0; 32768],
            vram: vec![0; 16384],
            bank: 1,
            sram: 0,
            ticks: 0,
            dots: 0,
            double: false,
            reload_delay: 0,
            reload_hold: 0,
        };
        b.ram[0xff40] = 0x91;
        b.ram[0xff00] = 0xcf;
        b.ram[0xff70] = 1;
        b
    }
    pub(super) fn read(&self, a: u16) -> u8 {
        let a = a as usize;
        if self.flat {
            return self.ram[a];
        }
        match a {
            0..=0x3fff => self.rom[a],
            0x4000..=0x7fff => self.rom[(self.bank * 0x4000 + a - 0x4000) % self.rom.len()],
            0x8000..=0x9fff => self.vram[(self.ram[0xff4f] as usize & 1) * 8192 + a - 0x8000],
            0xc000..=0xcfff => self.wram[a - 0xc000],
            0xd000..=0xdfff => {
                self.wram[(self.ram[0xff70].max(1) as usize & 7) * 4096 + a - 0xd000]
            }
            0xe000..=0xfdff => self.read((a - 8192) as u16),
            0xff00 => self.ram[a] | 15,
            0xff04 => (self.ticks >> 8) as u8,
            0xff44 => {
                if self.ram[0xff40] & 128 != 0 {
                    (self.dots / 456 % 154) as u8
                } else {
                    0
                }
            }
            0xff41 => {
                let ly = self.dots / 456 % 154;
                let dot = self.dots % 456;
                self.ram[a] & 0x78
                    | if ly >= 144 {
                        1
                    } else if dot < 80 {
                        2
                    } else if dot < 250 {
                        3
                    } else {
                        0
                    }
                    | if ly as u8 == self.ram[0xff45] { 4 } else { 0 }
            }
            0xff4d => (if self.double { 128 } else { 0 }) | self.ram[a] & 1,
            _ => self.ram[a],
        }
    }
    pub(super) fn write(&mut self, a: u16, v: u8) {
        let a = a as usize;
        if self.flat {
            self.ram[a] = v;
            return;
        }
        match a {
            0x2000..=0x3fff => self.bank = (v as usize & 127).max(1),
            0x4000..=0x5fff => self.sram = v as usize,
            0..=0x7fff => (),
            0x8000..=0x9fff => {
                let bank = self.ram[0xff4f] as usize & 1;
                self.vram[bank * 8192 + a - 0x8000] = v
            }
            0xc000..=0xcfff => self.wram[a - 0xc000] = v,
            0xd000..=0xdfff => {
                let bank = self.ram[0xff70].max(1) as usize & 7;
                self.wram[bank * 4096 + a - 0xd000] = v
            }
            0xe000..=0xfdff => self.write((a - 8192) as u16, v),
            0xff04 => {
                let edge = self.timer_signal();
                self.ticks = 0;
                if edge {
                    self.increment_timer();
                }
            }
            0xff05 => {
                if self.reload_hold == 0 {
                    self.ram[a] = v;
                    self.reload_delay = 0;
                }
            }
            0xff06 => {
                self.ram[a] = v;
                if self.reload_hold != 0 {
                    self.ram[0xff05] = v;
                }
            }
            0xff07 => {
                let edge = self.timer_signal();
                self.ram[a] = v & 7;
                if edge && !self.timer_signal() {
                    self.increment_timer();
                }
            }
            0xff46 => {
                for i in 0..160 {
                    self.ram[0xfe00 + i] = self.read((v as u16) * 256 + i as u16);
                }
            }
            0xff55 => {
                let src = ((self.ram[0xff51] as u16) << 8) | ((self.ram[0xff52] & 240) as u16);
                let dst = 0x8000
                    | (((self.ram[0xff53] & 31) as u16) << 8)
                    | ((self.ram[0xff54] & 240) as u16);
                for i in 0..((v as u16 & 127) + 1) * 16 {
                    let value = self.read(src.wrapping_add(i));
                    self.write(dst.wrapping_add(i), value);
                }
                self.ram[a] = 255
            }
            0xff70 => self.ram[a] = v & 7,
            _ => self.ram[a] = v,
        }
    }
    fn timer_signal(&self) -> bool {
        let bit = [9, 3, 5, 7][(self.ram[0xff07] & 3) as usize];
        self.ram[0xff07] & 4 != 0 && self.ticks & (1 << bit) != 0
    }
    fn increment_timer(&mut self) {
        let (value, overflow) = self.ram[0xff05].overflowing_add(1);
        self.ram[0xff05] = value;
        if overflow {
            self.reload_delay = 4;
        }
    }
    fn advance(&mut self, t: u8) {
        if self.flat {
            return;
        }
        for _ in 0..t {
            if self.reload_hold != 0 {
                self.reload_hold -= 1;
            }
            if self.reload_delay != 0 {
                self.reload_delay -= 1;
                if self.reload_delay == 0 {
                    self.ram[0xff05] = self.ram[0xff06];
                    self.ram[0xff0f] |= 4;
                    self.reload_hold = 4;
                }
            }
            let previous = self.timer_signal();
            self.ticks += 1;
            if previous && !self.timer_signal() {
                self.increment_timer();
            }
            if (!self.double || self.ticks % 2 == 0) && self.ram[0xff40] & 128 != 0 {
                self.dots += 1;
                let ly = self.dots / 456 % 154;
                let dot = self.dots % 456;
                if ly == 144 && dot == 0 {
                    self.ram[0xff0f] |= 1;
                }
                let stat = self.ram[0xff41];
                if (ly < 144 && dot == 250 && stat & 8 != 0)
                    || (dot == 0 && ly < 144 && stat & 32 != 0)
                    || (dot == 0 && ly == 144 && stat & 16 != 0)
                    || (dot == 0 && ly as u8 == self.ram[0xff45] && stat & 64 != 0)
                {
                    self.ram[0xff0f] |= 2;
                }
            }
        }
    }
}
pub(super) struct Cpu {
    r: [u8; 8],
    f: u8,
    pc: u16,
    sp: u16,
    ime: bool,
    ei: u8,
    halt: bool,
    b: Bus,
    total: u64,
}
impl Cpu {
    /// Set the LCD phase only for isolated source-routine calibration. Runtime
    /// traces always advance the clock by executing instructions.
    pub(super) fn fixture_lcd_phase(&mut self, phase: u64) {
        self.b.dots = phase;
    }
    pub(super) fn fixture_write(&mut self, address: u16, value: u8) {
        self.b.write(address, value);
    }
    pub(super) fn fixture_registers(&mut self, values:&[(&str,u16)])->Result<(),String> {
        for &(name,value) in values {
            match name {
                "BC"=>self.setpair(0,value),"DE"=>self.setpair(1,value),"HL"=>self.setpair(2,value),
                "A" if value<=255=>self.r[7]=value as u8,
                "F" if value<=255&&value&15==0=>self.f=value as u8,
                _=>return Err(format!("unsupported timing fixture register {name}={value}")),
            }
        } Ok(())
    }
    pub(super) fn prepare_call(&mut self, bank: usize, address: u16, writes: &[(u16, u8)]) {
        self.b.bank = bank.max(1);
        self.pc = address;
        self.sp = 0xfffc;
        self.ime = false;
        self.ei = 0;
        self.halt = false;
        self.b.write(0xffff, 0);
        self.b.write(0xfffc, 0);
        self.b.write(0xfffd, 0);
        for &(address, value) in writes {
            self.b.write(address, value);
        }
    }
    pub(super) fn location(&self) -> (usize, u16) {
        (if self.pc < 0x4000 { 0 } else { self.b.bank }, self.pc)
    }
    pub(super) fn clock(&self) -> (u64, u64) {
        (self.total, self.b.dots)
    }
    pub(super) fn registers(&self) -> (u16, u16, u16, u16) {
        (self.pair(0), self.pair(1), self.hl(), self.sp)
    }
    pub(super) fn read(&self, address: u16) -> u8 {
        self.b.read(address)
    }
    pub(super) fn read_symbol(&self, bank: usize, address: u16) -> u8 {
        if (0xd000..=0xdfff).contains(&address) && bank != 0 {
            self.b.wram[bank * 4096 + usize::from(address - 0xd000)]
        } else {
            self.b.read(address)
        }
    }
    pub(super) fn accumulator(&self) -> u8 {
        self.r[7]
    }
    pub(super) fn rom_bank(&self) -> usize {
        self.b.bank
    }
    pub(super) fn timer_calendar(&self) -> Option<(u64, u64)> {
        if self.b.ram[0xff07] & 4 == 0 {
            return None;
        }
        let increment = 1u64 << ([9, 3, 5, 7][(self.b.ram[0xff07] & 3) as usize] + 1);
        let period = (256 - self.b.ram[0xff06] as u64) * increment;
        let remaining = if self.b.reload_delay != 0 {
            self.b.reload_delay as u64
        } else {
            (256 - self.b.ram[0xff05] as u64) * increment - self.b.ticks % increment + 4
        };
        Some((period, remaining))
    }
    pub(super) fn instruction_ready(&self) -> bool {
        !self.halt && !(self.ime && self.b.ram[0xffff] & self.b.ram[0xff0f] & 31 != 0)
    }
    pub(super) fn next_interrupt(&self) -> Option<u32> {
        let pending = self.b.ram[0xffff] & self.b.ram[0xff0f] & 31;
        (self.ime && pending != 0).then(|| pending.trailing_zeros())
    }
    pub(super) fn idle_wait(&self) -> bool {
        self.halt && self.b.ram[0xffff] & self.b.ram[0xff0f] & 31 == 0
    }
    pub(super) fn new(rom: impl Into<std::sync::Arc<Vec<u8>>>) -> Self {
        Self {
            r: [0, 0, 255, 86, 0, 13, 0, 17],
            f: 128,
            pc: 256,
            sp: 65534,
            ime: false,
            ei: 0,
            halt: false,
            b: Bus::new(rom),
            total: 0,
        }
    }
    fn hl(&self) -> u16 {
        u16::from_be_bytes([self.r[4], self.r[5]])
    }
    fn read_cycle(&mut self, a: u16) -> u8 {
        self.tick(1);
        self.b.read(a)
    }
    fn write_cycle(&mut self, a: u16, v: u8) {
        self.tick(1);
        self.b.write(a, v)
    }
    fn get(&mut self, r: usize) -> u8 {
        if r == 6 {
            self.read_cycle(self.hl())
        } else {
            self.r[r]
        }
    }
    fn set(&mut self, r: usize, v: u8) {
        if r == 6 {
            self.write_cycle(self.hl(), v)
        } else {
            self.r[r] = v
        }
    }
    fn pair(&self, p: usize) -> u16 {
        if p == 3 {
            self.sp
        } else {
            u16::from_be_bytes([self.r[p * 2], self.r[p * 2 + 1]])
        }
    }
    fn setpair(&mut self, p: usize, v: u16) {
        if p == 3 {
            self.sp = v
        } else {
            let [h, l] = v.to_be_bytes();
            self.r[p * 2] = h;
            self.r[p * 2 + 1] = l;
        }
    }
    fn byte(&mut self) -> u8 {
        let v = self.read_cycle(self.pc);
        self.pc = self.pc.wrapping_add(1);
        v
    }
    fn word(&mut self) -> u16 {
        let l = self.byte();
        let h = self.byte();
        u16::from_le_bytes([l, h])
    }
    fn push(&mut self, v: u16) {
        let [h, l] = v.to_be_bytes();
        self.sp = self.sp.wrapping_sub(1);
        self.write_cycle(self.sp, h);
        self.sp = self.sp.wrapping_sub(1);
        self.write_cycle(self.sp, l)
    }
    fn pop(&mut self) -> u16 {
        let l = self.read_cycle(self.sp);
        self.sp = self.sp.wrapping_add(1);
        let h = self.read_cycle(self.sp);
        self.sp = self.sp.wrapping_add(1);
        u16::from_le_bytes([l, h])
    }
    fn cond(&self, c: usize) -> bool {
        match c {
            0 => self.f & 128 == 0,
            1 => self.f & 128 != 0,
            2 => self.f & 16 == 0,
            _ => self.f & 16 != 0,
        }
    }
    fn alu(&mut self, op: usize, v: u8) {
        let a = self.r[7];
        let carry = if op == 1 || op == 3 {
            (self.f >> 4) & 1
        } else {
            0
        };
        match op {
            0 | 1 => {
                let sum = a as u16 + v as u16 + carry as u16;
                let half = (a & 15) + (v & 15) + carry > 15;
                self.r[7] = sum as u8;
                self.f = if sum as u8 == 0 { 128 } else { 0 }
                    | if half { 32 } else { 0 }
                    | if sum > 255 { 16 } else { 0 };
            }
            2 | 3 | 7 => {
                let result = a.wrapping_sub(v).wrapping_sub(carry);
                self.f = 64
                    | if result == 0 { 128 } else { 0 }
                    | if (a & 15) < (v & 15) + carry { 32 } else { 0 }
                    | if (a as u16) < v as u16 + carry as u16 {
                        16
                    } else {
                        0
                    };
                if op != 7 {
                    self.r[7] = result;
                }
            }
            4 => {
                self.r[7] &= v;
                self.f = 32 | if self.r[7] == 0 { 128 } else { 0 };
            }
            5 => {
                self.r[7] ^= v;
                self.f = if self.r[7] == 0 { 128 } else { 0 };
            }
            6 => {
                self.r[7] |= v;
                self.f = if self.r[7] == 0 { 128 } else { 0 };
            }
            _ => unreachable!(),
        }
    }
    fn cb(&mut self) -> u8 {
        let op = self.byte();
        let x = op >> 6;
        let y = (op >> 3) & 7;
        let z = (op & 7) as usize;
        let value = self.get(z);
        match x {
            0 => {
                let carry = (self.f >> 4) & 1;
                let (v, c) = match y {
                    0 => (value.rotate_left(1), value >> 7),
                    1 => (value.rotate_right(1), value & 1),
                    2 => ((value << 1) | carry, value >> 7),
                    3 => ((value >> 1) | (carry << 7), value & 1),
                    4 => (value << 1, value >> 7),
                    5 => ((value >> 1) | (value & 128), value & 1),
                    6 => (value.rotate_left(4), 0),
                    _ => (value >> 1, value & 1),
                };
                self.set(z, v);
                self.f = if v == 0 { 128 } else { 0 } | if c != 0 { 16 } else { 0 };
            }
            1 => self.f = (self.f & 16) | 32 | if value & (1 << y) == 0 { 128 } else { 0 },
            2 => self.set(z, value & !(1 << y)),
            _ => self.set(z, value | (1 << y)),
        }
        if z == 6 {
            if x == 1 { 3 } else { 4 }
        } else {
            2
        }
    }
    pub(super) fn step(&mut self) -> Result<(), String> {
        let irq = self.b.ram[0xffff] & self.b.ram[0xff0f] & 31;
        if irq != 0 {
            self.halt = false;
            if self.ime {
                let bit = irq.trailing_zeros();
                self.b.ram[0xff0f] &= !(1 << bit);
                self.ime = false;
                self.push(self.pc);
                self.pc = 0x40 + bit as u16 * 8;
                self.tick(3);
                return Ok(());
            }
        }
        if self.halt {
            let mut t = 4;
            if self.b.ram[0xff40] & 128 != 0 && !self.b.double {
                let dot = self.b.dots % 456;
                for event in [250, 456] {
                    if dot < event && event - dot < 4 {
                        t = (event - dot) as u8;
                    }
                }
            }
            self.total += t as u64;
            self.b.advance(t);
            return Ok(());
        }
        let step_start = self.total;
        let addr = self.pc;
        let op = self.byte();
        let x = op >> 6;
        let y = ((op >> 3) & 7) as usize;
        let z = (op & 7) as usize;
        let p = y >> 1;
        let q = y & 1;
        let cycles = match x {
            0 => match z {
                0 => match y {
                    0 => 1,
                    1 => {
                        let a = self.word();
                        let [l, h] = self.sp.to_le_bytes();
                        self.write_cycle(a, l);
                        self.write_cycle(a.wrapping_add(1), h);
                        5
                    }
                    2 => {
                        self.pc = self.pc.wrapping_add(1);
                        if self.b.ram[0xff4d] & 1 != 0 {
                            self.b.double = !self.b.double;
                            self.b.ram[0xff4d] &= !1;
                        } else {
                            self.halt = true;
                        }
                        1
                    }
                    3 => {
                        let d = self.byte() as i8;
                        self.pc = self.pc.wrapping_add_signed(d as i16);
                        3
                    }
                    _ => {
                        let d = self.byte() as i8;
                        if self.cond(y - 4) {
                            self.pc = self.pc.wrapping_add_signed(d as i16);
                            3
                        } else {
                            2
                        }
                    }
                },
                1 => {
                    if q == 0 {
                        let v = self.word();
                        self.setpair(p, v);
                        3
                    } else {
                        let a = self.hl();
                        let b = self.pair(p);
                        let sum = a as u32 + b as u32;
                        self.f = (self.f & 128)
                            | if (a & 4095) + (b & 4095) > 4095 {
                                32
                            } else {
                                0
                            }
                            | if sum > 65535 { 16 } else { 0 };
                        self.setpair(2, sum as u16);
                        2
                    }
                }
                2 => {
                    let a = match p {
                        0 => self.pair(0),
                        1 => self.pair(1),
                        _ => self.hl(),
                    };
                    if q == 0 {
                        self.write_cycle(a, self.r[7])
                    } else {
                        self.r[7] = self.read_cycle(a)
                    }
                    if p == 2 {
                        self.setpair(2, a.wrapping_add(1))
                    }
                    if p == 3 {
                        self.setpair(2, a.wrapping_sub(1))
                    }
                    2
                }
                3 => {
                    let a = self.pair(p);
                    self.setpair(
                        p,
                        if q == 0 {
                            a.wrapping_add(1)
                        } else {
                            a.wrapping_sub(1)
                        },
                    );
                    2
                }
                4 | 5 => {
                    let a = self.get(y);
                    let v = if z == 4 {
                        a.wrapping_add(1)
                    } else {
                        a.wrapping_sub(1)
                    };
                    self.set(y, v);
                    self.f = (self.f & 16)
                        | if v == 0 { 128 } else { 0 }
                        | if z == 5 { 64 } else { 0 }
                        | if (z == 4 && a & 15 == 15) || (z == 5 && a & 15 == 0) {
                            32
                        } else {
                            0
                        };
                    if y == 6 { 3 } else { 1 }
                }
                6 => {
                    let v = self.byte();
                    self.set(y, v);
                    if y == 6 { 3 } else { 2 }
                }
                _ => {
                    let a = self.r[7];
                    match y {
                        0 => {
                            self.r[7] = a.rotate_left(1);
                            self.f = (a >> 7) * 16
                        }
                        1 => {
                            self.r[7] = a.rotate_right(1);
                            self.f = (a & 1) * 16
                        }
                        2 => {
                            self.r[7] = (a << 1) | ((self.f >> 4) & 1);
                            self.f = (a >> 7) * 16
                        }
                        3 => {
                            self.r[7] = (a >> 1) | ((self.f & 16) << 3);
                            self.f = (a & 1) * 16
                        }
                        4 => {
                            let mut adjust = 0;
                            let mut carry = self.f & 16 != 0;
                            if self.f & 64 == 0 {
                                if carry || a > 0x99 {
                                    adjust |= 0x60;
                                    carry = true;
                                }
                                if self.f & 32 != 0 || a & 15 > 9 {
                                    adjust |= 6;
                                }
                                self.r[7] = a.wrapping_add(adjust);
                            } else {
                                if carry {
                                    adjust |= 0x60
                                }
                                if self.f & 32 != 0 {
                                    adjust |= 6
                                }
                                self.r[7] = a.wrapping_sub(adjust);
                            }
                            self.f = (self.f & 64)
                                | if self.r[7] == 0 { 128 } else { 0 }
                                | if carry { 16 } else { 0 };
                        }
                        5 => {
                            self.r[7] = !a;
                            self.f |= 96;
                        }
                        6 => self.f = (self.f & 128) | 16,
                        _ => self.f = (self.f & 128) | if self.f & 16 == 0 { 16 } else { 0 },
                    }
                    1
                }
            },
            1 => {
                if y == 6 && z == 6 {
                    self.halt = true;
                    1
                } else {
                    let value = self.get(z);
                    self.set(y, value);
                    if y == 6 || z == 6 { 2 } else { 1 }
                }
            }
            2 => {
                let value = self.get(z);
                self.alu(y, value);
                if z == 6 { 2 } else { 1 }
            }
            _ => match z {
                0 => {
                    if y < 4 {
                        if self.cond(y) {
                            self.pc = self.pop();
                            5
                        } else {
                            2
                        }
                    } else if y == 4 || y == 6 {
                        let a = 0xff00 | self.byte() as u16;
                        if y == 4 {
                            self.write_cycle(a, self.r[7]);
                        } else {
                            self.r[7] = self.read_cycle(a);
                        }
                        3
                    } else {
                        let e = self.byte();
                        let sp = self.sp;
                        let value = sp.wrapping_add_signed(e as i8 as i16);
                        self.f = if (sp & 15) + (e as u16 & 15) > 15 {
                            32
                        } else {
                            0
                        } | if (sp & 255) + e as u16 > 255 { 16 } else { 0 };
                        if y == 5 {
                            self.sp = value;
                            4
                        } else {
                            self.setpair(2, value);
                            3
                        }
                    }
                }
                1 => {
                    if q == 0 {
                        let v = self.pop();
                        if p == 3 {
                            let [a, f] = v.to_be_bytes();
                            self.r[7] = a;
                            self.f = f & 240
                        } else {
                            self.setpair(p, v)
                        }
                        3
                    } else {
                        match p {
                            0 => {
                                self.pc = self.pop();
                                4
                            }
                            1 => {
                                self.pc = self.pop();
                                self.ime = true;
                                4
                            }
                            2 => {
                                self.pc = self.hl();
                                1
                            }
                            _ => {
                                self.sp = self.hl();
                                2
                            }
                        }
                    }
                }
                2 => {
                    if y < 4 {
                        let a = self.word();
                        if self.cond(y) {
                            self.pc = a;
                            4
                        } else {
                            3
                        }
                    } else {
                        let a = if y == 4 || y == 6 {
                            0xff00 | self.r[1] as u16
                        } else {
                            self.word()
                        };
                        if y < 6 {
                            self.write_cycle(a, self.r[7]);
                        } else {
                            self.r[7] = self.read_cycle(a);
                        }
                        if y == 4 || y == 6 { 2 } else { 4 }
                    }
                }
                3 => match y {
                    0 => {
                        self.pc = self.word();
                        4
                    }
                    1 => self.cb(),
                    6 => {
                        self.ime = false;
                        self.ei = 0;
                        1
                    }
                    7 => {
                        self.ei = 2;
                        1
                    }
                    _ => return Err(format!("illegal {op:02x} at {addr:04x}")),
                },
                4 => {
                    if y < 4 {
                        let a = self.word();
                        if self.cond(y) {
                            self.push(self.pc);
                            self.pc = a;
                            6
                        } else {
                            3
                        }
                    } else {
                        return Err(format!("illegal {op:02x}"));
                    }
                }
                5 => {
                    if q == 0 {
                        let v = if p == 3 {
                            u16::from_be_bytes([self.r[7], self.f])
                        } else {
                            self.pair(p)
                        };
                        self.push(v);
                        4
                    } else if p == 0 {
                        let a = self.word();
                        self.push(self.pc);
                        self.pc = a;
                        6
                    } else {
                        return Err(format!("illegal {op:02x}"));
                    }
                }
                6 => {
                    let v = self.byte();
                    self.alu(y, v);
                    2
                }
                _ => {
                    self.push(self.pc);
                    self.pc = y as u16 * 8;
                    4
                }
            },
        };
        let elapsed = self.total - step_start;
        assert!(
            elapsed <= cycles as u64 * 4,
            "opcode {op:02x} cost mismatch: {elapsed}>{}",
            cycles as u64 * 4
        );
        self.b.advance((cycles as u64 * 4 - elapsed) as u8);
        self.total = step_start + cycles as u64 * 4;
        if self.ei != 0 {
            self.ei -= 1;
            if self.ei == 0 {
                self.ime = true;
            }
        }
        Ok(())
    }
    fn tick(&mut self, m: u8) {
        self.total += m as u64 * 4;
        self.b.advance(m * 4);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[test]
    fn independent_cpu_vectors() {
        let mut count = 0;
        let Some(root) = std::env::var_os("CRYSTAL_SM83_TEST_ROOT") else {
            return;
        };
        for file in fs::read_dir(root).unwrap() {
            let file = file.unwrap().path(); // STOP/HALT fixtures include idle cycles rather than one instruction.
            if matches!(
                file.file_name().and_then(|name| name.to_str()),
                Some("10.json" | "76.json")
            ) {
                continue;
            }
            if file.extension().is_none_or(|v| v != "json") {
                continue;
            }
            let vectors: Vec<serde_json::Value> =
                serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
            for test in vectors {
                let initial = &test["initial"];
                let final_ = &test["final"];
                let mut cpu = Cpu::new(vec![0; 32768]);
                cpu.b.flat = true;
                cpu.pc = initial["pc"].as_u64().unwrap() as u16;
                cpu.sp = initial["sp"].as_u64().unwrap() as u16;
                cpu.f = initial["f"].as_u64().unwrap() as u8;
                for (index, name) in ["b", "c", "d", "e", "h", "l", "unused", "a"]
                    .iter()
                    .enumerate()
                {
                    if index != 6 {
                        cpu.r[index] = initial[name].as_u64().unwrap() as u8;
                    }
                }
                for pair in initial["ram"].as_array().unwrap() {
                    cpu.b.ram[pair[0].as_u64().unwrap() as usize] = pair[1].as_u64().unwrap() as u8;
                }
                cpu.step().unwrap();
                assert_eq!(
                    cpu.total,
                    test["cycles"].as_array().unwrap().len() as u64 * 4,
                    "{} cycles",
                    test["name"]
                );
                for (index, name) in ["b", "c", "d", "e", "h", "l", "unused", "a"]
                    .iter()
                    .enumerate()
                {
                    if index != 6 {
                        assert_eq!(
                            cpu.r[index],
                            final_[name].as_u64().unwrap() as u8,
                            "{} register {}",
                            test["name"],
                            name
                        );
                    }
                }
                assert_eq!(
                    cpu.f,
                    final_["f"].as_u64().unwrap() as u8,
                    "{} flags",
                    test["name"]
                );
                assert_eq!(
                    cpu.pc,
                    final_["pc"].as_u64().unwrap() as u16,
                    "{} PC",
                    test["name"]
                );
                assert_eq!(
                    cpu.sp,
                    final_["sp"].as_u64().unwrap() as u16,
                    "{} SP",
                    test["name"]
                );
                for pair in final_["ram"].as_array().unwrap() {
                    assert_eq!(
                        cpu.b.ram[pair[0].as_u64().unwrap() as usize],
                        pair[1].as_u64().unwrap() as u8,
                        "{} RAM",
                        test["name"]
                    );
                }
                count += 1;
            }
        }
        println!("{} independent CPU vectors passed", count);
    }
    #[test]
    fn timer_delayed_reload_and_write_windows() {
        let mut b = Bus::new(vec![0; 32768]);
        b.ram[0xff40] = 0;
        b.ram[0xff07] = 5;
        b.ram[0xff05] = 255;
        b.ram[0xff06] = 42;
        b.advance(16);
        assert_eq!(b.ram[0xff05], 0);
        assert_eq!(b.ram[0xff0f] & 4, 0);
        b.advance(3);
        assert_eq!(b.ram[0xff05], 0);
        b.advance(1);
        assert_eq!(b.ram[0xff05], 42);
        assert_eq!(b.ram[0xff0f] & 4, 4);
        b.write(0xff05, 99);
        assert_eq!(b.ram[0xff05], 42);
        b.write(0xff06, 17);
        assert_eq!(b.ram[0xff05], 17);
        b.advance(4);
        b.write(0xff05, 99);
        assert_eq!(b.ram[0xff05], 99);
    }
    #[test]
    fn timer_pending_reload_cancels_and_div_edge_increments() {
        let mut b = Bus::new(vec![0; 32768]);
        b.ram[0xff40] = 0;
        b.ram[0xff07] = 5;
        b.ram[0xff05] = 255;
        b.advance(16);
        b.write(0xff05, 99);
        b.advance(4);
        assert_eq!(b.ram[0xff05], 99);
        assert_eq!(b.ram[0xff0f] & 4, 0);
        b.advance(4);
        assert!(b.timer_signal());
        b.write(0xff04, 0);
        assert_eq!(b.ram[0xff05], 100);
        assert_eq!(b.ticks, 0);
    }
}
