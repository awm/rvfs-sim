//! A Library holds items and allows them to be checked out temporarily.

use crate::{Id, IdIter};

/// A container which allows items to be temporarily checked in and out by Id.
#[derive(Debug)]
pub struct Library<T> {
    /// The "stacks" or "shelves" of the Library.
    items: Vec<Option<T>>,
}

impl<T> Library<T> {
    /// Create a new Library instance.
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Add a new item to the Library's collection and provide the Id which can be used to look it up later.
    ///
    /// # Parameters
    ///
    /// - `item`: The new item to be owned by the Library.
    pub fn add(&mut self, item: T) -> Id {
        let result = self.items.len();
        self.items.push(Some(item));
        result
    }

    /// Obtain an iterator over the Library's Ids.
    pub fn iter(&self) -> IdIter {
        IdIter::new(self.items.len())
    }

    /// Inspect a Library item without checking it out.
    ///
    /// # Parameters
    ///
    /// - `id`: Id of the item to inspect.
    pub fn inspect(&self, id: Id) -> &Option<T> {
        if id < self.items.len() {
            // The item is on the shelf.
            &self.items[id]
        } else {
            // The item is currently checked out.
            &None
        }
    }

    /// Check an item out of the Library, leaving its space empty.
    ///
    /// # Parameters
    ///
    /// - `id`: Id of the item to check out.
    pub fn checkout(&mut self, id: Id) -> Option<T> {
        if id < self.items.len() {
            // The item is on the shelf.
            self.items[id].take()
        } else {
            // The item is currently checked out.
            None
        }
    }

    /// Check an item back into the Library.
    ///
    /// # Parameters
    ///
    /// - `id`: Id of the item to check in.
    /// - `item`: The item being returned to the Library.
    pub fn checkin(&mut self, id: Id, item: T) -> Result<Id, String> {
        if id < self.items.len() && self.items[id].is_none() {
            self.items[id] = Some(item);
            Ok(id)
        } else {
            Err("Item cannot be checked in with that ID!".to_string())
        }
    }

    /// Verify that all items are checked in and accounted for.
    pub fn audit(&self) -> Result<(), String> {
        if self.items.iter().any(|i| i.is_none()) {
            Err("Items missing from library!".to_string())
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn library_create() {
        // GIVEN a type to hold in the library
        // WHEN a library is created
        let lib = Library::<i32>::new();
        // THEN it is created and initially empty
        assert_eq!(0, lib.iter().count());
    }
    #[test]
    fn library_add() {
        // GIVEN a new library
        let mut lib = Library::<i32>::new();
        // WHEN some items are inserted
        lib.add(102834);
        lib.add(-766);
        lib.add(0);
        // THEN the count is correct
        assert_eq!(3, lib.iter().count());
    }
    #[test]
    fn library_inspect_valid_items() {
        // GIVEN a new library
        let mut lib = Library::<i32>::new();
        // WHEN some items are inserted
        lib.add(102834);
        lib.add(-766);
        lib.add(0);
        // THEN the items can be inspected
        let mut it = lib.iter();
        assert_eq!(Some(102834), *lib.inspect(it.next().unwrap()));
        assert_eq!(Some(-766), *lib.inspect(it.next().unwrap()));
        assert_eq!(Some(0), *lib.inspect(it.next().unwrap()));
    }
    #[test]
    fn library_inspect_invalid_item() {
        // GIVEN a new library
        let mut lib = Library::<i32>::new();
        // WHEN an item is inserted
        lib.add(102834);
        // THEN inspecting a non-existent item returns None
        let mut it = lib.iter();
        assert_eq!(Some(102834), *lib.inspect(it.next().unwrap()));
        assert_eq!(None, it.next());
        assert_eq!(None, *lib.inspect(17));
    }
    #[test]
    fn library_checkout() {
        // GIVEN a library containing some items
        let mut lib = Library::<i32>::new();
        lib.add(102834);
        let id = lib.add(-766);
        lib.add(0);
        // WHEN an item is checked out
        let item = lib.checkout(id);
        // THEN the checked out item has the expected value, and inspecting or checking out that ID returns None
        assert_eq!(Some(-766), item);
        assert_eq!(None, *lib.inspect(id));
        assert_eq!(None, lib.checkout(id));
    }
    #[test]
    fn library_checkout_invalid() {
        // GIVEN a library containing some items
        let mut lib = Library::<i32>::new();
        lib.add(102834);
        lib.add(-766);
        lib.add(0);
        // WHEN an invalid item is checked out
        let item = lib.checkout(7);
        // THEN the checked out item is None
        assert_eq!(None, item);
    }
    #[test]
    fn library_checkin() {
        // GIVEN a library containing some items, with an item checked out
        let mut lib = Library::<i32>::new();
        lib.add(102834);
        lib.add(-766);
        let item = lib.checkout(0);
        // WHEN the item is checked back in
        assert!(item.is_some());
        let result = lib.checkin(0, item.unwrap());
        // THEN check-in succeeds and it is back in the expected location
        assert!(result.is_ok());
        assert_eq!(Some(102834), *lib.inspect(0));
    }
    #[test]
    fn library_audit_missing() {
        // GIVEN a library containing some items, with an item checked out
        let mut lib = Library::<i32>::new();
        lib.add(102834);
        lib.add(-766);
        let item = lib.checkout(0);
        // WHEN the library is audited
        assert!(item.is_some());
        let result = lib.audit();
        // THEN the audit fails
        assert!(result.is_err());
    }
    #[test]
    fn library_audit_all_present() {
        // GIVEN a library containing some items, with an item checked out
        let mut lib = Library::<i32>::new();
        lib.add(102834);
        lib.add(-766);
        let item = lib.checkout(0);
        // WHEN item is checked in and the library is audited
        assert!(item.is_some());
        let result = lib.checkin(0, item.unwrap());
        assert!(result.is_ok());
        let result = lib.audit();
        // THEN the audit succeeds
        assert!(result.is_ok());
    }
}
