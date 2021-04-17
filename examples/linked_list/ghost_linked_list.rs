//  A GhostLinkedList, with externally supplied token.
//
//  A number of operations normally implemented by traits cannot be successfully implemented on this collection due to
//  the requirement of supplying the GhostToken externally.
//
//  While this may seem like a harsh requirement, it provides some flexibility; see the top-level LinkedList for a
//  stand-alone version.
//
//  #   Safety
//
//  There's a single line of `unsafe` code: the implementation of `GhostProject`.

use crate::ghost_cell::{GhostCell, GhostToken};
use crate::static_rc::StaticRc;

/// A safe implementation of a linked-list build upon `GhostCell` and `StaticRc`.
///
/// The future is now!
pub struct GhostLinkedList<'brand, T> {
    head_tail: Option<(HalfNodePtr<'brand, T>, HalfNodePtr<'brand, T>)>,
}

impl<'brand, T> GhostLinkedList<'brand, T> {
    /// Creates an instance.
    pub fn new() -> Self { Self { head_tail: None } }

    /// Creates an iterator over self.
    pub fn iter<'a>(&'a self, token: &'a GhostToken<'brand>) -> GhostLinkedListIterator<'a, 'brand, T> {
        let head_tail = self.head_tail.as_ref().map(|head_tail| {
            (&*head_tail.0, &*head_tail.1)
        });

        GhostLinkedListIterator { token, head_tail, }
    }

    /// Returns whether the list is empty, or not.
    pub fn is_empty(&self) -> bool { self.head_tail.is_none() }

    pub fn len(&self, token: &GhostToken<'brand>) -> usize { self.iter(token).count() }

    pub fn clear(&mut self, token: &mut GhostToken<'brand>) {
        while let Some(_) = self.pop_back(token) {}
    }

    pub fn front<'a>(&'a self, token: &'a GhostToken<'brand>) -> Option<&'a T> {
        self.head_tail.as_ref().map(|(head, _)| {
            &head.borrow(token).data
        })
    }

    pub fn front_mut<'a>(&'a mut self, token: &'a mut GhostToken<'brand>) -> Option<&'a mut T> {
        self.head_tail.as_mut().map(move |(head, _)| {
            &mut head.borrow_mut(token).data
        })
    }

    pub fn back<'a>(&'a self, token: &'a GhostToken<'brand>) -> Option<&'a T> {
        self.head_tail.as_ref().map(|(_, tail)| {
            &tail.borrow(token).data
        })
    }

    pub fn back_mut<'a>(&'a mut self, token: &'a mut GhostToken<'brand>) -> Option<&'a mut T> {
        self.head_tail.as_mut().map(move |(_, tail)| {
            &mut tail.borrow_mut(token).data
        })
    }

    pub fn push_front(&mut self, data: T, token: &mut GhostToken<'brand>) {
        let (one, two) = Self::new_halves(data);

        let head_tail = if let Some((head, tail)) = self.head_tail.take() {
            head.borrow_mut(token).prev = Some(one);

            two.borrow_mut(token).next = Some(head);

            (two, tail)
        } else {
            (one, two)
        };

        self.head_tail = Some(head_tail);
    }

    pub fn pop_front(&mut self, token: &mut GhostToken<'brand>) -> Option<T> {
        let (head, tail) = self.head_tail.take()?;

        if StaticRc::as_ptr(&head) == StaticRc::as_ptr(&tail) {
            return Some(Self::into_inner(head, tail));
        }

        let next = head.borrow_mut(token).next.take()
            .expect("Non-tail should have a next node");
        let other_head = next.borrow_mut(token).prev.take()
            .expect("Non-head should have a previous node");

        self.head_tail = Some((next, tail));

        Some(Self::into_inner(head, other_head))
    }

    pub fn push_back(&mut self, data: T, token: &mut GhostToken<'brand>) {
        let (one, two) = Self::new_halves(data);

        let head_tail = if let Some((head, tail)) = self.head_tail.take() {
            tail.borrow_mut(token).next = Some(one);

            two.borrow_mut(token).prev = Some(tail);

            (head, two)
        } else {
            (one, two)
        };

        self.head_tail = Some(head_tail);
    }

    pub fn pop_back(&mut self, token: &mut GhostToken<'brand>) -> Option<T> {
        let (head, tail) = self.head_tail.take()?;

        if StaticRc::as_ptr(&head) == StaticRc::as_ptr(&tail) {
            return Some(Self::into_inner(head, tail));
        }

        let prev = tail.borrow_mut(token).prev.take()
            .expect("Non-head should have a previous node");
        let other_tail = prev.borrow_mut(token).next.take()
            .expect("Non-tail should have a next node");

        self.head_tail = Some((head, prev));

        Some(Self::into_inner(tail, other_tail))
    }

    fn new_halves(data: T) -> (HalfNodePtr<'brand, T>, HalfNodePtr<'brand, T>) {
        let node = Node { data, prev: None, next: None, };
        let full = FullNodePtr::new(GhostNode::new(node));

        StaticRc::split::<1, 1>(full)
    }

    fn into_inner(left: HalfNodePtr<'brand, T>, right: HalfNodePtr<'brand, T>) -> T {
        let full = FullNodePtr::join(left, right);
        let ghost_cell = FullNodePtr::into_inner(full);
        let node = GhostNode::into_inner(ghost_cell);

        //  If the node still has a prev and next, they are leaked.
        debug_assert!(node.prev.is_none());
        debug_assert!(node.next.is_none());

        node.data
    }
}

impl<'brand, T> Default for GhostLinkedList<'brand, T> {
    fn default() -> Self { Self::new() }
}

/// An iterator over a GhostLinkedList, self-sufficient once created as it carries its own token.
pub struct GhostLinkedListIterator<'a, 'brand, T> {
    token: &'a GhostToken<'brand>,
    head_tail: Option<(&'a GhostNode<'brand, T>, &'a GhostNode<'brand, T>)>,
}

impl<'a, 'id, T> Iterator for GhostLinkedListIterator<'a, 'id, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((head, tail)) = self.head_tail.take() {
            let node = head.borrow(self.token);
            self.head_tail = node.next.as_ref().map(|n| {
                let n: &'a GhostNode<'_, _> = &*n;
                (n, tail)
            });
            Some(&node.data)
        } else {
            None
        }
    }
}

impl<'a, 'id, T> DoubleEndedIterator for GhostLinkedListIterator<'a, 'id, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if let Some((head, tail)) = self.head_tail.take() {
            let node = tail.borrow(self.token);
            self.head_tail = node.prev.as_ref().map(|n| {
                let n: &'a GhostNode<'_, _> = &*n;
                (head, n)
            });
            Some(&node.data)
        } else {
            None
        }
    }
}

//
//  Implementation
//

struct Node<'brand, T> {
    data: T,
    prev: Option<HalfNodePtr<'brand, T>>,
    next: Option<HalfNodePtr<'brand, T>>,
}

type GhostNode<'brand, T> = GhostCell<'brand, Node<'brand, T>>;
type HalfNodePtr<'brand, T> = StaticRc<GhostNode<'brand, T>, 1, 2>;
type FullNodePtr<'brand, T> = StaticRc<GhostNode<'brand, T>, 2, 2>;
