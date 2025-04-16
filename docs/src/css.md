# Css

> [!NOTE]
> Natrixses css bundlinging system requires the use of the natrix cli.
> As such css bundling will not work when embedding natrix in other frameworks.

Natrix uses a unique css bundling system that allows for css to be declared in rust files, but bundled at compile time.
This is very different from other rust frameworks, which either do runtime injection, or require static external css files. Both of which have downsides that natrix solves.

The main advantage of this design is that css for dependencies is bundled along with the code on crates.io and is automatically combined with your own at **compile time**.

## Global css

Global css is emitted using the `global_css!` macro, which takes a string literal.

```rust
# extern crate natrix;
# use natrix::prelude::*;
global_css!("
    body {
        background-color: red;
    }
");
```

> [!IMPORTANT]
> Due to css-tree shaking, dynamic classes might be stripped from the final css bundle.
> To avoid this use the custom `@keep` directive.
>
> ```rust
> # extern crate natrix;
> # use natrix::prelude::*;
> global_css!("
>     @keep dynamically_generated_class;
>     .dynamically_generated_class {
>         background-color: red;
>     }
> ");
> ```

## Scoped css

Scoped css is emitted using the `scoped_css!` macro, which takes a string literal. This uses [Css Modules](https://lightningcss.dev/css-modules.html) to generate unique class/id/variable names for each invocation at compile time. it then emits the transformed css to the build system. and expands to a set of constants mapping the initial name to the mangled one.

> [!TIP]
> Features such as the `:global` selector are supported as described in the [css modules documentation](https://lightningcss.dev/css-modules.html#global).

```rust
# extern crate natrix;
# use natrix::prelude::*;
scoped_css!("
    .my-class {
        background-color: red;
    }
");
```

This will emit a css file with the following contents:

```css
.SOME_HASH-my-class {
  background-color: red;
}
```

and will expand to the following in rust:

```rust
pub(crate) const MY_CLASS: &str = "SOME_HASH-my-class";
```

Which can then be used in a [`.class`](html_elements::HtmlElement::class) call.

> [!TIP]
> Use a module to make it more clear where the constants are coming from in the rest of your code.

```rust
# extern crate natrix;
# use natrix::prelude::*;
mod css {
    use natrix::prelude::scoped_css;
scoped_css!("
        .my-class {
            background-color: red;
        }
    ");
}

#[derive(Component)]
struct HelloWorld;
impl Component for HelloWorld {
    fn render() -> impl Element<Self> {
        e::h1().text("Hello World").class(css::MY_CLASS)
    }
}
```

## Inline css
Sometimes you only need some styles for a single element, you can use `scoped_css` with a id for this, or even a `.attr("style", ...)`.
But we provide another option, the `style!` macro:
```rust
# extern crate natrix;
# use natrix::prelude::*;
#[derive(Component)]
struct HelloWorld;
impl Component for HelloWorld {
    fn render() -> impl Element<Self> {
        e::h1()
            .text("Hello World")
            .class(style!("font-size: 4rem"))
    }
}
```
This will emit a css file with the following contents:

```css
.SOME_HASH {
  font-size: 4rem;
}
```
And will expand to the following in rust:

```rust
# extern crate natrix;
# use natrix::prelude::*;
#[derive(Component)]
struct HelloWorld;
impl Component for HelloWorld {
    fn render() -> impl Element<Self> {
        e::h1()
            .text("Hello World")
            .class("SOME_HASH")
    }
}
```

I.e it will **not** use the `style` attribute, but instead still emit css to the bundling system.
This is best used for short snippets, and `scoped_css` should still be used for longer styles, even if only for a single element.

> [!TIP]
> The class name is based on the hash of the style, this means that if multiple parts of the code use, say, `style!("font-size: 4rem")` they will all resolve to the same class name and the emitted css will be deduplicated.
