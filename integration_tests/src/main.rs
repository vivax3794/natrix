use natrix::prelude::*;

const HELLO_TEXT: &str = "HELLO WORLD, TEST TEST";
const HELLO_ID: &str = "HELLO";
const PANIC_ID: &str = "PANIC";
const BUTTON_ID: &str = "BUTTON";

global_css!("
    h1 {
        background-color: rgba(1,2,3,1);
    }
    .hello_world {
        width: 100px;
    }
");

scoped_css!("
    .hello {
        height: 300px;
    }
    .I_amNotUsed {
        height: 400px;
    }
");

#[derive(Component)]
struct NotUsed;

impl Component for NotUsed {
    type EmitMessage = NoMessages;
    type ReceiveMessage = NoMessages;
    fn render() -> impl Element<Self> {
        e::div().class(I_AM_NOT_USED)
    }
}

#[derive(Component)]
struct HelloWorld {
    counter: usize,
}

impl Component for HelloWorld {
    type EmitMessage = NoMessages;
    type ReceiveMessage = NoMessages;
    fn render() -> impl Element<Self> {
        e::div()
            .child(
                e::h1()
                    .text(HELLO_TEXT)
                    .id(HELLO_ID)
                    .class("hello_world")
                    .class(HELLO),
            )
            .child(C::new(integration_tests_dependency::DepComp))
            .child(
                e::button()
                    .id(PANIC_ID)
                    .on::<events::Click>(|_ctx: E<Self>, _| {
                        panic!("Panic button clicked!");
                    }),
            )
            .child(
                e::button()
                    .id(BUTTON_ID)
                    .on::<events::Click>(|ctx: E<Self>, _| {
                        *ctx.counter += 1;
                    })
                    .text(|ctx: R<Self>| *ctx.counter),
            )
    }
}

fn main() {
    mount(HelloWorld { counter: 0 });
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use thirtyfour::{By, ChromiumLikeCapabilities, DesiredCapabilities, WebDriver};
    use tokio::time::sleep;

    use crate::{BUTTON_ID, HELLO_ID, HELLO_TEXT, PANIC_ID};

    async fn create_client() -> WebDriver {
        let mut caps = DesiredCapabilities::chrome();
        caps.set_headless().unwrap();
        let driver = WebDriver::new("http://localhost:9999", caps)
            .await
            .expect("Failed to connect to chrome driver");

        loop {
            let res = driver.get("http://localhost:8000").await;
            sleep(Duration::from_millis(100)).await;
            if res.is_ok() {
                break;
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

    #[tokio::test]
    async fn primary_global_css() {
        let client = create_client().await;
        let element = client.find(By::Id(HELLO_ID)).await.unwrap();
        let text = element.css_value("background-color").await.unwrap();
        assert_eq!(text, "rgba(1, 2, 3, 1)");
    }

    #[tokio::test]
    async fn global_css_class() {
        let client = create_client().await;
        let element = client.find(By::Id(HELLO_ID)).await.unwrap();
        let text = element.css_value("width").await.unwrap();
        assert_eq!(text, "100px");
    }

    #[tokio::test]
    async fn scoped_css() {
        let client = create_client().await;
        let element = client.find(By::Id(HELLO_ID)).await.unwrap();
        let text = element.css_value("height").await.unwrap();
        assert_eq!(text, "300px");
    }

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

    #[tokio::test]
    async fn dep_global_css() {
        let client = create_client().await;
        let element = client.find(By::Id(HELLO_ID)).await.unwrap();
        let text = element.css_value("color").await.unwrap();
        assert_eq!(text, "rgba(9, 8, 7, 1)");
    }

    #[tokio::test]
    async fn dep_scoped_css() {
        let client = create_client().await;
        let element = client
            .find(By::Id(integration_tests_dependency::DEP_ID))
            .await
            .unwrap();
        let text = element.css_value("height").await.unwrap();
        assert_eq!(text, "600px");
    }

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
}
