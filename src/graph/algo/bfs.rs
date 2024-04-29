use crate::graph::{Graph, VertexIdx};
use std::collections::VecDeque;

#[derive(PartialEq)]
enum Color {
    White,
    Grey,
    Black,
}

pub struct EnhancedWeight<V> {
    parent: Option<VertexIdx>,
    distance: u32,
    color: Color,
    weight: V,
}

impl<V> EnhancedWeight<V> {
    pub fn parent(&self) -> Option<VertexIdx> {
        self.parent
    }

    pub fn distance(&self) -> u32 {
        self.distance
    }

    pub fn weight(&self) -> &V {
        &self.weight
    }

    pub fn into_weight(self) -> V {
        self.weight
    }
}

/// Performs BFS on a graph `graph` with source vertex `src_vertex_idx`.
///
/// Returns `None` if the source vertex doesn't exist in the graph.
pub fn breadth_first_search<V, E>(
    graph: &Graph<V, E>,
    src_vertex_idx: VertexIdx,
) -> Option<Graph<EnhancedWeight<&V>, &E>> {
    let mut graph: Graph<EnhancedWeight<&V>, &E> = graph.filter_map(
        |idx| {
            let vertex = graph.get_vertex(idx)?;

            Some(if idx == src_vertex_idx {
                EnhancedWeight {
                    weight: vertex.weight(),
                    color: Color::Grey,
                    distance: 0,
                    parent: None,
                }
            } else {
                EnhancedWeight {
                    weight: vertex.weight(),
                    color: Color::White,
                    distance: u32::MAX,
                    parent: None,
                }
            })
        },
        |_, _, edge| Some(edge),
    );

    if graph.get_vertex(src_vertex_idx).is_none() {
        return None;
    }

    let mut queue: VecDeque<VertexIdx> = VecDeque::from([src_vertex_idx]);
    let mut neighbors_cache = Vec::new();

    while let Some(vertex_idx) = queue.pop_front() {
        let vertex = graph.get_vertex(vertex_idx)?;
        let vertex_distance = vertex.weight().distance;

        // A workaround for borrowing a mutable reference while an immutable reference is still in use.
        // I don't like it, and I'm too lazy to refactor everything, but it works.
        neighbors_cache.extend(vertex.neighbors().map(|(idx, _)| idx));

        for neighbor_idx in neighbors_cache.drain(..) {
            let neighbor = graph.get_mut_vertex(neighbor_idx)?;
            let neighbor_weight = neighbor.weight_mut();

            if neighbor_weight.color == Color::White {
                neighbor_weight.distance = vertex_distance + 1;
                neighbor_weight.parent = Some(vertex_idx);
                neighbor_weight.color = Color::Grey;

                queue.push_back(neighbor_idx);
            }
        }

        // Can't move this to `vertex` without making the borrow checker mad
        graph.get_mut_vertex(vertex_idx)?.weight_mut().color = Color::Black;
    }

    Some(graph)
}
