//  Sea comes the idea of "Sea of Nodes", which is a tenuous idea, for sure, but as Ghost Sea evokes pirates and
//  adventures I'll cling to it!

use core::mem::{self, MaybeUninit};

use super::GhostToken;

/// Projects a non-branded type as a branded type.
///
/// #   Safety
///
/// The default implementation of the method is going to be a `mem::transmute`, the safety of which hinges on:
///
/// -   The fact that `Self::Result` should be `Self` which just a few lifetimes switched to `'id`, and therefore have
///     the same layout. Fingers crossed.
/// -   The implementer should _really_ take care that only the brands place-holders switch to `'id`.
/// -   The implementer promises that the type does not already carry a `GhostToken`.
///
/// #   Examples
///
/// The typical example is to use on a `GhostCell`:
///
/// ```
/// use crate::ghost_sea::{GhostCell, GhostProject};
///
/// struct Wrapper<'brand>(GhostCell<'brand, String>);
///
/// unsafe impl<'id> GhostProject<'id> for Wrapper<'static> {
///     type Branded = Wrapper<'id>;
/// }
/// ```
///
/// Since the type embedded within the `GhostCell` may itself refer to the `'brand`, it may be required to apply the
/// change in more depth.
///
/// ```
/// use core::ptr::NonNull;
/// use crate::ghost_sea::{GhostCell, GhostProject};
///
/// struct Node<'brand>(String, Option<NodePtr<'brand>>, Option<NodePtr<'brand>>);
///
/// type GhostNode<'brand> = GhostCell<'brand, Node<'brand>>;
/// type NodePtr<'brand> = NonNull<GhostNode<'brand>>;
///
/// unsafe impl<'id> GhostProject<'id> for Node<'static> {
///     type Branded = Node<'id>;
/// }
/// ```
pub unsafe trait GhostProject<'id> : Sized {
    /// The result of the various projections.
    ///
    /// 
    type Branded;

    /// Projects `self` as a branded `Branded`.
    ///
    /// #   Safety.
    ///
    /// Even assuming that `project` is correctly implemented, there are still multiple reasons for this to go wrong:
    ///
    /// -   Since `self` is potentially aliased, it should only be tied to an `'id` brand carried by a
    ///     `&GhostToken<'id>` to prevent mutation.
    #[inline(always)]
    unsafe fn project(&self) -> &Self::Branded { mem::transmute(self) }

    /// Projects `self` as a branded `Branded`.
    #[inline(always)]
    fn project_mut(&mut self) -> &mut Self::Branded {
        //  Safety:
        //  -   `self` is borrowed mutably for the duration.
        unsafe { mem::transmute(self) }
    }

    /// Projects `self` as a branded `Branded`.
    #[inline(always)]
    fn project_once(self) -> Self::Branded {
        assert_eq!(mem::size_of::<Self>(), mem::size_of::<Self::Branded>());

        let result = unsafe { mem::transmute_copy(&self) };
        mem::forget(self);

        result
    }
}

unsafe impl<'a, 'id> GhostProject<'id> for GhostToken<'static> {
    type Branded = GhostToken<'id>;
}

/// Trait family to describe the genericity over brands of the return types of the closures passed to the various
/// GhostSea callbacks.
pub trait GhostResult<'a> {
    /// Lifetime-parameterized output of the callback.
    type Output;
}

/// Ergonomic wrapper around the usage of `GhostCell` and `GhostToken`.
pub struct GhostSea<T> {
    token: GhostToken<'static>,
    value: T,
}

impl<T> GhostSea<T> {
    /// Creates a new instance.
    #[inline(always)]
    pub fn new(value: T) -> Self {
        //  Safety:
        //
        //  Creating our own token is _super_ sketchy, yet, I _think_ that given out use of it, it is actually okay
        //  in this case.
        //
        //  The key issue to look out for is to end in a situation where a GhostCell can be unlocked by two different
        //  tokens, simultaneously, as then it becomes impossible to enforce the Aliasing XOR Mutation property. This
        //  issue may manifest as either:
        //
        //  -   Having 2 GhostCell<'static, _> which can be re-branded separately.
        //  -   Having 2 GhostTokens generated with the same lifetime.
        //
        //  Those issues are tackled in 2 different ways:
        //
        //  -   If `apply` or `combine` return a `R` containing a `GhostCell<'static, _>`, then it was not
        //      simultaneously a `GhostCell<'id, _>` and therefore there is no dual access.
        //  -   `combine` consumes the other `GhostSea`.
        //  -   The callback in `apply` and `combine` must work with arbitrary `'id` -- even if in practice it is only
        //      ever used with a specific one -- and therefore cannot containing a matching `GhostToken` nor smuggle
        //      out specific references.
        let token = unsafe { Self::token() };

        Self { token, value, }
    }

