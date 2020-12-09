use crate::data::{Data, ParseBytes};
use crate::parser::Parser;

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

pub fn solve() -> (i64, i64) {
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

    for line in Data::read(4).lines() {
        if line.is_empty() {
            check_validity(&mut passport);
            continue;
        }

        let mut parser = Parser::new(&line);
        loop {
            let field = parser.consume_until(b':');
            match parser.try_consume_until(b' ') {
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
    fn update(&mut self, field: &[u8], value: &[u8]) {
        match field {
            b"byr" => self.byr = Passport::check_int(value, 1920, 2002),
            b"iyr" => self.iyr = Passport::check_int(value, 2010, 2020),
            b"eyr" => self.eyr = Passport::check_int(value, 2020, 2030),
            b"hgt" => self.hgt = Passport::check_hgt(value),
            b"hcl" => self.hcl = Passport::check_hcl(value),
            b"ecl" => self.ecl = Passport::check_ecl(value),
            b"pid" => self.pid = Passport::check_pid(value),
            b"cid" => self.cid = Field::Valid,
            _ => unreachable!(),
        }
    }

    fn check_hgt(value: &[u8]) -> Field {
        if value.ends_with(b"cm") {
            Passport::check_int(&value[..value.len() - 2], 150, 193)
        } else if value.ends_with(b"in") {
            Passport::check_int(&value[..value.len() - 2], 59, 76)
        } else {
            Field::Invalid
        }
    }

    fn check_hcl(value: &[u8]) -> Field {
        if value.starts_with(b"#")
            && value.len() == 7
            && value.iter().skip(1).all(|&byte| Passport::is_hex(byte))
        {
            Field::Valid
        } else {
            Field::Invalid
        }
    }

    fn check_int(value: &[u8], min: i32, max: i32) -> Field {
        match value.try_parse_bytes::<i32>() {
            Some(n) if n >= min && n <= max => Field::Valid,
            _ => Field::Invalid,
        }
    }

    fn is_hex(c: u8) -> bool {
        match c {
            b'0'..=b'9' | b'a'..=b'f' => true,
            _ => false,
        }
    }

    fn check_ecl(value: &[u8]) -> Field {
        match value {
            b"amb" | b"blu" | b"brn" | b"gry" | b"grn" | b"hzl" | b"oth" => Field::Valid,
            _ => Field::Invalid,
        }
    }

    fn check_pid(value: &[u8]) -> Field {
        if value.len() == 9 && value.iter().all(|&c| c.is_ascii_digit()) {
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
