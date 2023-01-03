use std::cmp::Ordering;

use ordered_float::OrderedFloat;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TrackEdge {
    pub target: usize,
    pub dotted: bool,
}

pub type OrdFloat = OrderedFloat<f32>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TrackNode {
    pub id: usize,
    pub x: OrdFloat,
    pub y: OrdFloat,
    pub edges: Vec<TrackEdge>,
}

impl TrackNode {
    pub fn new(id: usize, x: f32, y: f32, edges: Vec<TrackEdge>) -> TrackNode {
        TrackNode {
            id,
            x: OrderedFloat(x),
            y: OrderedFloat(y),
            edges,
        }
    }

    pub fn get_x(&self) -> f32 {
        self.x.into_inner()
    }

    pub fn get_y(&self) -> f32 {
        self.y.into_inner()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct State<'a> {
    pub cost: OrdFloat,
    pub node: &'a TrackNode,
}

impl<'a> PartialOrd for State<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let ordering = other.cost.partial_cmp(&self.cost);

        if ordering == Some(Ordering::Equal) {
            self.node.partial_cmp(other.node)
        } else {
            ordering
        }
    }
}

// The priority queue depends on `Ord`.
// Explicitly implement the trait so the queue becomes a min-heap
impl<'a> Ord for State<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        // Notice that the we flip the ordering on costs.
        // In case of a tie we compare positions - this step is necessary
        // to make implementations of `PartialEq` and `Ord` consistent.
        other
            .cost
            .cmp(&self.cost)
            .then_with(|| self.node.cmp(other.node))
    }
}

pub struct Track(pub Vec<TrackNode>);

impl Track {
    #[inline(always)]
    pub fn get_node_by_id(&self, id: usize) -> Option<&TrackNode> {
        self.0.get(id)
    }

    pub fn get_edges(&self, source: usize) -> Option<&Vec<TrackEdge>> {
        self.get_node_by_id(source).map(|node| &node.edges)
    }

    pub fn get_edge(&self, source: usize, target: usize) -> Option<&TrackEdge> {
        self.get_edges(source).and_then(|edges| edges.get(target))
    }

    pub fn find_closest_node(&self, x: f32, y: f32) -> Option<&TrackNode> {
        self.0.iter().min_by(|a, b| {
            (x - a.x.0)
                .hypot(y - a.y.0)
                .partial_cmp(&(x - b.x.0).hypot(y - b.y.0))
                .unwrap()
        })
    }
}
