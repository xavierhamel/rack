mod template;
pub mod asm;
pub mod internals;
pub mod err;

use std::fs::File;
use std::io::prelude::*;
use std::collections::HashMap;
use crate::parser::token;
use crate::type_checker;

enum Platform {
    Windows,
}

pub struct Compiler {
    output:String,
    platform:Platform,
    functions:HashMap<String, (usize, usize, Vec<Vec<type_checker::Typ>>, Vec<Vec<type_checker::Typ>>, bool)>,
    consts:HashMap<String, isize>,
    internals:internals::Internals,
}

impl Compiler {
    pub fn new(
        functions:HashMap<String, (usize, usize, Vec<Vec<type_checker::Typ>>, Vec<Vec<type_checker::Typ>>, bool)>,
        consts:HashMap<String, isize>,
    ) -> Self {
        Self {
            output:"".to_string(),
            platform:Platform::Windows,
            internals:internals::Internals::new(),
            functions,
            consts,
        }
    }

    pub fn compile<'a>(&mut self, tokens:Vec<token::Token<'a>>) {
        self.init_output();
        let mut functions = vec![];
        for (identifier, (start, end, _, _, _)) in self.functions.iter() {
            self.internals.idx = *start;
            functions.append(&mut vec![
                asm::Inst::Label(format!("{}:", identifier)),
                asm::Inst::Call(asm::Op::Label("_std@store_ret_ptr".to_string()))
            ]);
            while self.internals.idx < *end {
                match tokens[self.internals.idx].compile(&mut self.internals, &self.functions, &self.consts) {
                    Ok(mut toks) => functions.append(&mut toks),
                    Err(err) => {
                        err.panic();
                    }
                }
                self.internals.idx += 1;
            }
            functions.append(&mut vec![
                asm::Inst::Call(asm::Op::Label("_std@load_ret_ptr".to_string())),
                asm::Inst::Ret,
            ]);
        }
        functions.iter().for_each(|inst| {
            self.push_op(&inst.to_string());
        });
        self.push_op("\n");
        self.push_op("_start:\n");
        let mut output = vec![];
        self.internals.idx = 0;
        while self.internals.idx < tokens.len() {
            match tokens[self.internals.idx].compile(&mut self.internals, &self.functions, &self.consts) {
                Ok(mut toks) => output.append(&mut toks),
                Err(err) => err.panic()
            }
            self.internals.idx += 1;
        }
        output.iter().for_each(|inst| {
            self.push_op(&inst.to_string());
        });
        self.push_op("call _std@exit\n");
        self.compile_strings();
        self.write();
    }

    pub fn write(&self) {
        let mut file = File::create("output.asm").unwrap();
        file.write_all(self.output.as_bytes()).unwrap();
    }

    fn compile_strings(&mut self) {
        self.push_op("\n");
        self.push_op("segment .data\n\t_mem@ret_ptr_idx dw 0\n");
        let strings = self.internals.compile_strings();
        self.push_op(&strings);
    }



    fn push_op(&mut self, op:&str) {
        self.output.push_str(op);
    }

    fn init_output(&mut self) {
        match self.platform {
            Platform::Windows => {
                self.push_op(template::header());
                self.push_op(template::ret_ptr());
                self.push_op(template::print_int());
                self.push_op(template::variables());
                self.push_op(template::exit());
            }
        }
        self.push_op("\n");
    }
}
