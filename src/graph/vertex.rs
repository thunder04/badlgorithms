use super::VertexIdx;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Vertex<V, E> {
    pub(crate) weight: V,
    /// Edges from `self` to the vertices/keys with weight the value of an entry
    pub(crate) edges: HashMap<VertexIdx, E>,
}

impl<V, E> Vertex<V, E> {
    pub fn weight(&self) -> &V {
        &self.weight
    }

    pub fn weight_mut(&mut self) -> &mut V {
        &mut self.weight
    }

    pub fn into_weight(self) -> V {
        self.weight
    }

    pub fn neighbors(&self) -> impl Iterator<Item = (VertexIdx, &E)> {
        self.edges.iter().map(|(k, v)| (*k, v))
    }

    pub fn neighbors_mut(&mut self) -> impl Iterator<Item = (VertexIdx, &mut E)> {
        self.edges.iter_mut().map(|(k, v)| (*k, v))
    }

    pub fn into_neighbors(self) -> impl Iterator<Item = (VertexIdx, E)> {
        self.edges.into_iter()
    }

    pub fn is_adjacent(&self, to: VertexIdx) -> bool {
        self.edges.iter().any(|e| *e.0 == to)
    }
}
