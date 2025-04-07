use natrix::prelude::*;

global_css!(
    "
h1 {
    color: rgba(9,8,7,1);
}
"
);

pub const DEP_TEXT: &str = "NICE!";
pub const DEP_ID: &str = "DEP_TEXT";

#[derive(Component)]
pub struct DepComp;

impl Component for DepComp {
    fn render() -> impl Element<Self::Data> {
        e::h1().text(DEP_TEXT).id(DEP_ID)
    }
}
