# Panic Policy

The framework only makes use of `debug_assert!`, its our goal that any issues should be highlighted in debug builds, but not in release builds. On release builds natrix will silently fail in many cases, this is to ensure that the framework does not panic in production.

## When does Natrix panic (in debug builds)?

* **Js Environment Corruption** - If something causes requires javascript methods to be missing, or otherwise fail.
    * In release natrix will skip executing the action it attempted, for example creating a dom node.
* **Unexpected Dom State** - If natrix cant find a expected dom node or the node isnt of the expected type.
    * Natrix will skip updating that part of the dom tree
* **Internal Borrow Errors** - These should only be triggrable by misuse of [`ctx.deferred_borrow`](state::State::deferred_borrow)/[`ctx.use_async`](state::State::use_async).
    * Natrix will skip handling the event/message, this might lead to dropped messages.
* **User Borrow Errors** - If you use [`.borrow_mut`](state::DeferredCtx::borrow_mut) while a borrow is active (which again can only happen due to dev error) it will panic in debug builds.
    * In release builds it will return `None` to signal the calling context should cancel itself.

## When does Natrix panic (in release builds)?
* **Mount Not Found** - if [`mount`](component::mount) fails to find the standard natrix mount point it will error.
* **User Panics** - This one should be obvious.
* **Misused Guards** - If you use async or interor mutability to use a [Guard](state::Guard) outside of the context it was created in you are violating its contract, which might lead to panics.
* **Deferred Borrows After Panic** - If you use [`.borrow_mut`](state::DeferredCtx::borrow_mut) after a panic has happened it will cause another panic, as returning to the user code could cause undefined behaviour.
