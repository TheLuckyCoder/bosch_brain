use crate::track::data::{Track, TrackEdge, TrackNode};
use serde::Deserialize;

#[derive(Clone, Deserialize)]
struct ParsingNode {
    pub id: usize,
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Deserialize)]
struct ParsingEdge {
    pub source: usize,
    pub target: usize,
    pub dotted: bool,
}

#[derive(Deserialize)]
struct NodesAndEdges {
    pub nodes: Vec<ParsingNode>,
    pub edges: Vec<ParsingEdge>,
}

pub fn parse_track(path: &str, y_axis_size: f32) -> std::io::Result<Track> {
    let file = std::fs::read_to_string(path)?;
    let nodes_and_edges: NodesAndEdges = serde_json::from_str(&file)?;

    let mut no = nodes_and_edges.nodes;
    no.insert(
        0,
        ParsingNode {
            id: 0,
            x: 0.0,
            y: 0.0,
        },
    );
    no.sort_by(|a, b| a.id.cmp(&b.id));

    let nodes = no
        .iter()
        .map(|node| {
            TrackNode::new(
                node.id,
                node.x * 100.0,
                (y_axis_size - node.y) * 100.0,
                nodes_and_edges
                    .edges
                    .iter()
                    .filter(|edge| edge.source == node.id)
                    .map(|edge| TrackEdge {
                        target: edge.target,
                        dotted: edge.dotted,
                    })
                    .collect(),
            )
        })
        .collect();

    Ok(Track(nodes))
}
