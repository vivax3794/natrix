use std::hint::black_box;

use natrix::prelude::*;
mod reload_tests;

const HELLO_TEXT: &str = "HELLO WORLD, TEST TEST!";
const HELLO_ID: &str = "HELLO";
const PANIC_ID: &str = "PANIC";
const BUTTON_ID: &str = "BUTTON";
const RELOAD_ID: &str = "RELOAD";
const IMG_ID: &str = "IMG_ID";

// global_css!("
//     h1 {
//         background-color: rgba(1,2,3,1);
//     }
//     .hello_world {
//         width: 100px;
//     }
//
//     @keep dynamic;
//
//     .dynamic {
//         padding: 100px;
//     }
// ");
//
// scoped_css!("
//     .hello {
//         height: 300px;
//         font-size: var(--size);
//     }
//     .I_amNotUsed {
//         height: 400px;
//     }
// ");

#[derive(Component)]
struct NotUsed;

impl Component for NotUsed {
    fn render() -> impl Element<Self> {
        e::img()
        // .class(I_AM_NOT_USED)
    }
}

#[derive(Component)]
struct HelloWorld {
    counter: usize,
}

impl Component for HelloWorld {
    fn render() -> impl Element<Self> {
        e::div()
            .child(
                e::h1()
                    .text(HELLO_TEXT)
                    .id(HELLO_ID)
                    .class("hello_world")
                    // .class(HELLO)
                    // .class(USE_IMG)
                    .class(format!("dyn{}", black_box("amic"))), // .class(style!("margin: 1px 2px 3px 4px"))
                                                                 // .css_value(SIZE, css::values::Numeric::px(10)),
            )
            .child(SubComponent::new(integration_tests_dependency::DepComp))
            // .child(SubComponent::new(NotUsed))
            .child(
                e::button()
                    .id(PANIC_ID)
                    .on::<events::Click>(|_ctx: E<Self>, _, _| {
                        panic!("Panic button clicked!");
                    }),
            )
            .child(
                e::button()
                    .id(BUTTON_ID)
                    .on::<events::Click>(|ctx: E<Self>, _, _| {
                        *ctx.counter += 1;
                    })
                    .text(|ctx: R<Self>| *ctx.counter), // .class(HELLO),
            )
            .child(e::div().id(RELOAD_ID).text(reload_tests::VALUE))
            .child(
                e::img()
                    .src(natrix::asset!("../assets/logo.png"))
                    .id(IMG_ID),
            )
    }
}

fn main() {
    natrix::mount(HelloWorld { counter: 0 });
}

#[cfg(test)]
mod driver_tests {
    use std::time::{Duration, Instant};

    use thirtyfour::{By, ChromiumLikeCapabilities, DesiredCapabilities, WebDriver};
    use tokio::time::sleep;

    use crate::{BUTTON_ID, HELLO_ID, HELLO_TEXT, IMG_ID, PANIC_ID};

    async fn create_client() -> WebDriver {
        let url = if cfg!(feature = "build_test") {
            "http://localhost:8000/dist/"
        } else {
            "http://localhost:8000"
        };

        let mut caps = DesiredCapabilities::chrome();
        caps.set_headless().unwrap();

        let driver = WebDriver::new("http://localhost:9999", caps)
            .await
            .expect("Failed to connect to chrome driver");

        let start = Instant::now();
        loop {
            let res = driver.get(url).await;
            sleep(Duration::from_millis(100)).await;
            if res.is_ok() {
                break;
            }
            if start.elapsed().as_secs() > 20 {
                panic!("Loading URL took too long");
            }
        }

        let start = Instant::now();
        let mut last_refresh = Instant::now();
        loop {
            let element = driver.find(By::Id(HELLO_ID)).await;
            sleep(Duration::from_millis(100)).await;
            if element.is_ok() {
                break;
            }
            if start.elapsed().as_secs() > 20 {
                panic!("Loading WASM took too long");
            } else if last_refresh.elapsed().as_secs() > 3 {
                println!("Loading WASM taking too long - trying a refresh");
                driver.get(url).await.unwrap();
                sleep(Duration::from_millis(300)).await;
                last_refresh = Instant::now();
            }
        }

        driver
    }

    #[tokio::test]
    async fn loading_framework_works() {
        let client = create_client().await;
        let element = client.find(By::Id(HELLO_ID)).await.unwrap();
        let text = element.text().await.unwrap();
        assert_eq!(text, HELLO_TEXT);
    }

    // #[tokio::test]
    // async fn primary_global_css() {
    //     let client = create_client().await;
    //     let element = client.find(By::Id(HELLO_ID)).await.unwrap();
    //     let text = element.css_value("background-color").await.unwrap();
    //     assert_eq!(text, "rgba(1, 2, 3, 1)");
    // }
    //
    // #[tokio::test]
    // async fn global_css_class() {
    //     let client = create_client().await;
    //     let element = client.find(By::Id(HELLO_ID)).await.unwrap();
    //     let text = element.css_value("width").await.unwrap();
    //     assert_eq!(text, "100px");
    // }
    //
    // #[tokio::test]
    // async fn scoped_css() {
    //     let client = create_client().await;
    //     let element = client.find(By::Id(HELLO_ID)).await.unwrap();
    //     let text = element.css_value("height").await.unwrap();
    //     assert_eq!(text, "300px");
    // }
    //
    // #[tokio::test]
    // async fn inline_style() {
    //     let client = create_client().await;
    //     let element = client.find(By::Id(HELLO_ID)).await.unwrap();
    //     let text = element.css_value("margin").await.unwrap();
    //     assert_eq!(text, "1px 2px 3px 4px");
    // }

