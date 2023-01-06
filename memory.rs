pub struct Memory {
    // 64KB of memory
    mem: [u8; 0x10000],
    // Current bank number for banked memory
    current_bank: u8,
    // I/O registers
    registers: [u8; 0x100],
    // MBC (Memory Bank Controller)
    mbc: MBC,
    // Interrupt enable register
    interrupt_enable: u8,
    // DMA (Direct Memory Access) transfer in progress
    dma_transfer: bool,
}

impl Memory {
    pub fn new() -> Memory {
        Memory { mem: [0; 0x10000], current_bank: 0, registers: [0; 0x100], mbc: MBC::None, interrupt_enable: 0, dma_transfer: false }
    }

    // Read a byte from memory at the given address, taking memory banking, I/O registers, and MBC into account
    pub fn read_byte(&self, addr: u16) -> u8 {
        if self.dma_transfer {
            return 0xFF;
        }if addr < 0x4000 {
            // Non-banked memory, always accessible
            self.mem[addr as usize]
        } else if addr < 0x8000 {
            // Banked memory, bank number is determined by current_bank and MBC
            let bank_offset = (self.mbc.get_bank(self.current_bank) as u16) * 0x4000;
            self.mem[(addr + bank_offset) as usize]
        } else if addr < 0xFF00 {
            // Remaining memory is not banked
            self.mem[addr as usize]
        } else if addr < 0xFFFF {
            // I/O registers
            self.registers[addr as usize - 0xFF00]
        } else {
            // Interrupt enable register
            self.interrupt_enable
        }
    }

    // Write a byte to memory at the given address, taking memory banking, I/O registers, and MBC into account
    pub fn write_byte(&mut self, addr: u16, val: u8) {
        if self.dma_transfer {
            return;
        }
    if addr < 0x4000 {
            // Non-banked memory, always accessible
            self.mem[addr as usize] = val;
        } else if addr < 0x8000 {
            // Banked memory, bank number is determined by MBC
            self.mbc.set_bank(addr, val, &mut self.mem);
        } else if addr < 0xFF00 {
            // Remaining memory is not banked
            self.mem[addr as usize] = val;
        } else if addr < 0xFFFF {
            // I/O registers
            self.registers[addr as usize - 0xFF00] = val;
        } else {
            // Interrupt enable register
            self.interrupt_enable = val;
        }
    }

// Set the current bank number for banked memory
pub fn set_bank(&mut self, bank: u8) {
    self.current_bank = bank;
}

// Read a word (2 bytes) from memory at the given address, taking memory banking, I/O registers, and MBC into account
pub fn read_word(&self, addr: u16) -> u16 {
    let low = self.read_byte(addr) as u16;
    let high = self.read_byte(addr + 1) as u16;
    (high << 8) | low
}

// Write a word (2 bytes) to memory at the given address, taking memory banking, I/O registers, and MBC into account
pub fn write_word(&mut self, addr: u16, val: u16) {
    self.write_byte(addr, (val & 0xFF) as u8);
    self.write_byte(addr + 1, (val >> 8) as u8);
}

// Check if an interrupt is enabled and should be triggered
pub fn check_interrupt(&self, interrupt: Interrupt) -> bool {
    let interrupt_flag = match interrupt {
        Interrupt::VBlank => 0x01,
        Interrupt::LCDStat => 0x02,
        Interrupt::Timer => 0x04,
        Interrupt::Serial => 0x08,
        Interrupt::Joypad => 0x10,
    };
    (self.interrupt_enable & interrupt_flag) != 0
}

// Trigger an interrupt
pub fn trigger_interrupt(&mut self, interrupt: Interrupt) {
    if !self.check_interrupt(interrupt) {
        return;
    }
    // Set the interrupt flag in memory
    let flag_addr = match interrupt {
        Interrupt::VBlank => 0xFF0F,
        Interrupt::LCDStat => 0xFF0F,
        Interrupt::Timer => 0xFF0F,
        Interrupt::Serial => 0xFF0F,
        Interrupt::Joypad => 0xFF0F,
    };
    let flag = self.read_byte(flag_addr);
    self.write_byte(flag_addr, flag | 0x01);
}

// Perform a DMA transfer
pub fn dma_transfer(&mut self, source: u8) {
    if self.dma_transfer {
        // DMA transfer already in progress
        return;
    }
    self.dma_transfer = true;
    let source_addr = (source as u16) << 8;
    for i in 0xFE00..0xFEA0 {
        self.write_byte(i, self.read_byte(source_addr + (i - 0xFE00)));
    }
    self.dma_transfer = false;
}
}