    /// Returns the value contained within.
    #[inline(always)]
    pub fn into_inner(self) -> T { self.value }

    //  Returns a generated token.
    //
    //  #   Safety
    //
    //  -   Caller should ensure proper usage of the token.
    unsafe fn token() -> GhostToken<'static> {
        assert_eq!(0, mem::size_of::<GhostToken<'static>>());

        //  Safety:
        //  -   The token is stateless, hence it should be possible to create it out of thin air.
        MaybeUninit::uninit().assume_init()
    }
}

impl<T> GhostSea<T>
where
    for<'id> T: GhostProject<'id>,
{
    /// Apply the provided function, and return its result.
    #[inline(always)]
    pub fn apply_ref<'a, R, F>(&'a self, fun: F) -> <R as GhostResult<'a>>::Output
    where
        R: for<'id> GhostResult<'id>,
        F: for<'id> FnOnce(&'id <T as GhostProject<'id>>::Branded, &'id GhostToken<'id>) -> <R as GhostResult<'id>>::Output,
    {
        //  Safety:
        //  -   Pair &T with &GhostToken, so read-only.
        let token: &'a GhostToken<'a> = unsafe { self.token.project() };
        let value: &'a <T as GhostProject<'a>>::Branded = unsafe { self.value.project() };

        fun(value, token)
    }

    /// Apply the provided function, and return its result.
    #[inline(always)]
    pub fn apply_ref_opt_ref<'a, R, F>(&'a self, fun: F) -> Option<&'a R>
    where
        for<'id> F: FnOnce(&'id <T as GhostProject<'id>>::Branded, &'id GhostToken<'id>) -> Option<&'id R>,
    {
        //  FIXME: Remove this oddly specific method, and replace uses by `apply_ref`.
        //
        //  Unfortunately, this will require understanding why `LinkedList::front` fails with:
        //
        //  error: implementation of `FnOnce` is not general enough
        //  --> examples/linked_list/linked_list.rs:33:48
        //      |
        //  33  |       pub fn front(&self) -> Option<&T> { self.0.apply_ref(|ghost, token| ghost.front(token)) }
        //      |                                                  ^^^^^^^^^ implementation of `FnOnce` is not general enough
        //      |
        //      = note: `FnOnce<(&'0 <GhostImpl<'static, T> as GhostProject<'0>>::Branded, &'0 GhostToken<'0>)>`
        //              would have to be implemented for the type
        //              `for<'a> fn(&'a <GhostImpl<'static, T> as GhostProject<'a>>::Branded, &'a GhostToken<'a>) -> Option<&'a T>
        //              {linked_list::LinkedList::<T>::front_impl}`, for some specific lifetime `'0`...
        //
        //      = note: ...but `FnOnce<(&'id <GhostImpl<'static, T> as GhostProject<'id>>::Branded, &'id GhostToken<'id>)>`
        //              is actually implemented for the type
        //              `for<'a> fn(&'a <GhostImpl<'static, T> as GhostProject<'a>>::Branded, &'a GhostToken<'a>) -> Option<&'a T>
        //              {linked_list::LinkedList::<T>::front_impl}`

        //  Safety:
        //  -   Pair &T with &GhostToken, so read-only.
        let token: &'a GhostToken<'a> = unsafe { self.token.project() };
        let value: &'a <T as GhostProject<'a>>::Branded = unsafe { self.value.project() };

        fun(value, token)
    }

    /// Apply the provided function, and return its result.
    #[inline(always)]
    pub fn apply_mut<'a, R, F>(&'a mut self, fun: F) -> R
    where
        for<'id> F: FnOnce(&'id mut <T as GhostProject<'id>>::Branded, &'id mut GhostToken<'id>) -> R,
    {
        let token: &'a mut GhostToken<'a> = self.token.project_mut();
        let value: &'a mut <T as GhostProject<'a>>::Branded = self.value.project_mut();

        fun(value, token)
    }

    /// Apply the provided function, and return its result.
    #[inline(always)]
    pub fn apply_mut_opt_mut<'a, R, F>(&'a mut self, fun: F) -> Option<&'a mut R>
    where
        for<'id> F: FnOnce(&'id mut <T as GhostProject<'id>>::Branded, &'id mut GhostToken<'id>) -> Option<&'id mut R>,
    {
        //  FIXME: Remove this oddly specific method, and replace uses by `apply_ref`.
        //
        //  Unfortunately, this will require understanding why `LinkedList::front` fails with:
        //
        //  error: implementation of `FnOnce` is not general enough
        //  --> examples/linked_list/linked_list.rs:33:48
        //      |
        //  33  |       pub fn front(&self) -> Option<&T> { self.0.apply_ref(|ghost, token| ghost.front(token)) }
        //      |                                                  ^^^^^^^^^ implementation of `FnOnce` is not general enough
        //      |
        //      = note: `FnOnce<(&'0 <GhostImpl<'static, T> as GhostProject<'0>>::Branded, &'0 GhostToken<'0>)>`
        //              would have to be implemented for the type
        //              `for<'a> fn(&'a <GhostImpl<'static, T> as GhostProject<'a>>::Branded, &'a GhostToken<'a>) -> Option<&'a T>
        //              {linked_list::LinkedList::<T>::front_impl}`, for some specific lifetime `'0`...
        //
        //      = note: ...but `FnOnce<(&'id <GhostImpl<'static, T> as GhostProject<'id>>::Branded, &'id GhostToken<'id>)>`
        //              is actually implemented for the type
        //              `for<'a> fn(&'a <GhostImpl<'static, T> as GhostProject<'a>>::Branded, &'a GhostToken<'a>) -> Option<&'a T>
        //              {linked_list::LinkedList::<T>::front_impl}`

        let token = self.token.project_mut();
        let value = self.value.project_mut();

        fun(value, token)
    }

    /// Apply the provided function, and return its result.
    #[inline(always)]
    pub fn apply_once<R, F>(self, fun: F) -> R
    where
        for<'id> F: FnOnce(<T as GhostProject<'id>>::Branded, GhostToken<'id>) -> R,
    {
        let token = self.token.project_once();
        let value = self.value.project_once();

        fun(value, token)
    }

    /// Apply the provided function, and return its result.
    #[inline(always)]
    pub fn combine_ref<R, O, F>(&self, other: GhostSea<O>, fun: F) -> R
    where
        for<'id> O: GhostProject<'id>,
        for<'id> F: FnOnce(&'id <T as GhostProject<'id>>::Branded, <O as GhostProject<'id>>::Branded, &'id GhostToken<'id>) -> R,
    {
        //  Safety:
        //  -   Pair &T with &GhostToken, so read-only.
        let token = unsafe { self.token.project() };
        let value = unsafe { self.value.project() };

        let other = other.value.project_once();

        fun(value, other, token)
    }

    /// Apply the provided function, and return its result.
    #[inline(always)]
    pub fn combine_mut<R, O, F>(&mut self, other: GhostSea<O>, fun: F) -> R
    where
        for<'id> O: GhostProject<'id>,
        for<'id> F: FnOnce(&'id mut <T as GhostProject<'id>>::Branded, <O as GhostProject<'id>>::Branded, &'id mut GhostToken<'id>) -> R,
    {
        let token = self.token.project_mut();
        let value = self.value.project_mut();

        let other = other.value.project_once();

        fun(value, other, token)
    }

    /// Apply the provided function, and return its result.
    #[inline(always)]
    pub fn combine_once<R, O, F>(self, other: GhostSea<O>, fun: F) -> R
    where
        for<'id> O: GhostProject<'id>,
        for<'id> F: FnOnce(<T as GhostProject<'id>>::Branded, <O as GhostProject<'id>>::Branded, GhostToken<'id>) -> R,
    {
        let token = self.token.project_once();
        let value = self.value.project_once();

        let other = other.value.project_once();

        fun(value, other, token)
    }
}

