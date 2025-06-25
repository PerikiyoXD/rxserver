use crate::server::pipeline::handle_connection;
use tokio::net::TcpListener;

pub async fn run_tcp_server(addr: &str) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    println!("Listening on {}", addr);

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("Accepted connection from {}", addr);
        tokio::spawn(handle_connection(socket));
    }
}
