#![allow(clippy::naive_bytecount)]

use std::env::args;
use std::time::Instant;

mod data;
mod parser;

struct Day {
    label: &'static str,
    solve_fn: fn() -> (usize, usize),
    expected: (usize, usize),
}

impl Day {
    fn solve(&self) -> (usize, usize) {
        let start = Instant::now();
        let answer = (self.solve_fn)();
        println!("{} solved in {:?}", self.label, start.elapsed());
        answer
    }

    fn assert_solve(&self) {
        let answer = self.solve();
        assert_eq!(answer, self.expected, "when checking {}", self.label);
    }
}

macro_rules! days {
    ($($day:ident = ($part_1:expr, $part_2:expr)),* $(,)?) => {
        $(mod $day;)*

        const DAYS: &[Day] = &[
            $(Day {
                label: stringify!($day),
                solve_fn: $day::solve,
                expected: ($part_1, $part_2)
            }),*
        ];
    };
}

days! {
    day_1 = (788739, 178724430),
    day_2 = (424, 747),
    day_3 = (205, 3952146825),
    day_4 = (254, 184),
    day_5 = (861, 633),
    day_6 = (6170, 2947),
}

fn main() {
    match args().nth(1) {
        None => {
            println!("Will execute all days");
            let start = Instant::now();
            for day in DAYS {
                day.assert_solve();
            }
            println!("{} days solved in {:?}", DAYS.len(), start.elapsed());
        }
        Some(day) => {
            let day: usize = day.parse().unwrap();
            let (part_1, part_2) = DAYS[day - 1].solve();
            println!("Part 1 = {}, part 2 = {}", part_1, part_2);
        }
    }
}
