use crate::track;
use crate::track::find_path;

#[test]
fn follow_track() {
    let track = track::get_track();
    let start_node = track.get_node_by_id(24).unwrap();
    let end_node = track.get_node_by_id(60).unwrap();

    let path = find_path(track, start_node, end_node).unwrap();

    println!("{}", path.len());
}
