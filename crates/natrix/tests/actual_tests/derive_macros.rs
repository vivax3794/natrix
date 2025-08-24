use natrix::prelude::*;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

// Test the Project derive macro
#[derive(Project)]
enum TestEnum<T> {
    Value(T),
    Text(String),
    Empty,
}

#[wasm_bindgen_test]
fn project_derive_compiles() {
    // Test that the derive macro generates valid code
    // This is just a compilation test for now
    let _test_enum: TestEnum<i32> = TestEnum::Value(42);
}

#[derive(State)]
struct ProjectTest {
    test_enum: Signal<TestEnum<u32>>,
}

fn render_project_test() -> impl Element<ProjectTest> {
    |mut ctx: RenderCtx<ProjectTest>| {
        // Test that .project() works with our derived enum by using it in a simple context
        // For now, just test that the types work by using a basic display
        e::div().text(move |mut ctx: RenderCtx<ProjectTest>| {
            // Use the enum value directly for this simple test
            match &*ctx.test_enum {
                TestEnum::Value(v) => format!("Value: {}", v),
                TestEnum::Text(t) => format!("Text: {}", t),
                TestEnum::Empty => "Empty".to_string(),
            }
        })
    }
}

#[wasm_bindgen_test] 
fn project_derive_usage() {
    // Test that we can actually use the projected types 
    // For now this just tests that the generated code compiles and the types exist
    let _test_enum: TestEnum<u32> = TestEnum::Value(42);
    let _projected_type_exists: TestEnumProjected<u32> = TestEnumProjected::Value(natrix::access::Ref::Read(&42));
}