impl<T: Default> Default for GhostSea<T> {
    fn default() -> Self { Self::new(T::default()) }
}

/*
error: internal compiler error: compiler/rustc_trait_selection/src/traits/codegen.rs:78:17:

encountered error
    `OutputTypeParameterMismatch(
        Binder(
            <
                [closure@examples/linked_list/linked_list.rs:15:51: 15:78]
                as
                std::ops::FnOnce<(
                    &<ghost_linked_list::GhostLinkedList<std::string::String> as ghost_sea::GhostProject<'_>>::Branded,
                    &ghost_sea::GhostToken<'_>
                )>
            >
        ),
        Binder(
            <
                [closure@examples/linked_list/linked_list.rs:15:51: 15:78]
                as
                std::ops::FnOnce<(
                    &ghost_linked_list::GhostLinkedList<std::string::String>,
                    &ghost_sea::GhostToken
                )>
            >
        ),
        Sorts(
            ExpectedFound {
                    expected: ghost_linked_list::GhostLinkedList<std::string::String>,
                    found: <ghost_linked_list::GhostLinkedList<std::string::String> as ghost_sea::GhostProject<'_>>::Branded
            }
        )
    )`
selecting
    `Binder(
        <
            [closure@examples/linked_list/linked_list.rs:15:51: 15:78]
            as
            std::ops::FnOnce<(&ghost_linked_list::GhostLinkedList<std::string::String>, &ghost_sea::GhostToken)>
        >
    )`
during codegen
*/
