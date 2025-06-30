//! Run the CI pipeline using dagger

// TODO: `cargo-sweep` caches.

/// Common items
mod prelude {
    use dagger_sdk::HostDirectoryOpts;
    pub use dagger_sdk::{Container, ContainerWithExecOptsBuilder, Directory, Query, ReturnType};
    pub use eyre::Result;

    /// Result of executing a command in a container
    #[derive(Debug)]
    pub struct ExecutionResult {
        /// Exit code of the command
        pub exit_code: isize,
        /// Standard output of the command
        pub stdout: String,
        /// Standard error of the command
        pub stderr: String,
    }

    /// Extension trait for dagger containers
    pub trait ContainerExtension {
        /// Copy in the workspace and setup target cache
        fn with_workspace(&self, client: &Query) -> Container;

        /// Execute a command with default options (`ReturnType::Any`)
        fn with_exec_any(&self, args: Vec<&str>) -> Result<Container>;

        /// Get the full execution result (exit code, stdout, stderr) from the current container
        async fn get_result(&self) -> Result<ExecutionResult>;
    }

    impl ContainerExtension for Container {
        // MAYBE: Allow to specify needed crates.
        fn with_workspace(&self, client: &Query) -> Container {
            let workspace = client.host().directory_opts(
                ".",
                HostDirectoryOpts {
                    exclude: Some(vec!["target", "dist", "book"]),
                    include: None,
                    no_cache: None,
                },
            );
            self.with_directory("/app", workspace)
                .with_workdir("/app")
                .with_mounted_cache("/app/target", client.cache_volume("rust-target"))
        }

        fn with_exec_any(&self, args: Vec<&str>) -> Result<Container> {
            Ok(self.with_exec_opts(
                args,
                ContainerWithExecOptsBuilder::default()
                    .expect(ReturnType::Any)
                    .build()?,
            ))
        }

        async fn get_result(&self) -> Result<ExecutionResult> {
            let (exit_code, stdout, stderr) =
                tokio::try_join!(self.exit_code(), self.stdout(), self.stderr(),)?;

            Ok(ExecutionResult {
                exit_code,
                stdout,
                stderr,
            })
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
        let reports = report::run_all_tests(&client).await?;
        let report = report::generate_report(&client, reports)?;
        report::serve_report(&client, report).await?;
        Ok(())
    })
    .await
    .map_err(Into::into)
}
