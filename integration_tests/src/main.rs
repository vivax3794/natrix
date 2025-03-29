use natrix::prelude::*;

#[derive(Component)]
struct HelloWorld;

impl Component for HelloWorld {
    fn render() -> impl Element<Self::Data> {
        e::h1().text("HELLO WORLD").id("HELLO")
    }
}

fn main() {
    mount_component(HelloWorld, "mount");
}

#[cfg(test)]
mod tests {
    use std::sync::OnceLock;
    use std::time::Duration;

    use thirtyfour::{By, ChromiumLikeCapabilities, DesiredCapabilities, WebDriver};
    use tokio::time::sleep;

    async fn create_client() -> WebDriver {
        let mut caps = DesiredCapabilities::chrome();
        caps.set_headless().unwrap();
        let driver = WebDriver::new("http://localhost:9999", caps)
            .await
            .expect("Failed to connect to chrome driver");

        driver
            .get("http://localhost:4444")
            .await
            .expect("Failed to goto site");
        sleep(Duration::from_secs(1)).await;

        driver
    }

    static CLIENT: OnceLock<WebDriver> = OnceLock::new();

    async fn get_client() -> &'static WebDriver {
        if let Some(client) = CLIENT.get() {
            client
        } else {
            let client = create_client().await;
            CLIENT.set(client).unwrap();
            CLIENT.get().unwrap()
        }
    }

    #[tokio::test]
    async fn loading_framework_works() {
        let client = get_client().await;
        let element = client.find(By::Id("HELLO")).await.unwrap();
        let text = element.text().await.unwrap();
        assert_eq!(text, "HELLO WORLD");
    }
}
