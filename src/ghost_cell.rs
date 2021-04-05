//  The core of the library.
//
//  The code in this model was copied from http://plv.mpi-sws.org/rustbelt/ghostcell/.
//
//  If I have filled in the blanks incorrectly, the blame is mine.

use core::{
    cell::UnsafeCell,
    marker::PhantomData,
    mem,
};

/// A single "branded" permission to access the data structure.
/// Implemented with a phantom-lifetime marker type.
pub struct GhostToken<'id> { _marker: InvariantLifetime<'id> }

impl<'id> GhostToken<'id> {
    /// Creates a fresh token that GhostCells can be tied to later.
    pub fn new<R, F>(fun: F) -> R
    where
        for <'new_id> F: FnOnce(GhostToken<'new_id>) -> R
    {
        let token = Self { _marker: InvariantLifetime::default() };
        fun(token)
    }
}

/// Branded wrapper for the data structure's nodes, whose type is T.
pub struct GhostCell<'id, T: ?Sized> {
    _marker: InvariantLifetime<'id>,
    value: UnsafeCell<T>,
}

impl<'id, T> GhostCell<'id, T> {
    /// Wraps some data T into a GhostCell with brand`'id`.
    pub fn new(value: T) -> Self {
        let _marker = InvariantLifetime::default();
        let value = UnsafeCell::new(value);

        Self { _marker, value, }
    }

    /// Turns an owned GhostCell back into owned data.
    pub fn into_inner(self) -> T { self.value.into_inner() }

    /// Turns a mutably borrowed GhostCell to mutably borrowed data.
    pub fn get_mut(&mut self) -> &mut T { unsafe { mem::transmute(self) } }

    /// Turns mutably borrowed data to a mutably borrowed GhostCell.
    pub fn from_mut(t: &mut T) -> &mut Self { unsafe { mem::transmute(t) } }

    /// Immutably borrows the GhostCell with the same-branded token.
    pub fn borrow<'a>(&'a self, _: &'a GhostToken<'id>) -> &'a T {
        unsafe{ &*self.value.get() }
    }

    /// Mutably borrows the GhostCell with the same-branded token.
    pub fn borrow_mut<'a>(&'a self, _: &'a mut GhostToken<'id>) -> &'a mut T {
        unsafe{ &mut *self.value.get() }
    }
}

//
//  Implementation
//

type InvariantLifetime<'id> = PhantomData<fn(&'id ()) -> &'id ()>;
