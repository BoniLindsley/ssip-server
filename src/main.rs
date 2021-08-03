use ssip_server;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

type ExitCode = i32;
type RunResult = Result<(), Box<dyn std::error::Error>>;

fn main() {
    let run_result = run();
    let exit_code = parse_run_result(run_result);
    std::process::exit(exit_code);
}

fn parse_run_result(result: RunResult) -> ExitCode {
    match result {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("{}", error);
            1
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let tokio_runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .build()?;
    tokio_runtime.block_on(async_run())
}

async fn async_run() -> Result<(), Box<dyn std::error::Error>> {
    let mut output_module = ssip_server::OutputModule::new()?;
    let message = "Press Control C to exit.";
    output_module.speak(message)?;
    eprintln!("{}", message);
    let listener =
        tokio::net::TcpListener::bind("127.0.0.1:6560").await?;
    loop {
        let (stream, address) = listener.accept().await?;
        tokio::spawn(async move {
            let result = process_connection(stream, address).await;
            parse_run_result(result)
        });
    }
}

async fn process_connection(
    mut stream: tokio::net::TcpStream,
    address: std::net::SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Accepted connection from {}.", address);
    let mut buffer = [0; 1024];
    let read_size = stream.read(&mut buffer).await?;
    if read_size == 0 {
        return Ok(());
    }
    let message = &mut buffer[..read_size];
    println!("Received: {}", String::from_utf8_lossy(&message[..]));
    let response = if message.starts_with(b"QUIT\r\n") {
        String::from("200 QUIT DONE\r\n")
    } else {
        String::from("500 Unexpected message.\r\n")
    };
    println!("Sending: {:?}", response);
    stream.write_all(response.as_bytes()).await?;
    stream.flush().await?;
    stream.shutdown().await?;
    Ok(())
}
