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
    use std::thread::sleep;
    use std::time::Duration;

    use thirtyfour_sync::{By, DesiredCapabilities, WebDriver, WebDriverCommands};

    fn create_client() -> WebDriver {
        let mut caps = DesiredCapabilities::chrome();
        caps.set_headless().unwrap();
        let driver = WebDriver::new("http://localhost:9999", caps)
            .expect("Failed to connect to chrome driver");

        driver
            .get("http://localhost:4444")
            .expect("Failed to goto site");
        sleep(Duration::from_secs(1));

        driver
    }

    thread_local! {
        static CLIENT: WebDriver = create_client();
    }

    #[test]
    fn loading_framework_works() {
        CLIENT.with(|client| {
            let element = client.find_element(By::Id("HELLO")).unwrap();
            let text = element.text().unwrap();
            assert_eq!(text, "HELLO WORLD");
        })
    }
}
