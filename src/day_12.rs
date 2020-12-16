use crate::data::{Data, ParseBytes, TryFromBytes};
use crate::iter_utils::IterUtils;
use crate::parser::Parser;

#[derive(Debug, Copy, Clone)]
enum Command {
    /// (EW, NS): E and N are positive
    Move(i16, i16),
    /// Clock-wise: R is positive
    Turn(i16),
    Forward(i16),
}

#[derive(Debug, Copy, Clone, Default)]
struct Position {
    /// East is positive
    ew: i16,
    /// North is positive
    ns: i16,
    /// 0 = facing east, 90 = facing south
    direction: i16,
}

pub fn solve() -> (i64, i64) {
    let mut ship_p1 = Position::default();
    let mut ship_p2 = Position::default();
    let mut waypoint = Position::default();
    waypoint.ew = 10;
    waypoint.ns = 1;

    for command in Data::read(12).lines().parsed::<Command>() {
        command.apply_p1(&mut ship_p1);
        command.apply_p2(&mut ship_p2, &mut waypoint);
    }

    (
        ship_p1.manhattan_distance() as i64,
        ship_p2.manhattan_distance() as i64,
    )
}

impl TryFromBytes for Command {
    fn try_from_bytes(mut bytes: &[u8]) -> Option<Self> {
        let op = bytes.consume_byte();
        let num: i16 = bytes.parse_bytes();
        let result = match op {
            b'N' => Command::Move(0, num),
            b'S' => Command::Move(0, -num),
            b'E' => Command::Move(num, 0),
            b'W' => Command::Move(-num, 0),
            b'R' => Command::Turn(num),
            b'L' => Command::Turn(-num),
            b'F' => Command::Forward(num),
            _ => unreachable!(),
        };

        Some(result)
    }
}

impl Position {
    fn manhattan_distance(self) -> i16 {
        self.ns.abs() + self.ew.abs()
    }
}

impl Command {
    fn apply_p1(self, ship: &mut Position) {
        match self {
            Command::Move(ew, ns) => {
                ship.ew += ew;
                ship.ns += ns;
            }
            Command::Turn(t) => {
                ship.direction = (ship.direction + t + 360) % 360;
            }
            Command::Forward(f) => match ship.direction {
                0 => ship.ew += f,
                90 => ship.ns -= f,
                180 => ship.ew -= f,
                270 => ship.ns += f,
                _ => unreachable!(),
            },
        }
    }

    fn apply_p2(self, ship: &mut Position, waypoint: &mut Position) {
        match self {
            Command::Move(ew, ns) => {
                waypoint.ew += ew;
                waypoint.ns += ns;
            }
            Command::Turn(t) => {
                let (ew, ns) = (waypoint.ew, waypoint.ns);
                let t = (t + 360) % 360;
                match t {
                    90 => {
                        waypoint.ew = ns;
                        waypoint.ns = -ew;
                    }
                    180 => {
                        waypoint.ew = -ew;
                        waypoint.ns = -ns;
                    }
                    270 => {
                        waypoint.ew = -ns;
                        waypoint.ns = ew;
                    }
                    _ => unreachable!(),
                }
            }
            Command::Forward(f) => {
                ship.ew += f * waypoint.ew;
                ship.ns += f * waypoint.ns;
            }
        }
    }
}
