use crate::data::Data;
use itertools::Itertools;
use std::fmt;
use std::ops::Range;

#[derive(Debug, Copy, Clone, PartialEq)]
enum Cube {
    Active,
    Inactive,
}

#[derive(Clone)]
struct CubeSpace {
    /// `values[S * S * S * S * w + S * S * z + S * y + z]`
    values: Vec<Cube>,
    offset: i32,
    size: usize,
}

/// (x, y, z, w)
type Point = (i32, i32, i32, i32);

pub fn solve() -> (i64, i64) {
    const ITERATIONS: i32 = 6;
    const INITIAL_SIZE: i32 = 9;
    let mut space = CubeSpace::new(INITIAL_SIZE + 2 * ITERATIONS + 2);

    let data = Data::read(17);
    for (y, line) in data.lines().enumerate().take(INITIAL_SIZE as usize) {
        for (x, &cube) in line.iter().enumerate().take(INITIAL_SIZE as usize) {
            space.set(
                (
                    x as i32 - INITIAL_SIZE / 2,
                    y as i32 - INITIAL_SIZE / 2,
                    0,
                    0,
                ),
                match cube {
                    b'.' => Cube::Inactive,
                    b'#' => Cube::Active,
                    _ => unreachable!(),
                },
            );
        }
    }

    let mut space_p1 = space.clone();
    let mut space_p2 = space;
    for iteration in 0..ITERATIONS as i32 {
        let mut new_space_p1 = space_p1.clone();
        let mut new_space_p2 = space_p2.clone();

        let limit = INITIAL_SIZE as i32 / 2 + iteration + 1;
        let range = -limit..=limit;
        for x in range.clone() {
            for y in range.clone() {
                for z in range.clone() {
                    for w in range.clone() {
                        let point = (x, y, z, w);

                        if w == 0 {
                            space_p1.write_evolution(&mut new_space_p1, point);
                        }
                        space_p2.write_evolution(&mut new_space_p2, point);
                    }
                }
            }
        }

        space_p1 = new_space_p1;
        space_p2 = new_space_p2;
    }

    (space_p1.num_active() as i64, space_p2.num_active() as i64)
}

impl CubeSpace {
    fn new(size: i32) -> Self {
        let offset = size / 2;
        let size = size as usize;
        CubeSpace {
            offset,
            values: vec![Cube::Inactive; size * size * size * size],
            size,
        }
    }

    fn get(&self, point: Point) -> Cube {
        self.values[self.index(point)]
    }

    fn set(&mut self, point: Point, cube: Cube) {
        let index = self.index(point);
        self.values[index] = cube
    }

    fn index(&self, point: Point) -> usize {
        debug_assert!(self.range().contains(&point.0), "{:?} is invalid", point);
        debug_assert!(self.range().contains(&point.1), "{:?} is invalid", point);
        debug_assert!(self.range().contains(&point.2), "{:?} is invalid", point);
        debug_assert!(self.range().contains(&point.3), "{:?} is invalid", point);

        let x = (point.0 + self.offset) as usize;
        let y = (point.1 + self.offset) as usize;
        let z = (point.2 + self.offset) as usize;
        let w = (point.3 + self.offset) as usize;
        w * self.size * self.size * self.size + z * self.size * self.size + y * self.size + x
    }

    fn range(&self) -> Range<i32> {
        -self.offset..self.size as i32 - self.offset
    }

    fn active_neighbors(&self, point: Point) -> usize {
        let mut result = 0;

        macro_rules! check_xyzw {
            ($dx:literal, $dy:literal, $dz:literal, $dw:literal) => {
                let test_point = (point.0 + $dx, point.1 + $dy, point.2 + $dz, point.3 + $dw);
                let all_zeros = $dx == 0 && $dy == 0 && $dz == 0 && $dw == 0;
                if !all_zeros && self.get(test_point) == Cube::Active {
                    result += 1;
                }
            };
        }

        macro_rules! check_xyz {
            ($dx:literal, $dy:literal, $dz:literal) => {
                check_xyzw!($dx, $dy, $dz, -1);
                check_xyzw!($dx, $dy, $dz, 0);
                check_xyzw!($dx, $dy, $dz, 1);
            };
        }

        macro_rules! check_xy {
            ($dx:literal, $dy:literal) => {
                check_xyz!($dx, $dy, -1);
                check_xyz!($dx, $dy, 0);
                check_xyz!($dx, $dy, 1);
            };
        }

        macro_rules! check_x {
            ($dx:literal) => {
                check_xy!($dx, -1);
                check_xy!($dx, 0);
                check_xy!($dx, 1);
            };
        }

        check_x!(-1);
        check_x!(0);
        check_x!(1);

        result
    }

    fn write_evolution(&self, target: &mut Self, point: Point) {
        let active_neighbors = self.active_neighbors(point);
        match self.get(point) {
            Cube::Active => {
                if active_neighbors != 2 && active_neighbors != 3 {
                    target.set(point, Cube::Inactive);
                }
            }
            Cube::Inactive => {
                if active_neighbors == 3 {
                    target.set(point, Cube::Active);
                }
            }
        }
    }

    fn num_active(&self) -> usize {
        self.values
            .iter()
            .filter(|&&cube| cube == Cube::Active)
            .count()
    }
}

impl fmt::Debug for CubeSpace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for w in self.range() {
            for z in self.range() {
                let empty_page = self.range().all(|y| {
                    self.range()
                        .all(|x| self.get((x, y, z, w)) == Cube::Inactive)
                });
                if empty_page {
                    continue;
                }

                writeln!(f, "\nz={} w={}", z, w)?;
                for y in self.range() {
                    writeln!(
                        f,
                        "{}",
                        self.range()
                            .map(|x| match self.get((x, y, z, w)) {
                                Cube::Active => "#",
                                Cube::Inactive => ".",
                            })
                            .format("")
                    )?;
                }
            }
        }
        Ok(())
    }
}
