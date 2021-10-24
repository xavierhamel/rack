use crate::compiler::{asm::*, internals, err};

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    EndWhile,
    End,
    If,
    While,
    Do,
    Else,
    Macro,
    Fn,
    Const,
}

impl Token {
    pub fn compile(&self, jmp_idx:Option<usize>, internals:&mut internals::Internals) -> Result<Vec<Inst>, err::Err> {
        // unimplemented!("Control flow is not implemented yet");
        match self {
            Token::If | Token::Do => {
                let mut output = vec![
                    Inst::Pop(Op::Rax),
                    Inst::Inst2Op("test", Op::Rax, Op::Rax),
                ];
                if let Some(idx) = jmp_idx {
                    let address = internals.compute_address(idx);
                    output.push(
                        Inst::Inst1Op("jz", Op::Label(address))
                    );
                } else {
                    err::Err::new(
                        "Missing an `end` statement after the `if` or `do`. Should be in this format: `<condition> if <if_true> else <if_false> end`".to_string(),
                        internals.location.clone(),
                        2,
                    ).panic();
                }
                Ok(output)
            },
            Token::Else => {
                if let Some(idx) = jmp_idx {
                    let address = internals.compute_address(idx);
                    let label = internals.compute_address(internals.idx);
                    Ok(vec![
                        Inst::Inst1Op("jmp", Op::Label(address)),
                        Inst::Label(format!("{}:", label))
                    ])
                } else {
                    err::Err::new(
                        "Missing an `end` statement after the `else`. Should be in this format: `<condition> if <if_true> else <if_false> end`".to_string(),
                        internals.location.clone(),
                        4,
                    ).panic();
                    Ok(vec![])
                }
            },
            Token::While | Token::End => {
                let address = internals.compute_address(internals.idx);
                Ok(vec![Inst::Label(format!("{}:", address))])
            },
            Token::EndWhile => {
                let mut output = vec![];
                if let Some(idx) = jmp_idx {
                    let address = internals.compute_address(idx);
                    output.push(
                        Inst::Inst1Op("jmp", Op::Label(address))
                    );
                }
                let address = internals.compute_address(internals.idx);
                output.push(Inst::Label(format!("{}:", address)));
                Ok(output)
            },
            Token::Fn | Token::Const => {
                if let Some(idx) = jmp_idx {
                    internals.idx = idx;
                }
                Ok(vec![])
            },
            Token::Macro => {
                unimplemented!("functions are not implemented yet");
            }
        }
    }
}