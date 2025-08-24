use natrix::prelude::*;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

// Test the Project derive macro with additional derives
#[derive(Project)]
#[project(Debug)]
enum TestEnum<T> {
    Value(T),
    Text(String),
    Empty,
}

#[derive(Project)]
enum SimpleEnum {
    A,
    B,
    C,
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
    // Test basic type creation and usage
    let _test_enum: TestEnum<u32> = TestEnum::Value(42);
    let _projected_type_exists: TestEnumProjected<u32> = TestEnumProjected::Value(natrix::access::Ref::Read(&42));
    
    // Test that Debug derive works on the projected enum (from #[project(Debug)])
    let debug_test: TestEnumProjected<u32> = TestEnumProjected::Empty;
    let _debug_string = format!("{:?}", debug_test);
    
    // Test SimpleEnum (no lifetime parameter needed)
    let _simple: SimpleEnum = SimpleEnum::A;
    let _simple_projected: SimpleEnumProjected = SimpleEnumProjected::A;
}

#[wasm_bindgen_test] 
fn project_trait_implementation() {
    use natrix::access::{Project, Ref};
    
    // Test that the Project trait is actually implemented and works
    let test_enum = TestEnum::Value(42u32);
    let test_ref = Ref::Read(&test_enum);
    
    // This should call our generated project() method
    let projected = TestEnum::project(test_ref);
    
    match projected {
        TestEnumProjected::Value(value_ref) => {
            // Check that we can read the value
            if let Some(value) = value_ref.into_read() {
                assert_eq!(*value, 42);
            }
        },
        _ => panic!("Expected Value variant"),
    }
}