// Simple test file to understand the patterns
use natrix::prelude::*;

#[derive(Project, Downgrade)]
enum MyEnum<T> {
    Variant1(T),
    Variant2(String),
    Variant3,
}

fn main() {
    println!("Macro test compiles!");
}