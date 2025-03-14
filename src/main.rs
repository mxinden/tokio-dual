use socket2::{Domain, Socket, Type};
use std::net::{SocketAddr, TcpListener};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener as TokioTcpListener, TcpStream};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Create a dual-stack socket
    let socket = Socket::new(Domain::IPV6, Type::STREAM, Some(socket2::Protocol::TCP))?;
    socket.set_only_v6(false)?;

    // Bind to IPv6 localhost (this also accepts IPv4 due to set_only_v6(false))
    let address: SocketAddr = "[::]:0".parse().unwrap();
    socket.bind(&address.into())?;
    socket.listen(128)?;

    // Convert our Socket to a std::net::TcpListener, then a tokio::TcpListener
    let std_listener: TcpListener = socket.into();
    std_listener.set_nonblocking(true)?;
    let listener: TokioTcpListener = TokioTcpListener::from_std(std_listener)?;

    let address = listener.local_addr().unwrap();

    println!("Server listening on {address:?} (dual-stack).");

    // Spawn the server in a background task
    let server_task = tokio::spawn(server(listener));

    // Give the server a little time to process/log
    sleep(Duration::from_secs(1)).await;

    // --- Connect and exchange data via IPv6 ---
    {
        let mut v6_stream = TcpStream::connect(format!("[::1]:{}", address.port())).await?;
        println!("Connected via IPv6!");
        v6_stream.write_all(b"ping").await?;
        println!("Sent 'ping' (IPv6)");

        let mut buffer = [0u8; 4];
        v6_stream.read_exact(&mut buffer).await?;
        println!(
            "IPv6 client received: {:?}",
            String::from_utf8_lossy(&buffer)
        );
    }

    // --- Connect and exchange data via IPv4 ---
    {
        let mut v4_stream = TcpStream::connect(format!("127.0.0.1:{}", address.port()))
            .await
            .unwrap();
        println!("Connected via IPv4!");
        v4_stream.write_all(b"ping").await?;
        println!("Sent 'ping' (IPv4)");

        let mut buffer = [0u8; 4];
        v4_stream.read_exact(&mut buffer).await?;
        println!(
            "IPv4 client received: {:?}",
            String::from_utf8_lossy(&buffer)
        );
    }

    // Give the server a little time to process/log
    sleep(Duration::from_secs(1)).await;

    // Stop the server. In real code, you might use a graceful shutdown strategy.
    server_task.abort();
    println!("Done.");

    Ok(())
}

async fn server(listener: TokioTcpListener) -> std::io::Result<()> {
    loop {
        // Accept a connection (this call is awaitable)
        let (mut stream, addr) = match listener.accept().await {
            Ok(conn) => conn,
            Err(e) => {
                eprintln!("accept error: {}", e);
                continue;
            }
        };
        println!("Accepted connection from {addr}");

        // Handle the connection right here (no separate spawn).
        let mut buffer = [0u8; 4];
        if let Ok(_) = stream.read_exact(&mut buffer).await {
            if &buffer == b"ping" {
                println!("Server received 'ping' -> sending 'pong'.");
                if let Err(e) = stream.write_all(b"pong").await {
                    eprintln!("Failed to send 'pong': {}", e);
                }
            } else {
                eprintln!("Received something other than 'ping': {:?}", &buffer);
            }
        }
    }
}
