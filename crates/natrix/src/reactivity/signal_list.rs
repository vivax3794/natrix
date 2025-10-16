//! Fine-grained reactive list implementation.
//!
//! `SignalList` provides a reactive list where individual element access
//! is tracked separately, preventing unnecessary invalidations.

use std::cell::RefCell;
use std::ops::{Index, IndexMut};

use crate::error_handling::log_or_panic;
use crate::prelude::State;
use crate::reactivity::core;
use crate::reactivity::core::SignalDepList;

/// A reactive list with fine-grained dependency tracking.
///
/// Unlike wrapping a `Vec` in a `Signal`, `SignalList` tracks dependencies
/// per-index, so reading `list[0]` won't be invalidated when you modify `list[1]`.
///
/// # Example
///
/// ```rust
/// use natrix::prelude::*;
///
/// let mut list = SignalList::new();
/// list.push(1);
/// list.push(2);
/// list.push(3);
///
/// // Reading list[0] only subscribes to changes at index 0
/// let first = list.get(0);
///
/// // Pushing doesn't invalidate readers of existing elements
/// list.push(4); // Only length readers are notified
///
/// // Modifying an element only notifies readers of that element
/// if let Some(elem) = list.get_mut(1) {
///     *elem = 20; // Only readers of list[1] are notified
/// }
/// ```
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct SignalList<T> {
    /// The actual data storage
    items: Vec<T>,
    /// Dependency tracking for each index
    /// Uses a Vec to allow dynamic growth as list grows
    #[cfg_attr(feature = "serde", serde(skip))]
    deps: RefCell<Vec<SignalDepList>>,
    /// Dependencies for the list length
    /// Reading `.len()` subscribes here, so it updates when items are added/removed
    #[cfg_attr(feature = "serde", serde(skip))]
    len_deps: RefCell<SignalDepList>,
}

impl<T: std::fmt::Debug> std::fmt::Debug for SignalList<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignalList")
            .field("items", &self.items)
            .finish_non_exhaustive()
    }
}

