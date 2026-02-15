use crate::registers::Register;

#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    AddImmediate {
        res: Register,
        reg: Register,
        imm: i16,
    },
    AddUnsigned {
        res: Register,
        reg: Register,
        ret: Register,
    },
    LoadUpperImmediate {
        res: Register,
        imm: i16,
    },
    OrImmediate {
        res: Register,
        reg: Register,
        imm: i16,
    },
    SystemCall,
}
