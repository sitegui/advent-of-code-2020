use crate::data::Data;
use crate::iter_utils::IterUtils;

pub fn solve() -> (i64, i64) {
    let data = Data::read(25);

    let lines: Vec<u64> = data.lines().parsed().collect();
    let card_public_key = lines[0];
    let door_public_key = lines[1];

    let card_private_key = find_loop_size(card_public_key);
    let encryption_key = transform(door_public_key, card_private_key);

    (encryption_key as i64, 0)
}

fn find_loop_size(public_key: u64) -> u64 {
    let mut value = 1;
    let mut loops = 1;

    loop {
        value = (value * 7) % 20201227;

        if value == public_key {
            return loops;
        }
        loops += 1;
    }
}

fn transform(subject: u64, mut loops: u64) -> u64 {
    let mut value = 1;

    while loops > 0 {
        value = (value * subject) % 20201227;
        loops -= 1;
    }

    value
}
