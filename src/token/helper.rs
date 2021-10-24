use crate::compiler::{asm::*, err};
use crate::type_checker;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    ArgOpen,
    ArgClose,
    ArgSep,
    TypeAnnot(Vec<Vec<type_checker::Typ>>, Vec<Vec<type_checker::Typ>>, bool),
}

impl Token {
    pub fn compile(&self) -> Result<Vec<Inst>, err::Err> {
        Ok(vec![])
    }

    pub fn new_type_annot(value:&str) -> Result<Token, String> {
        let values = value.trim().split("->").collect::<Vec<&str>>();
        let mut args_type = values[0];
        let mut return_type = values[0].to_string();
        let mut ignore_return = false;
        if values.len() == 2 {
            return_type = values[1].to_string();
        } else {
            args_type = "";
        }
        if return_type.trim().starts_with("!") {
            ignore_return = true;
            return_type = return_type.trim()[1..].to_string();
        }
        match (Token::parse_annot(args_type), Token::parse_annot(&return_type)) {
            (Ok(args), Ok(ret)) => {
                Ok(Token::TypeAnnot(args, ret, ignore_return))
            },
            (_, Err(err)) => Err(err),
            (Err(err), _) => Err(err),
        }
    }

    fn parse_annot(value:&str) -> Result<Vec<Vec<type_checker::Typ>>, String> {
        let mut output = vec![];
        for typ in value.split(",") {
            if typ.trim().len() > 0 {
                match type_checker::Typ::try_from(typ.trim()) {
                    Ok(val) => output.push(val),
                    Err(err) => {
                        return Err(err);
                    }
                }
            }
        }
        Ok(output)
    }
}