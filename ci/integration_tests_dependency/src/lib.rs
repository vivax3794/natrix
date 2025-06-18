use natrix::prelude::*;

pub const DEP_TEXT: &str = "NICE!";
pub const DEP_ID: Id = natrix::id!();

#[derive(Component)]
pub struct DepComp;

impl Component for DepComp {
    fn render() -> impl Element<Self> {
        e::h1().text(DEP_TEXT).id(DEP_ID)
    }
}
