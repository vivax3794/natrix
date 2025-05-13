# Testing

Testing is a important part of any project. Natrix doesnt have a dedicated testing framework, instead we recommend you use [wasm-pack](https://rustwasm.github.io/wasm-pack/) to run your tests.
But natrix does provide the [`test_utils`](test_utils) module to help with testing, which is enabled with the `test_utils` feature flag.

The primary functions are [`mount_test`](test_utils::mount_test) and [`get`](test_utils::get).

## Example

```rust
# extern crate natrix;
# extern crate wasm_bindgen_test;
use natrix::prelude::*;

#[derive(Component)]
struct HelloWorld;

impl Component for HelloWorld {
    fn render() -> impl Element<Self> {
        e::div()
            .text("Hello World")
            .id("HELLO")
    }
}

mod tests {
    use super::*;
    use natrix::test_utils;
    use wasm_bindgen_test::wasm_bindgen_test;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_hello_world() {
        test_utils::mount_test(HelloWorld);
        let hello = test_utils::get("HELLO");
        assert_eq!(hello.text_content(), Some("Hello World".to_string()));
    }
}

# fn main() {}
```

This will mount the `HelloWorld` component and then check if the text content of the element with id `HELLO` is "Hello World". This is a simple test, but it shows how to use the `test_utils` module to test your components.
These tests can be run as follows:

```bash
wasm-pack test --headless --chrome --firefox
```

> [!NOTE]
> From out experience the firefox webdriver is very slow to spin up, and even fails at semmingly random times.

## Message Passing
Due to the fact message passing between components uses async, you will need to make your test async as well to observe the changes.
Luckily `wasm-bindgen-test` already natively supports async tests, so you can just use the `async` keyword in your test function.
To wait until all messages have been processed, you can use [`next_animation_frame`](async_utils::next_animation_frame) from the `async_utils` feature flag.

```rust
# extern crate natrix;
# extern crate wasm_bindgen_test;
# use natrix::prelude::*;
# 
# #[derive(Component)]
# struct Child;
#
# impl Component for Child {
#     type EmitMessage = u8;
#     fn render() -> impl Element<Self> {
#         e::div().id("CHILD").on::<events::Click>(|ctx: E<Self>, token, _| {
#             ctx.emit(1, token);
#         })
#     }
# }
# 
# #[derive(Component)]
# struct Parent {
#     state: u8,
# }
# 
# impl Component for Parent {
#     fn render() -> impl Element<Self> {
#         e::div()
#             .child(e::div().id("PARENT").text(|ctx: R<Self>| *ctx.state))
#             .child(SubComponent::new(Child).on(|ctx: E<Self>, msg, _| {
#                   *ctx.state += msg;
#             }))
#     }
# }
# 
# mod tests {
#     use super::*;
#     use natrix::test_utils;
use natrix::async_utils;
#     use wasm_bindgen_test::wasm_bindgen_test;
# 
#     wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn test_message_passing() {
    test_utils::mount_test(Parent {state: 0});

    let parent_element = test_utils::get("PARENT");
    let child_element = test_utils::get("CHILD");

    assert_eq!(parent_element.text_content(), Some("0".to_string()));
    child_element.click();
    assert_eq!(parent_element.text_content(), Some("0".to_string()));

    async_utils::next_animation_frame().await;
    assert_eq!(parent_element.text_content(), Some("1".to_string()));
}
# }

# fn main() {}
```
