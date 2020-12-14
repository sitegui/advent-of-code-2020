use crate::data::{Data, ParseBytes, TryFromBytes};
use crate::parser::Parser;
use std::collections::BTreeMap;

#[derive(Debug, Copy, Clone, Default, Ord, PartialOrd, Eq, PartialEq)]
struct Mask {
    /// The last 36 bits of the masks bellow show where each type of filter is.
    bitmap_0: i64,
    bitmap_1: i64,
    bitmap_x: i64,
}

#[derive(Debug, Copy, Clone)]
struct Command {
    address: i64,
    value: i64,
}

pub fn solve() -> (i64, i64) {
    let mut memory_p1 = BTreeMap::new();
    let mut memory_p2 = BTreeMap::new();

    let mut mask = Mask::default();

    for line in Data::read(14).lines() {
        let mut line = Parser::new(line);
        match line.try_consume_prefix(b"mask = ") {
            Some(_) => {
                // Read new mask
                mask = line.parse_bytes();
            }
            None => {
                let command: Command = line.parse_bytes();

                // Update memory for part 1
                memory_p1.insert(command.address, mask.apply_p1(command.value));

                // Update memory for part 2
                mask.apply_p2(command, &mut memory_p2);
            }
        }
    }

    let part_1 = memory_p1.values().copied().sum();
    let part_2 = memory_p2.values().copied().sum();

    (part_1, part_2)
}

impl Mask {
    fn apply_p1(&self, mut value: i64) -> i64 {
        value &= !self.bitmap_0;
        value |= self.bitmap_1;
        value
    }

    fn apply_p2(&self, command: Command, memory: &mut BTreeMap<i64, i64>) {
        let base = command.address | self.bitmap_1;

        fn apply_from_bit(
            bitmap_x: i64,
            bit: usize,
            address_tail: i64,
            memory: &mut BTreeMap<i64, i64>,
            value: i64,
            base: i64,
        ) {
            if bit == 36 {
                memory.insert(address_tail, value);
            } else if get_bit(bitmap_x, bit) == 0 {
                // No 'x', copy base bit and skip to next
                apply_from_bit(
                    bitmap_x,
                    bit + 1,
                    address_tail | (get_bit(base, bit) << bit),
                    memory,
                    value,
                    base,
                );
            } else {
                // Do both '0' and '1'
                apply_from_bit(bitmap_x, bit + 1, address_tail, memory, value, base);
                apply_from_bit(
                    bitmap_x,
                    bit + 1,
                    address_tail | (1 << bit),
                    memory,
                    value,
                    base,
                );
            }
        }

        apply_from_bit(self.bitmap_x, 0, 0, memory, command.value, base);
    }
}

fn get_bit(value: i64, bit: usize) -> i64 {
    (value >> bit) & 1
}

impl TryFromBytes for Mask {
    fn try_from_bytes(bytes: Parser<'_>) -> Option<Self> {
        if bytes.len() != 36 {
            return None;
        }

        let mut mask = Mask::default();
        for (i, bit) in bytes.iter().enumerate() {
            let filter = match bit {
                b'X' => &mut mask.bitmap_x,
                b'0' => &mut mask.bitmap_0,
                b'1' => &mut mask.bitmap_1,
                _ => return None,
            };

            *filter |= 1 << (35 - i);
        }

        Some(mask)
    }
}

impl TryFromBytes for Command {
    fn try_from_bytes(mut bytes: Parser<'_>) -> Option<Self> {
        bytes.try_consume_prefix(b"mem[")?;
        let address = bytes.consume_until(b']').try_parse_bytes()?;
        bytes.try_consume_prefix(b" = ")?;
        let value = bytes.try_parse_bytes()?;
        Some(Command { address, value })
    }
}
