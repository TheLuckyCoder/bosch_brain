#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrackEdge {
    pub target: usize,
    pub dotted: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrackNode {
    pub x: f32,
    pub y: f32,
    pub edges: Vec<TrackEdge>,
}

// Do mint that id 1 is stored at index 0
pub struct Track(pub Vec<TrackNode>);

impl Track {
    #[inline(always)]
    pub fn get_node(&self, id: usize) -> Option<&TrackNode> {
        self.0.get(id)
    }

    pub fn get_edges(&self, source: usize) -> Option<&Vec<TrackEdge>> {
        self.get_node(source).map(|node| &node.edges)
    }

    pub fn get_edge(&self, source: usize, target: usize) -> Option<&TrackEdge> {
        self.get_edges(source).and_then(|edges| edges.get(target))
    }

    pub fn find_closest_node(&self, x: f32, y: f32) -> Option<(usize, &TrackNode)> {
        self.0
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                (x - a.x)
                    .hypot(y - a.y)
                    .partial_cmp(&(x - b.x).hypot(y - b.y))
                    .unwrap()
            })
            .map(|(id, node)| (id, node))
    }
}
