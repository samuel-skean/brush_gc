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

unsafe fn mark_all(stack: &Vec<*mut TrackedObject>) {
    unsafe fn mark(object: *mut TrackedObject) {
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
    for object in stack {
        mark(*object);
    }
}

unsafe fn sweep(head: JustNonNull<Option<JustNonNull<TrackedObject>>>) {
    let mut current = head;
    while let Some(object) = *current.as_ptr() {
        if !(*object.as_ptr()).marked {
            // SAFETY: No one else is looking at this Box because it's from
            // somewhere else. FIXME: What am I talking about?
            let boxed_object = Box::from_raw(object.as_ptr());
            *current.as_ptr() = boxed_object.next;
        } else {
            // Clear marking for remaining objects, so they must be re-marked next time.
            (*object.as_ptr()).marked = false;
        }
        // FIXME: Audit this, I don't like making this temporary mutable
        // reference! Maybe replace with is_some?
        if let Some(ref mut new_current) = *current.as_ptr() {
            // SAFETY: Depends on JustNonNull taking advantage of the Null-Pointer
            // Optimization, where 0 is equivalent to null.
            current = JustNonNull::from_mut(std::mem::transmute(new_current));
        }
    }
}

pub fn collect_garbage(stack: &Vec<*mut TrackedObject>, head: JustNonNull<Option<JustNonNull<TrackedObject>>>) {
    unsafe {
        mark_all(stack);
        sweep(head);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let mut head = None;
        let _x = TrackedObject::new(&mut head, Object::Integer(21));
        let y = TrackedObject::new(&mut head, Object::Integer(42));
        let stack = vec![y.as_ptr()];
        collect_garbage(&stack, JustNonNull::from_mut(&mut head));
        unsafe { drop(Box::from_raw(y.as_ptr())); }
    }
}
