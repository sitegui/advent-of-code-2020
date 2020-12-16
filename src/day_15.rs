use crate::data::Data;
use crate::iter_utils::IterUtils;
use crate::parser::Parser;
use itertools::Itertools;
use std::mem;
use std::ops::Range;

const PART_1_TURNS: i32 = 2_020;
const PART_2_TURNS: i32 = 30_000_000;
const MAX_VALUE: i32 = 30_000_000;

struct Memory {
    /// `last_turns[value]` indicates the number of the last turn in which this value was saw, or
    /// zero if never.
    last_turns: Vec<i32>,
}

pub fn solve() -> (i64, i64) {
    let data = Data::read(15);
    let starting = data
        .bytes()
        .split_byte(b',', true)
        .parsed::<i32>()
        .collect_vec();

    let mut memory = Memory::new();

    for (turn_from_zero, &value) in starting[..starting.len() - 1].iter().enumerate() {
        let turn = turn_from_zero as i32 + 1;
        assert_eq!(memory.insert(value, turn), 0);
    }

    let next = starting[starting.len() - 1];
    let start_turn = starting.len() as i32;
    let part_1 = play(&mut memory, start_turn..PART_1_TURNS, next);
    let part_2 = play(&mut memory, PART_1_TURNS..PART_2_TURNS, part_1);

    (part_1 as i64, part_2 as i64)
}

fn play(memory: &mut Memory, turns: Range<i32>, mut next: i32) -> i32 {
    for turn in turns {
        next = memory.insert(next, turn);
    }
    next
}

impl Memory {
    fn new() -> Self {
        Memory {
            last_turns: vec![0; MAX_VALUE as usize],
        }
    }

    /// Insert in the memory and return its age (or zero if new)
    fn insert(&mut self, value: i32, turn: i32) -> i32 {
        match mem::replace(&mut self.last_turns[value as usize], turn) {
            0 => 0,
            last_turn => turn - last_turn,
        }
    }
}
