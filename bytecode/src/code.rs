pub const LOAD: u8 = 0x00;
pub const LOAD_SUPER: u8 = 0x01;
pub const LOAD_FIELD: u8 = 0x02;
pub const LOAD_ITEM: u8 = 0x03;
pub const LOAD_SLICE: u8 = 0x04;

pub const STORE: u8 = 0x08;
pub const STORE_SUPER: u8 = 0x09;
pub const STORE_FIELD: u8 = 0x0a;
pub const STORE_ITEM: u8 = 0x0b;
pub const STORE_SLICE: u8 = 0x0c;

pub const DUP: u8 = 0x10;
pub const DUP_PRE2: u8 = 0x11;
pub const DUP_PRE3: u8 = 0x12;
pub const DUP_PRE4: u8 = 0x13;
pub const POP: u8 = 0x14;

pub const PUSH_NULL: u8 = 0x20;
pub const PUSH_INT: u8 = 0x21;
pub const PUSH_CONST: u8 = 0x22;
pub const NEW_ARRAY: u8 = 0x23;

pub const PUSH_ARG: u8 = 0x28;
pub const PUSH_SELF: u8 = 0x29;
pub const PUSH_SUPER: u8 = 0x2a;
pub const PUSH_CLOSURE: u8 = 0x2b;

pub const JMP: u8 = 0x30;
pub const JN: u8 = 0x31;
pub const JT: u8 = 0x32;
pub const JF: u8 = 0x33;

pub const CALL: u8 = 0x38;
pub const RETURN: u8 = 0x39;

pub const ADD: u8 = 0x40;
pub const SUB: u8 = 0x41;
pub const MUL: u8 = 0x42;
pub const DIV: u8 = 0x43;
pub const MOD: u8 = 0x44;
pub const NEG: u8 = 0x45;

pub const CMP_EQ: u8 = 0x48;
pub const CMP_NE: u8 = 0x49;
pub const CMP_GT: u8 = 0x4a;
pub const CMP_LT: u8 = 0x4b;
pub const CMP_GE: u8 = 0x4c;
pub const CMP_LE: u8 = 0x4d;
pub const NOT: u8 = 0x4e;

pub const BAND: u8 = 0x50;
pub const BOR: u8 = 0x51;
pub const BXOR: u8 = 0x52;
pub const BINV: u8 = 0x53;
pub const SHL: u8 = 0x54;
pub const SHR: u8 = 0x55;

pub const TYPE: u8 = 0x58;
pub const LEN: u8 = 0x59;

pub const IN: u8 = 0x60;
pub const OUT: u8 = 0x61;
pub const LOAD_LIB: u8 = 0x62;

pub struct CodeInfo {
    pub name: &'static str,
    pub params: u32
}

