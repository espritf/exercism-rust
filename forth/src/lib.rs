use std::collections::HashMap;

pub type Value = i32;
pub type Result = std::result::Result<(), Error>;

pub struct Forth {
    stack: Vec<Value>,
    words: HashMap<String, Vec<String>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    DivisionByZero,
    StackUnderflow,
    UnknownWord,
    InvalidWord,
}

impl Forth {
    pub fn new() -> Forth {
        Forth {
            stack: Vec::new(),
            words: HashMap::new(),
        }
    }

    pub fn stack(&self) -> &[Value] {
        self.stack.as_slice()
    }

    pub fn eval(&mut self, input: &str) -> Result {
        let normalized = input.to_lowercase();
        let mut tokens = normalized.split(' ');
        while let Some(token) = tokens.next() {
            if let Some(exp) = self.words.get(token) {
                self.eval(exp.join(" ").as_str())?;
                continue;
            }

            match token {
                ":" => {
                    if let Some(word) = tokens.next() {
                        self.words.insert(word.to_string(), Vec::new());
                        while let Some(t) = tokens.next() {
                            if t == ";" {
                                break;
                            }
                            let ent = self.words.get_mut(word).unwrap();
                            ent.push(t.to_string());
                        }
                    }
                }
                "+" => match (self.stack.pop(), self.stack.pop()) {
                    (Some(x), Some(y)) => self.stack.push(x + y),
                    _ => return Err(Error::StackUnderflow),
                },
                "-" => match (self.stack.pop(), self.stack.pop()) {
                    (Some(x), Some(y)) => self.stack.push(y - x),
                    _ => return Err(Error::StackUnderflow),
                },
                "*" => match (self.stack.pop(), self.stack.pop()) {
                    (Some(x), Some(y)) => self.stack.push(y * x),
                    _ => return Err(Error::StackUnderflow),
                },
                "/" => match (self.stack.pop(), self.stack.pop()) {
                    (Some(0), Some(_)) => return Err(Error::DivisionByZero),
                    (Some(x), Some(y)) => self.stack.push(y / x),
                    _ => return Err(Error::StackUnderflow),
                },
                "dup" => match self.stack.last() {
                    Some(x) => self.stack.push(*x),
                    _ => return Err(Error::StackUnderflow),
                },
                "drop" => match self.stack.pop() {
                    Some(_) => {}
                    _ => return Err(Error::StackUnderflow),
                },
                "swap" => match (self.stack.pop(), self.stack.pop()) {
                    (Some(x), Some(y)) => {
                        self.stack.push(x);
                        self.stack.push(y);
                    }
                    _ => return Err(Error::StackUnderflow),
                },
                "over" => match self.stack[..] {
                    [.., x, _] => self.stack.push(x),
                    _ => return Err(Error::StackUnderflow),
                },
                _ => {
                    if let Ok(n) = token.parse() {
                        self.stack.push(n);
                    }
                }
            }
        }

        Ok(())
    }
}
