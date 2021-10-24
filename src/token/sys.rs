use std::collections::HashMap;
use crate::compiler::{asm::*, internals, err};

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Sys(String),
    Include,
}

impl Token {
    pub fn compile(&self, consts:&HashMap<String, isize>, internals:&mut internals::Internals) -> Result<Vec<Inst>, err::Err> {
        match self {
            Token::Sys(sys) => {
                match consts.get(sys) {
                    Some(&args) => {
                        let registers = [Op::Rcx, Op::Rdx, Op::R8, Op::R9];
                        let args_count = if 4 < args as usize { 4 } else { args as usize };
                        let mut output = (0..args_count).map(|idx| {
                            Inst::Pop(registers[idx].clone())
                        }).collect::<Vec<Inst>>();
                        output.append(&mut vec![
                            Inst::Call(Op::Label(sys.to_string())),
                            Inst::Push(Op::Rax)
                        ]);
                        Ok(output)
                    },
                    _ => {
                        Err(err::Err::new(
                            format!("Before using a sys call, you must defined a const with it's number of arguments. The const must have the same name as the sys call does. `{}` has no const assossiated.", sys),
                            internals.location.clone(),
                            sys.len(),
                        ))
                    }
                }
            },
            Token::Include => {
                Ok(vec![])
            }
        }
    }
}