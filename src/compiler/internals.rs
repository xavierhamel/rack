use std::collections::{HashMap};

/// Internals variables used during compilation.
pub struct Internals {
    pub strings:Vec<String>,
    pub idx:usize,
    pub addresses_usage:HashMap<usize, usize>,
    pub should_increment:bool,
    pub current_variable:Option<String>,
    pub variables:Vec<String>,
    pub location:(usize, usize, String),
}

impl Internals {
    pub fn new() -> Self {
        Self {
            strings:vec![],
            idx:0,
            addresses_usage:HashMap::new(),
            should_increment:false,
            current_variable:None,
            variables:vec![],
            location:(0, 0, "".to_string()),
        }
    }

    pub fn push_string(&mut self, string:String) -> usize {
        self.strings.push(string);
        self.strings.len() - 1
    }

    pub fn compile_strings(&mut self) -> String {
        let mut output = "".to_string();
        for (idx, string) in self.strings.iter_mut().enumerate() {
            output.push_str(&format!("\tstr_{} db \"{}\",0\n", idx, string));
        }
        output
    }

    /// Compute the addresses to be different in every macro, even if a macro is
    /// used more than once. Every time an address is called (an address is just
    /// the token idx), the counter for that address is incremented by one. We
    /// divise this count by 2 because it is called for the label and the jmp
    /// instruction with the same value.
    pub fn compute_address(&mut self, idx:usize) -> String {
        let usage = match self.addresses_usage.get_mut(&idx) {
            Some(usage) => {
                *usage += 1;
                *usage
            },
            None => {
                self.addresses_usage.insert(idx, 0);
                0
            }
        };
        format!("ADDR_{}_{}", idx, usage / 2)
    }
}