use core::fmt::{Debug, Formatter};
use crate::{Health, Entry};

pub struct Entries;

impl Entries {
    #[must_use]
    pub(crate) fn new() -> Self {
        todo!()
    }
}

impl Iterator for Entries {
    type Item = Entry;

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

impl ExactSizeIterator for Entries {
    fn len(&self) -> usize {
        todo!()
    }
}

impl Clone for Entries {
    fn clone(&self) -> Self {
        todo!()
    }
}

impl Debug for Entries {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        todo!()
    }
}
