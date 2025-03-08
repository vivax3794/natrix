//! Utility traits and structs

/// A version of the stdlib `Any` with no type ids, downcasting, etc
/// Its the minimal possible dyn object, mainly used for keep alive.
pub(crate) trait SmallAny {}
impl<T> SmallAny for T {}
