use std::ops::Deref;

pub struct Parser<'a>(&'a str);

impl<'a> Parser<'a> {
    pub fn new(inner: &'a str) -> Self {
        Parser(inner)
    }

    pub fn consume_until(&mut self, c: char) -> Option<&'a str> {
        match self.find(c) {
            None => None,
            Some(pos) => {
                let result = &self.0[..pos];
                self.0 = &self.0[pos + 1..];
                Some(result)
            }
        }
    }

    pub fn into_inner(self) -> &'a str {
        self.0
    }
}

impl<'a> Deref for Parser<'a> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
