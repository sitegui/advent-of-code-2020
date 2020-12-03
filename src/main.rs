use std::env::args;
use std::time::Instant;

mod parser;

struct Day {
    label: &'static str,
    solve_fn: fn(),
}

impl Day {
    fn solve(&self) {
        let start = Instant::now();
        (self.solve_fn)();
        println!("{} solved in {:?}", self.label, start.elapsed());
    }
}

macro_rules! days {
    ($($day:ident),* $(,)?) => {
        $(mod $day;)*

        const DAYS: &[Day] = &[
            $(Day {
                label: stringify!($day),
                solve_fn: $day::solve,
            }),*
        ];
    };
}

days! {day_1, day_2, day_3}

fn main() {
    match args().nth(1) {
        None => {
            println!("Will execute all days");
            let start = Instant::now();
            for day in DAYS {
                day.solve();
            }
            println!("{} days solved in {:?}", DAYS.len(), start.elapsed());
        }
        Some(day) => {
            let day: usize = day.parse().unwrap();
            DAYS[day - 1].solve();
        }
    }
}
