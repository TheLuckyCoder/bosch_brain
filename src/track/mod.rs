pub use self::data::*;
use std::collections::{BinaryHeap, HashMap};
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

pub fn find_path(
    track: &Track,
    start_coord: (NonNanF32, NonNanF32),
    end_coord: (NonNanF32, NonNanF32),
) -> Option<Vec<&TrackNode>> {
    let (_, start_node) =
        match track.find_closest_node(start_coord.0.get_f32(), start_coord.1.get_f32()) {
            Some(node) => node,
            None => panic!("start coord is not on track"),
        };

    let (_, end_node) = match track.find_closest_node(end_coord.0.get_f32(), end_coord.1.get_f32())
    {
        Some(node) => node,
        None => panic!("end coord is not on track"),
    };

    let mut prio_queue = BinaryHeap::new();

    prio_queue.push(State {
        cost: NonNanF32::new(0.0).expect("0.0 is not NaN"),
        node: start_node,
    });

    let mut came_from = HashMap::new();


    came_from.insert(start_node, Option::None);

    let mut cost_so_far = HashMap::new();

    cost_so_far.insert(start_node, NonNanF32::new(0.0).expect("0.0 is not NaN"));

    while let Some(State { cost: _cost, node }) = prio_queue.pop() {
        if node == end_node {
            break;
        }

        for edge in &node.edges {
            let new_cost = match cost_so_far.get(node) {
                Some(cost) => *cost,
                None => panic!("cost_so_far does not contain node"),
            };

            let next_node = match track.get_node_by_id(edge.target) {
                Some(node) => node,
                None => panic!("track does not contain node with id {}", edge.target),
            };

            if !cost_so_far.contains_key(next_node)
                || new_cost
                    < match cost_so_far.get(next_node) {
                        Some(cost) => *cost,
                        None => panic!("cost_so_far does not contain node"),
                    }
            {
                cost_so_far.insert(next_node, new_cost);

                let priority = new_cost + fast_euclidean_distance(next_node, end_node);

                prio_queue.push(State {
                    cost: priority,
                    node: next_node,
                });
                came_from.insert(next_node, Some(node));
            }
        }
    }

    if !came_from.contains_key(end_node) {
        return None;
    }

    let mut path = Vec::new();

    let mut current_node = end_node;

    while current_node != start_node {
        path.push(current_node);
        current_node = match came_from.get(current_node) {
            Some(node) => node.unwrap(),
            None => panic!("came_from does not contain node"),
        };
    }

    path.push(start_node);
    path.reverse();


    Some(path)
}

fn fast_euclidean_distance(node1: &TrackNode, node2: &TrackNode) -> NonNanF32 {
    (node1.x - node2.x).pow(2_f32) + (node1.y - node2.y).pow(2_f32)
}
