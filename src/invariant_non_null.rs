// This maybe belongs in a separate crate.
use std::{marker::PhantomData, ops::{Deref, DerefMut}, ptr::NonNull};

/// NonNull but invariant.
#[repr(transparent)]
pub struct InvariantNonNull<T> {
    inner: NonNull<T>,
    _marker: PhantomData<*mut T>,
}

// Copy and Clone derive macros didn't work, presumably because they're
// conditional on T being Clone and Copy. It's kinda surprising that this just works, though.
impl<T> Clone for InvariantNonNull<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for InvariantNonNull<T> { }

impl<T> InvariantNonNull<T> {
    pub fn new(ptr: *mut T) -> Option<Self> {
        NonNull::new(ptr).map(|non_null| {
            Self {
                inner: non_null,
                _marker: PhantomData,
            }
        })
    }

    pub fn from_mut(r: &mut T) -> Self {
        Self { inner: NonNull::new(r).expect("References should never be null"), _marker: PhantomData }
    }
}

impl<T> Deref for InvariantNonNull<T> {
    type Target = NonNull<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for InvariantNonNull<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

// NOTE: The below function should fail to compile, because InvariantNonNull
// should be invariant.
//
// I would make this a compile-fail doctest (or some other sort of thing), but
// since this type is not exported, it wouldn't work. Doctests are separate
// crates. I'll just have to remember to uncomment this every now and then and
// make sure it fails.
// fn covariant<'a, T>(x: InvariantNonNull<&'static T>) -> InvariantNonNull<&'a T> { x }