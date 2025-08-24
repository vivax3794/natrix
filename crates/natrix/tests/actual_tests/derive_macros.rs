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

// Test Downgrade derive
#[derive(Downgrade)]
enum DowngradeTestEnum<T> {
    Value(T),
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

#[derive(State)]
struct ProjectableSignalTest {
    choice: ProjectableSignal<TestEnum<u32>>,
    simple_choice: ProjectableSignal<SimpleEnum>,
}

#[wasm_bindgen_test]
fn projectable_signal_with_custom_enum() {
    use natrix::reactivity::{ProjectableSignal, ProjectIntoState};
    
    // Ensure our derived enum implements ProjectIntoState
    fn assert_project_into_state<T: ProjectIntoState>() {}
    assert_project_into_state::<TestEnum<u32>>();
    assert_project_into_state::<SimpleEnum>();
    
    // Test basic ProjectableSignal creation and usage
    let mut signal = ProjectableSignal::new(TestEnum::Value(42u32));
    
    // Test deref access
    match *signal {
        TestEnum::Value(ref value) => assert_eq!(*value, 42),
        _ => panic!("Expected Value variant"),
    }
    
    // Test updating the signal
    signal.update(TestEnum::Text("hello".to_string()));
    match *signal {
        TestEnum::Text(ref text) => assert_eq!(text, "hello"),
        _ => panic!("Expected Text variant"),
    }
}

#[wasm_bindgen_test]
fn projectable_signal_field_access() {
    use natrix::access::Ref;
    
    let mut signal = ProjectableSignal::new(TestEnum::Value(100u32));
    
    // Test field access and projection
    let signal_ref = field!(signal);
    let projected = signal_ref.project_signal();
    
    match projected {
        TestEnumProjected::Value(value_ref) => {
            if let Some(value) = value_ref.into_read() {
                assert_eq!(*value, 100);
            }
        },
        _ => panic!("Expected Value variant"),
    }
}

// Test complex enum with named fields for ProjectableSignal
#[derive(Project)]
#[project(Debug, PartialEq)]
enum UserAction<T> {
    Click { x: u32, y: u32 },
    Select { item: T },
    Scroll(f32),
    None,
}

#[wasm_bindgen_test] 
fn complex_enum_with_projectable_signal() {
    use natrix::reactivity::ProjectableSignal;
    
    let mut signal = ProjectableSignal::new(UserAction::Click { x: 100, y: 200 });
    
    // Test named field access
    match *signal {
        UserAction::Click { x, y } => {
            assert_eq!(x, 100);
            assert_eq!(y, 200);
        },
        _ => panic!("Expected Click variant"),
    }
    
    // Test projection with named fields
    let projected = field!(signal).project_signal();
    match projected {
        UserActionProjected::Click { x, y } => {
            if let (Some(x_val), Some(y_val)) = (x.into_read(), y.into_read()) {
                assert_eq!(*x_val, 100);
                assert_eq!(*y_val, 200);
            }
        },
        _ => panic!("Expected Click variant in projected"),
    }
    
    // Test updating to different variant
    signal.update(UserAction::Select { item: "test".to_string() });
    let projected = field!(signal).project_signal();
    match projected {
        UserActionProjected::Select { item } => {
            if let Some(item_val) = item.into_read() {
                assert_eq!(item_val, "test");
            }
        },
        _ => panic!("Expected Select variant in projected"),
    }
}

#[wasm_bindgen_test]
fn downgrade_derive_implementation() {
    use natrix::access::{Downgrade, Ref};
    
    // Test that Downgrade trait is implemented
    let test_enum = DowngradeTestEnum::Value(42u32);
    
    // Test the basic downgrade functionality
    let downgradeable: Ref<DowngradeTestEnum<u32>> = Ref::Read(&test_enum);
    let downgraded = downgradeable.into_read().unwrap();
    
    match downgraded {
        DowngradeTestEnumReadOutput::Value(val) => assert_eq!(*val, 42),
        _ => panic!("Expected Value variant"),
    }
    
    // Test Empty variant
    let empty_enum = DowngradeTestEnum::<u32>::Empty;
    let empty_ref: Ref<DowngradeTestEnum<u32>> = Ref::Read(&empty_enum);
    let empty_downgraded = empty_ref.into_read().unwrap();
    
    match empty_downgraded {
        DowngradeTestEnumReadOutput::Empty => {}, // This should work
        _ => panic!("Expected Empty variant"),
    }
}