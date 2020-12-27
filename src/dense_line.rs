use itertools::Itertools;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::mem::MaybeUninit;
use std::{fmt, mem, slice};

/// The number of children of each node.
/// Must be of the form `2 * N` with `N >= 1`
const SIZE: usize = 1024;
type Child<T> = Option<Box<Node<T>>>;
type Children<T> = [Child<T>; SIZE];

#[derive(Debug)]
pub struct DenseLine<T> {
    pub root: Node<T>,
}

#[derive(Debug)]
pub struct Iter<'a, T> {
    stack: Vec<IterStackState<'a, T>>,
}

#[derive(Debug, Clone)]
pub struct Coordinates(Vec<usize>);

#[derive(Debug)]
pub struct Get1Remove3<T> {
    gotten: T,
    removed: [T; 3],
    num_removed: usize,
}

#[derive(Debug)]
pub struct Insert3<T> {
    values: [T; 3],
    num_inserted: usize,
    new_coordinates: [Coordinates; 3],
}

pub struct Node<T> {
    pub value: Option<T>,
    pub children: Children<T>,
}

#[derive(Debug)]
struct IterStackState<'a, T> {
    next_coordinate: usize,
    remaining_children: slice::Iter<'a, Child<T>>,
}

impl<T: Copy + Default + Debug> DenseLine<T> {
    pub fn new(values: &[T]) -> Self {
        DenseLine {
            root: Node::with_values(values),
        }
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter::new(&self.root)
    }

    pub fn get_1_and_remove_3(&mut self, start: &Coordinates) -> Get1Remove3<T> {
        let coordinates = &start.0;

        fn get_1_and_remove_3<T: Copy + Default + Debug>(
            node: &mut Node<T>,
            coordinates: &[usize],
            result: &mut Get1Remove3<T>,
        ) {
            let (&coordinate, sub_coordinates) = coordinates.split_first().unwrap();

            if sub_coordinates.is_empty() {
                let child = node.children[coordinate as usize].as_mut().unwrap();
                result.gotten = child.value.unwrap();
                remove_3(&mut child.children, result);
            } else {
                let child = node.children[coordinate as usize].as_mut().unwrap();
                get_1_and_remove_3(child, sub_coordinates, result);
            }

            if !result.is_full() {
                let next_children = &mut node.children[coordinate as usize + 1..];
                remove_3(next_children, result);
            }
        }

        fn remove_3<T: Copy + Default>(nodes: &mut [Child<T>], result: &mut Get1Remove3<T>) {
            for (i, node) in nodes.iter_mut().enumerate() {
                if let Some(node) = node {
                    if let Some(value) = node.value.take() {
                        result.push_removed(value);
                        if result.is_full() {
                            return;
                        }
                    }

                    remove_3(&mut node.children, result);
                    if result.is_full() {
                        return;
                    }
                } else if i > 0 {
                    // Children (except the head) are filled from left to right
                    break;
                }
            }
        }

        let mut result = Get1Remove3::new();
        get_1_and_remove_3(&mut self.root, coordinates, &mut result);

        if !result.is_full() {
            remove_3(&mut self.root.children, &mut result);
        }

        result
    }

    pub fn insert_3(&mut self, after: &Coordinates, mut values: Insert3<T>) -> Insert3<T> {
        /// Navigate until the insertion point as described by the remaining coordinates
        fn navigate<T: Copy>(node: &mut Node<T>, coordinates: &[usize], values: &mut Insert3<T>) {
            let (&coordinate, sub_coordinates) = coordinates.split_first().unwrap();

            if sub_coordinates.is_empty() {
                insert_after(&mut node.children, coordinate, values);
            } else {
                let child = node.children[coordinate as usize].as_mut().unwrap();
                navigate(child, sub_coordinates, values);
            }
        }

        /// Start the insertion after the node `nodes[coordinate]`.
        fn insert_after<T: Copy>(
            nodes: &mut Children<T>,
            coordinate: usize,
            values: &mut Insert3<T>,
        ) {
            if values.is_empty() {
                return;
            }

            // Pick exclusive references to the target node and its right sibling (if any)
            let (node, maybe_sibling) = {
                let (before, after) = nodes.split_at_mut(coordinate + 1);
                (
                    before.last_mut().unwrap().as_mut().unwrap(),
                    after.first_mut(),
                )
            };

            #[allow(clippy::if_same_then_else)]
            if node.children[0].is_some() {
                // Follow down the head
                values.push_coordinate(0);
                insert_after(&mut node.children, 0, values);
            } else if let Some(neck) = &mut node.children[1] {
                if neck.value.is_some() {
                    // Has a full neck => create a head and follow it
                    node.children[0] = Some(Box::new(Node::new()));
                    values.push_coordinate(0);
                    insert_after(&mut node.children, 0, values);
                } else {
                    // Has a hollow neck => insert in it
                    insert_at_neck(node, values);
                }
            } else if node.children[2].is_some() {
                // Non leaf node => create a neck
                unreachable!("TODO: remove this arm")
            } else if maybe_sibling.is_none() {
                // Leaf node and no space to insert at right => create a neck
                insert_at_neck(node, values);
            } else if let Some(sibling) = maybe_sibling {
                let sibling = sibling.get_or_insert_with(|| Box::new(Node::new()));
                if sibling.value.is_none() {
                    // Hollow sibling => insert at it
                    values.pop_coordinate();
                    sibling.value = Some(values.pop(coordinate + 1));
                    insert_after(nodes, coordinate + 1, values);
                } else {
                    // Full sibling => create a neck
                    insert_at_neck(node, values);
                }
            } else {
                unreachable!()
            }
        }

        fn insert_at_neck<T: Copy>(node: &mut Node<T>, values: &mut Insert3<T>) {
            let neck = node.children[1].get_or_insert_with(|| Box::new(Node::new()));
            debug_assert!(neck.value.is_none());
            neck.value = Some(values.pop(1));
            insert_after(&mut node.children, 1, values);
        }

        values.set_base_coordinates(after);
        navigate(&mut self.root, &after.0, &mut values);

        values
    }

