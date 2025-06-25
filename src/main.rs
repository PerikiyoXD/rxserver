mod protocol;
mod server;

use server::tcp::run_tcp_server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Starting X11 protocol server...");
    run_tcp_server("127.0.0.1:6000").await?;
    Ok(())
}
