use std::collections::HashMap;
use std::rc::Rc;

pub type Value = i32;
pub type Result = std::result::Result<(), Error>;
pub type Func = Rc<dyn Fn(&mut Forth) -> Result>;

pub struct Forth {
    stack: Vec<Value>,
    words: HashMap<String, Func>,
}

#[derive(Clone)]
enum Token {
    Val(Value),
    Fun(Func),
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
        let mut words: HashMap<String, Func> = HashMap::new();
        words.insert("+".to_string(), Rc::new(Self::add));
        words.insert("-".to_string(), Rc::new(Self::sub));
        words.insert("*".to_string(), Rc::new(Self::mul));
        words.insert("/".to_string(), Rc::new(Self::div));
        words.insert("dup".to_string(), Rc::new(Self::dup));
        words.insert("drop".to_string(), Rc::new(Self::drop));
        words.insert("swap".to_string(), Rc::new(Self::swap));
        words.insert("over".to_string(), Rc::new(Self::over));
        Forth {
            stack: Vec::new(),
            words,
        }
    }

    pub fn stack(&self) -> &[Value] {
        self.stack.as_slice()
    }

    fn add(&mut self) -> Result {
        match (self.stack.pop(), self.stack.pop()) {
            (Some(x), Some(y)) => Ok(self.stack.push(x + y)),
            _ => Err(Error::StackUnderflow),
        }
    }

    fn sub(&mut self) -> Result {
        match (self.stack.pop(), self.stack.pop()) {
            (Some(x), Some(y)) => Ok(self.stack.push(y - x)),
            _ => Err(Error::StackUnderflow),
        }
    }

    fn mul(&mut self) -> Result {
        match (self.stack.pop(), self.stack.pop()) {
            (Some(x), Some(y)) => Ok(self.stack.push(y * x)),
            _ => Err(Error::StackUnderflow),
        }
    }

    fn div(&mut self) -> Result {
        match (self.stack.pop(), self.stack.pop()) {
            (Some(0), Some(_)) => return Err(Error::DivisionByZero),
            (Some(x), Some(y)) => Ok(self.stack.push(y / x)),
            _ => Err(Error::StackUnderflow),
        }
    }

    fn dup(&mut self) -> Result {
        match self.stack.last() {
            Some(x) => Ok(self.stack.push(*x)),
            _ => Err(Error::StackUnderflow),
        }
    }

    fn drop(&mut self) -> Result {
        match self.stack.pop() {
            Some(_) => Ok(()),
            _ => Err(Error::StackUnderflow),
        }
    }

    fn swap(&mut self) -> Result {
        match (self.stack.pop(), self.stack.pop()) {
            (Some(x), Some(y)) => {
                self.stack.push(x);
                self.stack.push(y);
                Ok(())
            }
            _ => Err(Error::StackUnderflow),
        }
    }

    fn over(&mut self) -> Result {
        match self.stack[..] {
            [.., x, _] => Ok(self.stack.push(x)),
            _ => return Err(Error::StackUnderflow),
        }
    }

    fn compile(&mut self, input: &Vec<&str>) -> std::result::Result<Vec<Token>, Error> {
        let mut ops: Vec<Token> = Vec::new();
        let mut tokens = input.iter();
        'outer: while let Some(&token) = tokens.next() {
            match token {
                ":" => {
                    if let Some(word) = tokens.next() {
                        if word.parse::<i32>().is_ok() {
                            return Err(Error::InvalidWord);
                        }

                        let mut definition: Vec<&str> = Vec::new();
                        while let Some(&t) = tokens.next() {
                            if t == ";" {
                                let compiled = self.compile(&definition)?;
                                self.words
                                    .insert(word.to_string(), Rc::new(move |s| s.exec(&compiled)));

                                continue 'outer;
                            }
                            definition.push(t);
                        }
                    }

                    return Err(Error::InvalidWord);
                }
                c => {
                    if let Ok(n) = c.parse() {
                        ops.push(Token::Val(n));
                        continue;
                    }
                    if let Some(op) = self.words.get(c) {
                        ops.push(Token::Fun(op.clone()));
                        continue;
                    }

                    return Err(Error::UnknownWord);
                }
            }
        }

        Ok(ops)
    }

    fn exec(&mut self, input: &Vec<Token>) -> Result {
        for op in input {
            match op {
                Token::Val(n) => self.stack.push(*n),
                Token::Fun(f) => f(self)?,
            }
        }

        Ok(())
    }

    pub fn eval(&mut self, input: &str) -> Result {
        let normalized = input.to_lowercase();
        let tokens: Vec<&str> = normalized.split(' ').collect();
        let tokens: Vec<Token> = self.compile(&tokens)?;

        self.exec(&tokens)
    }
}
