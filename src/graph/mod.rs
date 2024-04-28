use std::{
    collections::HashMap,
    sync::atomic::{AtomicUsize, Ordering},
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct VertexIdx(usize);

impl VertexIdx {
    pub(super) fn new() -> Self {
        static VERTEX_INCR_IDX: AtomicUsize = AtomicUsize::new(0);

        let mut idx = VERTEX_INCR_IDX.load(Ordering::Relaxed);

        // If it fails, another thread got first our idx. Try to get the new idx
        while let Err(new_idx) = VERTEX_INCR_IDX.compare_exchange(
            idx,
            (idx + 1) % usize::MAX,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            idx = new_idx;
        }

        Self(idx)
    }
}

#[derive(Debug, Default)]
pub struct Graph<V, E> {
    vertices: HashMap<VertexIdx, Vertex<V, E>>,
}

impl<V, E> Graph<V, E> {
    pub fn new() -> Self {
        Self {
            vertices: HashMap::new(),
        }
    }

    /// Retrieve an immutable reference to a vertex by its ID in `O(1)`.
    pub fn get_vertex(&self, vertex_idx: VertexIdx) -> Option<&Vertex<V, E>> {
        self.vertices.get(&vertex_idx)
    }

    /// Retrieve a mutable reference to a vertex by its ID in `O(1)`.
    pub fn get_mut_vertex(&mut self, vertex_idx: VertexIdx) -> Option<&mut Vertex<V, E>> {
        self.vertices.get_mut(&vertex_idx)
    }

    /// Insert a vertex with weight `V` in `O(1)`. Returns its ID.
    ///
    /// Use [`Self::insert_edge`] or [`Self::insert_edges`] to define its edges.
    pub fn insert_vertex(&mut self, weight: V) -> VertexIdx {
        let idx = VertexIdx::new();

        self.vertices.insert(
            idx,
            Vertex {
                edges: HashMap::new(),
                weight,
            },
        );

        idx
    }

    /// Remove a vertex by its ID in `O(|V|)`. Returns the [`Vertex`] itself, if it exists.
    pub fn remove_vertex(&mut self, vertex_idx: VertexIdx) -> Option<Vertex<V, E>> {
        let removed_vertex = self.vertices.remove(&vertex_idx)?;

        for vertex in &mut self.vertices {
            vertex.1.edges.remove(&vertex_idx);
        }

        Some(removed_vertex)
    }

    /// Insert or update an edge from a vertex `from` to a vertex `to` with weight `weight` in `O(1)`.
    ///
    /// Returns itself to allow for chaining [`Self::insert_edge`], if the vertices specified exist in this graph.
    pub fn insert_or_update_edge(
        &mut self,
        from: VertexIdx,
        to: VertexIdx,
        weight: E,
    ) -> Option<&mut Self> {
        if !self.vertices.contains_key(&to) {
            return None;
        }

        self.vertices.get_mut(&from)?.edges.insert(to, weight);
        Some(self)
    }

    /// A convenient method to insert or update multiple edges of type `(from, to, weight)` in `O(n)`, where `n` is the length of `edges`.
    ///
    /// Returns itself to allow for chaining [`Self::insert_edges`], if you are into it ¯\\\_(ツ)\_/¯
    pub fn insert_or_update_edges(
        &mut self,
        edges: impl IntoIterator<Item = (VertexIdx, VertexIdx, E)>,
    ) -> &mut Self {
        for (from, to, weight) in edges {
            if self.vertices.contains_key(&to) {
                if let Some(vertex) = self.vertices.get_mut(&from) {
                    vertex.edges.insert(to, weight);
                }
            }
        }

        self
    }

    /// Remove an edge from a vertex `from` to a vertex `to` in `O(1)`.
    ///
    /// Returns that edge's weight, if it exists.
    pub fn remove_edge(&mut self, from: VertexIdx, to: VertexIdx) -> Option<E> {
        self.vertices.get_mut(&from)?.edges.remove(&to)
    }
}

#[derive(Debug)]
pub struct Vertex<V, E> {
    weight: V,
    /// Edges from `self` to the vertices/keys with weight the value of an entry
    edges: HashMap<VertexIdx, E>,
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
        self.edges.iter().find(|e| *e.0 == to).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::{Graph, Vertex, VertexIdx};
    use helpers::test_neighbors;

    #[test]
    fn test_relationships() {
        // The end graph should look like this:
        //   [a: 23]-→[b: 1]
        //     ↑ ↓   ↖   ↑
        //   a[c: 7]  [d: 9] ↰
        //               ⤷---⤴
        let mut graph = Graph::<u16, ()>::new();

        let a_idx = graph.insert_vertex(23);
        let b_idx = graph.insert_vertex(1);
        let c_idx = graph.insert_vertex(7);
        let d_idx = graph.insert_vertex(9);

        graph.insert_or_update_edge(a_idx, b_idx, ());
        graph.insert_or_update_edges([
            (a_idx, b_idx, ()),
            (a_idx, c_idx, ()),
            (c_idx, a_idx, ()),
            (d_idx, a_idx, ()),
            (d_idx, b_idx, ()),
            (d_idx, d_idx, ()),
        ]);

        let temp_idx = graph.insert_vertex(u16::MAX);

        assert_eq!(
            graph.remove_vertex(temp_idx).map(|x| x.into_weight()),
            Some(u16::MAX),
            "Vertex temp has been already removed"
        );

        assert!(
            graph
                .insert_or_update_edge(temp_idx, temp_idx, ())
                .is_none(),
            "There can't be edges in non-existent vertices"
        );

        let a = graph.get_vertex(a_idx).expect("Vertex a doesn't exist");
        let b = graph.get_vertex(b_idx).expect("Vertex b doesn't exist");
        let c = graph.get_vertex(c_idx).expect("Vertex c doesn't exist");
        let d = graph.get_vertex(d_idx).expect("Vertex d doesn't exist");

        assert_eq!(*a.weight(), 23, "Wrong weight for vertex a");
        assert_eq!(*b.weight(), 1, "Wrong weight for vertex b");
        assert_eq!(*c.weight(), 7, "Wrong weight for vertex c");
        assert_eq!(*d.weight(), 9, "Wrong weight for vertex d");

        test_neighbors(a, &[b_idx, c_idx]);
        test_neighbors(b, &[]);
        test_neighbors(c, &[a_idx]);
        test_neighbors(d, &[b_idx, a_idx, d_idx]);
    }

    mod helpers {
        use super::{Vertex, VertexIdx};
        use std::{collections::HashSet, fmt::Debug};

        pub fn test_neighbors<V: Debug, E: Debug>(vertex: &Vertex<V, E>, expected: &[VertexIdx]) {
            let neighbors = vertex.neighbors().map(|(id, _)| id).collect::<HashSet<_>>();

            assert_eq!(
                neighbors.len(),
                expected.len(),
                "Unexpected number of neighbors in {vertex:?}"
            );

            for expected_neighbor in expected {
                assert!(
                    neighbors.contains(expected_neighbor),
                    "Neighbor {expected_neighbor:?} isn't in {vertex:?}'s neighbors"
                )
            }
        }
    }
}
