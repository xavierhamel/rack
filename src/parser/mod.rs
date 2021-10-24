pub mod token;
pub mod typ;
use crate::token::{control_flow, sys, helper};
use crate::compiler::err;
use crate::type_checker;

use std::fs;
use std::collections::HashMap;

/// Check for the `include` keyword at the begining of each file. When something
/// else is encountered (except for commented line), we stop checking for the
/// include keyword (it can only be at the top of the file).
pub fn parse_includes(main_file:&str, input:&str) -> String {
    let mut content = "".to_string();
    let mut can_include = true;
    for (row, line) in input.lines().enumerate() {
        if !can_include {
            if line.trim().starts_with("include") {
                err::Err::new(
                    "You can only include files at the begining of the file, before any other tokens.".to_string(),
                    (row, 0, main_file.to_string()), 7
                ).panic();
            }
            continue;
        }
        if !line.trim().starts_with("include") 
            && !line.trim().starts_with("___rk___") 
            && !line.trim().starts_with("#")
            && line.trim().len() > 0 {
            can_include = false;
            continue;
        }
        if line.trim().starts_with("___rk___") || line.trim().starts_with("#") || line.trim().len() == 0 {
            continue;
        }
        let values = line.trim().split("include").collect::<Vec<&str>>();
        if (values.len() != 2 && line.trim().starts_with("include")) || values[1].trim().len() == 0 {
            let message = match values.len() {
                1 | 2 => "You must put a string after the `include` keyword to include an other file. Example : `include \"std.rk\"".to_string(),
                tok_count => format!("You can only include one file per `include` keyword but {} tokens were found after the `include` keyword.", tok_count)
            };
            err::Err::new(
                message,
                (row, 0, main_file.to_string()),
                7
            ).print();
            continue;
        }
        if values[1].matches("\"").count() != 2 {
            err::Err::new(
                "Included file must be specified in a string. Example: `include \"std.rk\"".to_string(),
                (row, 6 + line.len() - values[1].len(), main_file.to_string()),
                values[1].trim().len(),
            ).print();
            continue;
        }
        let child_file = values[1].trim().replace("\"", "");
        if let Some(parent) = std::path::Path::new(main_file).parent() {
            let mut child_path = parent.to_path_buf();
            child_path.push(&child_file);
            if let Some(child_path_str) = child_path.to_str() {
                match fs::read_to_string(&child_path) {
                    Ok(child_content) => {
                        let child_input = format!(
                            "\n___rk___ __rk_newfile_rk__ ___rk___ {}\n{}",
                            child_path_str,
                            child_content,
                        );
                        content.push_str(&parse_includes(child_path_str, &child_input));
                    }
                    Err(_) => {
                        err::Err::command_line(
                            format!(
                                "The included file `{}` (`{}`) does not exists or is not able to being opened. Check the path and permission of the file.",
                                child_file,
                                child_path_str
                            )
                        ).panic();
                    }
                }
            } else {
                err::Err::new(
                    format!("An internal error occured while including `{}`.", child_file),
                    (row, 0, main_file.to_string()), 7
                ).panic();
            }
        } else {
            err::Err::new(
                format!("An internal error occured while parsing `{}`", main_file),
                (row, 0, main_file.to_string()), 7
            ).panic();
        }
    }
    content.push_str("\n");
    content.push_str(input);
    content
}

