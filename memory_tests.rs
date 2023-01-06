use crate::memory::Memory;

#[cfg(test)]
pub mod tests {
    use super::*; // Import the functions and types from the parent module

    #[test]
    pub fn test_non_banked_memory() {
        let mut memory = Memory::new();
        let test_addr = 0x2000; // Any address in the non-banked memory range (0x0000 - 0x3FFF)
        let test_val = 0xAA;

        // Write a value to the test address
        memory.write_byte(test_addr, test_val);

        // Read from the test address and verify the correct value is returned
        assert_eq!(memory.read_byte(test_addr), test_val);
    }

    #[test]
    pub fn test_banked_memory() {
        let mut memory = Memory::new();
        let test_addr = 0x4000; // Any address in the banked memory range (0x4000 - 0x7FFF)
        let test_val = 0xAA;

        // Write a value to the test address
        memory.write_byte(test_addr, test_val);

        // Change the current bank number and read from the same address
        memory.set_bank(1);
        assert_ne!(memory.read_byte(test_addr), test_val);

        // Change the current bank number back and verify the correct value is returned
        memory.set_bank(0);
        assert_eq!(memory.read_byte(test_addr), test_val);
    }

    #[test]
    pub fn test_io_registers() {
        let mut memory = Memory::new();
        let test_addr = 0xFF00; // Any address in the I/O registers range (0xFF00 - 0xFF7F)
        let test_val = 0xAA;

        // Write a value to the test address
        memory.write_byte(test_addr, test_val);

        // Read from the test address and verify the correct value is returned
        assert_eq!(memory.read_byte(test_addr), test_val);
    }

    #[test]
    pub fn test_read_write_word() {
        let mut memory = Memory::new();
        let test_addr = 0x2000; // Any address in the non-banked memory range (0x0000 - 0x3FFF)
        let test_val = 0xAABB;

        // Write a value to the test address using write_word
        memory.write_word(test_addr, test_val);

        // Read from the test address using read_word and verify the correct value is returned
        assert_eq!(memory.read_word(test_addr), test_val);
    }
}