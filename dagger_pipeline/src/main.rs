//! Run the CI pipeline using dagger

/// Common items
mod prelude {
    pub use dagger_sdk::{Container, Directory, Query};
    use dagger_sdk::{ContainerWithExecOptsBuilder, HostDirectoryOpts, ReturnType};
    pub use eyre::Result;
    use tokio::sync::Semaphore;

    /// Global mutexses and semphores
    pub struct GlobalState {
        /// Semaphore to hold during potentially expensive cpu actions
        cpu_work: Semaphore,
    }

    impl GlobalState {
        /// create a new version of the global state
        pub fn new() -> Self {
            Self {
                cpu_work: Semaphore::new(1),
            }
        }
    }

    /// Extension trait for dagger containers
    pub trait ContainerExtension {
        /// Run the given command while holding the mutex, forcing the sync point right away.
        async fn run_with_mutex(
            &self,
            state: &GlobalState,
            args: Vec<impl Into<String>>,
            fail_okay: bool,
        ) -> Result<Container>;

        /// Copy in the workspace and setup target cache
        fn with_workspace(&self, client: &Query) -> Container;
    }

    impl ContainerExtension for Container {
        async fn run_with_mutex(
            &self,
            state: &GlobalState,
            args: Vec<impl Into<String>>,
            fail_okay: bool,
        ) -> Result<Container> {
            let container = self.with_exec_opts(
                args,
                ContainerWithExecOptsBuilder::default()
                    .expect(if fail_okay {
                        ReturnType::Any
                    } else {
                        ReturnType::Success
                    })
                    .build()?,
            );

            {
                let lock = state.cpu_work.acquire().await?;
                container.sync().await?;
                drop(lock);
            }

            Ok(container)
        }

        fn with_workspace(&self, client: &Query) -> Container {
            let workspace = client.host().directory_opts(
                ".",
                HostDirectoryOpts {
                    exclude: Some(vec!["target", "dist", "book", "allure-report"]),
                    include: None,
                    no_cache: Some(true),
                },
            );
            self.with_directory("/app", workspace)
                .with_workdir("/app")
                .with_mounted_cache("/app/target", client.cache_volume("rust-target"))
        }
    }
}

mod base_images;
mod report;
mod targets;

use prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    dagger_sdk::connect(async |client| {
        let state = GlobalState::new();

        report::generate_report(&client, &state).await?;
        Ok(())
    })
    .await
    .map_err(Into::into)
}
