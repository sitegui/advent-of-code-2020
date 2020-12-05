use crate::parser::Parser;
use std::fs;
use std::str::FromStr;

/// Represents the input data
#[derive(Debug, Clone)]
pub struct Data {
    bytes: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
pub struct Lines<'a>(Parser<'a>);

impl Data {
    pub fn read(day: usize) -> Self {
        Data {
            bytes: fs::read(format!("data/input-{}", day)).unwrap(),
        }
    }

    pub fn lines(&self) -> Lines<'_> {
        Lines(Parser::new(&self.bytes))
    }
}

impl<'a> Iterator for Lines<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        self.0.try_consume_until(b'\n').or_else(|| {
            let rest = self.0.into_inner();
            if rest.is_empty() {
                None
            } else {
                Some(rest)
            }
        })
    }
}

pub trait ParseBytes {
    fn try_parse_bytes<F: FromStr>(&self) -> Option<F>;

    fn parse_bytes<F: FromStr>(&self) -> F {
        self.try_parse_bytes().unwrap()
    }
}

impl ParseBytes for [u8] {
    fn try_parse_bytes<F: FromStr>(&self) -> Option<F> {
        std::str::from_utf8(self).ok().and_then(|s| s.parse().ok())
    }
}
