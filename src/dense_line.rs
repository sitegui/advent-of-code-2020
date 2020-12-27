use itertools::Itertools;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::{fmt, slice};

#[derive(Debug)]
pub struct DenseLine<T> {
    root: Node<T>,
}

#[derive(Debug)]
pub struct Iter<'a, T> {
    stack: Vec<IterStackState<'a, T>>,
}

#[derive(Debug, Clone)]
pub struct Coordinates(Vec<usize>);

#[derive(Debug)]
pub struct Get1Remove3<T> {
    gotten: Option<T>,
    removed: [Option<T>; 3],
    num_removed: usize,
}

#[derive(Debug)]
pub struct Insert3<T> {
    values: [T; 3],
    num_inserted: usize,
    new_coordinates: [Coordinates; 3],
}

struct Node<T> {
    value: Option<T>,
    used_head: bool,
    children: Vec<Node<T>>,
}

#[derive(Debug)]
struct IterStackState<'a, T> {
    next_coordinate: usize,
    remaining_children: slice::Iter<'a, Node<T>>,
}

impl<T: Copy + Debug> DenseLine<T> {
    pub fn new(values: impl IntoIterator<Item = T>) -> Self {
        DenseLine {
            root: Node::with_values(values),
        }
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter::new(&self.root)
    }

    pub fn get_1_and_remove_3(&mut self, start: &Coordinates) -> Get1Remove3<T> {
        let coordinates = &start.0;

        fn get_1_and_remove_3<T: Copy + Debug>(
            node: &mut Node<T>,
            coordinates: &[usize],
            result: &mut Get1Remove3<T>,
        ) {
            let (&coordinate, sub_coordinates) = coordinates.split_first().unwrap();

            if sub_coordinates.is_empty() {
                let child = &mut node.children[coordinate as usize];
                result.gotten = Some(child.value.unwrap());
                remove_3(&mut child.children, result);
            } else {
                let child = &mut node.children[coordinate as usize];
                get_1_and_remove_3(child, sub_coordinates, result);
            }

            if !result.is_full() {
                let next_children = &mut node.children[coordinate as usize + 1..];
                remove_3(next_children, result);
            }
        }

        fn remove_3<T: Copy>(nodes: &mut [Node<T>], result: &mut Get1Remove3<T>) {
            for node in nodes {
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
                let child = &mut node.children[coordinate as usize];
                navigate(child, sub_coordinates, values);
            }
        }

        /// Start the insertion after the node `nodes[coordinate]`.
        fn insert_after<T: Copy>(
            nodes: &mut Vec<Node<T>>,
            coordinate: usize,
            values: &mut Insert3<T>,
        ) {
            if values.is_empty() {
                return;
            }

            // Pick exclusive references to the target node and its right sibling (if any)
            let (node, maybe_sibling) = {
                let (before, after) = nodes.split_at_mut(coordinate + 1);
                (before.last_mut().unwrap(), after.first_mut())
            };

            if node.is_leaf() {
                match maybe_sibling {
                    None => {
                        // Free space at right => insert new node
                        values.pop_coordinate();
                        let new_coordinate = nodes.len();
                        let value = values.pop(new_coordinate);
                        nodes.push(Node::with_value(value));
                        insert_after(nodes, new_coordinate, values);
                    }
                    Some(sibling) => {
                        if sibling.value.is_none() {
                            // Hollow sibling => insert at it
                            values.pop_coordinate();
                            sibling.value = Some(values.pop(coordinate + 1));
                            insert_after(nodes, coordinate + 1, values);
                        } else {
                            // Full sibling => create a neck
                            node.children.push(Node::new());
                            node.children.push(Node::with_value(values.pop(1)));
                            insert_after(&mut node.children, 1, values);
                        }
                    }
                }
            } else if node.used_head {
                // Follow down the head
                values.push_coordinate(0);
                insert_after(&mut node.children, 0, values);
            } else {
                let neck = &mut node.children[1];
                if neck.value.is_some() {
                    // Has a full neck => follow down the head
                    node.used_head = true;
                    values.push_coordinate(0);
                    insert_after(&mut node.children, 0, values);
                } else {
                    // Has a hollow neck => insert in it
                    neck.value = Some(values.pop(1));
                    insert_after(&mut node.children, 1, values);
                }
            }
        }

        values.set_base_coordinates(after);
        navigate(&mut self.root, &after.0, &mut values);

        values
    }

