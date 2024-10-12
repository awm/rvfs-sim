// pub mod ipin;
mod library;
pub mod sim;
pub mod wire;
pub mod wirevalue;

/// Identifier used to look up simulation components.
pub type Id = usize;

/// Iterator over a sequence of Ids.
pub struct IdIter {
    /// Present Id.
    id: Id,
    /// Iteration terminator.
    end: Id,
}

impl IdIter {
    /// Create a new iterator.
    ///
    /// # Parameters
    ///
    /// - `end`: Terminating value of the iteration (non-inclusive).
    fn new(end: Id) -> Self {
        Self { id: 0, end }
    }
}

impl Iterator for IdIter {
    type Item = Id;

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.id;
        if id < self.end {
            self.id += 1;
            Some(id)
        } else {
            None
        }
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_iter_create() {
        // GIVEN an Id endpoint
        let end: Id = 7;
        // WHEN an iterator is created
        let it = IdIter::new(end);
        // THEN creation succeeds and the iterator has "end" number of entries
        assert_eq!(end, it.count());
    }
    #[test]
    fn id_iter_iterate() {
        // GIVEN an initialized iterator
        let mut it = IdIter::new(4);
        // THEN the iterator has the expected entries
        assert_eq!(Some(0), it.next());
        assert_eq!(Some(1), it.next());
        assert_eq!(Some(2), it.next());
        assert_eq!(Some(3), it.next());
        assert_eq!(None, it.next());
    }
}
