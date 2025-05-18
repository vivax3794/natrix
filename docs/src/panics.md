# Panic Policy

The framework only makes use of `debug_assert!`, its our goal that any issues should be highlighted in debug builds, but not in release builds. On release builds natrix will silently fail in many cases, this is to ensure that the framework does not panic in production.

## When does Natrix panic (in debug builds)?

### Very unlikely
- **Js Environment Corruption** - If something causes requires javascript methods to be missing, or otherwise fail.
  - In release natrix will skip executing the action it attempted, for example creating a dom node.
- **Unexpected Dom State** - If natrix cant find a expected dom node or the node isnt of the expected type.
  - Natrix will skip updating that part of the dom tree

### User Errors
- **Internal Borrow Errors** - These should only be triggrable by misuse of [`ctx.deferred_borrow`](reactivity::state::State::deferred_borrow)/[`ctx.use_async`](reactivity::state::State::use_async).
  - Natrix will skip handling the event/message, this might lead to dropped messages.
- **User Borrow Errors** - If you use [`.borrow_mut`](reactivity::state::DeferredCtx::borrow_mut) while a borrow is active (which again can only happen due to dev error) it will panic in debug builds.
  - In release builds it will return `None` to signal the calling context should cancel itself.
- **Other Validations** A few methods have debug_asserts, listing all of them would be impractical.

## When does Natrix panic (in release builds)?

### Very unlikely
- **Window or Document Not Found** - If the window or document is not found, natrix will panic.
- **Mount Not Found** - if [`mount`](reactivity::component::mount) fails to find the standard natrix mount point it will error.

### User Errors
- **User Panics** - This one should be obvious.
- **Moving values outside intended scope** - Certain values are intended to only be valid in a given scope.
    - Using interior mutability to move a [`EventToken`](reactivity::state::EventToken) outside its intended scope will likely lead to bugs if used to call apis in non-event contexts.
    - Using interior mutability to move a [`Guard`](reactivity::state::Guard), or using it after a `.await`, will invalidate its guarantees.

## What does natrix does in the case of a panic?
Unlike native rust, a panic in wasm does not prevent the program from continuing. This can lead to unexpected behavior if state is left in a invalid state, or worse lead to undefined behavior.
Therefor natrix will always do its best to prevent further rust execution after a panic, this is done by checking a panic flag at the start of every event handler, natrix also effectively freezes all async code using a special wrapping future that stops propagation of `.poll` calls on panic. 
