use std::string::String;
use std::fmt::{Display, Formatter};
use colored::Colorize;

// there are a lot of times when I want to handle escape sequences in strings and find the length of
// the visible characters only, so this struct exists to make that simpler to manage and store

// HOW IT WORKS:
// overwrite the colouring functions such that they can be applied directly to the ColouredString
// which will then colour the internal String and not add anything to the counter.
// only adding strings will change the length counter

#[derive(Clone)]
pub struct ColouredString{
    s: String,
    l: usize,
}

impl ColouredString {
    pub const fn new() -> ColouredString {
        return ColouredString{
            s: String::new(),
            l: 0
        }
    }
    pub fn from_string(string: String) -> ColouredString {
        let len = string.len();
        return ColouredString{
            s: String::from(string),
            l: len
        }
    }
    pub fn from_str(string: &str) -> ColouredString {
        return ColouredString::from_string(string.to_string())
    }
    pub fn as_str(&self) -> &str {
        return self.s.as_str()
    }
    pub fn to_string(&self) -> String {
        return self.s.to_string()
    }
    pub fn push_str(&mut self, string: &str) {
        self.s.push_str(string);
        self.l += string.len();
    }
    pub fn push_string(&mut self, string: String) {
        self.push_str(string.as_str())
    }
    pub fn push_coloured_string(&mut self, coloured_string: ColouredString) {
        // provide way to join multiple strings that have been modified with colour already
        self.s.push_str(coloured_string.as_str());
        self.l += coloured_string.len();
    }
    pub fn len(&self) -> usize {
        return self.l
    }
    pub fn bodge_alter_len(&mut self, difference: usize) {
        self.l -= difference;
    }
}

impl Display for ColouredString { // just pass the value of the string to the formatter
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.s)
    }
}

// metaprogramming / macro to spare massive amount of repeated code
macro_rules! enroll {
    ($func_name:ident) => {
        pub fn $func_name(&self) -> ColouredString{
            return ColouredString {
                s: format!("{}", self.s.as_str().$func_name()),
                l: self.l
            }
        }
    };
}

impl ColouredString {
    enroll!(bold);
    enroll!(italic);

    enroll!(black);
    enroll!(red);
    enroll!(green);
    enroll!(yellow);
    enroll!(blue);
    // enroll!(magenta);
    enroll!(purple);
    enroll!(cyan);
    // enroll!(white);

    // enroll!(on_black);
    enroll!(on_red);
    enroll!(on_green);
    enroll!(on_yellow);
    // enroll!(on_blue);
    // enroll!(on_magenta);
    // enroll!(on_purple);
    enroll!(on_cyan);
    enroll!(on_white);
}