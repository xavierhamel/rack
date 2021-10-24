use crate::compiler::{asm::*, err};

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Plus,
    Minus,
    Div,
    Mul,
    Mod,
    LogicalAnd,
    LogicalOr,
}

impl Token {
    pub fn compile(&self) -> Result<Vec<Inst>, err::Err> {
        let mut output = vec![
            Inst::Pop(Op::Rbx),
            Inst::Pop(Op::Rax),
        ];  
        match self {
            Token::Plus => output.push(Inst::Add(Op::Rax, Op::Rbx)),
            Token::Minus => output.push(Inst::Sub(Op::Rax, Op::Rbx)),
            Token::Mul => output.push(Inst::Mul(Op::Rbx)),
            Token::Div => {
                output.append(&mut vec![
                    Inst::Xor(Op::Rdx, Op::Rdx),
                    Inst::Div(Op::Rbx),
                ]);
            }
            Token::Mod => {
                output.append(&mut vec![
                    Inst::Xor(Op::Rdx, Op::Rdx),
                    Inst::Div(Op::Rbx),
                    Inst::Mov(Op::Rax, Op::Rdx),
                ]);
            }
            Token::LogicalAnd => output.push(Inst::Inst2Op("and", Op::Rax, Op::Rbx)),
            Token::LogicalOr => output.push(Inst::Inst2Op("or", Op::Rax, Op::Rbx)),
        };
        output.push(Inst::Push(Op::Rax));
        Ok(output)
    }
}