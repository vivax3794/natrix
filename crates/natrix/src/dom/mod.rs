//! DOM-related modules for rendering HTML elements.

pub mod attributes;
pub mod classes;
pub mod element;
pub mod events;
pub mod html_elements;

pub use attributes::ToAttribute;
pub use classes::ToClass;
pub use element::{Element, MaybeStaticElement};
pub use events::EventHandler;
pub use html_elements::HtmlElement;
