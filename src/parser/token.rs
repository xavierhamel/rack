use crate::parser::typ;
use crate::compiler::{asm, internals, err};
use crate::type_checker;
use std::collections::HashMap;

/// A token represent a keyword, string or number. Keywords are separated in
/// different category (memory, control flow, etc...) only to be easier to find
/// in the files but are all essentially the same. `jmp_idx` is only used by
/// control flow tokens for now.
#[derive(Clone, Debug)]
pub struct Token<'a> {
    pub typ:typ::Typ<'a>,
    pub col:usize,
    pub row:usize,
    pub filename:String,
    pub jmp_idx:Option<usize>,
}

impl<'a> Token<'a> {
    pub fn new(value:&'a str, is_string:bool, is_char:bool, is_type_annot:bool, row:usize, col:usize, filename:&str) -> Result<Self, ()> {
        match typ::Typ::try_from(value, is_string, is_char, is_type_annot) {
            Ok(typ) => {
                Ok(
                    Self {
                        typ,
                        row,
                        col,
                        filename:filename.to_string(),
                        jmp_idx:None,
                    }
                )
            },
            Err(()) => {
                Err(())
            } 
        }

    }

    /// Will return the length of the token as the token appears in a file. This
    /// is useful for printing helpful error message.
    pub fn len(&self) -> usize {
        match self.typ.clone() {
            typ::Typ::Arithmetic(_) => 1,
            typ::Typ::Str(string) => string.len(),
            typ::Typ::Identifier(identifier) => identifier.len(),
            typ::Typ::Int(integer) => integer.to_string().len(),
            typ::Typ::Helper(_) => 1,
            typ::Typ::Sys(typ::sys::Token::Sys(identifier)) => identifier.len() + 5,
            typ::Typ::Sys(typ::sys::Token::Include) => 7,
            typ::Typ::Ignore => 0,
            typ::Typ::Memory(token) => {
                match token {
                    typ::mem::Token::Dup | typ::mem::Token::Rot | typ::mem::Token::Put | typ::mem::Token::Mem => 3,
                    typ::mem::Token::InternalMem | typ::mem::Token::Swap 
                    | typ::mem::Token::Load(_) | typ::mem::Token::Drop | typ::mem::Token::Over => 4,
                    typ::mem::Token::Store(_) | typ::mem::Token::Fetch => 5,
                }
            }
            typ::Typ::ControlFlow(token) => {
                match token {
                    typ::control_flow::Token::Do | typ::control_flow::Token::Fn | typ::control_flow::Token::If => 2,
                    typ::control_flow::Token::End | typ::control_flow::Token::EndWhile => 3,
                    typ::control_flow::Token::Else => 4,
                    typ::control_flow::Token::Const | typ::control_flow::Token::Macro 
                    | typ::control_flow::Token::While => 5,

                }
            }
            typ::Typ::Comparison(token) => {
                match token {
                    typ::cmp::Token::Eq | typ::cmp::Token::Gt | typ::cmp::Token::Lt => 1,
                    typ::cmp::Token::NotEq | typ::cmp::Token::Ge | typ::cmp::Token::Le => 2,

                }
            }
        }
    }

