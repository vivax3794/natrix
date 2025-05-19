# Sub Components

Components wouldnt be very useful if we could not compose them. Due to trait limitations we cant use components as [`Element`](dom::element::Element)s directly. But there is a simple [`SubComponent`](reactivity::component::SubComponent) wrapper to facilitate this.

```rust
# extern crate natrix;
# use natrix::prelude::*;

#[derive(Component)]
struct MyChild {
    /* Any State You Want */
}

impl Component for MyChild {
    fn render() -> impl Element<Self> {
        /* Your Component */
        # e::div()
    }
}

#[derive(Component)]
struct MyParent {
    /* Any State You Want */
}

impl Component for MyParent {
    fn render() -> impl Element<Self> {
        e::div()
            .child(SubComponent::new(MyChild {
                /* Initial Child State */
            }))
    }
}
```

## Message Passing

A common requirement is communication between components. This is where the [`EmitMessage`](reactivity::component::Component::EmitMessage) and [`ReceiveMessage`](reactivity::component::Component::ReceiveMessage) associated types come in. These are used to declare what type is used for message passing to and from the component. The `NoMessages` type is a enum with no variants (i.e similar to [`Infallible`](std::convert::Infallible)) and is used when you do not need to pass messages.

### Child to Parent

Define the `EmitMessage` type to the type of the message you will be emitting and then use [`ctx.emit`](reactivity::state::State::emit), you can then use [`.on`](reactivity::component::SubComponent::on) to listen for the message in the parent component.

```rust
# extern crate natrix;
# use natrix::prelude::*;
#[derive(Component)]
struct MyChild;

impl Component for MyChild {
    type EmitMessage = usize;

    fn render() -> impl Element<Self> {
        e::button()
            .text("Click Me")
            .on::<events::Click>(|ctx: E<Self>, token, _| {
                ctx.emit(10, token);
            })
    }
}

#[derive(Component)]
struct MyParent {
    state: usize,
};

impl Component for MyParent {
    fn render() -> impl Element<Self> {
        e::div()
            .child(SubComponent::new(MyChild).on(|ctx: E<Self>, msg, _| {
                ctx.state += msg;
            }))
    }
}
```

### Parent to Child

Similaryly you can use [`ReceiveMessage`](reactivity::component::Component::ReceiveMessage) to listen for messages from the parent component. You overwrite the default [`handle_message`](reactivity::component::Component::handle_message) method to handle the message. In the parent you use [`.sender`](reactivity::component::SubComponent::sender) to get a sender for the child component.

```rust
# extern crate natrix;
# use natrix::prelude::*;
use natrix::reactivity::state::EventToken;

#[derive(Component, Default)]
struct MyChild {
    state: usize,
}

impl Component for MyChild {
    type ReceiveMessage = usize;

    fn render() -> impl Element<Self> {
        e::div()
            .text(|ctx: R<Self>| *ctx.state)
    }

    fn handle_message(ctx: E<Self>, msg: Self::ReceiveMessage, token: EventToken) {
        *ctx.state += msg;
    }
}

#[derive(Component)]
struct MyParent;

impl Component for MyParent {
    fn render() -> impl Element<Self> {
        let (child, sender) = SubComponent::new(MyChild::default()).sender();
        e::div()
            .child(child)
            // We use `move` to move ownership of the sender into the closure
            .on::<events::Click>(move |ctx: E<Self>, token, _| {
                sender.send(10, token);
            })
    }
}
```

As you see this generally requires you to use a `let` binding to split the return of `.sender`. The [`Sender`](reactivity::component::Sender) is also cloneable.

### When do messages get processed?

Messages passing uses async channels internally, this means the messages will be processed once the current components reactivity cycle is finished. This will still run before the next reflow of the browser, and all messages are batched for efficiency.
