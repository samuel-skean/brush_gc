use just_non_null::JustNonNull;

mod just_non_null;

pub enum Object {
    Integer(i64),
    Pair(*mut TrackedObject, *mut TrackedObject),
}

pub struct TrackedObject {
    marked: bool,
    object: Object,
    next: Option<JustNonNull<TrackedObject>>,
}

impl TrackedObject {
    pub fn new(head: &mut Option<JustNonNull<TrackedObject>>, object: Object) -> JustNonNull<TrackedObject> {
        let next = *head;
        *head = JustNonNull::new(Box::into_raw(Box::new(Self {
            marked: false,
            object,
            next,
        })));
        head.unwrap()
    }
}

unsafe fn mark_all(stack: &[*mut TrackedObject]) {
    unsafe fn mark(object: *mut TrackedObject) {
        if object.is_null() {
            return;
        }
        if (*object).marked {
            // We've already been here. If we tried marking it again, we'd be
            // stuck in a cycle.
            return;
        }
        (*object).marked = true;
        match (*object).object {
            Object::Integer(_) => {}
            Object::Pair(first, second) => {
                mark(first);
                mark(second);
            }
        }
    }
    for object in stack.iter().copied() {
        mark(object);
    }
}

unsafe fn sweep(head: JustNonNull<Option<JustNonNull<TrackedObject>>>) {
    let mut current = head;
    while let Some(object) = *current.as_ptr() {
        if !(*object.as_ptr()).marked {
            // SAFETY: No one else is looking at this Box because it's from
            // somewhere else. FIXME: What am I talking about?
            let unreached = Box::from_raw(object.as_ptr());
            *current.as_ptr() = unreached.next;
        } else {
            // Clear marking for remaining objects, so they must be re-marked
            // next collection.
            (*object.as_ptr()).marked = false;
            // SAFETY: I mean, we just created this pointer to a valid thing,
            // it's gonna be valid. This is just to avoid a temporary mutable
            // reference, and all the assertions that makes about memory.
            current = JustNonNull::new(&raw mut (*object.as_ptr()).next).unwrap_unchecked();
        }
    }
}

pub fn collect_garbage(stack: &[*mut TrackedObject], head: JustNonNull<Option<JustNonNull<TrackedObject>>>) {
    unsafe {
        mark_all(stack);
        sweep(head);
    }
}

#[cfg(test)]
mod tests {
    use super::{*, Object::*};

    // For now, everything not collected, I need to collect manually.

    #[test]
    fn int_basic() {
        let mut head = None;
        let _x = TrackedObject::new(&mut head, Integer(21));
        let y = TrackedObject::new(&mut head, Integer(42));
        let stack = vec![y.as_ptr()];
        collect_garbage(&stack, JustNonNull::from_mut(&mut head));
        unsafe { drop(Box::from_raw(y.as_ptr())); }
    }

    #[test]
    fn int_all_reachable() {
        let mut head = None;
        let x = TrackedObject::new(&mut head, Integer(21));
        let y = TrackedObject::new(&mut head, Integer(42));
        let stack = vec![x.as_ptr(), y.as_ptr()];
        collect_garbage(&stack, JustNonNull::from_mut(&mut head));
        unsafe { 
            drop(Box::from_raw(y.as_ptr()));
            drop(Box::from_raw(x.as_ptr()));
        }
    }

    #[test]
    fn pair_all_reachable() {
        let mut head = None;
        let x = TrackedObject::new(&mut head, Integer(21));
        let y = TrackedObject::new(&mut head, Integer(42));
        let pair = TrackedObject::new(&mut head, Pair(x.as_ptr(), y.as_ptr()));
        let stack = vec![pair.as_ptr(), y.as_ptr()];
        collect_garbage(&stack, JustNonNull::from_mut(&mut head));

        // Everything was reachable.
        unsafe {
            drop(Box::from_raw(x.as_ptr()));
            drop(Box::from_raw(y.as_ptr()));
            drop(Box::from_raw(pair.as_ptr()));
        }
    }

    #[test]
    fn chain_of_pairs_all_reachable() {
        let mut head = None;
        let x = TrackedObject::new(&mut head, Integer(23));
        let y = TrackedObject::new(&mut head, Integer(42));
        let p1 = TrackedObject::new(&mut head, Pair(x.as_ptr(), y.as_ptr()));
        let p2 = TrackedObject::new(&mut head, Pair(p1.as_ptr(), x.as_ptr()));
        let p3 = TrackedObject::new(&mut head, Pair(p2.as_ptr(), y.as_ptr()));
        let p4 = TrackedObject::new(&mut head, Pair(p3.as_ptr(), y.as_ptr()));

        let stack = vec![p4.as_ptr()];
        collect_garbage(&stack, JustNonNull::from_mut(&mut head));

        // Everything was reachable.
        unsafe {
            drop(Box::from_raw(x.as_ptr()));
            drop(Box::from_raw(y.as_ptr()));
            drop(Box::from_raw(p1.as_ptr()));
            drop(Box::from_raw(p2.as_ptr()));
            drop(Box::from_raw(p3.as_ptr()));
            drop(Box::from_raw(p4.as_ptr()));
        }
    }

    #[test]
    fn cycle_collect() {
        let mut head = None;
        let x = TrackedObject::new(&mut head, Integer(42));
        let y = TrackedObject::new(&mut head, Integer(21));
        let p1 = TrackedObject::new(&mut head, Pair(x.as_ptr(), y.as_ptr()));

        // It seems like the only way to make a cycle is to mutate the pair,
        // since we don't have the equivalent of OCaml's `let rec`.
        unsafe {
            let Pair(ref mut p1_first, _) = (*p1.as_ptr()).object else {
                unreachable!("The pair didn't exist.")
            };
            *p1_first = p1.as_ptr();
        }

        let stack = vec![x.as_ptr()];
        collect_garbage(&stack, JustNonNull::from_mut(&mut head));

        // Only the integer we could directly reach stayed alive.

        unsafe {
            drop(Box::from_raw(x.as_ptr()))
        }

    }

}
