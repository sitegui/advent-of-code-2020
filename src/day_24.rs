use crate::data::Data;
use crate::parser::Parser;
use std::collections::BTreeSet;

/// In axial coordinates
/// https://www.redblobgames.com/grids/hexagons/#coordinates-axial
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
struct TilePosition(i16, i16);

#[derive(Debug, Clone, Default)]
struct Floor {
    black_tiles: BTreeSet<TilePosition>,
}

pub fn solve() -> (i64, i64) {
    let data = Data::read(24);

    let mut floor = Floor::default();
    for line in data.lines() {
        floor.flip_tile(line);
    }

    let part_1 = floor.num_black() as i64;

    for _ in 0..100 {
        floor.evolve();
    }

    let part_2 = floor.num_black() as i64;
    (part_1, part_2)
}

impl Floor {
    fn flip_tile(&mut self, mut directions: &[u8]) {
        let mut pos = TilePosition::default();
        while !directions.is_empty() {
            pos = match directions.consume_byte() {
                b'e' => pos.east(),
                b'w' => pos.west(),
                b'n' => match directions.consume_byte() {
                    b'e' => pos.northeast(),
                    b'w' => pos.northwest(),
                    _ => unreachable!(),
                },
                b's' => match directions.consume_byte() {
                    b'e' => pos.southeast(),
                    b'w' => pos.southwest(),
                    _ => unreachable!(),
                },
                _ => unreachable!(),
            };
        }

        if !self.black_tiles.insert(pos) {
            // Already present => flip to white
            self.black_tiles.remove(&pos);
        }
    }

    fn num_black(&self) -> usize {
        self.black_tiles.len()
    }

    fn is_black(&self, pos: TilePosition) -> bool {
        self.black_tiles.contains(&pos)
    }

    fn evolve(&mut self) {
        let mut visited_white = BTreeSet::new();
        let mut new_black_tiles = BTreeSet::new();

        fn inspect_black(
            floor: &Floor,
            visited_white: &mut BTreeSet<TilePosition>,
            new_black_tiles: &mut BTreeSet<TilePosition>,
            pos: TilePosition,
        ) {
            // Check white tiles that are bordering this one
            for &neighbor in pos.neighbors().iter() {
                if !floor.is_black(neighbor) {
                    inspect_white(floor, visited_white, new_black_tiles, neighbor);
                }
            }

            if !floor.should_flip_from_back(pos) {
                new_black_tiles.insert(pos);
            }
        }

        fn inspect_white(
            floor: &Floor,
            visited_white: &mut BTreeSet<TilePosition>,
            new_black_tiles: &mut BTreeSet<TilePosition>,
            pos: TilePosition,
        ) {
            if visited_white.insert(pos) && floor.should_flip_from_white(pos) {
                new_black_tiles.insert(pos);
            }
        }

        for &pos in &self.black_tiles {
            inspect_black(self, &mut visited_white, &mut new_black_tiles, pos);
        }

        self.black_tiles = new_black_tiles;
    }

    fn should_flip_from_back(&self, pos: TilePosition) -> bool {
        let mut black_neighbors = 0;
        for &pos in pos.neighbors().iter() {
            if self.is_black(pos) {
                black_neighbors += 1;

                if black_neighbors > 2 {
                    return true;
                }
            }
        }

        black_neighbors == 0
    }

    fn should_flip_from_white(&self, pos: TilePosition) -> bool {
        let mut black_neighbors = 0;
        for &pos in pos.neighbors().iter() {
            if self.is_black(pos) {
                black_neighbors += 1;

                if black_neighbors > 2 {
                    return false;
                }
            }
        }

        black_neighbors == 2
    }
}

/// Based on:
/// https://www.redblobgames.com/grids/hexagons/#neighbors-axial
impl TilePosition {
    fn east(self) -> Self {
        TilePosition(self.0 + 1, self.1)
    }
    fn west(self) -> Self {
        TilePosition(self.0 - 1, self.1)
    }
    fn northeast(self) -> Self {
        TilePosition(self.0 + 1, self.1 - 1)
    }
    fn northwest(self) -> Self {
        TilePosition(self.0, self.1 - 1)
    }
    fn southeast(self) -> Self {
        TilePosition(self.0, self.1 + 1)
    }
    fn southwest(self) -> Self {
        TilePosition(self.0 - 1, self.1 + 1)
    }
    fn neighbors(self) -> [Self; 6] {
        [
            self.east(),
            self.west(),
            self.northeast(),
            self.northwest(),
            self.southeast(),
            self.southwest(),
        ]
    }
}