impl<T> SignalList<T> {
    /// Create a new empty `SignalList`
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            deps: RefCell::new(Vec::new()),
            len_deps: RefCell::new(SignalDepList::new()),
        }
    }

    /// Create a new `SignalList` with the specified capacity
    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(capacity),
            deps: RefCell::new(Vec::with_capacity(capacity)),
            len_deps: RefCell::new(SignalDepList::new()),
        }
    }

    /// Get the length of the list.
    /// This subscribes to length changes (push/pop/clear).
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        // Track length dependency
        if let Some(hook) = core::statics::current_hook() {
            if let Ok(mut len_deps) = self.len_deps.try_borrow_mut() {
                len_deps.insert(hook);
            } else {
                log_or_panic!("Length deps list already borrowed");
            }
        }
        self.items.len()
    }

    /// Check if the list is empty.
    /// This subscribes to length changes.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Push an element to the end of the list.
    /// This only invalidates length readers, not element readers.
    pub fn push(&mut self, value: T) {
        self.items.push(value);
        
        // Ensure deps vec is large enough
        if let Ok(mut deps) = self.deps.try_borrow_mut() {
            deps.push(SignalDepList::new());
        } else {
            log_or_panic!("Deps list already borrowed during push");
        }

        // Only invalidate length subscribers
        core::statics::reg_dirty_list(|| {
            self.len_deps.get_mut().create_iter_and_clear()
        });
    }

    /// Pop an element from the end of the list.
    /// This invalidates length readers and the last element's readers.
    pub fn pop(&mut self) -> Option<T> {
        let result = self.items.pop();
        
        if result.is_some() {
            // Remove the dep list for the removed element and invalidate it
            if let Ok(mut deps) = self.deps.try_borrow_mut() {
                if let Some(mut last_deps) = deps.pop() {
                    // Invalidate both the removed element and length
                    core::statics::reg_dirty_list(|| last_deps.create_iter_and_clear());
                    core::statics::reg_dirty_list(|| {
                        self.len_deps.get_mut().create_iter_and_clear()
                    });
                } else {
                    // Only invalidate length if no element deps to remove
                    core::statics::reg_dirty_list(|| {
                        self.len_deps.get_mut().create_iter_and_clear()
                    });
                }
            } else {
                log_or_panic!("Deps list already borrowed during pop");
            }
        }
        
        result
    }

    /// Get an immutable reference to an element at the specified index.
    /// This subscribes to changes at that specific index.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.items.len() {
            return None;
        }

        // Track dependency for this specific index
        if let Some(hook) = core::statics::current_hook() {
            if let Ok(mut deps) = self.deps.try_borrow_mut() {
                if let Some(dep_list) = deps.get_mut(index) {
                    dep_list.insert(hook);
                } else {
                    log_or_panic!("Index out of bounds in deps tracking");
                }
            } else {
                log_or_panic!("Deps list already borrowed during get");
            }
        }

        Some(&self.items[index])
    }

    /// Get a mutable reference to an element at the specified index.
    /// This invalidates all readers of that specific index.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.items.len() {
            return None;
        }

        // Invalidate readers of this specific index
        if let Some(deps) = self.deps.get_mut().get_mut(index) {
            core::statics::reg_dirty_list(|| deps.create_iter_and_clear());
        } else {
            log_or_panic!("Index out of bounds in deps during get_mut");
        }

        Some(&mut self.items[index])
    }

    /// Clear all elements from the list.
    /// This invalidates all readers.
    pub fn clear(&mut self) {
        if self.items.is_empty() {
            return;
        }

        self.items.clear();
        
        // Invalidate all element readers and length readers
        if let Ok(mut deps) = self.deps.try_borrow_mut() {
            for mut dep_list in deps.drain(..) {
                core::statics::reg_dirty_list(|| dep_list.create_iter_and_clear());
            }
        } else {
            log_or_panic!("Deps list already borrowed during clear");
        }
        
        core::statics::reg_dirty_list(|| {
            self.len_deps.get_mut().create_iter_and_clear()
        });
    }

    /// Insert an element at the specified index.
    /// WARNING: This invalidates readers at this index and all subsequent indices,
    /// since their indices shift.
    pub fn insert(&mut self, index: usize, value: T) {
        if index > self.items.len() {
            log_or_panic!("Index {} out of bounds for insert (len {})", index, self.items.len());
            return;
        }

        self.items.insert(index, value);
        
        // Insert a new dep list at this position
        if let Ok(mut deps) = self.deps.try_borrow_mut() {
            deps.insert(index, SignalDepList::new());
        } else {
            log_or_panic!("Deps list already borrowed during insert");
        }

        // Invalidate readers from this index onwards (since indices shifted)
        if let Ok(mut deps) = self.deps.try_borrow_mut() {
            for dep_list in deps.iter_mut().skip(index) {
                core::statics::reg_dirty_list(|| dep_list.create_iter_and_clear());
            }
        } else {
            log_or_panic!("Deps list already borrowed during insert invalidation");
        }
        
        core::statics::reg_dirty_list(|| {
            self.len_deps.get_mut().create_iter_and_clear()
        });
    }

    /// Remove an element at the specified index.
    /// WARNING: This invalidates readers at this index and all subsequent indices.
    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index >= self.items.len() {
            log_or_panic!("Index {} out of bounds for remove (len {})", index, self.items.len());
            return None;
        }

        let result = self.items.remove(index);
        
        // Remove the dep list for this index and invalidate it immediately
        if let Ok(mut deps) = self.deps.try_borrow_mut() {
            let mut removed_deps = deps.remove(index);
            core::statics::reg_dirty_list(|| removed_deps.create_iter_and_clear());
        } else {
            log_or_panic!("Deps list already borrowed during remove");
        }
        
        // Invalidate readers from this index onwards
        if let Ok(mut deps) = self.deps.try_borrow_mut() {
            for dep_list in deps.iter_mut().skip(index) {
                core::statics::reg_dirty_list(|| dep_list.create_iter_and_clear());
            }
        } else {
            log_or_panic!("Deps list already borrowed during remove invalidation");
        }
        
        core::statics::reg_dirty_list(|| {
            self.len_deps.get_mut().create_iter_and_clear()
        });

        Some(result)
    }

    /// Iterate over the list elements.
    /// This does NOT track dependencies - use explicit `.get(i)` calls for reactive iteration.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.iter()
    }

    /// Iterate over the list elements mutably.
    /// This does NOT track dependencies or invalidate readers.
    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.items.iter_mut()
    }

    /// Get the internal capacity of the list
    #[inline]
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.items.capacity()
    }

    /// Reserve capacity for at least `additional` more elements
    pub fn reserve(&mut self, additional: usize) {
        self.items.reserve(additional);
        if let Ok(mut deps) = self.deps.try_borrow_mut() {
            deps.reserve(additional);
        } else {
            log_or_panic!("Deps list already borrowed during reserve");
        }
    }

    /// Extend the list with the contents of an iterator.
    /// This only invalidates length readers, not element readers.
    pub fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let iter = iter.into_iter();
        let (lower_bound, _) = iter.size_hint();
        self.reserve(lower_bound);

        let old_len = self.items.len();
        self.items.extend(iter);
        let new_len = self.items.len();

        // Add dep lists for new elements
        if let Ok(mut deps) = self.deps.try_borrow_mut() {
            for _ in old_len..new_len {
                deps.push(SignalDepList::new());
            }
        } else {
            log_or_panic!("Deps list already borrowed during extend");
        }

        // Only invalidate length if we actually added elements
        if new_len > old_len {
            core::statics::reg_dirty_list(|| {
                self.len_deps.get_mut().create_iter_and_clear()
            });
        }
    }
}

