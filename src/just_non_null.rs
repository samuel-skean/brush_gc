// This maybe belongs in a separate crate.
use std::{marker::PhantomData, ops::{Deref, DerefMut}, ptr::NonNull};

/// NonNull but invariant.
#[repr(transparent)]
pub struct JustNonNull<T> {
    inner: NonNull<T>,
    _marker: PhantomData<*mut T>,
}

// Copy and Clone derive macros didn't work, presumably because they're
// conditional on T being Clone and Copy. It's kinda surprising that this just works, though.
impl<T> Clone for JustNonNull<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for JustNonNull<T> { }

impl<T> JustNonNull<T> {
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

impl<T> Deref for JustNonNull<T> {
    type Target = NonNull<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for JustNonNull<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}