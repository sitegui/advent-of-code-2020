#![allow(dead_code)]
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone)]
pub struct Matrix<T> {
    values: Vec<T>,
    width: usize,
    height: usize,
}

impl<T> Matrix<T> {
    pub fn new(values: Vec<T>, width: usize) -> Self {
        let height = values.len() / width;
        assert_eq!(values.len(), width * height);
        Matrix {
            values,
            width,
            height,
        }
    }

    pub fn new_with_fn<F: FnMut(usize, usize) -> T>(
        mut generate: F,
        width: usize,
        height: usize,
    ) -> Self {
        let mut values = Vec::with_capacity(width * height);
        for y in 0..height {
            for x in 0..width {
                values.push(generate(x, y));
            }
        }
        Matrix {
            values,
            width,
            height,
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        self.try_coords_to_index(x, y)
            .map(|index| &self.values[index])
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        self.try_coords_to_index(x, y)
            .map(move |index| &mut self.values[index])
    }

    pub fn try_coords_to_index(&self, x: usize, y: usize) -> Option<usize> {
        if x >= self.width || y >= self.height {
            None
        } else {
            Some(y * self.width + x)
        }
    }

    pub fn try_index_to_coords(&self, index: usize) -> Option<(usize, usize)> {
        if index >= self.values.len() {
            None
        } else {
            let x = index % self.width;
            let y = index / self.width;
            Some((x, y))
        }
    }

    pub fn values(&self) -> &[T] {
        &self.values
    }

    pub fn into_values(self) -> Vec<T> {
        self.values
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn row(&self, y: usize) -> &[T] {
        assert!(y < self.height);
        &self.values[y * self.width..(y + 1) * self.width]
    }

    pub fn column(&self, x: usize) -> impl Iterator<Item = &T> {
        assert!(x < self.width);
        (0..self.height).map(move |y| &self[(x, y)])
    }

    pub fn rows(&self) -> impl Iterator<Item = &[T]> {
        (0..self.height).map(move |y| self.row(y))
    }
}

impl<T> Index<(usize, usize)> for Matrix<T> {
    type Output = T;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        self.get(index.0, index.1).unwrap()
    }
}

impl<T> IndexMut<(usize, usize)> for Matrix<T> {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        self.get_mut(index.0, index.1).unwrap()
    }
}