pub const CODE_INFO: &[CodeInfo] = &[
    CodeInfo { name: "LOAD", params: 1 },
    CodeInfo { name: "LOAD_SUPER", params: 1 },
    CodeInfo { name: "LOAD_FIELD", params: 1 },
    CodeInfo { name: "LOAD_ITEM", params: 0 },
    CodeInfo { name: "LOAD_SLICE", params: 0 },
    CodeInfo { name: "0x05", params: 0 },
    CodeInfo { name: "0x06", params: 0 },
    CodeInfo { name: "0x07", params: 0 },
    CodeInfo { name: "STORE", params: 1 },
    CodeInfo { name: "STORE_SUPER", params: 1 },
    CodeInfo { name: "STORE_FIELD", params: 1 },
    CodeInfo { name: "STORE_ITEM", params: 0 },
    CodeInfo { name: "STORE_SLICE", params: 0 },
    CodeInfo { name: "0x0d", params: 0 },
    CodeInfo { name: "0x0e", params: 0 },
    CodeInfo { name: "0x0f", params: 0 },
    CodeInfo { name: "DUP", params: 0 },
    CodeInfo { name: "DUP_PRE2", params: 0 },
    CodeInfo { name: "DUP_PRE3", params: 0 },
    CodeInfo { name: "DUP_PRE4", params: 0 },
    CodeInfo { name: "POP", params: 0 },
    CodeInfo { name: "0x15", params: 0 },
    CodeInfo { name: "0x16", params: 0 },
    CodeInfo { name: "0x17", params: 0 },
    CodeInfo { name: "0x18", params: 0 },
    CodeInfo { name: "0x19", params: 0 },
    CodeInfo { name: "0x1a", params: 0 },
    CodeInfo { name: "0x1b", params: 0 },
    CodeInfo { name: "0x1c", params: 0 },
    CodeInfo { name: "0x1d", params: 0 },
    CodeInfo { name: "0x1e", params: 0 },
    CodeInfo { name: "0x1f", params: 0 },
    CodeInfo { name: "PUSH_NULL", params: 0 },
    CodeInfo { name: "PUSH_INT", params: 1 },
    CodeInfo { name: "PUSH_CONST", params: 1 },
    CodeInfo { name: "NEW_ARRAY", params: 1 },
    CodeInfo { name: "0x24", params: 0 },
    CodeInfo { name: "0x25", params: 0 },
    CodeInfo { name: "0x26", params: 0 },
    CodeInfo { name: "0x27", params: 0 },
    CodeInfo { name: "PUSH_ARG", params: 1 },
    CodeInfo { name: "PUSH_SELF", params: 0 },
    CodeInfo { name: "PUSH_SUPER", params: 1 },
    CodeInfo { name: "PUSH_CLOSURE", params: 1 },
    CodeInfo { name: "0x2c", params: 0 },
    CodeInfo { name: "0x2d", params: 0 },
    CodeInfo { name: "0x2e", params: 0 },
    CodeInfo { name: "0x2f", params: 0 },
    CodeInfo { name: "JMP", params: 1 },
    CodeInfo { name: "JN", params: 1 },
    CodeInfo { name: "JT", params: 1 },
    CodeInfo { name: "JF", params: 1 },
    CodeInfo { name: "0x34", params: 0 },
    CodeInfo { name: "0x35", params: 0 },
    CodeInfo { name: "0x36", params: 0 },
    CodeInfo { name: "0x37", params: 0 },
    CodeInfo { name: "CALL", params: 1 },
    CodeInfo { name: "RETURN", params: 0 },
    CodeInfo { name: "0x3a", params: 0 },
    CodeInfo { name: "0x3b", params: 0 },
    CodeInfo { name: "0x3c", params: 0 },
    CodeInfo { name: "0x3d", params: 0 },
    CodeInfo { name: "0x3e", params: 0 },
    CodeInfo { name: "0x3f", params: 0 },
    CodeInfo { name: "ADD", params: 0 },
    CodeInfo { name: "SUB", params: 0 },
    CodeInfo { name: "MUL", params: 0 },
    CodeInfo { name: "DIV", params: 0 },
    CodeInfo { name: "MOD", params: 0 },
    CodeInfo { name: "NEG", params: 0 },
    CodeInfo { name: "0x46", params: 0 },
    CodeInfo { name: "0x47", params: 0 },
    CodeInfo { name: "CMP_EQ", params: 0 },
    CodeInfo { name: "CMP_NE", params: 0 },
    CodeInfo { name: "CMP_GT", params: 0 },
    CodeInfo { name: "CMP_LT", params: 0 },
    CodeInfo { name: "CMP_GE", params: 0 },
    CodeInfo { name: "CMP_LE", params: 0 },
    CodeInfo { name: "NOT", params: 0 },
    CodeInfo { name: "0x4f", params: 0 },
    CodeInfo { name: "BAND", params: 0 },
    CodeInfo { name: "BOR", params: 0 },
    CodeInfo { name: "BXOR", params: 0 },
    CodeInfo { name: "BINV", params: 0 },
    CodeInfo { name: "SHL", params: 0 },
    CodeInfo { name: "SHR", params: 0 },
    CodeInfo { name: "0x56", params: 0 },
    CodeInfo { name: "0x57", params: 0 },
    CodeInfo { name: "TYPE", params: 0 },
    CodeInfo { name: "LEN", params: 0 },
    CodeInfo { name: "0x5a", params: 0 },
    CodeInfo { name: "0x5b", params: 0 },
    CodeInfo { name: "0x5c", params: 0 },
    CodeInfo { name: "0x5d", params: 0 },
    CodeInfo { name: "0x5e", params: 0 },
    CodeInfo { name: "0x5f", params: 0 },
    CodeInfo { name: "IN", params: 0 },
    CodeInfo { name: "OUT", params: 0 },
    CodeInfo { name: "LOAD_LIB", params: 1 },
];
