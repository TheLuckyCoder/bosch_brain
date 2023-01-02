use std::cmp::Ordering;
use std::hash::Hash;
use std::ops::{Add, Sub};

#[derive(Debug, Copy, Clone)]
pub struct NonNanF32(f32);

impl NonNanF32 {
    pub fn new(val: f32) -> Option<NonNanF32> {
        if val.is_nan() {
            None
        } else {
            Some(NonNanF32(val))
        }
    }

    pub fn pow(self, exp: f32) -> NonNanF32 {
        NonNanF32(self.0.powf(exp))
    }

    pub fn get_f32(self) -> f32 {
        self.0
    }
}

impl PartialEq for NonNanF32 {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for NonNanF32 {}

impl Add for NonNanF32 {
    type Output = NonNanF32;

    fn add(self, rhs: Self) -> Self::Output {
        NonNanF32(self.0 + rhs.0)
    }
}

impl Sub for NonNanF32 {
    type Output = NonNanF32;

    fn sub(self, rhs: Self) -> Self::Output {
        NonNanF32(self.0 - rhs.0)
    }
}

impl PartialOrd for NonNanF32 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for NonNanF32 {
    fn cmp(&self, other: &NonNanF32) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Hash for NonNanF32 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TrackEdge {
    pub target: usize,
    pub dotted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TrackNode {
    pub id: usize,
    pub x: NonNanF32,
    pub y: NonNanF32,
    pub edges: Vec<TrackEdge>,
}

impl TrackNode {
    pub fn new(id: usize, x: f32, y: f32, edges: Vec<TrackEdge>) -> Option<TrackNode> {
        if x.is_nan() || y.is_nan() {
            None
        } else {
            Some(TrackNode {
                id,
                x: NonNanF32(x),
                y: NonNanF32(y),
                edges,
            })
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct State<'a> {
    pub(crate) cost: NonNanF32,
    pub(crate) node: &'a TrackNode,
}

impl<'a> PartialOrd for State<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let option = other.cost.partial_cmp(&self.cost);

        if option.is_none() {
            None
        } else if option == Some(Ordering::Equal) {
            self.node.partial_cmp(other.node)
        } else {
            option
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

    pub fn find_closest_node(&self, x: f32, y: f32) -> Option<(usize, &TrackNode)> {
        self.0
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                (x - a.x.0)
                    .hypot(y - a.y.0)
                    .partial_cmp(&(x - b.x.0).hypot(y - b.y.0))
                    .unwrap()
            })
            .map(|(id, node)| (id, node))
    }
}
