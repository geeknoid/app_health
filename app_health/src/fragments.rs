use core::fmt::{Debug, Formatter};

pub struct Fragments;

impl Fragments {
    #[must_use]
    pub(crate) fn new() -> Self {
        todo!()
    }
}

impl Iterator for Fragments {
    type Item = (String, usize);

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        todo!()
    }

    fn count(self) -> usize {
        todo!()
    }

    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        todo!()
    }
}

impl ExactSizeIterator for Fragments {
    fn len(&self) -> usize {
        todo!()
    }
}

impl Clone for Fragments {
    fn clone(&self) -> Self {
        todo!()
    }
}

impl Debug for Fragments {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        todo!()
    }
}
