pub type Value = i32;
pub type Result = std::result::Result<(), Error>;

pub struct Forth(Vec<Value>);

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    DivisionByZero,
    StackUnderflow,
    UnknownWord,
    InvalidWord,
}

impl Forth {
    pub fn new() -> Forth {
        Forth(Vec::new())
    }

    pub fn stack(&self) -> &[Value] {
        self.0.as_slice()
    }

    pub fn eval(&mut self, input: &str) -> Result {
        for word in input.to_lowercase().split(' ') {
            match word {
                "+" => match (self.0.pop(), self.0.pop()) {
                    (Some(x), Some(y)) => self.0.push(x + y),
                    _ => return Err(Error::StackUnderflow),
                },
                "-" => match (self.0.pop(), self.0.pop()) {
                    (Some(x), Some(y)) => self.0.push(y - x),
                    _ => return Err(Error::StackUnderflow),
                },
                "*" => match (self.0.pop(), self.0.pop()) {
                    (Some(x), Some(y)) => self.0.push(y * x),
                    _ => return Err(Error::StackUnderflow),
                },
                "/" => match (self.0.pop(), self.0.pop()) {
                    (Some(0), Some(_)) => return Err(Error::DivisionByZero),
                    (Some(x), Some(y)) => self.0.push(y / x),
                    _ => return Err(Error::StackUnderflow),
                },
                "dup" => match self.0.last() {
                    Some(x) => self.0.push(*x),
                    _ => return Err(Error::StackUnderflow),
                },
                "drop" => match self.0.pop() {
                    Some(_) => {}
                    _ => return Err(Error::StackUnderflow),
                },
                "swap" => match (self.0.pop(), self.0.pop()) {
                    (Some(x), Some(y)) => {
                        self.0.push(x);
                        self.0.push(y);
                    }
                    _ => return Err(Error::StackUnderflow),
                },
                "over" => match self.0[..] {
                    [.., x, _] => self.0.push(x),
                    _ => return Err(Error::StackUnderflow),
                },
                _ => {
                    if let Ok(n) = word.parse() {
                        self.0.push(n);
                    }
                }
            }
        }

        Ok(())
    }
}
