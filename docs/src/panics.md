# Panic Policy

The framework only makes use of `debug_assert!`, its our goal that any issues should be highlighted in debug builds, but not in release builds. On release builds natrix will silently fail in many cases, this is to ensure that the framework does not panic in production.

## When does Natrix panic (in debug builds)?

* **Js Environemt Corruption** - If something causes requires javascript methods to be missing, or otherwise fail.
    * In release natrix will skip executing the action it attempted, for example creating a dom node.
* **Unexpected Dom State** - If natrix cant find a expected dom node or the node isnt of the expected type.
    * Natrix will skip updating that part of the dom tree
* **Internal Borrow Errors** - These should only be triggrable by missuse of `ctx.defered_borrow`/`ctx.use_async`.
    * Natrix will skip handling the event/message, this might lead to dropped messages.
* **User Borrow Errors** - If you use `.borrow_mut` while a borrow is active (which again can only happen due to dev error) it will panic in debug builds.
    * In release builds it will return `None` to signal the calling context should cancel itself.

## When does Natrix panic (in release builds)?
* **Missing Mount Point** - If you try to mount a component to a non-existent mount point
* **User Panics** - This one should be obvious.
> [!TIP]
> If you are certain your code will not cause panics you can disable the `panic_hook` feature to get a smaller binary size, see [features](features.md) for warnings on doing this.

