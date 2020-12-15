use crate::data::Data;
use crate::iter_utils::IterUtils;
use itertools::Itertools;
use std::collections::BTreeMap;
use std::ops::Range;

const PART_1_TURNS: i32 = 2_020;
const PART_2_TURNS: i32 = 30_000_000;

pub fn solve() -> (i64, i64) {
    let data = Data::read(15);
    let starting = data.split(b',', true).parsed::<i32>().collect_vec();

    let mut memory = BTreeMap::new();

    for (turn_from_zero, &value) in starting[..starting.len() - 1].iter().enumerate() {
        let turn = turn_from_zero as i32 + 1;
        assert!(memory.insert(value, turn).is_none());
    }

    let next = starting[starting.len() - 1];
    let start_turn = memory.len() as i32 + 1;
    let part_1 = play(&mut memory, start_turn..PART_1_TURNS, next);
    let part_2 = play(&mut memory, PART_1_TURNS..PART_2_TURNS, part_1);

    (part_1 as i64, part_2 as i64)
}

fn play(memory: &mut BTreeMap<i32, i32>, turns: Range<i32>, mut next: i32) -> i32 {
    for turn in turns {
        match memory.insert(next, turn) {
            None => next = 0,
            Some(prev_round) => next = turn - prev_round,
        }
    }
    next
}