    pub fn next(&self, base: &mut Coordinates) -> T {
        fn find_next<T: Copy + Default + Debug>(
            nodes: &[Child<T>],
            coordinates: &mut Vec<usize>,
            level: usize,
        ) -> Option<T> {
            match coordinates.get(level) {
                Some(&coordinate) => {
                    if let Some(value) = find_next(
                        &nodes[coordinate as usize].as_ref().unwrap().children,
                        coordinates,
                        level + 1,
                    ) {
                        return Some(value);
                    }

                    coordinates.pop();
                    find_first(
                        &nodes[coordinate as usize + 1..],
                        coordinates,
                        level,
                        coordinate + 1,
                    )
                }
                None => find_first(nodes, coordinates, level, 0),
            }
        }

        fn find_first<T: Copy + Default + Debug>(
            nodes: &[Child<T>],
            coordinates: &mut Vec<usize>,
            level: usize,
            coordinate_offset: usize,
        ) -> Option<T> {
            coordinates.push(0);
            for (i, child) in nodes.iter().enumerate() {
                if let Some(child) = child {
                    if let Some(value) = child.value {
                        coordinates[level] = coordinate_offset + i;
                        return Some(value);
                    }

                    if let Some(value) = find_first(&child.children, coordinates, level + 1, 0) {
                        coordinates[level] = coordinate_offset + i;
                        return Some(value);
                    }
                } else if i > 0 {
                    // Children (except the head) are filled from left to right
                    break;
                }
            }
            coordinates.pop();
            None
        }

        find_next(&self.root.children, &mut base.0, 0)
            .or_else(|| {
                base.0.clear();
                find_first(&self.root.children, &mut base.0, 0, 0)
            })
            .unwrap()
    }
}

impl<T: Copy> Node<T> {
    pub fn iter_children(&self) -> Iter<'_, T> {
        Iter::new(&self)
    }

    fn new() -> Self {
        // Create an uninitialized array of `MaybeUninit`. The `assume_init` is
        // safe because the type we are claiming to have initialized here is a
        // bunch of `MaybeUninit`s, which do not require initialization.
        let mut data: [MaybeUninit<Child<T>>; SIZE] =
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
        let children = unsafe { mem::transmute_copy::<_, [Child<T>; SIZE]>(&data) };

        Node {
            value: None,
            children,
        }
    }

    fn with_value(value: T) -> Self {
        let mut node = Node::new();
        node.value = Some(value);
        node
    }

    fn with_values(values: &[T]) -> Self {
        let mut node = Node::new();

        if values.len() < SIZE {
            // Can fit in a single node
            for (i, &value) in values.iter().enumerate() {
                node.children[i + 1] = Some(Box::new(Node::with_value(value)));
            }
        } else {
            // Recursively split the values
            let chunk_size = values.len() / SIZE + 1;
            for (i, chunk) in values.chunks(chunk_size).enumerate() {
                if i == 0 {
                    node.children[i] = Some(Box::new(Node::with_values(chunk)));
                } else {
                    let (&first, tail) = chunk.split_first().unwrap();
                    let mut child = Node::with_values(tail);
                    child.value = Some(first);
                    node.children[i] = Some(Box::new(child));
                }
            }
        }

        node
    }
}

