use itertools::Itertools;
use std::fmt::Debug;
use std::mem::MaybeUninit;
use std::{mem, slice};

/// The number of children of each node.
/// Must be of the form `2 * N` with `N >= 1`
const SIZE: usize = 10;
type Tail<T> = [Option<NodeTail<T>>; SIZE - 1];

#[derive(Debug)]
pub struct DenseLine<T> {
    head: NodeHead<T>,
}

#[derive(Debug)]
pub struct Iter<'a, T> {
    stack: Vec<IterStackState<'a, T>>,
}

#[derive(Debug)]
pub struct Coordinates(Vec<u8>);

#[derive(Debug)]
pub struct Get1Remove3<T> {
    get_1: T,
    remove_3: [T; 3],
    num_removed: usize,
}

#[derive(Debug)]
pub struct Insert3<T> {
    values: [T; 3],
    num_inserted: usize,
    new_coordinates: [Coordinates; 3],
}

#[derive(Debug)]
struct NodeHead<T> {
    head: Option<Box<NodeHead<T>>>,
    tail: Tail<T>,
}

#[derive(Debug)]
struct NodeTail<T> {
    value: Option<T>,
    head: Option<Box<NodeHead<T>>>,
}

#[derive(Debug)]
struct IterStackState<'a, T> {
    prev_coordinate: u8,
    remaining_tail: slice::Iter<'a, Option<NodeTail<T>>>,
}

impl<T: Copy + Default + Debug> DenseLine<T> {
    fn new(values: &[T]) -> Self {
        DenseLine {
            head: NodeHead::new(values),
        }
    }

    fn iter(&self) -> Iter<'_, T> {
        Iter::new(self)
    }

    fn get_1_and_remove_3(&mut self, start: &Coordinates) -> Get1Remove3<T> {
        let coordinates = &start.0;

        fn get_1_and_remove_3<T: Copy + Default + Debug>(
            node: &mut NodeHead<T>,
            coordinates: &[u8],
            result: &mut Get1Remove3<T>,
        ) {
            let (&coordinate, sub_coordinates) = coordinates.split_first().unwrap();

            let next_tails;
            if sub_coordinates.is_empty() {
                let child = node.tail[coordinate as usize - 1].as_mut().unwrap();
                result.get_1 = child.value.unwrap();

                if let Some(sub_head) = &mut child.head {
                    remove_3_head(sub_head, result);
                }
                next_tails = &mut node.tail[coordinate as usize..];
            } else if coordinate == 0 {
                let child = node.head.as_mut().unwrap();
                get_1_and_remove_3(child, sub_coordinates, result);
                next_tails = &mut node.tail[..];
            } else {
                let child = node.tail[coordinate as usize - 1]
                    .as_mut()
                    .and_then(|tail| tail.head.as_mut())
                    .unwrap();
                get_1_and_remove_3(child, sub_coordinates, result);
                next_tails = &mut node.tail[coordinate as usize..];
            }

            if !result.is_full() {
                for tail in next_tails {
                    if let Some(tail) = tail {
                        remove_3_tail(tail, result);
                        if result.is_full() {
                            return;
                        }
                    }
                }
            }
        }

        fn remove_3_head<T: Copy + Default>(node: &mut NodeHead<T>, result: &mut Get1Remove3<T>) {
            if let Some(head) = &mut node.head {
                remove_3_head(head, result);
                if result.is_full() {
                    return;
                }
            }

            for tail in &mut node.tail {
                if let Some(tail) = tail {
                    remove_3_tail(tail, result);
                    if result.is_full() {
                        return;
                    }
                }
            }
        }

        fn remove_3_tail<T: Copy + Default>(node: &mut NodeTail<T>, result: &mut Get1Remove3<T>) {
            if let Some(value) = node.value.take() {
                result.push_removed(value);
                if result.is_full() {
                    return;
                }
            }

            if let Some(head) = &mut node.head {
                remove_3_head(head, result);
            }
        }

        let mut result = Get1Remove3::new();
        get_1_and_remove_3(&mut self.head, coordinates, &mut result);

        if !result.is_full() {
            remove_3_head(&mut self.head, &mut result);
        }

        result
    }

    fn insert_3(&mut self, after: &Coordinates, values: &mut Insert3<T>) {
        fn navigate<T: Copy + Default + Debug>(
            node: &mut NodeHead<T>,
            coordinates: &[u8],
            values: &mut Insert3<T>,
        ) {
            let (&coordinate, sub_coordinates) = coordinates.split_first().unwrap();

            if sub_coordinates.is_empty() {
                insert(node, coordinate, values);
            } else if coordinate == 0 {
                let child = node.head.as_mut().unwrap();
                navigate(child, sub_coordinates, values);
            } else {
                let child = node.tail[coordinate as usize - 1]
                    .as_mut()
                    .and_then(|tail| tail.head.as_mut())
                    .unwrap();
                navigate(child, sub_coordinates, values);
            }
        }

        fn insert<T: Copy + Default>(
            node: &mut NodeHead<T>,
            coordinate: u8,
            values: &mut Insert3<T>,
        ) {
            if values.is_empty() {
                return;
            }

            let (temp, next_tails) = node.tail.split_at_mut(coordinate as usize);
            let child = temp.last_mut().unwrap().as_mut().unwrap();
            debug_assert!(child.value.is_some());

            match &mut child.head {
                None => {
                    let vacuum_size = next_tails
                        .iter()
                        .enumerate()
                        .find_map(|(i, tail)| match tail {
                            None => None,
                            Some(NodeTail { value: None, .. }) => Some(i + 1),
                            Some(NodeTail { value: Some(_), .. }) => Some(i),
                        })
                        .unwrap_or(next_tails.len());

                    if vacuum_size == 0 {
                        let mut new_head = NodeHead::empty();
                        let index = new_head.tail.len() / 2;
                        let new_coordinate = index as u8 + 1;
                        new_head.tail[index] =
                            Some(NodeTail::new_single(values.pop(new_coordinate)));
                        child.head = Some(Box::new(new_head));

                        insert(child.head.as_mut().unwrap(), new_coordinate, values);
                    } else {
                        let index = vacuum_size / 2;
                        let new_coordinate = index as u8 + 1;
                        values.pop_coordinate();
                        let value = values.pop(new_coordinate);
                        match &mut next_tails[index] {
                            insertion_point @ None => {
                                *insertion_point = Some(NodeTail::new_single(value));
                            }
                            Some(tail) => {
                                debug_assert!(tail.value.is_none());
                                tail.value = Some(value);
                            }
                        }

                        insert(node, new_coordinate, values);
                    }
                }
                Some(sub_head) => match &mut sub_head.head {
                    None => {
                        let vacuum_size = sub_head
                            .tail
                            .iter()
                            .enumerate()
                            .find_map(|(i, tail)| match tail {
                                None => None,
                                Some(NodeTail { value: None, .. }) => Some(i + 1),
                                Some(NodeTail { value: Some(_), .. }) => Some(i),
                            })
                            .unwrap_or(next_tails.len());

                        if vacuum_size == 0 {
                            let mut new_head = NodeHead::empty();
                            let index = new_head.tail.len() / 2;
                            values.push_coordinate(0);
                            let new_coordinate = index as u8 + 1;
                            new_head.tail[index] =
                                Some(NodeTail::new_single(values.pop(new_coordinate)));
                            child.head = Some(Box::new(new_head));

                            insert(child.head.as_mut().unwrap(), new_coordinate, values);
                        } else {
                            let index = vacuum_size / 2;
                            let new_coordinate = index as u8 + 1;
                            values.pop_coordinate();
                            let value = values.pop(new_coordinate);
                            match &mut next_tails[index] {
                                insertion_point @ None => {
                                    *insertion_point = Some(NodeTail::new_single(value));
                                }
                                Some(tail) => {
                                    debug_assert!(tail.value.is_none());
                                    tail.value = Some(value);
                                }
                            }

                            insert(node, new_coordinate, values);
                        }
                    }
                    Some(_) => {}
                },
            }
        }

        values.set_base_coordinates(after);
        navigate(&mut self.head, &after.0, values)
    }
}

