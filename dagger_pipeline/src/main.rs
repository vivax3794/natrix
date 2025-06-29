//! Run the CI pipeline using dagger

use eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    dagger_sdk::connect(async |client| Ok(()))
        .await
        .map_err(Into::into)
}