impl<'a, T: Copy> Iter<'a, T> {
    fn new(root: &'a Node<T>) -> Self {
        Iter {
            stack: vec![IterStackState::new(&root)],
        }
    }
}

impl<'a, T: Copy> IterStackState<'a, T> {
    fn new(node: &'a Node<T>) -> Self {
        IterStackState {
            next_coordinate: 0,
            remaining_children: node.children.iter(),
        }
    }

    fn next_node(&mut self) -> Option<&'a Node<T>> {
        loop {
            match self.remaining_children.next() {
                None => return None,
                Some(maybe_node) => {
                    self.next_coordinate += 1;
                    match maybe_node {
                        // Children (except the head) are filled from left to right
                        None if self.next_coordinate > 1 => return None,
                        None => {}
                        Some(node) => return Some(node),
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
                Some(state) => match state.next_node() {
                    None => {
                        self.stack.pop();
                    }
                    Some(child_node) => {
                        let coordinates = self
                            .stack
                            .iter()
                            .map(|state| state.next_coordinate - 1)
                            .collect_vec();

                        self.stack.push(IterStackState::new(child_node));

                        if let Some(value) = child_node.value {
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
            gotten: T::default(),
            removed: [T::default(); 3],
            num_removed: 0,
        }
    }

    fn push_removed(&mut self, value: T) {
        self.removed[self.num_removed] = value;
        self.num_removed += 1;
    }

    fn is_full(&self) -> bool {
        self.num_removed == self.removed.len()
    }

    pub fn gotten(&self) -> T {
        self.gotten
    }

    pub fn removed(&self) -> [T; 3] {
        self.removed
    }
}

impl<T: Copy> Insert3<T> {
    pub fn new(values: [T; 3], new_coordinates_buffers: [Coordinates; 3]) -> Self {
        Insert3 {
            values,
            num_inserted: 0,
            new_coordinates: new_coordinates_buffers,
        }
    }

    pub fn into_new_coordinates(self) -> [Coordinates; 3] {
        self.new_coordinates
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

    fn push_coordinate(&mut self, coordinate: usize) {
        for buffer in &mut self.new_coordinates[self.num_inserted..] {
            buffer.0.push(coordinate);
        }
    }

    fn pop(&mut self, push_coordinate: usize) -> T {
        self.push_coordinate(push_coordinate);
        let value = self.values[self.num_inserted];
        self.num_inserted += 1;
        value
    }

    fn is_empty(&self) -> bool {
        self.num_inserted == self.values.len()
    }
}

impl<T: Debug> fmt::Debug for Node<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let children: BTreeMap<_, _> = self
            .children
            .iter()
            .enumerate()
            .filter_map(|(i, child)| child.as_ref().map(|child| (i, child.as_ref())))
            .collect();

        match (&self.value, children.is_empty()) {
            (None, true) => write!(f, "_"),
            (Some(value), true) => write!(f, "{:?}", value),
            (None, false) => f.debug_map().entries(children).finish(),
            (Some(value), false) => f
                .debug_struct("Node")
                .field("value", value)
                .field("children", &children)
                .finish(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;
    use std::collections::BTreeMap;

    #[test]
    fn test() {
        let mut values = vec![];
        for i in 0..(SIZE * (SIZE - 1)) {
            values.push(i);
        }

        let mut line = DenseLine::new(&values);
        println!("{:?}", line.iter().format("\n"));
        assert_eq!(line.iter().map(|(_, value)| value).collect_vec(), values);

        let mut coordinates_by_value: BTreeMap<_, _> = line.iter().map(|(c, v)| (v, c)).collect();
        let result = line.get_1_and_remove_3(coordinates_by_value.get(&SIZE).unwrap());
        assert_eq!(result.gotten(), SIZE);
        assert_eq!(result.removed(), [SIZE + 1, SIZE + 2, SIZE + 3]);

        let c0 = coordinates_by_value.remove(&result.removed()[0]).unwrap();
        let c1 = coordinates_by_value.remove(&result.removed()[1]).unwrap();
        let c2 = coordinates_by_value.remove(&result.removed()[2]).unwrap();
        let insert3 = Insert3::new(result.removed(), [c0, c1, c2]);
        let insert3 = line.insert_3(coordinates_by_value.get(&(SIZE / 2)).unwrap(), insert3);
        let new_coordinates = insert3.into_new_coordinates();
        eprintln!("new_coordinates = {:?}", new_coordinates);
        println!("{:?}", line.iter().format("\n"));

        let target = SIZE * (SIZE - 1) - 2;
        let coordinates = line.iter().find(|&(_, value)| value == target).unwrap().0;
        let result = line.get_1_and_remove_3(&coordinates);
        assert_eq!(result.gotten(), target);
        assert_eq!(result.removed(), [target + 1, 0, 1]);
    }
}