impl<T: Copy> NodeHead<T> {
    fn new(values: &[T]) -> Self {
        let mut head = None;
        let mut tail = empty_tail();

        if values.len() < SIZE {
            // Can fit in a single node
            for (i, &value) in values.iter().enumerate() {
                tail[i] = Some(NodeTail::new_single(value));
            }
        } else {
            // Recursively split the values
            let chunk_size = values.len() / SIZE;
            for (i, chunk) in values.chunks(chunk_size).enumerate() {
                if i == 0 {
                    head = Some(Box::new(NodeHead::new(chunk)));
                } else {
                    tail[i - 1] = Some(NodeTail::new(chunk));
                }
            }
        }

        NodeHead { head, tail }
    }

    fn empty() -> Self {
        NodeHead {
            head: None,
            tail: empty_tail(),
        }
    }
}

impl<T: Copy> NodeTail<T> {
    fn new_single(value: T) -> Self {
        NodeTail {
            value: Some(value),
            head: None,
        }
    }

    fn new(values: &[T]) -> Self {
        assert!(values.len() > 1);
        let (&value, values) = values.split_first().unwrap();
        NodeTail {
            value: Some(value),
            head: Some(Box::new(NodeHead::new(values))),
        }
    }
}

impl<'a, T: Copy> Iter<'a, T> {
    fn new(dense_line: &'a DenseLine<T>) -> Self {
        let mut iter = Iter { stack: vec![] };

        iter.prepare_head_visit(&dense_line.head);

        iter
    }

