use crate::parser::{token, typ};
use crate::compiler::err;

use std::collections::HashMap;
use colored::*;

#[derive(Clone, PartialEq, Debug)]
pub enum Typ {
    Ptr,
    Int,
    Str,
    Void,
    Float,
    Any,
    Char,
}

impl Typ {
    pub fn to_string(&self) -> String {
        match self {
            Typ::Ptr => "ptr".to_string(),
            Typ::Str => "str".to_string(),
            Typ::Int => "int".to_string(),
            Typ::Void => "void".to_string(),
            Typ::Float => "float".to_string(),
            Typ::Any => "any".to_string(),
            Typ::Char => "char".to_string(),
        }
    }

    pub fn try_from(value:&str) -> Result<Vec<Self>, String> {
        let mut output = vec![];
        for v in value.split("|") {
            output.push(
                match v {
                    "ptr" => Typ::Ptr,
                    "int" => Typ::Int,
                    "str" => Typ::Str,
                    "float" => Typ::Float,
                    "void" => Typ::Void,
                    "any" => Typ::Any,
                    "char" => Typ::Char,
                    _ => {
                        return Err(format!("Types can only be `ptr`, `int`, `str`, `float`, `void` or `any` but {} was found. You can also put multiple types separated by `|`.", v));
                    }
                }
            );
        }
        Ok(output)
    }
}

