use crate::data::Data;
use itertools::Itertools;
use std::iter;

type Cup = u32;
const NUM_CUPS_P1: usize = 9;
const NUM_CUPS_P2: usize = 1_000_000;
const NUM_MOVES_P1: usize = 100;
const NUM_MOVES_P2: usize = 10_000_000;

#[derive(Debug)]
struct Cups {
    next_by_cup: Vec<Cup>,
    max_cup: Cup,
    current: Cup,
}

pub fn solve() -> (i64, i64) {
    let data = Data::read(23);

    let base_cups: Vec<Cup> = data
        .lines()
        .next()
        .unwrap()
        .iter()
        .map(|&b| (b - b'0') as u32)
        .collect();

    let mut cups_p1 = Cups::new(&base_cups, NUM_CUPS_P1);
    let mut cups_p2 = Cups::new(&base_cups, NUM_CUPS_P2);

    for _ in 0..NUM_MOVES_P1 {
        cups_p1.apply_move();
    }
    let part_1 = cups_p1.get(1, 8).into_iter().join("").parse().unwrap();

    for _ in 0..NUM_MOVES_P2 {
        cups_p2.apply_move();
    }
    let after_1 = cups_p2.get(1, 2);
    let part_2 = after_1[0] as i64 * after_1[1] as i64;

    (part_1, part_2)
}

impl Cups {
    fn new(base_cups: &[Cup], num_cups: usize) -> Self {
        let more_cups = base_cups.len() as u32 + 1..=num_cups as u32;
        let cups = base_cups
            .iter()
            .copied()
            .chain(more_cups)
            .chain(iter::once(base_cups[0]));

        let mut next_by_cup = vec![0; num_cups + 1];
        for (cup, next_cup) in cups.tuple_windows::<(_, _)>() {
            next_by_cup[cup as usize] = next_cup;
        }

        Cups {
            next_by_cup,
            max_cup: num_cups as u32,
            current: base_cups[0],
        }
    }

    fn get(&self, start: Cup, len: usize) -> Vec<Cup> {
        let mut result = Vec::with_capacity(len);

        let mut current = start;
        while result.len() < len {
            current = self.next_by_cup[current as usize];
            result.push(current);
        }

        result
    }

    fn apply_move(&mut self) {
        // Determine the destination cup
        let mut destination = self.current;
        let x = self.next_by_cup[self.current as usize];
        let y = self.next_by_cup[x as usize];
        let z = self.next_by_cup[y as usize];
        loop {
            destination -= 1;
            if destination == 0 {
                destination = self.max_cup;
            }
            if destination != x && destination != y && destination != z {
                break;
            }
        }

        let after_destination = self.next_by_cup[destination as usize];
        let after_z = self.next_by_cup[z as usize];

        self.next_by_cup[destination as usize] = x;
        self.next_by_cup[z as usize] = after_destination;
        self.next_by_cup[self.current as usize] = after_z;
        self.current = after_z;
    }
}
