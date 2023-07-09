use crate::ColouredString;

#[derive(Clone)]
pub struct SideBySide {
    lines: Vec<ColouredString>,
    width: usize,
}

impl SideBySide {
    pub fn new(w: usize) -> SideBySide {
        return SideBySide {
            lines: Vec::new(),
            width: w
        }
    }
    pub fn add_line(&mut self, line: ColouredString) {
        self.lines.push(line);
    }
    pub fn print(s: &SideBySide) {
        for line in s.lines.to_vec().into_iter() {
            println!("{}", line); // TODO wrap lines or truncate or whatever
        }
    }
    pub fn print2(s1: &SideBySide,s2: &SideBySide, separator: &str) {
        let s1lines = s1.lines.to_vec();
        let s2lines = s2.lines.to_vec();
        let mut iter1 = s1lines.iter();
        let mut iter2 = s2lines.iter();
        let mut running = true;
        while running {
            match iter1.next() {
                Some(cstring1) => {
                    print!("{}", cstring1);
                    if cstring1.len() < s1.width {
                        for _ in 0..(s1.width - cstring1.len()) {
                            print!(" "); // padding according to s1
                        }
                    }
                    print!("{}",separator);
                    match iter2.next() {
                        Some(cstring2) => {
                            println!("{}", cstring2);
                        }
                        None => {
                            println!();
                        }
                    }
                }
                None => {
                    match iter2.next() {
                        Some(cstring2) => {
                            for _ in 0..(s1.width) {print!(" ")} // padding according to s1
                            print!("{}",separator);
                            println!("{}", cstring2);
                        }
                        None => {
                            running = false;
                        }
                    }
                }
            }
        }
    }
}