    #[tokio::test]
    async fn simple_dep() {
        let client = create_client().await;
        let element = client
            .find(By::Id(integration_tests_dependency::DEP_ID))
            .await
            .unwrap();
        let text = element.text().await.unwrap();
        assert_eq!(text, integration_tests_dependency::DEP_TEXT);
    }

    // #[tokio::test]
    // async fn dep_global_css() {
    //     let client = create_client().await;
    //     let element = client.find(By::Id(HELLO_ID)).await.unwrap();
    //     let text = element.css_value("color").await.unwrap();
    //     assert_eq!(text, "rgba(9, 8, 7, 1)");
    // }
    //
    // #[tokio::test]
    // async fn dep_scoped_css() {
    //     let client = create_client().await;
    //     let element = client
    //         .find(By::Id(integration_tests_dependency::DEP_ID))
    //         .await
    //         .unwrap();
    //     let text = element.css_value("height").await.unwrap();
    //     assert_eq!(text, "600px");
    // }

    #[tokio::test]
    async fn panic_button() {
        let client = create_client().await;

        let panic_button = client.find(By::Id(PANIC_ID)).await.unwrap();
        let button = client.find(By::Id(BUTTON_ID)).await.unwrap();

        button.click().await.unwrap();
        let text = button.text().await.unwrap();
        assert_eq!(text, "1");

        panic_button.click().await.unwrap();
        button.click().await.unwrap();
        let text = button.text().await.unwrap();
        assert_eq!(
            text, "1",
            "Panic should have prevented further rust execution"
        );
    }

    // #[tokio::test]
    // async fn dynamic_class() {
    //     let client = create_client().await;
    //     let element = client.find(By::Id(HELLO_ID)).await.unwrap();
    //     let text = element.css_value("padding").await.unwrap();
    //     assert_eq!(text, "100px");
    // }

    #[tokio::test]
    async fn assets() {
        let client = create_client().await;
        let element = client.find(By::Id(IMG_ID)).await.unwrap();
        let rect = element.rect().await.unwrap();
        let width = rect.width;

        assert!(width >= 100.0, "Img width too small {width}");
    }

    // #[tokio::test]
    // async fn dynamic_css_var() {
    //     let client = create_client().await;
    //     let element = client.find(By::Id(HELLO_ID)).await.unwrap();
    //
    //     let font_size = element.css_value("font-size").await.unwrap();
    //     assert_eq!(font_size, "10px");
    // }

    // #[tokio::test]
    // async fn concat_asset_works() {
    //     let client = create_client().await;
    //     let element = client.find(By::Id(HELLO_ID)).await.unwrap();
    //     let text = element.css_value("background-image").await.unwrap();
    //     assert!(
    //         text.contains("logo.png"),
    //         "{text} expected to point to .*-logo.png"
    //     );
    // }

    #[tokio::test]
    #[cfg(not(feature = "build_test"))]
    async fn reload() {
        use crate::{RELOAD_ID, reload_tests};

        if option_env!("TEST_KIND_BUILD").is_some() {
            return;
        }

        let client = create_client().await;
        let element = client.find(By::Id(RELOAD_ID)).await.unwrap();

        let text = element.text().await.unwrap();
        assert_eq!(text, reload_tests::VALUE);

        let new_text = format!("{}E", reload_tests::VALUE);
        std::fs::write(
            "src/reload_tests.rs",
            format!("pub const VALUE: &str = \"{new_text}\";\n"),
        )
        .unwrap();

        // Wait for the file to be reloaded

        let start = Instant::now();
        loop {
            sleep(Duration::from_millis(100)).await;
            if let Ok(element) = client.find(By::Id(RELOAD_ID)).await {
                if let Ok(text) = element.text().await {
                    if text == new_text {
                        break;
                    }
                }
            }

            if start.elapsed().as_secs() > 20 {
                panic!("Reloading took too long");
            }
        }

        // Reset the file to its original state
        std::fs::write(
            "src/reload_tests.rs",
            format!("pub const VALUE: &str = \"{}\";\n", reload_tests::VALUE),
        )
        .unwrap();

        let start = Instant::now();
        loop {
            sleep(Duration::from_millis(100)).await;
            if let Ok(element) = client.find(By::Id(RELOAD_ID)).await {
                if let Ok(text) = element.text().await {
                    if text == reload_tests::VALUE {
                        break;
                    }
                }
            }
            if start.elapsed().as_secs() > 20 {
                panic!("Reloading took too long");
            }
        }
    }
}

#[cfg(test)]
#[cfg(feature = "build_test")]
mod dist_tests {
    // use crate::I_AM_NOT_USED;

    // #[test]
    // fn unused_css() {
    //     let css_file = std::fs::read_to_string("./dist/styles.css").unwrap();
    //     assert!(
    //         !css_file.contains(I_AM_NOT_USED),
    //         "Unused CSS class should not be present in the dist file"
    //     );
    // }

    #[test]
    fn duplicate_assets_calls() {
        let mut amount_logo = 0;
        for file in std::fs::read_dir("./dist").unwrap() {
            let file = file.unwrap();
            if file.file_name().to_str().unwrap().contains("logo.png") {
                amount_logo += 1;
            }
        }

        assert_eq!(
            amount_logo, 1,
            "There should be only one logo.png in the dist folder, even if included multiple times"
        );
    }
}
