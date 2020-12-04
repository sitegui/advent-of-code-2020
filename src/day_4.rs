use crate::parser::Parser;
use itertools::Itertools;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn solve() -> (usize, usize) {
    let contents = fs::read_to_string("data/input-4").unwrap();

    let mut parser = Parser::new(&contents);

    todo!()
}

fn field_id(field: &str) -> usize {
    let index = match field {
        "byr" => 0,
        "iyr" => 1,
        "eyr" => 2,
        "hgt" => 3,
        "hcl" => 4,
        "ecl" => 5,
        "pid" => 6,
        "cid" => 7,
        _ => unreachable!(),
    };
    2 << index
}
