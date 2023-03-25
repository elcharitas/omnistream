use omnistream::handler::search_handler;
use vercel_runtime::{run, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(search_handler).await
}
