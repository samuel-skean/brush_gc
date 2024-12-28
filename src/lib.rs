use just_non_null::JustNonNull;

mod just_non_null;

enum Object {
    Integer(i64),
    Pair(*mut TrackedObject, *mut TrackedObject),
}

pub struct TrackedObject {
    marked: bool,
    object: Object,
    next: Option<JustNonNull<TrackedObject>>,
}

impl TrackedObject {
    fn new(head: &mut Option<JustNonNull<TrackedObject>>, object: Object) -> JustNonNull<TrackedObject> {
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

unsafe fn sweep(head: *mut Option<JustNonNull<TrackedObject>>) {
    let mut current = head;
    while let Some(object) = *current {
        let boxed_object;
        if !(*object.as_ptr()).marked {
            // SAFETY: No one else is looking at this Box because it's from
            // somewhere else.
            boxed_object = Box::from_raw(object.as_ptr());
            *current = boxed_object.next;
        } else {
            // Clear marking for remaining objects.
            (*object.as_ptr()).marked = false;
        }
        // There's something wrong with this in the case of skipping one of the
        // objects. I think it's not as uniform of a thing as I might want.
        current = &raw mut (*object.as_ptr()).next;
    }
}

pub fn collect_garbage(stack: &Vec<*mut TrackedObject>, head: *mut Option<JustNonNull<TrackedObject>>) {
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
        collect_garbage(&stack, &raw mut head);
        unsafe { drop(Box::from_raw(y.as_ptr())); }
    }
}
