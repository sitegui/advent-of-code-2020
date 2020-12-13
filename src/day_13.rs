use crate::data::{Data, ParseBytes};

pub fn solve() -> (i64, i64) {
    let data = Data::read(13);
    let mut lines = data.lines();

    let earliest_departure: i64 = lines.next().unwrap().parse_bytes();
    let mut matching_departure: i64 = 0;
    let mut matching_steps: i64 = 1;

    let best = lines
        .next()
        .unwrap()
        .split(|&byte| byte == b',')
        .enumerate()
        .filter(|&(_, id)| id != b"x")
        .map(|(column, bus)| {
            let bus: i64 = bus.parse_bytes();

            // Calculate the wait time, for part 1
            let wait = match earliest_departure % bus {
                0 => 0,
                diff => bus - diff,
            };

            // Calculate the nest matching departure, for part 2
            let column = column as i64;
            let target_delay = (-column).rem_euclid(bus);
            while matching_departure % bus != target_delay as i64 {
                matching_departure += matching_steps;
            }
            matching_steps *= bus;

            (bus, wait)
        })
        .min_by_key(|&(_, wait)| wait)
        .unwrap();

    let part_1 = best.0 as i64 * best.1 as i64;

    (part_1, matching_departure)
}
