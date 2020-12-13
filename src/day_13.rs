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

            // Calculate the nest matching departure, for part 2. `matching_departure` is the answer
            // so far, considering the buses before this one. If that answer is increased of
            // `matching_steps`, it is still a valid answer. However, it is no longer the *least*
            // possible value
            // For this new bus, we need to guarantee one extra condition:
            // `new_matching_departure + column = K * bus`, for some integer K
            // Taking module `bus` from both sides:
            // `new_matching_departure + column = 0 (mod bus)`
            // Rewriting to have a simpler equation:
            // `new_matching_departure % bus = reminder(-column, bus)`, where `reminder()` gives a
            // positive reminder, even for negative arguments.
            // Since we want to respect this new constraint, while respecting the previous ones, we
            // must have:
            // `new_matching_departure = matching_departure + K * matching_steps`, so we simply
            // iterate over values for `K` to find the answer :)
            // Then we know that `new_matching_steps` is the greatest common multiplier of all buses
            // so far. By inspection, they seem to be prime numbers, so that greatly simplify to
            // their product.
            let target_delay = (-(column as i64)).rem_euclid(bus);
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
