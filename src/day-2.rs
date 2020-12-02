mod parser;

use crate::parser::Parser;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() -> Result<(), Box<dyn Error>> {
    let file = BufReader::new(File::open("data/input-2")?);

    let mut valid_part_1 = 0;
    let mut valid_part_2 = 0;
    for line in file.lines() {
        let line = line?;
        let mut parser = Parser::new(&line);

        let n1: usize = parser.consume_until('-').unwrap().parse().unwrap();
        let n2: usize = parser.consume_until(' ').unwrap().parse().unwrap();
        let letter: char = parser.consume_until(':').unwrap().parse().unwrap();
        let pass = &parser.into_inner()[1..];

        let num = pass.matches(letter).count();
        if num >= n1 && num <= n2 {
            valid_part_1 += 1;
        }

        if (pass.chars().nth(n1 - 1) == Some(letter)) ^ (pass.chars().nth(n2 - 1) == Some(letter)) {
            valid_part_2 += 1;
        }
    }

    eprintln!("valid_part_1 = {:?}", valid_part_1);
    eprintln!("valid_part_2 = {:?}", valid_part_2);

    Ok(())
}
