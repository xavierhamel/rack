use crate::compiler::{asm::*, err};

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Eq,
    NotEq,
    Gt,
    Lt,
    Le,
    Ge,
}

impl Token {
    pub fn compile(&self) -> Result<Vec<Inst>, err::Err> {
        let mut output = vec![
            Inst::Xor(Op::Rcx, Op::Rcx),
            Inst::Mov(Op::Rdx, Op::Immediate(1)),
            Inst::Pop(Op::Rbx),
            Inst::Pop(Op::Rax),
            Inst::Cmp(Op::Rax, Op::Rbx),
        ];  
        let inst = match self {
            Token::Eq => "cmove",
            Token::NotEq => "cmovne",
            Token::Le => "cmovle",
            Token::Lt => "cmovl",
            Token::Ge => "cmovge",
            Token::Gt => "cmovg",
        };
        output.push(Inst::Inst2Op(inst, Op::Rcx, Op::Rdx));
        output.push(Inst::Push(Op::Rcx));
        Ok(output)
    }
}