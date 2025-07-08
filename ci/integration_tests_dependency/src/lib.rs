use natrix::prelude::*;

pub const DEP_TEXT: &str = "NICE!";
pub const DEP_ID: Id = natrix::id!();

pub fn dep_component<C: State>() -> impl Element<C> {
    e::h1().text(DEP_TEXT).id(DEP_ID)
}