impl<T> Default for SignalList<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Index<usize> for SignalList<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).unwrap_or_else(|| {
            panic!("Index {index} out of bounds for SignalList of length {}", self.items.len())
        })
    }
}

impl<T> IndexMut<usize> for SignalList<T> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let len = self.items.len();
        self.get_mut(index).unwrap_or_else(|| {
            panic!("Index {index} out of bounds for SignalList of length {len}")
        })
    }
}

impl<T> From<Vec<T>> for SignalList<T> {
    fn from(vec: Vec<T>) -> Self {
        let len = vec.len();
        let mut deps = Vec::with_capacity(len);
        for _ in 0..len {
            deps.push(SignalDepList::new());
        }
        
        Self {
            items: vec,
            deps: RefCell::new(deps),
            len_deps: RefCell::new(SignalDepList::new()),
        }
    }
}

impl<T> FromIterator<T> for SignalList<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::from(iter.into_iter().collect::<Vec<_>>())
    }
}

impl<T: 'static> State for SignalList<T> {
    fn set(&mut self, new: Self) {
        // Clear and rebuild
        *self = new;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reactivity::core::{HookKey, statics};
    use std::collections::HashSet;

    #[test]
    fn push_doesnt_invalidate_element_readers() {
        let mut list = SignalList::new();
        list.push(1);
        list.push(2);

        let hook_elem_0 = HookKey::new(0, 0);
        let hook_elem_1 = HookKey::new(1, 0);
        let hook_len = HookKey::new(2, 0);

        // Subscribe to different elements
        statics::with_hook(hook_elem_0, || {
            let _ = list.get(0);
        });
        statics::with_hook(hook_elem_1, || {
            let _ = list.get(1);
        });
        statics::with_hook(hook_len, || {
            let _ = list.len();
        });

        // Push a new element
        let (dirty, ()) = statics::with_dirty_tracking(|| {
            list.push(3);
        });

        // Only length reader should be invalidated
        let hooks: HashSet<_> = dirty.into_iter().flatten().collect();
        assert_eq!(hooks, HashSet::from([hook_len]));
    }

    #[test]
    fn modifying_element_only_invalidates_that_index() {
        let mut list = SignalList::from(vec![1, 2, 3]);

        let hook_elem_0 = HookKey::new(0, 0);
        let hook_elem_1 = HookKey::new(1, 0);
        let hook_elem_2 = HookKey::new(2, 0);

        statics::with_hook(hook_elem_0, || {
            let _ = list.get(0);
        });
        statics::with_hook(hook_elem_1, || {
            let _ = list.get(1);
        });
        statics::with_hook(hook_elem_2, || {
            let _ = list.get(2);
        });

        // Modify index 1
        let (dirty, ()) = statics::with_dirty_tracking(|| {
            if let Some(elem) = list.get_mut(1) {
                *elem = 20;
            }
        });

        // Only hook_elem_1 should be invalidated
        let hooks: HashSet<_> = dirty.into_iter().flatten().collect();
        assert_eq!(hooks, HashSet::from([hook_elem_1]));
    }

    #[test]
    fn pop_invalidates_length_and_last_element() {
        let mut list = SignalList::from(vec![1, 2, 3]);

        let hook_elem_2 = HookKey::new(0, 0);
        let hook_len = HookKey::new(1, 0);

        statics::with_hook(hook_elem_2, || {
            let _ = list.get(2);
        });
        statics::with_hook(hook_len, || {
            let _ = list.len();
        });

        let (dirty, ()) = statics::with_dirty_tracking(|| {
            list.pop();
        });

        let hooks: HashSet<_> = dirty.into_iter().flatten().collect();
        assert_eq!(hooks, HashSet::from([hook_elem_2, hook_len]));
    }

    #[test]
    fn clear_invalidates_all_readers() {
        let mut list = SignalList::from(vec![1, 2, 3]);

        let hook_elem_0 = HookKey::new(0, 0);
        let hook_elem_1 = HookKey::new(1, 0);
        let hook_len = HookKey::new(2, 0);

        statics::with_hook(hook_elem_0, || {
            let _ = list.get(0);
        });
        statics::with_hook(hook_elem_1, || {
            let _ = list.get(1);
        });
        statics::with_hook(hook_len, || {
            let _ = list.len();
        });

        let (dirty, ()) = statics::with_dirty_tracking(|| {
            list.clear();
        });

        let hooks: HashSet<_> = dirty.into_iter().flatten().collect();
        assert_eq!(hooks, HashSet::from([hook_elem_0, hook_elem_1, hook_len]));
    }

    #[test]
    fn indexing_tracks_dependencies() {
        let mut list = SignalList::from(vec![1, 2, 3]);
        let hook = HookKey::new(0, 0);

        statics::with_hook(hook, || {
            let _val = list[1];
        });

        let (dirty, ()) = statics::with_dirty_tracking(|| {
            list[1] = 20;
        });

        let hooks: HashSet<_> = dirty.into_iter().flatten().collect();
        assert_eq!(hooks, HashSet::from([hook]));
    }

    #[test]
    fn extend_only_invalidates_length() {
        let mut list = SignalList::from(vec![1, 2]);

        let hook_elem_0 = HookKey::new(0, 0);
        let hook_len = HookKey::new(1, 0);

        statics::with_hook(hook_elem_0, || {
            let _ = list.get(0);
        });
        statics::with_hook(hook_len, || {
            let _ = list.len();
        });

        let (dirty, ()) = statics::with_dirty_tracking(|| {
            list.extend(vec![3, 4, 5]);
        });

        let hooks: HashSet<_> = dirty.into_iter().flatten().collect();
        assert_eq!(hooks, HashSet::from([hook_len]));
    }

    #[test]
    fn from_iter_works() {
        let list: SignalList<i32> = (1..=5).collect();
        assert_eq!(list.len(), 5);
        assert_eq!(list.get(0), Some(&1));
        assert_eq!(list.get(4), Some(&5));
    }
}