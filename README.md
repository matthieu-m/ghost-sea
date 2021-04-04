A novel safe and zero-cost borrow-checking paradigm based on the
[`GhostCell`](http://plv.mpi-sws.org/rustbelt/ghostcell/) paper.

#   Motivating Example

A number of collections, such as linked-lists, binary-trees, or B-Trees are most easily implemented with aliasing
pointers.

Traditionally, this means using run-time borrow-checking in order to still be able to mutate said structures, or using
`unsafe` in the name of performance.

By using _brands_, `GhostCell` separate the data from the permission to mutate it, and uses a unique `GhostToken` to
model this permission, tied at compile-time to a number of said `GhostCell` via the _brand_.

Unfortunately, whilst theoretically sound, the pattern requires a very restricting programming style.


#   Could it be more ergonomic?

This is the idea behind `GhostSea`:

-   Wrap an _un-branded_ version of the type.
-   When the user wishes to access the data:
    -   Create a `GhostToken` on the fly.
    -   _Brand_ the un-branded type to match, via a simple (if `unsafe`) projection trait.
    -   Call a user-supplied action with _newly-branded_ type and matching `GhostToken`.

This works to a degree: in the `examples/linked_list` folder one can see a `GhostLinkedList<'brand, T>` used as the
basis for a `LinkedList<T>`. A single of `unsafe` code to implement `GhostProject` and here you go.

There's a **catch**, though: references to the insides of `GhostLinkedList` (nodes, or elements) are prevented by the
borrow-checker from leaking through the interface of `LinkedList` -- because the borrow is bounded to the lifetime of
the token, which is ephemeral.


#   That's all folks!

And thanks for reading.
