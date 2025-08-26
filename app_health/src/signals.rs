use crate::signal::Signal;
use core::fmt::{Debug, Formatter};
use core::slice::Iter;

/// An iterator over publisher signals and the number of times each has been reported.
pub struct Signals<'a> {
    iter: Iter<'a, (Signal, usize)>,
}

impl<'a> Signals<'a> {
    #[must_use]
    pub(crate) fn new(messages: &'a [(Signal, usize)]) -> Self {
        Self { iter: messages.iter() }
    }
}

impl Iterator for Signals<'_> {
    type Item = (Signal, usize);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(msg, count)| (msg.clone(), *count))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn count(self) -> usize {
        self.iter.count()
    }

    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter.fold(init, |acc, (msg, count)| f(acc, (msg.clone(), *count)))
    }
}

impl ExactSizeIterator for Signals<'_> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl Clone for Signals<'_> {
    fn clone(&self) -> Self {
        Self { iter: self.iter.clone() }
    }
}

impl Debug for Signals<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.iter.fmt(f)
    }
}
