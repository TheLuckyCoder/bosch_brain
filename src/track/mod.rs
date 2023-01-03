use std::collections::{BinaryHeap, HashMap};
use std::mem::MaybeUninit;
use std::sync::Once;

use ordered_float::OrderedFloat;

pub use self::data::*;

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
    start_coord: (f32, f32),
    end_coord: (f32, f32),
) -> Result<Vec<&TrackNode>, String> {
    let start_node = track
        .find_closest_node(start_coord.0, start_coord.1)
        .ok_or_else(|| "Didn't find start node".to_string())?;
    let end_node = track
        .find_closest_node(end_coord.0, end_coord.1)
        .ok_or_else(|| "Didn't find start node".to_string())?;
    let mut prio_queue = BinaryHeap::new();

    prio_queue.push(State {
        cost: OrderedFloat(0.0),
        node: start_node,
    });

    let mut came_from = HashMap::new();
    came_from.insert(start_node, track.0.first().unwrap());

    let mut cost_so_far = HashMap::new();
    cost_so_far.insert(start_node, OrderedFloat(0.0));

    while let Some(State { cost: _cost, node }) = prio_queue.pop() {
        if node == end_node {
            break;
        }

        for edge in &node.edges {
            let new_cost = *cost_so_far
                .get(node)
                .expect("cost_so_far does not contain node");

            let next_node = match track.get_node_by_id(edge.target) {
                Some(node) => node,
                None => panic!("track does not contain node with id {}", edge.target),
            };

            let next_node_cost = cost_so_far.get(next_node);

            if next_node_cost.is_none() || new_cost < *next_node_cost.unwrap() {
                cost_so_far.insert(next_node, new_cost);

                let priority = new_cost + fast_euclidean_distance(next_node, end_node);

                prio_queue.push(State {
                    cost: priority,
                    node: next_node,
                });
                came_from.insert(next_node, node);
            }
        }
    }

    if !came_from.contains_key(end_node) {
        return Err("No path found".to_string());
    }

    let mut path = Vec::new();

    let mut current_node = end_node;

    while current_node != start_node {
        path.push(current_node);
        current_node = match came_from.get(current_node) {
            Some(node) => node,
            _ => panic!("came_from does not contain node"),
        };
    }

    path.push(start_node);
    path.reverse();

    Ok(path)
}

fn fast_euclidean_distance(node1: &TrackNode, node2: &TrackNode) -> f32 {
    (node1.x - node2.x).powf(2_f32) + (node1.y - node2.y).powf(2_f32)
}

mod tests {
    use super::*;

    #[test]
    fn test_find_path() {
        let track = get_track();
        let start_node = match track.get_node_by_id(24) {
            Some(node) => node,
            None => panic!("start node not found"),
        };

        let end_node = match track.get_node_by_id(60) {
            Some(node) => node,
            None => panic!("end node not found"),
        };

        let path = find_path(
            track,
            (start_node.get_x(), start_node.get_y()),
            (end_node.get_x(), end_node.get_y()),
        )
        .unwrap();

        assert_eq!(path.len(), 6);
    }
}
