use std::ops::Deref;

#[derive(Debug, Copy, Clone)]
pub struct Parser<'a>(&'a [u8]);

impl<'a> Parser<'a> {
    pub fn new(inner: &'a [u8]) -> Self {
        Parser(inner)
    }

    pub fn consume_until(&mut self, target_byte: u8) -> &'a [u8] {
        self.try_consume_until(target_byte).unwrap()
    }

    pub fn try_consume_until(&mut self, target_byte: u8) -> Option<&'a [u8]> {
        match self.iter().position(|&byte| byte == target_byte) {
            None => None,
            Some(pos) => {
                let result = &self.0[..pos];
                self.0 = &self.0[pos + 1..];
                Some(result)
            }
        }
    }

    pub fn into_inner(self) -> &'a [u8] {
        self.0
    }
}

impl<'a> Deref for Parser<'a> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