    /// Will compile the token to it's assembly reprensentation. During this
    /// phase we are not directly compiling to assembly string because it will
    /// be easier to optimize the code later that way.
    pub fn compile(
        &self, 
        internals:&mut internals::Internals, 
        functions:&HashMap<String, (usize, usize, Vec<Vec<type_checker::Typ>>, Vec<Vec<type_checker::Typ>>, bool)>,
        consts:&HashMap<String, isize>,
    ) -> Result<Vec<asm::Inst>, err::Err> {
        let mut was_identifier = false;
        if internals.current_variable.is_some() && !self.typ.is_affected_by_identifier() {
            let identifier = internals.current_variable.clone().unwrap();
            return Err(
                err::Err::new(
                    format!("An identifier cannot be free standing, it should be a `fn` or have a `put` or `fetch` (or `!`) after it. `{}` is free standing. Check the spelling of the identifier.", identifier),
                    internals.location.clone(),
                    identifier.len()
                )
            )
        }
        internals.location = (self.row, self.col, self.filename.clone());
        let toks = match &self.typ {
            typ::Typ::Memory(typ) => typ.compile(internals),
            typ::Typ::ControlFlow(typ) => typ.compile(self.jmp_idx, internals),
            typ::Typ::Comparison(typ) => typ.compile(),
            typ::Typ::Arithmetic(typ) => typ.compile(),
            typ::Typ::Sys(typ) => typ.compile(consts, internals),
            typ::Typ::Str(string) => {
                let str_idx = internals.push_string(string.to_string());
                Ok(vec![
                    asm::Inst::Lea(asm::Op::Rax, asm::Op::Memory(format!("[str_{}]", str_idx))),
                    asm::Inst::Push(asm::Op::Rax)
                ])
            },
            typ::Typ::Int(integer) => {
                Ok(vec![
                    asm::Inst::Mov(asm::Op::Rax, asm::Op::Immediate(*integer)),
                    asm::Inst::Push(asm::Op::Rax)
                ])
            }
            typ::Typ::Identifier(identifier) => {
                was_identifier = true;
                let id = identifier.to_string();
                if let Some(_) = functions.get(&id) {
                    Ok(vec![
                        asm::Inst::Call(asm::Op::Label(id))
                    ])
                } else if let Some(value) = consts.get(&id) {
                    Ok(vec![
                        asm::Inst::Push(asm::Op::Immediate(*value))
                    ])
                } else {
                    internals.current_variable = Some(identifier.to_string());
                    Ok(vec![])
                }
            },
            typ::Typ::Ignore => Ok(vec![]),
            typ::Typ::Helper(_) => Ok(vec![])
        };
        if !was_identifier {
            internals.current_variable = None;
        }
        toks
    }
}

