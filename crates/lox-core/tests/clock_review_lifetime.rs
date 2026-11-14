#![cfg(feature = "runtime-blocking")]

use lox_core::runtime::{Cancellation, ClockClient, ClockDriver, ClockOptions};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::time::{Duration, Instant};

fn options(host: String) -> ClockOptions {
    let mut options = ClockOptions::new(host, "review", "synthetic-only");
    options.use_system_proxy = false;
    options.timeout = Duration::from_secs(3);
    options.shutdown_timeout = Duration::from_secs(2);
    options.retry_delays.clear();
    options
}

fn headers(socket: &mut TcpStream) {
    let mut bytes = Vec::new();
    while !bytes.ends_with(b"\r\n\r\n") && bytes.len() < 8192 {
        let mut byte = [0];
        socket.read_exact(&mut byte).unwrap();
        bytes.push(byte[0]);
    }
}

fn eof(socket: &mut TcpStream) -> bool {
    match socket.read(&mut [0]) {
        Ok(0) => true,
        Err(e)
            if matches!(
                e.kind(),
                std::io::ErrorKind::ConnectionReset | std::io::ErrorKind::ConnectionAborted
            ) =>
        {
            true
        }
        _ => false,
    }
}

async fn receive<T>(receiver: &mpsc::Receiver<T>) -> T {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match receiver.try_recv() {
            Ok(value) => return value,
            Err(mpsc::TryRecvError::Empty) if Instant::now() < deadline => {
                tokio::time::sleep(Duration::from_millis(2)).await
            }
            _ => panic!("fixture deadline/disconnect"),
        }
    }
}

#[tokio::test(flavor = "current_thread")]
async fn dropping_a_running_async_read_without_request_close_disconnects_the_socket() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let host = format!("http://{}", listener.local_addr().unwrap());
    let (start_tx, started) = mpsc::sync_channel(1);
    let (end_tx, ended) = mpsc::sync_channel(1);
    let worker = std::thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        headers(&mut socket);
        start_tx.send(()).unwrap();
        end_tx.send(eof(&mut socket)).unwrap();
    });
    let client = ClockClient::new(options(host)).unwrap();
    let handle = client.clone();
    let task = tokio::spawn(async move { handle.read_clock(&Cancellation::new()).await });
    receive(&started).await;
    task.abort();
    assert!(task.await.unwrap_err().is_cancelled());
    assert!(receive(&ended).await);
    assert!(!client.is_closed());
    assert_eq!(client.active_operations(), 0);
    client.close().await.unwrap();
    worker.join().unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn dropping_an_unclosed_owned_driver_under_tokio_disconnects_a_pooled_socket() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let host = format!("http://{}", listener.local_addr().unwrap());
    let (end_tx, ended) = mpsc::sync_channel(1);
    let worker = std::thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        for _ in 0..2 {
            headers(&mut socket);
            let body = r#"{"LL":{"value":"fixture","Code":200}}"#;
            write!(
                socket,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{body}",
                body.len()
            )
            .unwrap();
        }
        end_tx.send(eof(&mut socket)).unwrap();
    });
    let driver = ClockDriver::new(options(host)).unwrap();
    driver
        .read_clock(&Cancellation::new())
        .unwrap()
        .validate()
        .unwrap();
    let start = Instant::now();
    drop(driver);
    assert!(start.elapsed() < Duration::from_secs(2));
    assert!(receive(&ended).await);
    worker.join().unwrap();
}
