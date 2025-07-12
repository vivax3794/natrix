# Lenses

Lenses are effectively a typed zero-cost composiable abstraction over getter closures.
Because lenses use generics for their functions, even when composed, they only hold the state any closures might have captured (in say a `.map`), and hold no function pointers. This means that the large majority of lenses are in fact zero-sized, and will most likely compile down to a direct field reference.
This does mean one pays for this with monophorzation

## Using lenses
Lenses will generally use the [`Lens`](lens::Lens) trait, the first generic is the source type, and the second is the target.
Generally the source will be global state type, but can also be other structs when composing lenses.
Lenses are usually best defined using `impl Lens<...>`, and access using [`ctx.get`](prelude::EventCtx::get).

such as:
```rust
# extern crate natrix;
# use natrix::prelude::*;
# 
# #[derive(State)]
# struct App;
# 
fn title(content: impl Lens<App, String>) -> impl Element<App> {
    e::h1().text(move |ctx: &mut RenderCtx<App>| ctx.get(content.clone()).clone())
}
```

If you are writing a component library or generally a generic component, then you would naturally use `impl Lens<S, String>` instead.

> [!TIP]
> All lenses are `Clone`, but you can require them to be `Copy` for ergonomics if you know the lenses you use are `Copy`.
> (Only lenses that capture non-`Copy` data in closure via say [`.map`](lens::Lens::map) are not `Copy`)

## Defining Lenses
> [!NOTE]
> These examples use the [`impl_trait_in_bindings`](https://github.com/rust-lang/rust/issues/63065) feature to let us easially show what kind of lens is created by the expressions.

### Field access lenses
Access to fields is done via the `lens!` macro.

```rust
# #![feature(impl_trait_in_bindings)]
# extern crate natrix;
# use natrix::prelude::*;
struct Book {
    author: String,
    title: String,
}

let get_author: impl Lens<Book, String> = lens!(Book => .author);
```
you can access multiple levels of fields at once:

```rust
# #![feature(impl_trait_in_bindings)]
# extern crate natrix;
# use natrix::prelude::*;
struct Book {
    author: String,
    title: String,
}

struct MyBooks {
    rust: Book,
    natrix: Book,
}

let get_natrix_author: impl Lens<MyBooks, String> = lens!(MyBooks => .natrix.author);
```

### Dereferencing
Notably its best practice for render functions to take `impl Lens<..., u8>`, i.e not `Signal<u8>`, but as we know natrix state should be in signals.
This is where [`.deref`](lens::Lens::deref) comes in, it wraps the lens in a way that dereferences. the target value.

```rust
# #![feature(impl_trait_in_bindings)]
# extern crate natrix;
# use natrix::prelude::*;
#[derive(State)]
struct Book {
    author: Signal<String>,
    title: Signal<String>,
}

let get_author_signal: impl Lens<Book, Signal<String>> = lens!(Book => .author);
let get_author: impl Lens<Book, String> = get_author_signal.deref();
```
>  And of course you can do that as one chain, `lens!(Book => .author).deref()`

### Chaining lenses
If say you are getting a lens to some sub-state, and want to call other components that need more narrow lenses, how would you do that?
you can chain lenses together.

```rust
# #![feature(impl_trait_in_bindings)]
# extern crate natrix;
# use natrix::prelude::*;
struct Book {
    author: String,
    title: String,
}

struct MyBooks {
    rust: Book,
    natrix: Book,
}

fn uses_book(book: impl Lens<MyBooks, Book>) {
    let author = book.then(lens!(Book => .author)).deref();
}
```

### Custom lens function
Sometimes you are working with data where there is no good lens abstraction for yet.
In these cases you can use [`.map`](lens::Lens::map), as you will notice lenses are always mut getter internally ([`RenderCtx::get`](prelude::RenderCtx::get) is actually just a downgrade), the lens builder abstractions ensure pure access in most cases, but `.map` gives you the full power to mess with stuff, the getter should be that, a getter, i.e pure.

```rust
# #![feature(impl_trait_in_bindings)]
# extern crate natrix;
# use natrix::prelude::*;
mod library {
    pub struct Data {
        field: u8
    }

    impl Data {
        pub fn get_mut(&mut self) -> &mut u8 {
            &mut self.field
        }
    }
}

struct App {
    data: library::Data,
}

let data_lens: impl Lens<App, u8> = lens!(App => .data).map(|data| data.get_mut());
```
