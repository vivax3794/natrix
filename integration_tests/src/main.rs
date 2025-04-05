use natrix::prelude::*;

const HELLO_TEXT: &str = "HELLO WORLD, TEST TEST";
const HELLO_ID: &str = "HELLO";

global_css!(
    "
h1 {
    background-color: blue;
}
"
);

#[derive(Component)]
struct HelloWorld {
    counter: usize,
}

impl Component for HelloWorld {
    fn render() -> impl Element<Self::Data> {
        e::div()
            .child(e::h1().text(HELLO_TEXT).id(HELLO_ID))
            .child(C(integration_tests_dependency::DepComp))
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

    use crate::{HELLO_ID, HELLO_TEXT};

    async fn create_client() -> WebDriver {
        let mut caps = DesiredCapabilities::chrome();
        caps.set_headless().unwrap();
        let driver = WebDriver::new("http://localhost:9999", caps)
            .await
            .expect("Failed to connect to chrome driver");

        loop {
            let res = driver.get("http://localhost:8000").await;
            sleep(Duration::from_secs(1)).await;
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
    async fn simple_dep() {
        let client = create_client().await;
        let element = client
            .find(By::Id(integration_tests_dependency::DEP_ID))
            .await
            .unwrap();
        let text = element.text().await.unwrap();
        assert_eq!(text, integration_tests_dependency::DEP_TEXT);
    }
}
