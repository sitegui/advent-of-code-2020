use crate::parser::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Default, Debug)]
struct Passport {
    byr: Field,
    iyr: Field,
    eyr: Field,
    hgt: Field,
    hcl: Field,
    ecl: Field,
    pid: Field,
    cid: Field,
}

#[derive(Debug, PartialEq)]
enum Field {
    Missing,
    Invalid,
    Valid,
}

impl Default for Field {
    fn default() -> Self {
        Field::Missing
    }
}

pub fn solve() -> (usize, usize) {
    let file = BufReader::new(File::open("data/input-4").unwrap());

    let mut passport = Passport::default();
    let mut num_valid_1 = 0;
    let mut num_valid_2 = 0;

    let mut check_validity = |passport: &mut Passport| {
        if passport.is_valid_part_1() {
            num_valid_1 += 1;
        }
        if passport.is_valid_part_2() {
            num_valid_2 += 1;
        }
        *passport = Passport::default();
    };

    for line in file.lines() {
        let line = line.unwrap();

        if line.is_empty() {
            check_validity(&mut passport);
            continue;
        }

        let mut parser = Parser::new(&line);
        loop {
            let field = parser.consume_until(':').unwrap();
            match parser.consume_until(' ') {
                None => {
                    passport.update(field, parser.into_inner());
                    break;
                }
                Some(value) => {
                    passport.update(field, value);
                }
            }
        }
    }

    check_validity(&mut passport);

    (num_valid_1, num_valid_2)
}

impl Passport {
    fn update(&mut self, field: &str, value: &str) {
        match field {
            "byr" => self.byr = Passport::check_int(value, 1920, 2002),
            "iyr" => self.iyr = Passport::check_int(value, 2010, 2020),
            "eyr" => self.eyr = Passport::check_int(value, 2020, 2030),
            "hgt" => self.hgt = Passport::check_hgt(value),
            "hcl" => self.hcl = Passport::check_hcl(value),
            "ecl" => self.ecl = Passport::check_ecl(value),
            "pid" => self.pid = Passport::check_pid(value),
            "cid" => self.cid = Field::Valid,
            _ => unreachable!(),
        }
    }

    fn check_hgt(value: &str) -> Field {
        if value.ends_with("cm") {
            Passport::check_int(&value[..value.len() - 2], 150, 193)
        } else if value.ends_with("in") {
            Passport::check_int(&value[..value.len() - 2], 59, 76)
        } else {
            Field::Invalid
        }
    }

    fn check_hcl(value: &str) -> Field {
        if value.starts_with('#') && value.len() == 7 && value.chars().skip(1).all(Passport::is_hex)
        {
            Field::Valid
        } else {
            Field::Invalid
        }
    }

    fn check_int(value: &str, min: i32, max: i32) -> Field {
        match value.parse::<i32>() {
            Ok(n) if n >= min && n <= max => Field::Valid,
            _ => Field::Invalid,
        }
    }

    fn is_hex(c: char) -> bool {
        match c {
            '0'..='9' | 'a'..='f' => true,
            _ => false,
        }
    }

    fn check_ecl(value: &str) -> Field {
        match value {
            "amb" | "blu" | "brn" | "gry" | "grn" | "hzl" | "oth" => Field::Valid,
            _ => Field::Invalid,
        }
    }

    fn check_pid(value: &str) -> Field {
        if value.len() == 9 && value.chars().all(|c| c.is_ascii_digit()) {
            Field::Valid
        } else {
            Field::Invalid
        }
    }

    fn is_valid_part_1(&self) -> bool {
        self.byr != Field::Missing
            && self.iyr != Field::Missing
            && self.eyr != Field::Missing
            && self.hgt != Field::Missing
            && self.hcl != Field::Missing
            && self.ecl != Field::Missing
            && self.pid != Field::Missing
    }

    fn is_valid_part_2(&self) -> bool {
        self.byr == Field::Valid
            && self.iyr == Field::Valid
            && self.eyr == Field::Valid
            && self.hgt == Field::Valid
            && self.hcl == Field::Valid
            && self.ecl == Field::Valid
            && self.pid == Field::Valid
    }
}
