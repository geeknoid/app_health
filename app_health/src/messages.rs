use crate::PublisherMessage;
use core::fmt::{Debug, Formatter};
use core::slice::Iter;

/// An iterator over publisher messages and the number of times each has been reported.
pub struct Messages<'a> {
    iter: Iter<'a, (PublisherMessage, usize)>,
}

impl<'a> Messages<'a> {
    #[must_use]
    pub(crate) fn new(messages: &'a [(PublisherMessage, usize)]) -> Self {
        Self {
            iter: messages.iter(),
        }
    }
}

impl Iterator for Messages<'_> {
    type Item = (PublisherMessage, usize);

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
        self.iter
            .fold(init, |acc, (msg, count)| f(acc, (msg.clone(), *count)))
    }
}

impl ExactSizeIterator for Messages<'_> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl Clone for Messages<'_> {
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
        }
    }
}

impl Debug for Messages<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.iter.fmt(f)
    }
}
