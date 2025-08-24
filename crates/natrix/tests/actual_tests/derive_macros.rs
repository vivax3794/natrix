use natrix::prelude::*;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

// This should not compile - pure-unit enum (uncomment to test compile error)
// #[derive(Project)]
// enum PureUnitEnum {
//     A,
//     B,
//     C,
// }

// Test the Project derive macro with additional derives
#[derive(Project, ProjectIntoState)]
#[project(derive(Debug))]
enum TestEnum<T> {
    Value(T),
    Text(String),
    Empty,
}

#[derive(Project, ProjectIntoState)]
enum SimpleEnum {
    A(i32),
    B,
    C,
}

// Test Downgrade derive on a projected enum with Ref fields
#[derive(Downgrade)]
enum DowngradeTestProjected<'a> {
    Value(natrix::access::Ref<'a, u32>),
    Empty,
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
    
    // Test SimpleEnum 
    let _simple: SimpleEnum = SimpleEnum::A(42);
    let _simple_projected: SimpleEnumProjected = SimpleEnumProjected::A(natrix::access::Ref::Read(&42));
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
    choice: natrix::reactivity::signal::ProjectableSignal<TestEnum<u32>>,
    simple_choice: natrix::reactivity::signal::ProjectableSignal<SimpleEnum>,
}

#[wasm_bindgen_test]
fn projectable_signal_with_custom_enum() {
    use natrix::reactivity::signal::ProjectableSignal;
    
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
    use natrix::access::{Ref, Project};
    use natrix::reactivity::signal::ProjectableSignal;
    
    let mut signal = ProjectableSignal::new(TestEnum::Value(100u32));
    
    // Test direct project access on the enum
    let test_ref = Ref::Read(&*signal);
    let projected = TestEnum::project(test_ref);
    
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
#[derive(Project, ProjectIntoState)]
#[project(derive(Debug, PartialEq))]
enum UserAction<T> {
    Click { x: u32, y: u32 },
    Select { item: T },
    Scroll(f32),
    None,
}

#[wasm_bindgen_test] 
fn complex_enum_with_projectable_signal() {
    use natrix::reactivity::signal::ProjectableSignal;
    use natrix::access::Project;
    
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
    let test_ref = natrix::access::Ref::Read(&*signal);
    let projected = UserAction::project(test_ref);
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
    let test_ref = natrix::access::Ref::Read(&*signal);
    let projected = UserAction::project(test_ref);
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
    
    // Test that Downgrade trait is implemented on projected enum with Ref fields
    let test_enum = DowngradeTestProjected::Value(Ref::Read(&42u32));
    
    // Test the basic downgrade functionality
    let downgraded = test_enum.into_read().unwrap();
    
    match downgraded {
        DowngradeTestProjectedReadOutput::Value(mut ref_val) => {
            // The ref_val should be the downgraded version of the Ref
            if let Some(inner_val) = ref_val.into_read() {
                assert_eq!(**inner_val, 42);
            }
        },
        _ => panic!("Expected Value variant"),
    }
    
    // Test Empty variant
    let empty_enum = DowngradeTestProjected::Empty;
    let empty_downgraded = empty_enum.into_read().unwrap();
    
    match empty_downgraded {
        DowngradeTestProjectedReadOutput::Empty => {}, // This should work
        _ => panic!("Expected Empty variant"),
    }
}