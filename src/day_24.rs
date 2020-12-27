use crate::data::Data;
use crate::parser::Parser;
use std::mem;

const MAX_ABS_Q: i16 = 72;
const MAX_ABS_R: i16 = 72;
const TILES_VEC_LEN: usize = 4 * (MAX_ABS_Q as usize) * (MAX_ABS_R as usize);

/// In axial coordinates
/// https://www.redblobgames.com/grids/hexagons/#coordinates-axial
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
struct TilePosition {
    q: i16,
    r: i16,
    /// A "linear" index
    index: usize,
}

#[derive(Debug, Clone)]
struct Floor {
    black_tiles: Vec<bool>,
    new_black_tiles: Vec<bool>,
    visited_white: Vec<bool>,
}

pub fn solve() -> (i64, i64) {
    let data = Data::read(24);

    let mut floor = Floor::new();
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
    fn new() -> Self {
        Floor {
            black_tiles: vec![false; TILES_VEC_LEN],
            new_black_tiles: vec![false; TILES_VEC_LEN],
            visited_white: vec![false; TILES_VEC_LEN],
        }
    }

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

        let index = pos.index();
        self.black_tiles[index] = !self.black_tiles[index];
    }

    fn num_black(&self) -> usize {
        self.black_tiles.iter().filter(|&&t| t).count()
    }

    fn is_black(&self, pos: TilePosition) -> bool {
        self.black_tiles[pos.index()]
    }

    fn evolve(&mut self) {
        for cell in &mut self.visited_white {
            *cell = false;
        }
        for cell in &mut self.new_black_tiles {
            *cell = false;
        }

        for (index, &is_black) in self.black_tiles.iter().enumerate() {
            if is_black {
                let pos = TilePosition::from_index(index);

                // Check white tiles that are bordering this one
                for &neighbor in pos.neighbors().iter() {
                    if !self.is_black(neighbor) && !self.visited_white[neighbor.index()] {
                        self.visited_white[neighbor.index()] = true;
                        self.new_black_tiles[neighbor.index()] =
                            self.should_flip_from_white(neighbor);
                    }
                }

                self.new_black_tiles[pos.index()] = !self.should_flip_from_back(pos);
            }
        }

        mem::swap(&mut self.black_tiles, &mut self.new_black_tiles);
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
    fn from_index(index: usize) -> Self {
        let x = index % (2 * MAX_ABS_Q as usize);
        let y = index / (2 * MAX_ABS_Q as usize);

        let q = x as i16 - MAX_ABS_Q;
        let r = y as i16 - MAX_ABS_R;

        TilePosition { q, r, index }
    }

    fn east(self) -> Self {
        self.with_offset(1, 0)
    }
    fn west(self) -> Self {
        self.with_offset(-1, 0)
    }
    fn northeast(self) -> Self {
        self.with_offset(1, -1)
    }
    fn northwest(self) -> Self {
        self.with_offset(0, -1)
    }
    fn southeast(self) -> Self {
        self.with_offset(0, 1)
    }
    fn southwest(self) -> Self {
        self.with_offset(-1, 1)
    }
    fn with_offset(self, dq: i16, dr: i16) -> Self {
        let index = self.index as isize + dq as isize + dr as isize * (2 * MAX_ABS_Q as isize);
        TilePosition {
            q: self.q + dq,
            r: self.r + dr,
            index: index as usize,
        }
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

    /// Convert from `(q, r)` to a vector index
    fn index(self) -> usize {
        debug_assert!(self.q >= -MAX_ABS_Q);
        debug_assert!(self.q < MAX_ABS_Q);
        debug_assert!(self.r >= -MAX_ABS_R);
        debug_assert!(self.r < MAX_ABS_R);

        let x = (self.q + MAX_ABS_Q) as usize;
        let y = (self.r + MAX_ABS_R) as usize;

        y * (2 * MAX_ABS_Q as usize) + x
    }
}
