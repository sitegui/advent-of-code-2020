use crate::data::Data;
use crate::parser::Parser;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Token {
    Add,
    Mul,
    Digit(i64),
    OpenPar,
    ClosePar,
}

pub fn solve() -> (i64, i64) {
    let data = Data::read(18);

    let mut total_p1 = 0;
    let mut total_p2 = 0;
    for mut line in data.lines() {
        let mut line_p2 = line;
        total_p1 += eval_level_p1(&mut line);
        total_p2 += eval_level_p2(&mut line_p2);
    }

    (total_p1, total_p2)
}

/// Evaluate the expression until the end of the parenthesis close, whichever comes first
fn eval_level_p1(expr: &mut &[u8]) -> i64 {
    let next_value = |sub_expr: &mut &[u8]| match Token::next(sub_expr) {
        Some(Token::OpenPar) => eval_level_p1(sub_expr),
        Some(Token::Digit(d)) => d,
        _ => unreachable!(),
    };

    let mut result = next_value(expr);

    loop {
        match Token::next(expr) {
            None => return result,
            Some(Token::Mul) => result *= next_value(expr),
            Some(Token::Add) => result += next_value(expr),
            Some(Token::ClosePar) => return result,
            _ => unreachable!(),
        }
    }
}

/// Evaluate the expression until the end of the parenthesis close, whichever comes first
fn eval_level_p2(expr: &mut &[u8]) -> i64 {
    let next_value = |sub_expr: &mut &[u8]| match Token::next(sub_expr) {
        Some(Token::OpenPar) => eval_level_p2(sub_expr),
        Some(Token::Digit(d)) => d,
        _ => unreachable!(),
    };

    // Added values, that will be multiplied at the end
    let mut to_multiply = Vec::with_capacity(32);

    // Values being added
    let mut accumulator = next_value(expr);

    loop {
        match Token::next(expr) {
            None => break,
            Some(Token::Mul) => {
                to_multiply.push(accumulator);
                accumulator = next_value(expr);
            }
            Some(Token::Add) => accumulator += next_value(expr),
            Some(Token::ClosePar) => break,
            _ => unreachable!(),
        }
    }

    to_multiply.push(accumulator);
    to_multiply.into_iter().product()
}

impl Token {
    fn next(expr: &mut &[u8]) -> Option<Self> {
        loop {
            let byte = match expr.try_consume_byte() {
                None => return None,
                Some(b' ') => continue,
                Some(byte) => byte,
            };

            return match byte {
                b'+' => Some(Token::Add),
                b'*' => Some(Token::Mul),
                b'(' => Some(Token::OpenPar),
                b')' => Some(Token::ClosePar),
                d @ b'0'..=b'9' => Some(Token::Digit((d - b'0') as i64)),
                _ => unreachable!(),
            };
        }
    }
}
