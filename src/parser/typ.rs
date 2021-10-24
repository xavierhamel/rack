pub use crate::token::*;

#[derive(Debug, PartialEq, Clone)]
pub enum Typ<'a> {
    Memory(mem::Token),
    ControlFlow(control_flow::Token),
    Comparison(cmp::Token),
    Arithmetic(arith::Token),
    Sys(sys::Token),
    Helper(helper::Token),
    Str(&'a str),
    Int(isize),
    Identifier(&'a str),
    Ignore,
}

impl<'a> Typ<'a> {
    /// Will try to find what type of token the given string is. Currently it
    /// never fails but this may change soon when some checks are added
    /// (especially for char autorized in identifiers).
    pub fn try_from(value:&'a str, is_string:bool, is_char:bool, is_type_annot:bool) -> Result<Self, ()> {
        if value.trim().len() == 0 {
            Err(())
        } else if is_char {
            let int = value.chars().nth(0).unwrap() as isize;
            Ok(Typ::Int(int))
        } else if is_string {
            if value.len() > 0 {
                Ok(Typ::Str(&value))
            } else {
                Err(())
            }
        } else if is_type_annot {
            match helper::Token::new_type_annot(value) {
                Ok(annot) => Ok(Typ::Helper(annot)),
                // TODO: This error is never shown when it happen.
                Err(_str) => {
                    println!("{}", _str);
                    unimplemented!("type annotation error");
                },
            }
        } else {
            if let Ok(integer) = value.parse::<isize>() {
                Ok(Typ::Int(integer))
            } else {
                let typ = match value {
                    "mem" =>   Typ::Memory(mem::Token::Mem),
                    "_mem" =>   Typ::Memory(mem::Token::InternalMem),
                    "swap" =>  Typ::Memory(mem::Token::Swap),
                    "dup" =>   Typ::Memory(mem::Token::Dup),
                    "over" =>  Typ::Memory(mem::Token::Over),
                    "drop" =>  Typ::Memory(mem::Token::Drop),
                    "rot" =>   Typ::Memory(mem::Token::Rot),
                    "store" => Typ::Memory(mem::Token::Store(mem::Size::Qword)),
                    "load" =>  Typ::Memory(mem::Token::Load(mem::Size::Qword)),
                    "store32" => Typ::Memory(mem::Token::Store(mem::Size::Dword)),
                    "load32" =>  Typ::Memory(mem::Token::Load(mem::Size::Dword)),
                    "store16" => Typ::Memory(mem::Token::Store(mem::Size::Word)),
                    "load16" =>  Typ::Memory(mem::Token::Load(mem::Size::Word)),
                    "store8" => Typ::Memory(mem::Token::Store(mem::Size::Byte)),
                    "load8" =>  Typ::Memory(mem::Token::Load(mem::Size::Byte)),
                    "put" =>   Typ::Memory(mem::Token::Put),
                    "fetch" => Typ::Memory(mem::Token::Fetch),
                    "!" => Typ::Memory(mem::Token::Fetch),
                    "&" => Typ::Arithmetic(arith::Token::LogicalAnd),
                    "|" => Typ::Arithmetic(arith::Token::LogicalOr),
                    "+" => Typ::Arithmetic(arith::Token::Plus),
                    "-" => Typ::Arithmetic(arith::Token::Minus),
                    "*" => Typ::Arithmetic(arith::Token::Mul),
                    "/" => Typ::Arithmetic(arith::Token::Div),
                    "%" => Typ::Arithmetic(arith::Token::Mod),
                    "=" => Typ::Comparison(cmp::Token::Eq),
                    "!=" => Typ::Comparison(cmp::Token::NotEq),
                    "<=" => Typ::Comparison(cmp::Token::Le),
                    "<" => Typ::Comparison(cmp::Token::Lt),
                    ">" => Typ::Comparison(cmp::Token::Gt),
                    ">=" => Typ::Comparison(cmp::Token::Ge),
                    "macro" => Typ::ControlFlow(control_flow::Token::Macro),
                    "fn" => Typ::ControlFlow(control_flow::Token::Fn),
                    "const" => Typ::ControlFlow(control_flow::Token::Const),
                    "if" => Typ::ControlFlow(control_flow::Token::If),
                    "else" => Typ::ControlFlow(control_flow::Token::Else),
                    "while" => Typ::ControlFlow(control_flow::Token::While),
                    "do" => Typ::ControlFlow(control_flow::Token::Do),
                    "end" => Typ::ControlFlow(control_flow::Token::End),
                    "sys" => Typ::Sys(sys::Token::Sys("".to_string())),
                    "include" => Typ::Sys(sys::Token::Include),
                    "(" => Typ::Helper(helper::Token::ArgOpen),
                    ")" => Typ::Helper(helper::Token::ArgClose),
                    "," => Typ::Helper(helper::Token::ArgSep),
                    _ => {
                        let path:Vec<&str> = value.split("::").collect();
                        if path.len() > 1 {
                            match path[0] {
                                "sys" => Typ::Sys(sys::Token::Sys(path[1].to_string())),
                                _ => Typ::Identifier(value),
                            }
                        } else {
                            Typ::Identifier(value)
                        }
                    }
                };
                Ok(typ)
            }
        }
    }

    /// Return if the current token is affected by an identifier as the previous
    /// token.
    pub fn is_affected_by_identifier(&self) -> bool {
        self == &Typ::Memory(mem::Token::Put) || self == &Typ::Memory(mem::Token::Fetch)
    }
}
