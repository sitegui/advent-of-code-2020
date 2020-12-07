use std::ops::Deref;

#[derive(Debug, Copy, Clone)]
pub struct Parser<'a>(&'a [u8]);

impl<'a> Parser<'a> {
    pub fn new(inner: &'a [u8]) -> Self {
        Parser(inner)
    }

    pub fn consume_bytes(&mut self, n: usize) -> &'a [u8] {
        let result = &self.0[..n];
        self.0 = &self.0[n..];
        result
    }

    pub fn consume_until(&mut self, target_byte: u8) -> &'a [u8] {
        self.try_consume_until(target_byte).unwrap()
    }

    pub fn try_consume_until(&mut self, target_byte: u8) -> Option<&'a [u8]> {
        match self.iter().position(|&byte| byte == target_byte) {
            None => None,
            Some(pos) => Some(&self.consume_bytes(pos + 1)[..pos]),
        }
    }

    pub fn consume_words(&mut self, n: usize) -> &'a [u8] {
        self.try_consume_words(n).unwrap()
    }

    pub fn try_consume_words(&mut self, mut n: usize) -> Option<&'a [u8]> {
        let pos = self.iter().position(|&byte| {
            if byte == b' ' {
                n -= 1;
            }
            n == 0
        });

        if let Some(pos) = pos {
            // Stopped early: found all words
            Some(&self.consume_bytes(pos + 1)[..pos])
        } else if n == 1 && !self.is_empty() {
            // Last word has no spaces
            Some(self.consume_bytes(self.len()))
        } else {
            None
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consume_until() {
        let mut parser = Parser::new(b"ba be bi bo bu");

        assert_eq!(parser.consume_bytes(3), b"ba ");
        assert_eq!(parser.consume_until(b' '), b"be");
        assert_eq!(parser.into_inner(), b"bi bo bu");
    }

    #[test]
    fn consume_words() {
        let mut parser = Parser::new(b"ba be bi bo bu");

        assert_eq!(parser.consume_words(1), b"ba");
        assert_eq!(parser.consume_words(2), b"be bi");
        assert_eq!(parser.consume_words(2), b"bo bu");
        assert_eq!(parser.try_consume_words(1), None);
        assert_eq!(parser.try_consume_words(2), None);

        let mut parser = Parser::new(b"ba be bi bo bu");
        assert_eq!(parser.consume_words(5), b"ba be bi bo bu");
    }
}
