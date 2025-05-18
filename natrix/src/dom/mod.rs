//! DOM-related modules for rendering HTML elements.

pub mod element;
pub mod events;
pub mod html_elements;
pub mod list;

// Re-export the public items
pub use element::{Element, MaybeStaticElement};
pub use html_elements::{HtmlElement, ToAttribute, ToClass, ToCssValue};
pub use list::List;
