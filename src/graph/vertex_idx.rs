use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct VertexIdx(pub(crate) usize);

impl VertexIdx {
    pub(crate) fn new() -> Self {
        pub(crate) static VERTEX_INCR_IDX: AtomicUsize = AtomicUsize::new(0);

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