    fn prepare_head_visit(&mut self, mut head: &'a NodeHead<T>) {
        loop {
            self.stack.push(IterStackState {
                prev_coordinate: 0,
                remaining_tail: head.tail.iter(),
            });
            match &head.head {
                None => return,
                Some(sub_head) => head = sub_head,
            }
        }
    }
}

impl<'a, T: Copy> IterStackState<'a, T> {
    fn next_tail(&mut self) -> Option<&'a NodeTail<T>> {
        loop {
            match self.remaining_tail.next() {
                None => return None,
                Some(maybe_tail) => {
                    self.prev_coordinate += 1;
                    match maybe_tail {
                        None => {}
                        Some(tail) => return Some(tail),
                    }
                }
            }
        }
    }
}

impl<'a, T: Copy> Iterator for Iter<'a, T> {
    type Item = (Coordinates, T);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.stack.last_mut() {
                None => return None,
                Some(state) => match state.next_tail() {
                    None => {
                        self.stack.pop();
                    }
                    Some(tail) => {
                        let coordinates = self
                            .stack
                            .iter()
                            .map(|state| state.prev_coordinate)
                            .collect_vec();
                        if let Some(head) = &tail.head {
                            self.prepare_head_visit(head);
                        }

                        if let Some(value) = tail.value {
                            return Some((Coordinates(coordinates), value));
                        }
                    }
                },
            }
        }
    }
}

impl<T: Copy + Default> Get1Remove3<T> {
    fn new() -> Self {
        Get1Remove3 {
            get_1: T::default(),
            remove_3: [T::default(); 3],
            num_removed: 0,
        }
    }

    fn push_removed(&mut self, value: T) {
        self.remove_3[self.num_removed] = value;
        self.num_removed += 1;
    }

    fn is_full(&self) -> bool {
        self.num_removed == self.remove_3.len()
    }

    fn get_1(&self) -> T {
        self.get_1
    }

    fn remove_3(&self) -> [T; 3] {
        self.remove_3
    }
}

impl<T: Copy> Insert3<T> {
    pub fn new(values: [T; 3], mut new_coordinates_buffers: [Coordinates; 3]) -> Self {
        Insert3 {
            values,
            num_inserted: 0,
            new_coordinates: new_coordinates_buffers,
        }
    }

    fn set_base_coordinates(&mut self, coordinates: &Coordinates) {
        for buffer in &mut self.new_coordinates {
            buffer.0.clear();
            buffer.0.extend_from_slice(&coordinates.0);
        }
    }

    fn pop_coordinate(&mut self) {
        for buffer in &mut self.new_coordinates[self.num_inserted..] {
            buffer.0.pop();
        }
    }

    fn push_coordinate(&mut self, coordinate: u8) {
        for buffer in &mut self.new_coordinates[self.num_inserted..] {
            buffer.0.push(coordinate);
        }
    }

    fn pop(&mut self, push_coordinate: u8) -> T {
        self.push_coordinate(push_coordinate);
        let value = self.values[self.num_inserted];
        self.num_inserted += 1;
        value
    }

    fn is_empty(&self) -> bool {
        self.num_inserted == self.values.len()
    }
}

fn empty_tail<T>() -> Tail<T> {
    // Create an uninitialized array of `MaybeUninit`. The `assume_init` is
    // safe because the type we are claiming to have initialized here is a
    // bunch of `MaybeUninit`s, which do not require initialization.
    let mut data: [MaybeUninit<Option<NodeTail<T>>>; SIZE - 1] =
        unsafe { MaybeUninit::uninit().assume_init() };

    // Dropping a `MaybeUninit` does nothing. Thus using raw pointer
    // assignment instead of `ptr::write` does not cause the old
    // uninitialized value to be dropped. Also if there is a panic during
    // this loop, we have a memory leak, but there is no memory safety
    // issue.
    for elem in &mut data[..] {
        *elem = MaybeUninit::new(None);
    }

    // Everything is initialized. Transmute the array to the
    // initialized type.
    unsafe { mem::transmute_copy::<_, [Option<NodeTail<T>>; SIZE - 1]>(&data) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;

    #[test]
    fn test() {
        let mut values = vec![];
        for i in 0..(SIZE * (SIZE - 1)) {
            values.push(i);
        }

        let mut line = DenseLine::new(&values);
        assert_eq!(line.iter().map(|(_, value)| value).collect_vec(), values);

        let coordinates = line.iter().find(|&(_, value)| value == SIZE).unwrap().0;
        let result = line.get_1_and_remove_3(coordinates);
        assert_eq!(result.get_1(), SIZE);
        assert_eq!(result.remove_3(), [SIZE + 1, SIZE + 2, SIZE + 3]);

        let target = SIZE * (SIZE - 1) - 2;
        let coordinates = line.iter().find(|&(_, value)| value == target).unwrap().0;
        let result = line.get_1_and_remove_3(coordinates);
        assert_eq!(result.get_1(), target);
        assert_eq!(result.remove_3(), [target + 1, 0, 1]);
    }
}
