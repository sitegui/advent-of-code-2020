use crate::data::{Data, ParseBytes};
use crate::parser::Parser;

pub fn solve() -> (i64, i64) {
    let mut valid_part_1 = 0;
    let mut valid_part_2 = 0;
    for line in Data::read(2).lines() {
        let mut parser = Parser::new(&line);

        let n1: usize = parser.consume_until(b'-').parse_bytes();
        let n2: usize = parser.consume_until(b' ').parse_bytes();
        let letter = parser.consume_until(b':').parse_bytes::<char>() as u8;
        let pass = &parser.into_inner()[1..];

        let num = pass.iter().filter(|&&b| b == letter).count();
        if num >= n1 && num <= n2 {
            valid_part_1 += 1;
        }

        if (pass.get(n1 - 1) == Some(&letter)) ^ (pass.get(n2 - 1) == Some(&letter)) {
            valid_part_2 += 1;
        }
    }

    (valid_part_1, valid_part_2)
}