// MBC (Memory Bank Controller) enum
enum MBC {
None,
MBC1,
MBC2,
MBC3,
MBC5,
}
impl MBC {
    // Get the current bank number for banked memory
    pub fn get_bank(&self, bank: u8) -> u8 {
        match self {
            MBC::None => bank,
            MBC::MBC1 => {
                // MBC1 can use either the lower 5 bits of the bank number or the upper 2 bits, depending on the ROM/RAM mode
                if self.rom_mode {
                    bank & 0x1F
                } else {
                    (bank & 0x03) | (self.ram_bank as u8)
                }
            }
            MBC::MBC2 => bank & 0x0F,
            MBC::MBC3 => bank,
            MBC::MBC5 => bank,
        }
    }
    // Set the current bank number for banked memory
    pub fn set_bank(&mut self, addr: u16, val: u8, mem: &mut [u8]) {
        match self {
            MBC::None => {}
            MBC::MBC1 => {
                if addr < 0x2000 {
                    // Enable/disable RAM
                    self.ram_enable = val & 0x0F == 0x0A;
                } else if addr < 0x4000 {
                    // Set ROM bank number
                    self.rom_bank = (self.rom_bank & 0x60) | (val & 0x1F);
                } else if addr < 0x6000 {
                    // Set RAM bank number or ROM/RAM mode
                    if self.rom_mode {
                        self.rom_bank = (self.rom_bank & 0x1F) | ((val & 0x03) << 5);
                    } else {
                        self.ram_bank = val & 0x03;
                    }
                } else if addr < 0x8000 {
                    // Set ROM/RAM mode
                    self.rom_mode = val & 0x01 == 0x00;
                }
            }
            MBC::MBC2 => {
                if addr < 0x2000 {
                    // Enable/disable RAM
                    self.ram_enable = val & 0x0F == 0x0A;
                } else if addr < 0x4000 {
                    // Set ROM bank number (lower 4 bits only)
                    self.rom_bank = val & 0x0F;
                }
            }
            MBC::MBC3 => {
                if addr < 0x2000 {
                    // Enable/disable RAM
                    self.ram_enable = val & 0x0F == 0x0A;
                } else if addr < 0x4000 {
                    // Set ROM bank number
                    self.rom_bank = val & 0x7F;
                } else if addr < 0x6000 {
                    // Set RAM bank number
                    self.ram_bank = val & 0x03;
                }
            }
            MBC::MBC5 => {
                if addr < 0x2000 {
                    // Enable/disable RAM
                    self.ram_enable = val & 0x0F == 0x0A;
                } else if addr < 0x3000 {
                    // Set lower 8 bits of ROM bank number
                    self.rom_bank = (self.rom_bank & 0x0100) | val;
                } else if addr < 0x4000 {
                    // Set upper bit of ROM bank number
                    self.rom_bank = (self.rom_bank & 0x00FF) | ((val & 0x01) << 8);
                } else if addr < 0x6000 {
                    // Set RAM bank number
                    self.ram_bank = val & 0x0F;
                }
            }
        }
    }
}

// Interrupt enum
enum Interrupt {
    VBlank,
    LCDStat,
    Timer,
    Serial,
    Joypad,
}