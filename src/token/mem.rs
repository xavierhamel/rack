use crate::compiler::{asm::*, internals, err};

#[derive(Debug, PartialEq, Clone)]
pub enum Size {
    Qword,
    Dword,
    Word,
    Byte
}

impl Size {
    pub fn to_string(&self) -> String {
        match self {
            Size::Qword => "qword".to_string(),
            Size::Dword => "dword".to_string(),
            Size::Word => "word".to_string(),
            Size::Byte => "byte".to_string(),
        }
    }

    pub fn to_rbx(&self) -> Op {
        match self {
            Size::Qword => Op::Rbx,
            Size::Dword => Op::Memory("ebx".to_string()),
            Size::Word => Op::Memory("bx".to_string()),
            Size::Byte => Op::Memory("bl".to_string()),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    InternalMem,
    Mem,
    Swap,
    Dup,
    Over,
    Rot,
    Drop,
    Load(Size),
    Store(Size),
    Put,
    Fetch,
}

impl Token {
    pub fn compile(&self, internals:&mut internals::Internals) -> Result<Vec<Inst>, err::Err>  {
        match self {
            Token::InternalMem => {
                Ok(vec![
                    Inst::Lea(Op::Rax, Op::Memory("[_mem@internal]".to_string())),
                    Inst::Push(Op::Rax)
                ])
            },
            Token::Mem => {
                Ok(vec![
                    Inst::Lea(Op::Rax, Op::Memory("[_mem@mem]".to_string())),
                    Inst::Push(Op::Rax)
                ])
            },
            Token::Dup => {
                Ok(vec![
                    Inst::Pop(Op::Rax),
                    Inst::Mov(Op::Rbx, Op::Rax),
                    Inst::Push(Op::Rax),
                    Inst::Push(Op::Rbx),
                ])
            },
            Token::Swap => {
                Ok(vec![
                    Inst::Pop(Op::Rax),
                    Inst::Pop(Op::Rbx),
                    Inst::Push(Op::Rax),
                    Inst::Push(Op::Rbx),
                ])
            }
            Token::Over => {
                Ok(vec![
                    Inst::Pop(Op::Rax),
                    Inst::Pop(Op::Rbx),
                    Inst::Push(Op::Rbx),
                    Inst::Push(Op::Rax),
                    Inst::Push(Op::Rbx),
                ])
            }
            Token::Load(size) => {
                Ok(vec![
                    Inst::Xor(Op::Rbx, Op::Rbx),
                    Inst::Pop(Op::Rax),
                    Inst::Mov(size.to_rbx(), Op::Memory(format!("{} [rax]", size.to_string()))),
                    Inst::Push(Op::Rbx),
                ])
            }
            Token::Store(size) => {
                Ok(vec![
                    Inst::Pop(Op::Rax),
                    Inst::Pop(Op::Rbx),
                    Inst::Mov(Op::Memory(format!("{} [rax]", size.to_string())), size.to_rbx()),
                ])
            }
            Token::Drop => {
                Ok(vec![
                    Inst::Pop(Op::Rax),
                ])
            }
            Token::Put => {
                match &internals.current_variable {
                    Some(variable) => {
                        // If the variable does not exist, we create it.
                        let idx = match internals.variables.iter().position(|r| r == variable) {
                            Some(idx) => idx,
                            _ => {
                                internals.variables.push(variable.clone());
                                internals.variables.len() - 1
                            }
                        };
                        Ok(vec![
                            Inst::Push(Op::Immediate(idx as isize)),
                            Inst::Call(Op::Label("_std@put_variable".to_string())),
                        ])
                    }
                    _ => {
                        Err(
                            err::Err::new(
                                "`put` should be preceeded by an identifier but was not.".to_string(),
                                internals.location.clone(),
                                3
                            )
                        )
                    }
                }
            }
            Token::Fetch => {
                match &internals.current_variable {
                    Some(variable) => {
                        match internals.variables.iter().position(|r| r == variable) {
                            Some(idx) => {
                                Ok(vec![
                                    Inst::Push(Op::Immediate(idx as isize)),
                                    Inst::Call(Op::Label("_std@fetch_variable".to_string())),
                                ])
                            },
                            _ => {
                                Err(
                                    err::Err::new(
                                        format!("The variable {} was not declared in the current scope. Declare your variables with `put`", variable),
                                        internals.location.clone(),
                                        5,
                                    )
                                )
                            }
                        }
                    }
                    _ => {
                        Err(
                            err::Err::new(
                                "`fetch` should be preceeded by an identifier but was not.".to_string(),
                                internals.location.clone(),
                                5,
                            )
                        )
                    },
                }
            }
            Token::Rot => {
                Ok(vec![
                    Inst::Pop(Op::Rax),
                    Inst::Pop(Op::Rbx),
                    Inst::Pop(Op::Rcx),
                    Inst::Push(Op::Rbx),
                    Inst::Push(Op::Rax),
                    Inst::Push(Op::Rcx),
                ])
            }
        }
    }
}