use crate::data::{Data, ParseBytes, TryFromBytes};
use itertools::Itertools;

struct BoardingPass {
    row: u8,
    column: u8,
}

#[derive(Copy, Clone, PartialEq)]
enum SeatState {
    Taken,
    Available,
}

pub fn solve() -> (i64, i64) {
    let mut seat_states = vec![SeatState::Available; BoardingPass::NUM_SEATS];
    let mut max_used_seat = 0;

    for line in Data::read(5).lines() {
        let boarding_pass: BoardingPass = line.parse_bytes();
        let seat_id = boarding_pass.seat_id();
        seat_states[seat_id] = SeatState::Taken;
        max_used_seat = max_used_seat.max(seat_id);
    }

    let mut my_seat_id = 0;
    for available_id in seat_states
        .iter()
        .positions(|&state| state == SeatState::Available)
    {
        if available_id > 1
            && seat_states[available_id - 1] == SeatState::Taken
            && seat_states[available_id + 1] == SeatState::Taken
        {
            my_seat_id = available_id;
            break;
        }
    }

    (max_used_seat as i64, my_seat_id as i64)
}

impl TryFromBytes for BoardingPass {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 10 {
            return None;
        }

        let row_bit = |pos: usize| ((bytes[pos] == b'B') as u8) << (6 - pos);
        let row = row_bit(0)
            + row_bit(1)
            + row_bit(2)
            + row_bit(3)
            + row_bit(4)
            + row_bit(5)
            + row_bit(6);

        let column_bit = |pos: usize| ((bytes[pos + 7] == b'R') as u8) << (2 - pos);
        let column = column_bit(0) + column_bit(1) + column_bit(2);

        Some(BoardingPass { row, column })
    }
}

impl BoardingPass {
    const NUM_SEATS: usize = 1 << 10;

    fn seat_id(&self) -> usize {
        self.row as usize * 8 + self.column as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let pass: BoardingPass = b"FBFBBFFRLR".parse_bytes();
        assert_eq!(pass.row, 44);
        assert_eq!(pass.column, 5);
    }
}
