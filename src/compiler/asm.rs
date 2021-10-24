#[derive(Clone, PartialEq)]
pub enum Op {
    Rax,
    Rbx,
    Rcx,
    Rdx,
    Rsi,
    Rsp,
    Rbp,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
    Label(String),
    Immediate(isize),
    Memory(String),
    Ignore,
}

impl Op {
    pub fn to_string(&self) -> String {
        match self {
            Op::Rax => "rax".to_string(),
            Op::Rbx => "rbx".to_string(),
            Op::Rcx => "rcx".to_string(),
            Op::Rdx => "rdx".to_string(),
            Op::Rsi => "rsi".to_string(),
            Op::Rsp => "rsp".to_string(),
            Op::Rbp => "rbp".to_string(),
            Op::R8 => "r8".to_string(),
            Op::R9 => "r9".to_string(),
            Op::R10 => "r10".to_string(),
            Op::R11 => "r11".to_string(),
            Op::R12 => "r12".to_string(),
            Op::R13 => "r13".to_string(),
            Op::R14 => "r14".to_string(),
            Op::R15 => "r15".to_string(),
            Op::Label(label) => label.to_string().replace("::", "_"),
            Op::Immediate(integer) => integer.to_string(),
            Op::Memory(mem) => mem.clone(),
            Op::Ignore => "".to_string(),
        }   
    }
    pub fn is_register(&self) -> bool {
        match self {
            Op::Rax | Op::Rbx | Op::Rcx | Op::Rdx | Op::Rsi | Op::Rsp |
            Op::Rbp | Op::R8  | Op::R9  | Op::R10 | Op::R11 | Op::R12 |
            Op::R13 | Op::R14 | Op::R15 => true,
            _ => false,
        }
    }
}

#[derive(PartialEq)]
pub enum Inst {
    Push(Op),
    Pop(Op),
    Mov(Op, Op),
    Lea(Op, Op),
    Call(Op),
    Add(Op, Op),
    Sub(Op, Op),
    Mul(Op),
    Div(Op),
    Xor(Op, Op),
    Cmp(Op, Op),
    Inst2Op(&'static str, Op, Op),
    Inst1Op(&'static str, Op),
    Label(String),
    Ret,
    Ignore,
}

impl Inst {
    pub fn to_string(&self) -> String {
        match self {
            Inst::Push(op) => format!("\tpush {}\n", op.to_string()),
            Inst::Pop(op) => format!("\tpop {}\n", op.to_string()),
            Inst::Mov(dest, src) => format!("\tmov {}, {}\n", dest.to_string(), src.to_string()),
            Inst::Lea(dest, src) => format!("\tlea {}, {}\n", dest.to_string(), src.to_string()),
            Inst::Call(op) => format!("\tcall {}\n", op.to_string()),
            Inst::Add(dest, src) => format!("\tadd {}, {}\n", dest.to_string(), src.to_string()),
            Inst::Sub(dest, src) => format!("\tsub {}, {}\n", dest.to_string(), src.to_string()),
            Inst::Mul(op) => format!("\tmul {}\n", op.to_string()),
            Inst::Div(op) => format!("\tdiv {}\n", op.to_string()),
            Inst::Xor(dest, src) => format!("\txor {}, {}\n", dest.to_string(), src.to_string()),
            Inst::Cmp(dest, src) => format!("\tcmp {}, {}\n", dest.to_string(), src.to_string()),
            Inst::Inst2Op(inst, dest, src) => format!("\t{} {}, {}\n", inst, dest.to_string(), src.to_string()),
            Inst::Inst1Op(inst, op) => format!("\t{} {}\n", inst, op.to_string()),
            Inst::Label(label) => format!("{}\n", label).replace("::", "_"),
            Inst::Ret => "\tret\n".to_string(),
            Inst::Ignore => "".to_string(),
        }
    }
}