pub struct TypeChecker {
    pub stack:Vec<Typ>,
    errors:Vec<err::Err>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            stack:vec![],
            errors:vec![],
        }
    }

    pub fn check_stack(
        &mut self,
        keyword:&str,
        typs_allowed:Vec<Vec<Typ>>, 
        tok:&token::Token
    ) -> bool {
        if self.check_stack_len(keyword, typs_allowed.len(), tok) {
            let mut error = false;
            let mut found_types = vec![];
            for typs in typs_allowed.iter() {
                let typ = self.stack.pop().unwrap();
                found_types.push(typ.clone());
                if !typs.contains(&typ) && !typs.contains(&Typ::Any) && typ != Typ::Any {
                    error = true;
                }
            }
            if error {
                self.errors.push(
                err::Err::new(
                    format!(
                        "To use `{}` you need values of type `{}` on the stack but values of types `{}` were found",
                        keyword, 
                        self.allowed_types_to_string(typs_allowed), 
                        self.found_types_to_string(found_types),
                    ),
                    (tok.row, tok.col, tok.filename.clone()),
                    tok.len()
                ));
                return false;
            }
            true
        } else {
            false
        }
    }


    pub fn check_return_stack(
        &mut self,
        keyword:&str,
        typs_allowed:Vec<Vec<Typ>>, 
        tok:&token::Token
    ) {
        let mut found_types = vec![];
        for typs in typs_allowed.iter() {
            let typ = self.stack.pop().unwrap();
            found_types.push(typ.clone());
            if !typs.contains(&typ) && !typs.contains(&Typ::Any) && typ != Typ::Any {
                err::Err::new(
                    format!(
                        "The function `{}` should return types of `{}` but returned types of `{}` on the stack.",
                        keyword, 
                        self.allowed_types_to_string(typs_allowed.clone()), 
                        self.found_types_to_string(found_types.clone()),
                    ),
                    (tok.row, tok.col, tok.filename.clone()),
                    tok.len()
                ).panic();
            }
        }
    }

    pub fn allowed_types_to_string(&self, allowed:Vec<Vec<Typ>>) -> String {
        let mut output = "".to_string();
        allowed.iter().for_each(|args| {
            args.iter().for_each(|arg| {
                output.push_str(&format!("{}|", arg.to_string()));
            });
            output.pop();
            output.push_str(", ");
        });
        output.pop();
        output.pop();
        output
    }

    pub fn found_types_to_string(&self, found:Vec<Typ>) -> String {
        let mut output = "".to_string();
        found.iter().for_each(|arg| {
            output.push_str(&format!("{}, ", arg.to_string()));
        });
        output.pop();
        output.pop();
        output
    }

    pub fn check_stack_len(&self, keyword:&str, min_len:usize, tok:&token::Token) -> bool {
        if self.stack.len() < min_len {
            err::Err::new(
                format!("You cannot use `{}` because the minimum length of the stack is {} but {} value was found on the stack.", keyword, min_len, self.stack.len()),
                (tok.row, tok.col, tok.filename.clone()),
                tok.len()
            ).panic();
            false
        } else {
            true
        }
    }

    pub fn checks(
        &mut self,
        tokens:&Vec<token::Token>,
        functions:&HashMap<String, (usize, usize, Vec<Vec<Typ>>, Vec<Vec<Typ>>, bool)>,
        consts:&HashMap<String, isize>,
    ) {
        for (identifier, func) in functions.clone().into_iter() {
            let args: Vec<Typ> = func.2.iter().rev().map(|arg| {
                if arg.len() > 1 {
                    Typ::Any
                } else {
                    arg[0].clone()
                }
            }).collect();
            self.stack = args;
            self.check(&tokens, functions, consts, func.0, func.1, false);
            if func.4 {
                continue;
            }
            if func.3 == vec![vec![Typ::Void]] {
                if self.stack.len() != 0 {
                    err::Err::new(
                        format!("`{}` should return an empty stack, but it returns {} values on the stack", identifier, self.stack.len()),
                        (tokens[func.0 - 2].row, tokens[func.0 - 2].col, tokens[func.0 - 2].filename.clone()), identifier.len()
                    ).panic();
                }
            } else if self.stack.len() != func.3.len() {
                err::Err::new(
                    format!("`{}` should return {} values on the stack, but it returns {} values on the stack", identifier, func.3.len(), self.stack.len()),
                    (tokens[func.0 - 2].row, tokens[func.0 - 2].col, tokens[func.0 - 2].filename.clone()), identifier.len()
                ).panic();
            } else if self.stack.len() == func.3.len() {
                self.check_return_stack(&identifier, func.3.clone(), &tokens[func.0 - 2]);
            }
        }
        self.stack = vec![];
        self.check(&tokens, functions, consts, 0, tokens.len(), false);
    }

    pub fn check(
        &mut self,
        tokens:&[token::Token],
        functions:&HashMap<String, (usize, usize, Vec<Vec<Typ>>, Vec<Vec<Typ>>, bool)>,
        consts:&HashMap<String, isize>,
        start:usize,
        end:usize,
        debug:bool,
    ) {
        let mut variables_types:HashMap<String, Typ> = HashMap::new();
        let mut current_variable:Option<String> = None;
        let mut idx = start;

        if debug {
            println!(
                "{: <24} {} {}\n{}",
                "Token".cyan().bold(),
                "|".cyan(),
                "Stack".cyan().bold(),
                "--------------------------------------------------".cyan().bold(),
            );
        }
        while idx < end {
            let mut was_identifier = false;
            let token = &tokens[idx];
            match &tokens[idx].typ {
                typ::Typ::Int(_) => self.stack.push(Typ::Int),
                typ::Typ::Str(_) => self.stack.push(Typ::Str),
                typ::Typ::Memory(tok) => {
                    match tok {
                        typ::mem::Token::Mem => self.stack.push(Typ::Ptr),
                        typ::mem::Token::InternalMem => self.stack.push(Typ::Ptr),
                        typ::mem::Token::Dup => {
                            if self.check_stack_len("dup", 1, &token) {
                                self.stack.push(self.stack[self.stack.len() - 1].clone());
                            }
                        },
                        typ::mem::Token::Drop => {
                            if self.check_stack("drop", vec![vec![Typ::Any]], &token) {
                            }
                        },
                        typ::mem::Token::Fetch => {
                            match &current_variable {
                                Some(variable) => {
                                    match variables_types.get(&variable.to_string()) {
                                        Some(typ) => {
                                            self.stack.push(typ.clone());
                                        },
                                        _ => {
                                            err::Err::new(
                                                format!("You have to define a variable before using it, but {} was not found before.", variable),
                                                (token.row, token.col, token.filename.clone()),
                                                token.len()
                                            ).panic();
                                        }
                                    }
                                },
                                _ => {
                                    err::Err::new(
                                        "You need an identifier before the `put` keyword.".to_string(),
                                        (token.row, token.col, token.filename.clone()),
                                        token.len()
                                    ).panic();
                                }
                            }

                        }
                        typ::mem::Token::Load(_) => {
                            self.check_stack("load", vec![vec![Typ::Ptr, Typ::Int]], &token);
                            self.stack.push(Typ::Any);
                        },
                        typ::mem::Token::Over => {
                            if self.check_stack_len("over", 2, &token) {
                                self.stack.push(self.stack[self.stack.len() - 2].clone());
                            }
                        },
                        typ::mem::Token::Put =>  {
                            match &current_variable {
                                Some(variable) => {
                                    if let Some(typ) = self.stack.pop() {
                                        variables_types.insert(variable.clone(), typ);
                                    } else {
                                        err::Err::new(
                                            "You need a value on the stack to put a variable, but the stack was empty.".to_string(),
                                            (token.row, token.col, token.filename.clone()),
                                            token.len()
                                        ).panic();
                                    }
                                },
                                _ => {
                                    err::Err::new(
                                        "You need an identifier before the `put` keyword.".to_string(),
                                        (token.row, token.col, token.filename.clone()),
                                        token.len()
                                    ).panic();
                                }
                            }
                        }
                        typ::mem::Token::Rot => {
                            let len = self.stack.len();
                            if self.check_stack_len("rot", 3, &token) {
                                let tmp = self.stack[self.stack.len() - 3].clone();
                                self.stack[len - 3] = self.stack[self.stack.len() - 2].clone();
                                self.stack[len - 2] = self.stack[self.stack.len() - 1].clone();
                                self.stack[len - 1] = tmp;
                            }
                        }
                        typ::mem::Token::Store(_) => {
                            self.check_stack("store", vec![vec![Typ::Ptr, Typ::Int], vec![Typ::Any]], &token);
                        }
                        typ::mem::Token::Swap => {
                            let len = self.stack.len();
                            if self.check_stack_len("swap", 2, &token) {
                                let tmp = self.stack[self.stack.len() - 2].clone();
                                self.stack[len - 2] = self.stack[self.stack.len() - 1].clone();
                                self.stack[len - 1] = tmp;
                            }
                        }
                    }
                }
                typ::Typ::Ignore => {},
                typ::Typ::Arithmetic(_) => {
                    let is_ptr = self.stack[self.stack.len() - 1] == Typ::Ptr || self.stack[self.stack.len() - 2] == Typ::Ptr;
                    self.check_stack("arithmetic", vec![vec![Typ::Int, Typ::Ptr], vec![Typ::Int, Typ::Ptr]], &token);
                    if is_ptr {
                        self.stack.push(Typ::Ptr);
                    } else {
                        self.stack.push(Typ::Int);
                    }
                }
                typ::Typ::Comparison(_) => {
                    self.check_stack("cmp", vec![vec![Typ::Int, Typ::Ptr], vec![Typ::Int, Typ::Ptr]], &token);
                    self.stack.push(Typ::Int);
                }
                typ::Typ::ControlFlow(tok) => {
                    match tok {
                        typ::control_flow::Token::If => {
                            // TODO: Add internal check (both branch push the same thing)
                            self.check_stack("if", vec![vec![Typ::Int]], &token);
                        },
                        typ::control_flow::Token::Else => {
                            if let Some(jmp_idx) = tokens[idx].jmp_idx {
                                idx = jmp_idx;
                            }
                        }
                        typ::control_flow::Token::Do => {
                            self.check_stack("do", vec![vec![Typ::Int]], &token);
                            if let Some(jmp_idx) = tokens[idx].jmp_idx {
                                idx = jmp_idx;
                            }
                        },
                        typ::control_flow::Token::Fn | typ::control_flow::Token::Const => {
                            // TODO: Add internal check (check with what goes in and what goes out)
                            if let Some(jmp_idx) = tokens[idx].jmp_idx {
                                idx = jmp_idx;
                            }
                        },
                        typ::control_flow::Token::While => {
                            // TODO: Add internal check (no more value after and before), internal stack ?
                        }
                        _ => {}
                    }
                }
                typ::Typ::Helper(_) => {}
                typ::Typ::Identifier(identifier) => {
                    was_identifier = true;
                    let id = identifier.to_string();
                    if let Some(func) = functions.get(&id) {
                        self.check_stack(identifier, func.2.clone(), &token);
                        let mut ret_type = func.3.clone().iter().map(|t| {
                            if t.len() > 1 {
                                Typ::Any
                            } else {
                                t[0].clone()
                            }
                        }).collect::<Vec<Typ>>();
                        if ret_type != vec![Typ::Void] {
                            self.stack.append(&mut ret_type);
                        }
                    } else if let Some(_) = consts.get(&id) {
                        self.stack.push(Typ::Int);
                    } else {
                        current_variable = Some(identifier.to_string());
                    }
                }
                typ::Typ::Sys(typ::sys::Token::Sys(sys)) => {
                    if let Some(value) = consts.get(sys) {
                        if value > &0 {
                            let values_to_pop = (0..*value as usize).map(|_| vec![Typ::Any]).collect();
                            self.check_stack(sys, values_to_pop, &token);
                        }
                        self.stack.push(Typ::Any);
                    }
                }
                _ => {}
            }
            if !was_identifier {
                current_variable = None;
            }
            // TODO: Create a debug tool to show the stack like this, very
            // useful to debug some functions.
            if debug {
                println!(
                    "{: <24} {} {}",
                    format!("{:?}", tokens[idx].typ),
                    "|".cyan(),
                    format!("{:?}", self.stack),
                );
            }
            idx += 1;
        }

        if self.errors.len() > 0 {
            self.errors.iter().for_each(|err| {
                err.print();
            });
            err::Err::exit();
        }
    }
}