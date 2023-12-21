pub const LOAD: u8 = 0x00;
pub const LOAD_SUPER: u8 = 0x01;
pub const LOAD_FIELD: u8 = 0x02;
pub const LOAD_ITEM: u8 = 0x03;

pub const STORE: u8 = 0x08;
pub const STORE_SUPER: u8 = 0x09;
pub const STORE_FIELD: u8 = 0x0a;
pub const STORE_ITEM: u8 = 0x0b;

pub const DUP: u8 = 0x10;
pub const POP: u8 = 0x11;

pub const PUSH_INT: u8 = 0x20;
pub const PUSH_CONST: u8 = 0x21;
pub const NEW_ARRAY: u8 = 0x22;

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

pub const EQ: u8 = 0x48;
pub const NE: u8 = 0x49;
pub const GT: u8 = 0x4a;
pub const LT: u8 = 0x4b;
pub const GE: u8 = 0x4c;
pub const LE: u8 = 0x4d;
pub const NOT: u8 = 0x4e;

pub const BAND: u8 = 0x50;
pub const BOR: u8 = 0x51;
pub const BXOR: u8 = 0x52;
pub const BINV: u8 = 0x53;
pub const SHL: u8 = 0x54;
pub const SHR: u8 = 0x55;
pub const USHR: u8 = 0x56;

pub const LEN: u8 = 0x58;

pub const IN: u8 = 0x60;
pub const OUT: u8 = 0x61;
pub const LOAD_LIB: u8 = 0x68;