/// Convert the text input of the file to a string of tokens. The string of
/// token will then be consume one by one to compile the program.
pub fn tokenize(input:&str) -> Result<Vec<Token>, Vec<err::Err>> {
    let mut tokens = vec![];
    let mut errors = vec![];
    let mut filename = "";
    let mut row_offset = 0;
    input.lines().enumerate().for_each(|(row, line)| {
        let mut start = 0;
        let mut in_comment = false;
        let mut is_string = false;
        let mut is_char = false;
        let mut is_type_annot = false;
        let mut is_two_char_tok = false;
        let mut ignore_equal = false;
        // This delimit a file, (when included, file are all put in the same
        // 'input' variable). This give us the name of the file (it cames after
        // this long line in the input). `in_comment` is then put to true to
        // ignore the line.
        let filedelemiter = line.split("___rk___ __rk_newfile_rk__ ___rk___ ").collect::<Vec<&str>>();
        if filedelemiter.len() == 2 {
            filename = filedelemiter[1];
            in_comment = true;
            row_offset = row;
        }
        line.chars().enumerate().for_each(|(col, c)| {
            if !in_comment {
                if is_two_char_tok {
                    let mut end = col;
                    let two_tok_start = start;
                    if c == '=' {
                        ignore_equal = true;
                        end += 1;
                    } else {
                        start = col;
                    }
                    let value = &line[two_tok_start..end].trim();
                    if let Ok(tok) = Token::new(value, false, false, false, row - row_offset, col, filename) {
                        tokens.push(tok);
                    }
                    is_two_char_tok = false;
                }
                match c {
                    '#' => {
                        in_comment = true;
                    }
                    '!' | '>' | '<' => if !is_string && !is_char && !is_type_annot {
                        is_two_char_tok = true;
                        let value = &line[start..col].trim();
                        if let Ok(tok) = Token::new(value, false, false, false, row - row_offset, col, filename) {
                            tokens.push(tok);
                        }
                        if col == line.len() - 1 {
                            if let Ok(tok) = Token::new(&line[col..], false, false, false, row - row_offset, col, filename) {
                                tokens.push(tok);
                            }
                        }
                        start = col;
                    },
                    '(' | ')' | '&' | '|' | '+' | '*'
                    | '/' | '%' | '=' | ',' | ' ' | '\t' => {
                        if !is_char && !is_string && !is_type_annot {
                            if ignore_equal {
                                ignore_equal = false;
                                start = col + 1;
                            } else {
                                let value = &line[start..col].trim();
                                if let Ok(tok) = Token::new(value, false, false, false, row - row_offset, col, filename) {
                                    tokens.push(tok);
                                }
                                if let Ok(tok) = Token::new(&line[col..col + 1], false, false, false, row - row_offset, col, filename) {
                                    tokens.push(tok);
                                }
                                start = col + 1;
                            }
                        }
                    },
                    '[' => {
                        let value = &line[start..col].trim();
                        if let Ok(tok) = Token::new(value, false, false, false, row - row_offset, col, filename) {
                            tokens.push(tok);
                        }
                        if !is_string && !is_char {
                            start = col + 1;
                            is_type_annot = true;
                        }
                    },
                    ']' => {
                        if is_type_annot {
                            let value = &line[start..col].trim();
                            if let Ok(tok) = Token::new(value, false, false, true, row - row_offset, col, filename) {
                                tokens.push(tok);
                            }
                            start = col + 1;
                            is_type_annot = false;
                        }
                    }
                    '"' => {
                        if is_string {
                            let value = &line[start..col].trim();
                            if let Ok(tok) = Token::new(value, true, false, false, row - row_offset, col, filename) {
                                tokens.push(tok);
                            }
                            start = col + 1;
                            is_string = false;
                        } else if !is_char && !is_type_annot {
                            let value = &line[start..col].trim();
                            if let Ok(tok) = Token::new(value, false, false, false, row - row_offset, col, filename) {
                                tokens.push(tok);
                            }
                            start = col + 1;
                            is_string = true;
                        }
                    },
                    '\'' => {
                        if is_char {
                            if col - start == 1 {
                                let value = &line[col - 1..col];
                                if let Ok(tok) = Token::new(value, false, true, false, row - row_offset, col, filename) {
                                    tokens.push(tok);
                                }
                                start = col + 1;
                                is_char = false;
                            } else {
                                errors.push(
                                    err::Err::new(
                                        "a char should be 1 charater long. To have longer strings, use \" not \'.".to_string(),
                                        (row - row_offset, start + 1, filename.to_string()),
                                        col - start,
                                    )
                                );
                                is_char = false;
                            }
                        } else if !is_string && !is_type_annot {
                            let value = &line[start..col];
                            if let Ok(tok) = Token::new(value, false, false, false, row - row_offset, col, filename) {
                                tokens.push(tok);
                            }
                            start = col + 1;
                            is_char = true;
                        }
                    }
                    _ => {
                        if col == line.len() - 1 {
                            if !is_string && !is_char && !is_type_annot {
                                let value = &line[start..];
                                if let Ok(tok) = Token::new(value, false, false, false, row - row_offset, col, filename) {
                                    tokens.push(tok);
                                }
                            } else if !is_type_annot {
                                errors.push(
                                    err::Err::new(
                                        "a string can only be on one line and should be closed before the end of the line".to_string(),
                                        (row - row_offset, col, filename.to_string()), 1
                                    )
                                )
                            } else {
                                errors.push(
                                    err::Err::new(
                                        "An opening braket '[' must be matched with a closing one ']'.".to_string(),
                                        (row - row_offset, col, filename.to_string()), 1
                                    )
                                )
                            }
                        }
                    }
                }                     
            }
        });
    });
    // tokens.iter().for_each(|tok| {
        // println!("{:?}", tok);
    // });
    if errors.len() > 0 {
        Err(errors)
    } else {
        Ok(tokens)
    }
}