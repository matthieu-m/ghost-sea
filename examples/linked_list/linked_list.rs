//  A linked-list implemented in entirely safe code.

#![macro_use]

use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use ghost_cell::GhostToken;
use ghost_sea::{GhostProject, GhostSea};
use ghost_sea::{GhostApplyMut, GhostApplyRef};

use super::GhostLinkedList;

/// A typically self-sufficient linked list, written in safe code.
pub struct LinkedList<T>(GhostSea<GhostImpl<'static, T>>);

impl<T> LinkedList<T> {
    /// Creates an empty instance.
    pub fn new() -> Self { Self(GhostSea::default()) }

    /// Returns whether the list is empty.
    pub fn is_empty(&self) -> bool { self.0.apply_ref(RetValueRef::new(|ghost, _| ghost.is_empty())) }

    /// Returns the length of the list.
    ///
    /// #   Complexity
    ///
    /// O(N)
    pub fn len(&self) -> usize { self.0.apply_ref(RetValueRef::new(|ghost, token| ghost.len(token))) }

    /// Clears the list.
    pub fn clear(&mut self) { self.0.apply_mut(RetValueMut::new(|ghost, token| ghost.clear(token))) }
    
    /// Pushes an item at the front of the list.
    pub fn push_front(&mut self, value: T) { self.0.apply_mut(RetValueMut::new(|ghost, token| ghost.push_front(value, token))) }

    /// Pushes an item at the back of the list.
    pub fn push_back(&mut self, value: T) { self.0.apply_mut(RetValueMut::new(|ghost, token| ghost.push_back(value, token))) }
}

impl<T: 'static> LinkedList<T> {
    /// Returns the front item, if any.
    pub fn front(&self) -> Option<&T> { self.0.apply_ref(RetOptionalRef::new(|ghost, token| ghost.front(token))) }

    /// Returns the front item, if any.
    pub fn front_mut(&mut self) -> Option<&mut T> { self.0.apply_mut(RetOptionalMut::new(|ghost, token| ghost.front_mut(token))) }

    /// Returns the back item, if any.
    pub fn back(&self) -> Option<&T> { self.0.apply_ref(RetOptionalRef::new(|ghost, token| ghost.back(token))) }

    /// Returns the back item, if any.
    pub fn back_mut(&mut self) -> Option<&mut T> { self.0.apply_mut(RetOptionalMut::new(|ghost, token| ghost.back_mut(token))) }

    /// Pops the front item of the list, if any.
    pub fn pop_front(&mut self) -> Option<T> { self.0.apply_mut(RetValueMut::new(|ghost, token| ghost.pop_front(token))) }

    /// Pops the back item of the list, if any.
    pub fn pop_back(&mut self) -> Option<T> { self.0.apply_mut(RetValueMut::new(|ghost, token| ghost.pop_back(token))) }
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

macro_rules! call_forwarder {
    (apply_ref, $forwarder:ident, $result:ident, $target:lifetime, $target_output:ty, $brand:lifetime, $brand_output:ty) => {
        struct $forwarder<F, $result, T>(F, PhantomData<*const $result>, PhantomData<*const T>);

        impl<F, $result, T> $forwarder<F, $result, T>
        where
            F: for<$brand> FnOnce(&$brand GhostImpl<$brand, T>, &$brand GhostToken<$brand>) -> $brand_output,
        {
            fn new(fun: F) -> Self { Self(fun, PhantomData, PhantomData) }
        }

        impl<$target, F, $result, T> GhostApplyRef<$target, GhostImpl<'static, T>> for $forwarder<F, $result, T>
        where
            R: $target,
            F: for<$brand> FnOnce(&$brand GhostImpl<$brand, T>, &$brand GhostToken<$brand>) -> $brand_output,
        {
            type Output = $target_output;
        
            fn call(self, ghost: &$target GhostImpl<$target, T>, token: &$target GhostToken<$target>) -> Self::Output {
                (self.0)(ghost, token)
            }
        }
    };
    (apply_mut, $forwarder:ident, $result:ident, $target:lifetime, $target_output:ty, $brand:lifetime, $brand_output:ty) => {
        struct $forwarder<F, $result, T>(F, PhantomData<*const $result>, PhantomData<*const T>);

        impl<F, $result, T> $forwarder<F, $result, T>
        where
            F: for<$brand> FnOnce(&$brand mut GhostImpl<$brand, T>, &$brand mut GhostToken<$brand>) -> $brand_output,
        {
            fn new(fun: F) -> Self { Self(fun, PhantomData, PhantomData) }
        }

        impl<$target, F, $result, T> GhostApplyMut<$target, GhostImpl<'static, T>> for $forwarder<F, $result, T>
        where
            R: $target,
            F: for<$brand> FnOnce(&$brand mut GhostImpl<$brand, T>, &$brand mut GhostToken<$brand>) -> $brand_output,
        {
            type Output = $target_output;
        
            fn call(self, ghost: &$target mut GhostImpl<$target, T>, token: &$target mut GhostToken<$target>) -> Self::Output {
                (self.0)(ghost, token)
            }
        }
    };
}

call_forwarder!(apply_ref, RetValueRef, R, 'id, R, 'x, R);
call_forwarder!(apply_mut, RetValueMut, R, 'id, R, 'x, R);

call_forwarder!(apply_ref, RetOptionalRef, R, 'id, Option<&'id R>, 'x, Option<&'x R>);
call_forwarder!(apply_mut, RetOptionalMut, R, 'id, Option<&'id mut R>, 'x, Option<&'x mut R>);
