mod parser;
mod compiler;
pub mod token;
mod type_checker;

use colored::*;
use std::fs;
use std::env;
use std::collections::HashMap;

// nasm -f win64 output.asm -o output.obj
// link output.obj /subsystem:console /entry:main /out:output.exe
// calling conventions:
// https://docs.microsoft.com/en-us/cpp/build/x64-calling-convention?view=msvc-160

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        compiler::err::Err::command_line(
            "No file was specified to be compiled.\n\tCommand usage: `rack <file_name>.rk`".to_string(),
        ).panic();
    }
    let mut is_debug_stack = false;
    let filename = &args[1];
    if args.len() > 2 && args[2] == "--debug-stack" {
        if args.len() != 4 {
            compiler::err::Err::command_line(
                "The `--debug-stack` flag only works for functions.\n\tTo debug a function, call `rack <file_name> --debug-stack <function_name>.".to_string(),
            ).panic();
        }
        is_debug_stack = true;
    }
    let mut input = format!("___rk___ __rk_newfile_rk__ ___rk___ {}\n", filename);
    match fs::read_to_string(filename) {
        Ok(content) => {
            input.push_str(&content);
            input = parser::parse_includes(&filename, &input);
            let (tokens, functions, consts) = parser::parse(&input);
            let type_checker = type_checker::TypeChecker::new();
            if is_debug_stack {
                debug_stack(type_checker, tokens, functions, consts, &args[3]);
            } else {
                compile(filename, type_checker, tokens, functions, consts);
            }
        },
        Err(_) => {
            compiler::err::Err::command_line(
                format!(
                    "The file `{}` does not exists or is not able to being opened. Check the path and permission of the file.",
                    filename,
                )
            ).panic();
        }
    }
}

fn compile(
    filename:&String,
    mut type_checker:type_checker::TypeChecker,
    tokens:Vec<parser::token::Token>,
    functions:HashMap<String, (usize, usize, Vec<Vec<type_checker::Typ>>, Vec<Vec<type_checker::Typ>>, bool)>,
    consts:HashMap<String, isize>,
) {
    println!(
        "{} {}",
        "compiling".green().bold(),
        filename,
    );
    type_checker.checks(&tokens, &functions, &consts);
    let mut compiler = compiler::Compiler::new(functions, consts);
    compiler.compile(tokens);
    println!(
        "{} {}",
        "finished ".green().bold(),
        filename,
    );
}

fn debug_stack(
    mut type_checker:type_checker::TypeChecker,
    tokens:Vec<parser::token::Token>,
    functions:HashMap<String, (usize, usize, Vec<Vec<type_checker::Typ>>, Vec<Vec<type_checker::Typ>>, bool)>,
    consts:HashMap<String, isize>,
    function_to_debug:&String,
) {
    match functions.get(function_to_debug) {
        Some((start, end, stack, _, _)) => {
            let args: Vec<type_checker::Typ> = stack.iter().rev().map(|arg| {
                if arg.len() > 1 {
                    type_checker::Typ::Any
                } else {
                    arg[0].clone()
                }
            }).collect();
            type_checker.stack = args;
            type_checker.check(&tokens, &functions, &consts, *start, *end, true);
        }
        _ => {
            compiler::err::Err::command_line(
                format!(
                    "The function `{}` does not exists. Check the of the function.",
                    function_to_debug,
                )
            ).panic();
        }
    }
}