use colored::*;

pub enum ErrTyp {
    Internal,
    User,
    CommandLine,
    Function,
}

pub struct Err {
    typ:ErrTyp,
    message:String,
    location:(usize, usize),
    filename:String,
    token_len:usize,
    function_line:Option<usize>,
}

impl Err {
    pub fn new(message:String, location:(usize, usize, String), token_len:usize) -> Self {
        Self {
            typ:ErrTyp::User,
            message,
            location:(location.0, location.1),
            filename:location.2,
            token_len,
            function_line:None,
        }
    }

    pub fn function(message:String, location:(usize, usize, String), token_len:usize, func_line:usize) -> Self {
        Self {
            typ:ErrTyp::Function,
            message,
            location:(location.0, location.1),
            filename:location.2,
            token_len,
            function_line:Some(func_line),
        }
    }

    pub fn typ(message:String, location:(usize, usize, String), token_len:usize, func_line:usize, is_function:bool) -> Self {
        if is_function {
            Self::function(message, location, token_len, func_line)
        } else {
            Self::new(message, location, token_len)
        }
    }


    pub fn internal(location:(usize, usize, String)) -> Self {
        Self {
            typ:ErrTyp::Internal,
            message:"An internal error occured while parsing or compiling the program".to_string(),
            location:(location.0, location.1),
            filename:location.2,
            token_len:1,
            function_line:None,
        }
    }

    pub fn command_line(message:String) -> Self {
        Self {
            typ:ErrTyp::CommandLine,
            message,
            location:(0, 0),
            filename:"".to_string(),
            token_len:0,
            function_line:None,
        }
    }


    pub fn panic(&self) {
        self.print();
        Err::exit();
   }

   pub fn print(&self) {
        match self.typ {
            ErrTyp::CommandLine => {
                println!(
                    "{}: {}",
                    "error".red().bold(),
                    self.message.bold(),
                )
            }
            ErrTyp::User => {
                println!(
                    "{}: {}\n {} {}\n{}",
                    "error".red().bold(),
                    self.message.bold(),
                    "-->".cyan(),
                    self.location_string(),
                    self.line_with_error(),
                )
            },
            ErrTyp::Function => {
                println!(
                    "{}: {}\n {} {}\n{}\n{: >5}{}\n{}",
                    "error".red().bold(),
                    self.message.bold(),
                    "-->".cyan(),
                    self.location_string(),
                    self.function_line(),
                    " ",
                    "...".cyan(),
                    self.line_with_error(),
                )
            },
            ErrTyp::Internal => {
                println!(
                    "{}: {}\n {} {}",
                    "internal error".red().bold(),
                    "An internal error occured while parsing or compiling the program. This is a bug, please report it.".bold(),
                    "-->".cyan(),
                    self.location_string()
                )
            }
        }

   }

    fn location_string(&self) -> String {
        format!("{}:{}:{}", self.filename, self.location.0, self.location.1)
    }

    fn line_with_error(&self) -> String {
        match std::fs::read_to_string(&self.filename) {
            Ok(filecontent) => {
                match filecontent.lines().nth(self.location.0 - 1) {
                    Some(line) => {
                        let mut column = 0;
                        if self.location.1 + 2 >= self.token_len {
                            column = self.location.1 + 2 - self.token_len;
                        }
                        let line_number = self.location.0.to_string();
                        format!(
                            "{: >5} {}\n{: >5} {} {}\n{: >5} {:col$} {}",
                            "", "|".cyan(),
                            line_number.cyan(), "|".cyan(), line.bold(),
                            "", "|".cyan(), "^".repeat(self.token_len).red().bold(),
                            col=column
                        )
                    },
                    _ => "".to_string(),
                }
            },
            _ => "".to_string(),
        }
    }

    fn function_line(&self) -> String {
        match (std::fs::read_to_string(&self.filename), self.function_line) {
            (Ok(filecontent), Some(func_line)) => {
                match filecontent.lines().nth(func_line) {
                    Some(line) => {
                        let line_number = func_line.to_string();
                        format!(
                            "{: >5} {} {}",
                            line_number.cyan(), "|".cyan(), line.bold(),
                        )
                    },
                    _ => "".to_string(),
                }
            },
            _ => "".to_string(),
        }

    }

    pub fn exit() {
        std::process::exit(0);
    }
}

