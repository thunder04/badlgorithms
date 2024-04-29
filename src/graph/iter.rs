use super::{Vertex, VertexIdx};
use std::collections::hash_map::{
    IntoIter as HashMapIntoIter, Iter as HashMapIter, IterMut as HashMapIterMut,
};

pub struct IntoVerticesIterator<V, E>(pub(super) HashMapIntoIter<VertexIdx, Vertex<V, E>>);

impl<V, E> Iterator for IntoVerticesIterator<V, E> {
    type Item = (VertexIdx, Vertex<V, E>);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<V, E> ExactSizeIterator for IntoVerticesIterator<V, E> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

pub struct VerticesIter<'a, V, E>(pub(super) HashMapIter<'a, VertexIdx, Vertex<V, E>>);

impl<'a, V, E> Iterator for VerticesIter<'a, V, E> {
    type Item = (VertexIdx, &'a Vertex<V, E>);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(idx, v)| (*idx, v))
    }
}

impl<'a, V, E> ExactSizeIterator for VerticesIter<'a, V, E> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

pub struct VerticesIterMut<'a, V, E>(pub(super) HashMapIterMut<'a, VertexIdx, Vertex<V, E>>);

impl<'a, V, E> Iterator for VerticesIterMut<'a, V, E> {
    type Item = (VertexIdx, &'a mut Vertex<V, E>);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(idx, v)| (*idx, v))
    }
}

impl<'a, V, E> ExactSizeIterator for VerticesIterMut<'a, V, E> {
    fn len(&self) -> usize {
        self.0.len()
    }
}