pub fn parse<'a>(input:&'a str) -> (
    Vec<token::Token<'a>>,
    HashMap<String, (usize, usize, Vec<Vec<type_checker::Typ>>, Vec<Vec<type_checker::Typ>>, bool)>,
    HashMap<String, isize>
) {
    let mut errors = vec![];
    let mut tokens = match token::tokenize(input) {
        Ok(toks) => toks,
        Err(errs) => {
            errs.iter().for_each(|error| {
                error.print();
            });
            err::Err::exit();
            vec![]
        }
    };
    let mut args_stack = vec![];
    let mut stack:Vec<(usize, control_flow::Token)> = vec![];
    let mut macros:HashMap<String, (usize, usize)> = HashMap::new();
    let mut functions:HashMap<String, (usize, usize, Vec<Vec<type_checker::Typ>>, Vec<Vec<type_checker::Typ>>, bool)> = HashMap::new();
    let mut consts:HashMap<String, isize> = HashMap::new();
    for idx in 0..tokens.len() {
        if let typ::Typ::Helper(helper) = &tokens[idx].typ {
            match helper {
                helper::Token::ArgOpen => {
                    if idx > 0 {
                        args_stack.push(idx);
                    } else {
                        errors.push(
                            err::Err::new(
                                "arguments must be after at least one other element.".to_string(),
                                (tokens[idx].row, tokens[idx].col, tokens[idx].filename.to_string()), tokens[idx].len()
                            )
                        )
                    }
                },
                helper::Token::ArgClose => {
                    match args_stack.pop() {
                        Some(open_idx) => {
                            let mut offset = 0;
                            for arg_idx in open_idx + 1..idx {
                                if let typ::Typ::Helper(helper::Token::ArgSep) = &tokens[arg_idx].typ {
                                    offset = 0;
                                } else {
                                    let tok = tokens.remove(arg_idx);
                                    tokens.insert(open_idx - 1 + offset, tok);
                                    offset += 1;
                                }
                            }
                        },
                        None => {
                            errors.push(
                                err::Err::new(
                                    "A closing parenthesis ')' must be matched with an opening one '('.".to_string(),
                                    (tokens[idx].row, tokens[idx].col, tokens[idx].filename.to_string()), tokens[idx].len()
                                )
                            )
                        }
                    }
                },
                _ => {}
            }
        } else if let typ::Typ::ControlFlow(keyword) = &tokens[idx].typ {
            match keyword {
                control_flow::Token::Else => {
                    match stack.pop() {
                        Some((op_idx, control_flow::Token::If)) => {
                            tokens[op_idx].jmp_idx = Some(idx);
                            stack.push((idx, control_flow::Token::Else));
                        },
                        _ => {
                            errors.push(
                                err::Err::new(
                                    "Missing an `if` statement before the `else`. Should be in this format: `<condition> if <if_true> else <if_false> end`".to_string(),
                                    (tokens[idx].row, tokens[idx].col, tokens[idx].filename.to_string()), tokens[idx].len()
                                )
                            )
                        }
                    }
                },
                control_flow::Token::End => {
                    match stack.pop() {
                        Some((op_idx, control_flow::Token::If))
                        | Some((op_idx, control_flow::Token::Else))  => {
                            tokens[op_idx].jmp_idx = Some(idx);
                        },
                        Some((op_idx, control_flow::Token::Do)) => {
                            match stack.pop() {
                                Some((while_idx, control_flow::Token::While)) => {
                                    tokens[idx].jmp_idx = Some(while_idx);
                                    tokens[idx].typ = typ::Typ::ControlFlow(control_flow::Token::EndWhile);
                                    tokens[op_idx].jmp_idx = Some(idx);
                                }
                                _ => {
                                    errors.push(
                                        err::Err::new(
                                            "Missing a `while` statement before the `do`. Should be in this format: `while <condition> do <if_true> end`".to_string(),
                                            (tokens[idx].row, tokens[idx].col, tokens[idx].filename.to_string()), tokens[idx].len()
                                        )
                                    )
                                }
                            }
                        },
                        Some((op_idx, control_flow::Token::Macro)) => {
                            if let typ::Typ::Identifier(identifier) = tokens[op_idx + 1].typ {
                                macros.insert(identifier.to_string(), (op_idx + 2, idx));
                            } else {
                                errors.push(
                                    err::Err::new(
                                        "Missing an `identifier` just after the `macro` keyword. Should be in this format: `macro <identifier> <statements> end`".to_string(),
                                        (tokens[idx].row, tokens[idx].col, tokens[idx].filename.to_string()), tokens[idx].len()
                                    )
                                )
                            }
                        },
                        Some((op_idx, control_flow::Token::Fn)) => {
                            if let typ::Typ::Identifier(identifier) = tokens[op_idx + 1].typ {
                                if let typ::Typ::Helper(helper::Token::TypeAnnot(args_type, ret_type, ignore_return)) = tokens[op_idx + 2].typ.clone() {
                                    tokens[op_idx].jmp_idx = Some(idx);
                                    functions.insert(identifier.to_string(), (op_idx + 3, idx, args_type.clone(), ret_type.clone(), ignore_return));
                                } else {
                                    errors.push(
                                        err::Err::new(
                                            "Missing type annotation just after the identifier. Should be in this format: `fn <identifier>[<type_annotation>] <statements> end`".to_string(),
                                            (tokens[op_idx + 1].row, tokens[op_idx + 1].col, tokens[op_idx + 1].filename.to_string()), tokens[op_idx + 1].len()
                                        )
                                    )
                                }
                            } else {
                                errors.push(
                                    err::Err::new(
                                        "Missing an `identifier` just after the `fn` keyword. Should be in this format: `fn <identifier>[<type_annotation>] <statements> end`".to_string(),
                                        (tokens[idx].row, tokens[idx].col, tokens[idx].filename.to_string()), tokens[idx].len()
                                    )
                                )
                            }
                        },
                        Some((op_idx, control_flow::Token::Const)) => {
                            if let typ::Typ::Identifier(identifier) = tokens[op_idx + 1].typ {
                                tokens[op_idx].jmp_idx = Some(idx);
                                if let typ::Typ::Int(integer) = tokens[op_idx + 2].typ {
                                    consts.insert(identifier.to_string(), integer);
                                } else {
                                    errors.push(
                                        err::Err::new(
                                            "A `const` can only be of type `int`.".to_string(),
                                            (tokens[idx].row, tokens[idx].col, tokens[idx].filename.to_string()), tokens[idx].len()
                                        )
                                    )
                                }
                            } else {
                                errors.push(
                                    err::Err::new(
                                        "Missing an `identifier` just after the `const` keyword. Should be in this format: `const <identifier> <int> end`".to_string(),
                                        (tokens[idx].row, tokens[idx].col, tokens[idx].filename.to_string()), tokens[idx].len()
                                    )
                                )
                            }
                        },
                        _ => {
                            errors.push(
                                err::Err::new(
                                    "The `end` keyword did not match any opening statement (like `if`, `while`, `const` or `fn`).".to_string(),
                                    (tokens[idx].row, tokens[idx].col, tokens[idx].filename.to_string()), tokens[idx].len()
                                )
                            )

                        },
                    }
                },
                control_flow::Token::If | control_flow::Token::While | control_flow::Token::Do
                | control_flow::Token::Fn | control_flow::Token::Const => {
                    stack.push((idx, keyword.clone()));
                },
                _ => {}
            }
        } else  if let typ::Typ::Sys(sys::Token::Include) = &tokens[idx].typ {
            tokens[idx].typ = typ::Typ::Ignore;
            tokens[idx + 1].typ = typ::Typ::Ignore;
        }
    }

    if args_stack.len() > 0 {
        errors.push(
            err::Err::new(
                "All opening parenthesis '(' must be matched with a closing one ')'.".to_string(),
                (tokens[tokens.len() - 1].row, tokens[tokens.len() - 1].col, tokens[tokens.len() - 1].filename.to_string()),
                tokens[tokens.len()].len()
            )
        )
    }
    
    if errors.len() > 0 {
        errors.iter().for_each(|err| {
            err.print();
        });
        err::Err::exit();
    }

    (tokens, functions, consts)
}