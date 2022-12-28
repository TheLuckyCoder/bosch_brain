use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
pub use self::data::*;
use std::mem::MaybeUninit;
use std::sync::Once;


mod data;
mod parsing;

pub fn get_track() -> &'static Track {
    static mut SINGLETON: MaybeUninit<Track> = MaybeUninit::uninit();
    static ONCE: Once = Once::new();

    unsafe {
        ONCE.call_once(|| {
            SINGLETON.write(parsing::parse_track("res/tracks/test_track.json").unwrap());
        });

        SINGLETON.assume_init_ref()
    }
}

#[derive(Clone, PartialEq)]
struct State<'a> {
    cost: f32,
    node: &'a TrackNode,
}

impl<'a> Eq for State<'a> {}

// `PartialOrd` needs to be implemented as well.
impl<'a> PartialOrd for State<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// The priority queue depends on `Ord`.
// Explicitly implement the trait so the queue becomes a min-heap
// instead of a max-heap.
impl<'a> Ord for State<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        // Notice that the we flip the ordering on costs.
        // In case of a tie we compare positions - this step is necessary
        // to make implementations of `PartialEq` and `Ord` consistent.
        other.cost.partial_cmp(&self.cost).unwrap()
            .then_with(|| self.node.cmp(other.node))
    }
}



pub fn find_path(track: &Track, start_coord: (f32, f32), end_coord: (f32, f32)) {
    // for i in &track.0 {
    //     println!("{:?}",i);
    // }

    let start_node = track.find_closest_node(start_coord.0, start_coord.1).unwrap().1;
    let end_node = track.find_closest_node(end_coord.0, end_coord.1).unwrap().1;
    println!("Start node: {:?}", start_node);
    println!("End node: {:?}", end_node);

    let mut prio_queue = BinaryHeap::new();

    prio_queue.push(State { cost: 0.0, node: start_node });

    let mut came_from = HashMap::new();

    let temp = TrackNode { id: 0, x: 0.0, y: 0.0, edges: Vec::new() };

    came_from.insert(start_node, &temp);

    let mut cost_so_far = HashMap::new();

    cost_so_far.insert(start_node, 0.0);

    while let Some(State { cost: _cost, node }) = prio_queue.pop() {
        if node == end_node {
            break;
        }

        for edge in &node.edges {
            let new_cost = *cost_so_far.get(node).unwrap();

            let next_node = track.get_node_by_id(edge.target).unwrap();

            if !cost_so_far.contains_key(next_node) || new_cost < *cost_so_far.get(next_node).unwrap() {
                cost_so_far.insert(next_node, new_cost);
                let priority = new_cost + fast_euclidean_distance(next_node, end_node);
                prio_queue.push(State { cost: priority, node: next_node});
                came_from.insert(next_node, node);
            }
        }
    }

    if !came_from.contains_key(end_node) {
        println!("No path found");
        return;
    }

    let mut path = Vec::new();

    let mut current_node = end_node;

    while current_node != start_node {
        path.push(current_node);
        current_node = came_from.get(current_node).unwrap();
    }

    path.push(start_node);
    path.reverse();

    println!("{}",path.len());

    for node in path {
        println!("{:?}",node);
    }

}

fn fast_euclidean_distance(node1: &TrackNode, node2: &TrackNode) -> f32 {
    (node1.x - node2.x).powf(2_f32) + (node1.y - node2.y).powf(2_f32)
}
