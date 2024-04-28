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

    /// Insert or update a vertex with weight `V` in `O(1)`. Returns its ID.
    ///
    /// Use [`Self::insert_edge`] or [`Self::insert_edges`] to define its edges.
    pub fn insert_or_update_vertex(&mut self, weight: V) -> VertexIdx {
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

    /// Insert or update a vertex with weight `V` and a predefined ID in `O(1)`. Returns `self`.
    ///
    /// Use [`Self::insert_edge`] or [`Self::insert_edges`] to define its edges.
    pub fn insert_or_update_vertex_with_id(&mut self, weight: V, idx: VertexIdx) -> &mut Self {
        self.vertices.insert(
            idx,
            Vertex {
                edges: HashMap::new(),
                weight,
            },
        );

        self
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

    /// Filter and optionally map to new types, vertices and edges of this graph in `O(|V||E|)`.
    ///
    /// - `vertex_map(vertex_idx) -> Option<new_vertex_weight>`
    ///     - Return `None` to exclude the vertex with ID `vertex_idx`.
    ///     - The mapped vertex is guaranteed to have the same ID.
    /// - `edge_map((from_vertex_idx, from_vertex), (to_vertex_idx, to_vertex), old_edge_weight) -> Option<new_edge_weight>`
    ///     - It is called only if `vertex_map` returned `Some(_)` for both `from_vertex_idx` and `to_vertex_idx`.
    ///     - Return `None` to exclude the edge `(from_vertex_idx, to_vertex_idx)`.
    pub fn filter_map<F, G, NV, NE>(&self, mut vertex_map: F, mut edge_map: G) -> Graph<NV, NE>
    where
        F: FnMut(VertexIdx) -> Option<NV>,
        G: FnMut((VertexIdx, &Vertex<NV, NE>), (VertexIdx, &Vertex<NV, NE>), &E) -> Option<NE>,
    {
        let mut graph = Graph::new();

        for (&vertex_idx, _) in &self.vertices {
            if let Some(new_vertex_weight) = vertex_map(vertex_idx) {
                graph.insert_or_update_vertex_with_id(new_vertex_weight, vertex_idx);
            }
        }

        'outer: for (&from_vertex_idx, old_vertex) in &self.vertices {
            for (&to_vertex_idx, old_edge) in &old_vertex.edges {
                // Skip edges that their `from` part doesn't exist in the new graph
                //
                // This statement could have been outside of this for loop if it weren't
                // for the rustc compiler complaining about:
                //     error[E0502]: cannot borrow `graph` as mutable because it is also borrowed as immutable
                let Some(from_vertex) = graph.get_vertex(from_vertex_idx) else {
                    continue 'outer;
                };

                // Skip edges that their `to` part doesn't exist in the new graph
                let Some(to_vertex) = graph.get_vertex(to_vertex_idx) else {
                    continue;
                };

                if let Some(new_edge_weight) = edge_map(
                    (from_vertex_idx, from_vertex),
                    (to_vertex_idx, to_vertex),
                    old_edge,
                ) {
                    graph.insert_or_update_edge(from_vertex_idx, to_vertex_idx, new_edge_weight);
                }
            }
        }

        graph
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
        // The graph should look like this:
        //   [a: 23]-→[b: 1]
        //     ↑ ↓   ↖   ↑
        //   a[c: 7]  [d: 9] ↰
        //               ⤷---⤴
        let mut graph = Graph::<u16, ()>::new();

        let a_idx = graph.insert_or_update_vertex(23);
        let b_idx = graph.insert_or_update_vertex(1);
        let c_idx = graph.insert_or_update_vertex(7);
        let d_idx = graph.insert_or_update_vertex(9);

        graph.insert_or_update_edge(a_idx, b_idx, ());
        graph.insert_or_update_edges([
            (a_idx, b_idx, ()),
            (a_idx, c_idx, ()),
            (c_idx, a_idx, ()),
            (d_idx, a_idx, ()),
            (d_idx, b_idx, ()),
            (d_idx, d_idx, ()),
        ]);

        let temp_idx = graph.insert_or_update_vertex(u16::MAX);

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

    #[test]
    fn test_filter_map() {
        // The graph will look like this:
        //
        //              6
        //         _________
        //        /         \
        //        ↓    3    |
        //   [c: 8] ------> [a: 3]
        //    ↑   |
        //  2 |   | 4
        //    |   ↓
        //   [b: 6] ↰
        //     ⤷---⤴
        //       8
        let mut graph = Graph::<u16, u8>::new();

        let a_idx = graph.insert_or_update_vertex(3);
        let b_idx = graph.insert_or_update_vertex(6);
        let c_idx = graph.insert_or_update_vertex(8);

        graph.insert_or_update_edges([
            (b_idx, c_idx, 2),
            (c_idx, b_idx, 4),
            (a_idx, c_idx, 6),
            (c_idx, a_idx, 3),
            (b_idx, b_idx, 8),
        ]);

        // The graph should look like this:
        //
        //              1
        //         _________
        //        /         \
        //        ↓    2    |
        //   [c: 4] ------> [b: 3]
        let mapped_graph: Graph<u16, i8> = graph.filter_map(
            |idx| {
                let weight = graph.get_vertex(idx)?.weight();

                (weight % 2 == 0).then_some(weight / 2)
            },
            |(from_vertex_idx, _), (to_vertex_idx, _), old_edge_weight| {
                // Remove cycles
                (from_vertex_idx != to_vertex_idx).then_some(*old_edge_weight as i8 / 2)
            },
        );

        assert!(
            mapped_graph.get_vertex(a_idx).is_none(),
            "Vertex a shouldn't exist"
        );

        let b = mapped_graph
            .get_vertex(b_idx)
            .expect("Vertex b doesn't exist");
        let c = mapped_graph
            .get_vertex(c_idx)
            .expect("Vertex c doesn't exist");

        assert_eq!(*b.weight(), 3, "Wrong weight for vertex b");
        assert_eq!(*c.weight(), 4, "Wrong weight for vertex c");

        test_neighbors(b, &[c_idx]);
        test_neighbors(c, &[b_idx]);
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
