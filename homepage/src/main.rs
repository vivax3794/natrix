#![feature(must_not_suspend)]
#![warn(must_not_suspend)]
#![deny(clippy::panic, clippy::unwrap_used, clippy::expect_used)]
#![warn(
    clippy::arithmetic_side_effects,
    clippy::indexing_slicing,
    clippy::unreachable
)]

use natrix::prelude::*;

global_css!(
    "
    body {
        margin: 0;
        padding: 0;
        font-family: Arial, sans-serif;
    }

    :root {
        --primary-color: #1e1b4b;
        --secondary-color: oklch(71.4% 0.203 305.504);
        --background-color: #1e293b;
    }
"
);

fn link_button<C: Component>(text: &'static str, link: &'static str) -> impl Element<C> {
    scoped_css!(
        "
        .link {
            color: var(--secondary-color from global);
            font-size: 2rem;
            padding: 10px;
            align-content: center;

            transition: color 0.3s ease, background-color 0.3s ease;
            border-radius: 5px;
            text-decoration: none;
            font-weight: bold;
        }
        .link:hover {
            color: var(--primary-color from global);
            background-color: var(--secondary-color from global);
        }
    "
    );

    e::a().text(text).class(LINK).href(link)
}

fn top_bar<C: Component>() -> impl Element<C> {
    scoped_css!(
        "
        #parent {
            background-color: var(--primary-color from global);
            box-shadow: 0 4px 8px rgba(0, 0, 0, 0.6);
            display: flex;
            padding: 15px;
            gap: 15px;
        }
        .heading {
            color: var(--secondary-color from global);
            margin: 0px 0px;
            align-content: end;
        }
    "
    );

    e::div()
        .id(PARENT)
        .child(
            e::h1()
                .text("Natrix")
                .class(HEADING)
                .class(style!("font-size: 4rem")),
        )
        .child(
            e::h1()
                .text("The Rust first frontend framework")
                .class(HEADING)
                .class(style!("transform: translateY(-0.5rem); opacity: 0.6;")),
        )
        .child(e::div().class(style!("flex: 1")))
        .child(link_button("Book", "TODO"))
        .child(link_button("Docs", "https://docs.rs/natrix/latest/natrix/"))
}

#[derive(Component)]
struct HelloWorld;

impl Component for HelloWorld {
    fn render() -> impl Element<Self> {
        scoped_css!(
            "
            #parent {
                background-color: var(--background-color from global);
                height: 100vh;
                width: 100%;
            }
        "
        );
        e::div().id(PARENT).child(top_bar())
    }
}

fn main() {
    mount(HelloWorld);
}
