// Simple test file to understand the patterns
use natrix::prelude::*;

#[derive(Project)]
enum MyEnum<T> {
    Variant1(T),
    Variant2(String),
    Variant3,
}

fn main() {
    println!("Macro test compiles!");
}