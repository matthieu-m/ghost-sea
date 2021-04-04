//  A linked-list implemented in entirely safe code.

#![macro_use]

use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use ghost_sea::{GhostProject, GhostResult, GhostSea, GhostToken};

use super::GhostLinkedList;

/// A typically self-sufficient linked list, written in safe code.
pub struct LinkedList<T>(GhostSea<GhostImpl<'static, T>>);

impl<T> LinkedList<T> {
    /// Creates an empty instance.
    pub fn new() -> Self { Self(GhostSea::default()) }

    /// Returns whether the list is empty.
    pub fn is_empty(&self) -> bool { self.apply_ref_ret_value::<bool, _>(|ghost, _| ghost.is_empty()) }

    /// Returns the length of the list.
    ///
    /// #   Complexity
    ///
    /// O(N)
    //pub fn len(&self) -> usize { self.0.apply_ref::<Value::<usize>, _>(|ghost, token| ghost.len(token)) }

    /// Clears the list.
    pub fn clear(&mut self) { self.0.apply_mut(|ghost, token| ghost.clear(token)) }

    /// Returns the front item, if any.
    pub fn front(&self) -> Option<&T> { self.0.apply_ref_opt_ref(|ghost, token| ghost.front(token)) }

    /// Returns the front item, if any.
    pub fn front_mut(&mut self) -> Option<&mut T> { self.0.apply_mut_opt_mut(|ghost, token| ghost.front_mut(token)) }

    /// Returns the back item, if any.
    pub fn back(&self) -> Option<&T> { self.0.apply_ref_opt_ref(|ghost, token| ghost.back(token)) }

    /// Returns the back item, if any.
    pub fn back_mut(&mut self) -> Option<&mut T> { self.0.apply_mut_opt_mut(|ghost, token| ghost.back_mut(token)) }

    /// Pushes an item at the front of the list.
    pub fn push_front(&mut self, value: T) { self.0.apply_mut(|ghost, token| ghost.push_front(value, token)) }

    /// Pops the front item of the list, if any.
    pub fn pop_front(&mut self) -> Option<T> { self.0.apply_mut(|ghost, token| ghost.pop_front(token)) }

    /// Pushes an item at the back of the list.
    pub fn push_back(&mut self, value: T) { self.0.apply_mut(|ghost, token| ghost.push_back(value, token)) }

    /// Pops the back item of the list, if any.
    pub fn pop_back(&mut self) -> Option<T> { self.0.apply_mut(|ghost, token| ghost.pop_back(token)) }

    //  Implementation.
    fn apply_ref_ret_value<R, F>(&self, fun: F) -> R
    where
        Value<R>: for<'a> GhostResult<'a, Output = R>,
        F: for<'a> FnOnce(&'a <GhostImpl<'static, T> as GhostProject<'a>>::Branded, &'a GhostToken<'a>) -> R,
    {
        let result: <Value<R> as GhostResult<'_>>::Output = self.0.apply_ref::<Value<R>, _>(fun);

        result
    }
}

impl<T> Default for LinkedList<T> {
    fn default() -> Self { Self::new() }
}

//
//  Implementation
//

//  Wrapper to implement GhostProject.
struct GhostImpl<'brand, T>(GhostLinkedList<'brand, T>);

//  Safety:
//  -   `'static` is the brand, and only the brand.
unsafe impl<'id, T> GhostProject<'id> for GhostImpl<'static, T> {
    type Branded = GhostImpl<'id, T>;
}

impl<'brand, T> Default for GhostImpl<'brand, T> {
    fn default() -> Self { Self(Default::default()) }
}

impl<'brand, T> Deref for GhostImpl<'brand, T> {
    type Target = GhostLinkedList<'brand, T>;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl<'brand, T> DerefMut for GhostImpl<'brand, T> {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

//  Wrapper to return plain values.
struct Value<T>(PhantomData<*const T>);

impl<'a, T> GhostResult<'a> for Value<T> {
    type Output = T;
}
