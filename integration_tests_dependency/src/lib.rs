use natrix::prelude::*;

global_css!("
    h1 {
        color: rgba(9,8,7,1);
    }
");

scoped_css!("
    .hello {
        height: 600px;
    }
");

pub const DEP_TEXT: &str = "NICE!";
pub const DEP_ID: &str = "DEP_TEXT";

#[derive(Component)]
pub struct DepComp;

impl Component for DepComp {
    type EmitMessage = NoMessages;
    fn render() -> impl Element<Self> {
        e::h1().text(DEP_TEXT).id(DEP_ID).class(HELLO)
    }
}
