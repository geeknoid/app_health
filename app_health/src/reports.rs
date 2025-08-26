use crate::Report;
use core::fmt::{Debug, Formatter};
use std::vec::IntoIter;

/// An iterator over component reports.
pub struct Reports {
    iter: IntoIter<Report>,
}

impl Reports {
    #[must_use]
    pub(crate) fn new(reports: Vec<Report>) -> Self {
        Self { iter: reports.into_iter() }
    }
}

impl Iterator for Reports {
    type Item = Report;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn count(self) -> usize {
        self.iter.count()
    }

    fn fold<B, F>(self, init: B, f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter.fold(init, f)
    }
}

impl ExactSizeIterator for Reports {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl Clone for Reports {
    fn clone(&self) -> Self {
        Self { iter: self.iter.clone() }
    }
}

impl Debug for Reports {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.iter.fmt(f)
    }
}
