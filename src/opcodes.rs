//! FLUX VM opcode definitions.
//! Based on the cuda-instruction-set ISA.

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    NOP             = 0x00,
    HALT            = 0x01,
    LOAD_CONST      = 0x10,
    LOAD_REG        = 0x11,
    STORE_REG       = 0x12,
    CMP             = 0x20,
    JLT             = 0x21,
    JGT             = 0x22,
    JEQ             = 0x23,
    JNE             = 0x24,
    JMP             = 0x25,
    JLE             = 0x26,
    JGE             = 0x27,
    ADD             = 0x30,
    SUB             = 0x31,
    MUL             = 0x32,
    DIV             = 0x33,
    CONF_GET        = 0x40,
    CONF_SET        = 0x41,
    CONF_MUL        = 0x42,
    CONF_ADD        = 0x43,
    TRUST_COMPARE   = 0x50,
    TRUST_FLOOR     = 0x51,
    ENERGY_REPORT   = 0x60,
    ENERGY_READ     = 0x61,
    INST_MODULATE   = 0x70,
    INST_QUERY      = 0x71,
    DELEGATE        = 0x80,
    REPLY           = 0x81,
    PROCESS_TASK    = 0x82,
    READ_SENSOR     = 0x90,
    SETUP_CONST     = 0xA0,
    // Pseudo-ops for string storage
    STORE_STRING    = 0xF0,
}

impl Opcode {
    pub fn to_byte(self) -> u8 {
        self as u8
    }
}
