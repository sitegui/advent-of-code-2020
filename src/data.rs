use crate::parser::Parser;
use std::cell::RefCell;
use std::fs;
use std::rc::Rc;
use std::str::FromStr;

/// Represents the input data
#[derive(Debug, Clone)]
pub struct Data {
    bytes: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
pub struct Split<'a> {
    parser: Parser<'a>,
    separator: u8,
    ignore_last_empty: bool,
}

#[derive(Debug, Clone)]
pub struct Paragraphs<'a>(Rc<RefCell<Split<'a>>>);

#[derive(Debug)]
pub struct ParagraphLines<'a>(Rc<RefCell<Split<'a>>>);

impl Data {
    pub fn read(day: usize) -> Self {
        Data {
            bytes: fs::read(format!("data/input-{}", day)).unwrap(),
        }
    }

    #[allow(dead_code)]
    pub fn read_example() -> Self {
        Data {
            bytes: fs::read("data/example").unwrap(),
        }
    }

    pub fn split(&self, separator: u8, ignore_last_empty: bool) -> Split<'_> {
        Split {
            parser: Parser::new(&self.bytes),
            separator,
            ignore_last_empty,
        }
    }

    pub fn lines(&self) -> Split<'_> {
        self.split(b'\n', true)
    }

    pub fn paragraphs(&self) -> Paragraphs<'_> {
        Paragraphs(Rc::new(RefCell::new(self.lines())))
    }
}

impl<'a> Iterator for Split<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        self.parser.try_consume_until(self.separator).or_else(|| {
            let rest = self.parser.into_inner();
            let ignore_empty = self.ignore_last_empty;
            self.ignore_last_empty = true;
            if rest.is_empty() && ignore_empty {
                None
            } else {
                Some(rest)
            }
        })
    }
}

impl<'a> Iterator for Paragraphs<'a> {
    type Item = ParagraphLines<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let parser = &self.0.borrow().parser;
        if parser.is_empty() {
            None
        } else {
            Some(ParagraphLines(self.0.clone()))
        }
    }
}

impl<'a> Iterator for ParagraphLines<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.borrow_mut().next() {
            Some(line) if !line.is_empty() => Some(line),
            _ => None,
        }
    }
}

pub trait TryFromBytes: Sized {
    fn try_from_bytes(bytes: Parser<'_>) -> Option<Self>;
}

impl<T: FromStr> TryFromBytes for T {
    fn try_from_bytes(bytes: Parser<'_>) -> Option<Self> {
        std::str::from_utf8(bytes.into_inner())
            .ok()
            .and_then(|s| s.parse().ok())
    }
}

pub trait ParseBytes {
    fn try_parse_bytes<F: TryFromBytes>(&self) -> Option<F>;

    fn parse_bytes<F: TryFromBytes>(&self) -> F {
        self.try_parse_bytes().unwrap()
    }
}

impl ParseBytes for [u8] {
    fn try_parse_bytes<F: TryFromBytes>(&self) -> Option<F> {
        F::try_from_bytes(Parser::new(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;

    #[test]
    fn paragraphs() {
        let data = Data {
            bytes: "abc\n\na\nb\nc\n\nab\nac\n\nb\n".to_owned().into_bytes(),
        };

        let lines = data.paragraphs().map(|p| p.collect_vec()).collect_vec();

        assert_eq!(
            lines,
            vec![
                vec![b"abc".as_ref()],
                vec![b"a", b"b", b"c"],
                vec![b"ab", b"ac"],
                vec![b"b"]
            ]
        );
    }
}