    pub fn next(&self, base: &mut Coordinates) -> T {
        fn find_next<T: Copy + Debug>(
            nodes: &[Node<T>],
            coordinates: &mut Vec<usize>,
            level: usize,
        ) -> Option<T> {
            match coordinates.get(level) {
                Some(&coordinate) => {
                    if let Some(value) =
                        find_next(&nodes[coordinate as usize].children, coordinates, level + 1)
                    {
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

        fn find_first<T: Copy + Debug>(
            nodes: &[Node<T>],
            coordinates: &mut Vec<usize>,
            level: usize,
            coordinate_offset: usize,
        ) -> Option<T> {
            coordinates.push(0);
            for (i, child) in nodes.iter().enumerate() {
                if let Some(value) = child.value {
                    coordinates[level] = coordinate_offset + i;
                    return Some(value);
                }

                if let Some(value) = find_first(&child.children, coordinates, level + 1, 0) {
                    coordinates[level] = coordinate_offset + i;
                    return Some(value);
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

    pub fn get(&self, coordinates: &Coordinates) -> T {
        let mut node = &self.root;
        for &coordinate in &coordinates.0 {
            node = &node.children[coordinate];
        }
        node.value.unwrap()
    }

    pub fn print_stats(&self) {
        let mut dist: BTreeMap<_, usize> = BTreeMap::new();

        for child in &self.root.children {
            *dist.entry(child.children.len()).or_default() += 1;
        }

        println!(
            "root_len={}, fragmentation={}, child_lens={:?}",
            self.root.children.len(),
            self.root.fragmentation(),
            dist
        );
    }
}

impl<T: Copy> Node<T> {
    fn new() -> Self {
        Node {
            value: None,
            used_head: false,
            children: vec![],
        }
    }

    fn with_value(value: T) -> Self {
        let mut node = Node::new();
        node.value = Some(value);
        node
    }

    fn with_values(values: impl IntoIterator<Item = T>) -> Self {
        let mut node = Node::new();

        // Add empty head
        node.children.push(Node::new());
        node.children
            .extend(values.into_iter().map(Node::with_value));

        node
    }

    fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    fn fragmentation(&self) -> f32 {
        if self.children.is_empty() {
            0.
        } else {
            self.children
                .iter()
                .enumerate()
                .filter(|&(i, child)| i > 0 && child.children.is_empty() && child.value.is_none())
                .count() as f32
                / self.children.len() as f32
        }
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
        let result = self.remaining_children.next();

        if result.is_some() {
            self.next_coordinate += 1;
        }

        result
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

impl<T: Copy> Get1Remove3<T> {
    fn new() -> Self {
        Get1Remove3 {
            gotten: None,
            removed: [None; 3],
            num_removed: 0,
        }
    }

    fn push_removed(&mut self, value: T) {
        self.removed[self.num_removed] = Some(value);
        self.num_removed += 1;
    }

    fn is_full(&self) -> bool {
        self.num_removed == self.removed.len()
    }

    pub fn gotten(&self) -> T {
        self.gotten.unwrap()
    }

    pub fn removed(&self) -> [T; 3] {
        [
            self.removed[0].unwrap(),
            self.removed[1].unwrap(),
            self.removed[2].unwrap(),
        ]
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

impl Coordinates {
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T: Debug> fmt::Debug for Node<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let children: BTreeMap<_, _> = self.children.iter().enumerate().collect();

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
