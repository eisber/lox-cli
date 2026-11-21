use assert_cmd::Command;
use httpmock::{Method::GET, MockServer};
use serde_json::Value;
use std::fs;
use std::process::Output;
use std::time::Duration;
use tempfile::TempDir;

const UUID: &str = "01234567-0123-0123-0123456789abcdef";
const GET_PATH: &str = "/jdev/sps/io/01234567-0123-0123-0123456789abcdef";

struct Sandbox {
    root: TempDir,
}

impl Sandbox {
    fn new(host: &str) -> Self {
        let root = tempfile::Builder::new()
            .prefix(".clock-review-")
            .tempdir_in(env!("CARGO_MANIFEST_DIR"))
            .unwrap();
        fs::create_dir(root.path().join("home")).unwrap();
        fs::write(
            root.path().join("config.yaml"),
            format!("host: {host}\nuser: review\npass: synthetic-only\nverify_ssl: true\ntelemetry: false\n"),
        ).unwrap();
        Self { root }
    }

    fn run(&self, time: bool, verbosity: bool, json: bool) -> Output {
        let root = self.root.path();
        let mut command = Command::cargo_bin("lox").unwrap();
        command
            .current_dir(root)
            .env("LOX_CONFIG", root.join("config.yaml"))
            .env("LOX_HOME", root.join("home"))
            .env("HOME", root.join("home"))
            .env("USERPROFILE", root.join("home"))
            .env("LOX_TELEMETRY", "0")
            .env_remove("HOMEDRIVE")
            .env_remove("HOMEPATH")
            .env_remove("LOX_HOST")
            .env_remove("LOX_USER")
            .env_remove("LOX_PASS")
            .env_remove("LOX_SERIAL")
            .env("HTTP_PROXY", "")
            .env("HTTPS_PROXY", "")
            .env("ALL_PROXY", "")
            .env("NO_PROXY", "127.0.0.1,localhost")
            .timeout(Duration::from_secs(15))
            .arg("--quiet");
        if verbosity {
            command.arg("-vv");
        }
        if json {
            command.arg("--json");
        }
        if time {
            command.args(["live", "time"]);
        } else {
            command.args(["live", "get", UUID]);
        }
        command.output().unwrap()
    }
}

#[test]
fn legacy_vv_raw_body_is_no_longer_preserved() {
    let server = MockServer::start();
    let sandbox = Sandbox::new(&server.base_url());
    for path in [GET_PATH, "/jdev/sys/date", "/jdev/sys/time"] {
        server.mock(|when, then| {
            when.method(GET).path(path);
            then.body(r#"{"LL":{"value":"fixture-only","Code":200}}"#);
        });
    }
    let reference = sandbox.run(false, true, false);
    let migrated = sandbox.run(true, true, false);
    assert!(reference.status.success());
    assert!(migrated.status.success());
    let old = String::from_utf8_lossy(&reference.stderr);
    let new = String::from_utf8_lossy(&migrated.stderr);
    println!("REFERENCE unchanged LoxClient::get_json -vv:\n{old}");
    println!("MIGRATED ClockDriver -vv:\n{new}");
    assert!(old.contains(r#"body: {"LL":{"value":"fixture-only","Code":200}}"#));
    assert!(
        new.contains(r#"body: {"LL":{"value":"fixture-only","Code":200}}"#),
        "compatibility regression: legacy -vv body output removed"
    );
}

#[test]
fn legacy_vv_lossy_utf8_success_is_no_longer_preserved() {
    let server = MockServer::start();
    let sandbox = Sandbox::new(&server.base_url());
    let body = b"{\"LL\":{\"value\":\"fixture-\xff\",\"Code\":200}}";
    for path in [GET_PATH, "/jdev/sys/date", "/jdev/sys/time"] {
        server.mock(|when, then| {
            when.method(GET).path(path);
            then.body(body);
        });
    }
    let reference = sandbox.run(false, true, false);
    let migrated = sandbox.run(true, true, false);
    println!(
        "REFERENCE unchanged LoxClient::get_json -vv: status={:?}, stdout={}, stderr={}",
        reference.status,
        String::from_utf8_lossy(&reference.stdout),
        String::from_utf8_lossy(&reference.stderr)
    );
    println!(
        "MIGRATED ClockDriver -vv: status={:?}, stdout={}, stderr={}",
        migrated.status,
        String::from_utf8_lossy(&migrated.stdout),
        String::from_utf8_lossy(&migrated.stderr)
    );
    assert!(
        reference.status.success(),
        "baseline reference must succeed"
    );
    assert_eq!(
        String::from_utf8_lossy(&reference.stdout),
        "fixture-\u{fffd}\n"
    );
    assert!(
        migrated.status.success(),
        "compatibility regression: -vv legacy text decode now fails"
    );
}

#[test]
fn refused_connection_error_category_still_matches_legacy() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let host = format!("http://{}", listener.local_addr().unwrap());
    drop(listener);
    let sandbox = Sandbox::new(&host);
    let reference = sandbox.run(false, false, true);
    let migrated = sandbox.run(true, false, true);
    assert_eq!(reference.status.code(), Some(1));
    assert_eq!(migrated.status.code(), Some(1));
    let old: Value = serde_json::from_slice(&reference.stdout).unwrap();
    let new: Value = serde_json::from_slice(&migrated.stdout).unwrap();
    println!("REFERENCE refused connection: {old}");
    println!("MIGRATED refused connection: {new}");
    assert_eq!(old["error"], new["error"]);
}

#[test]
fn truncated_response_body_error_category_still_matches_legacy() {
    use std::io::{Read, Write};
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let host = format!("http://{}", listener.local_addr().unwrap());
    let stop = Arc::new(AtomicBool::new(false));
    let halt = stop.clone();
    let worker = std::thread::spawn(move || {
        let deadline = std::time::Instant::now() + Duration::from_secs(30);
        let mut hits = 0;
        while !halt.load(Ordering::Relaxed) && std::time::Instant::now() < deadline {
            match listener.accept() {
                Ok((mut socket, _)) => {
                    socket.set_nonblocking(false).unwrap();
                    socket
                        .set_read_timeout(Some(Duration::from_secs(2)))
                        .unwrap();
                    let mut request = Vec::new();
                    while !request.ends_with(b"\r\n\r\n") && request.len() < 8192 {
                        let mut byte = [0];
                        socket.read_exact(&mut byte).unwrap();
                        request.push(byte[0]);
                    }
                    socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 1000\r\nConnection: close\r\n\r\n{\"LL\":").unwrap();
                    hits += 1;
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(2))
                }
                Err(e) => panic!("fixture accept error: {e}"),
            }
        }
        hits
    });
    let sandbox = Sandbox::new(&host);
    let reference = sandbox.run(false, false, true);
    let migrated = sandbox.run(true, false, true);
    stop.store(true, Ordering::Relaxed);
    let hits = worker.join().unwrap();
    let old: Value = serde_json::from_slice(&reference.stdout).unwrap();
    let new: Value = serde_json::from_slice(&migrated.stdout).unwrap();
    println!("REFERENCE truncated response: {old}");
    println!("MIGRATED truncated response: {new}");
    assert_eq!(hits, 8);
    assert_eq!(reference.status.code(), Some(1));
    assert_eq!(migrated.status.code(), Some(1));
    assert_eq!(
        old["error"], new["error"],
        "body transport category changed"
    );
}

#[test]
fn legacy_vv_utf8_body_truncation_matches_legacy_transport() {
    let server = MockServer::start();
    let sandbox = Sandbox::new(&server.base_url());
    let body = serde_json::json!({"LL":{"value":"é".repeat(300),"Code":200}}).to_string();
    for path in [GET_PATH, "/jdev/sys/date", "/jdev/sys/time"] {
        server.mock(|when, then| {
            when.method(GET).path(path);
            then.body(&body);
        });
    }
    let reference = sandbox.run(false, true, false);
    let migrated = sandbox.run(true, true, false);
    assert!(reference.status.success());
    assert!(migrated.status.success());
    let old = String::from_utf8_lossy(&reference.stderr);
    let new = String::from_utf8_lossy(&migrated.stderr);
    let old_bodies: Vec<_> = old
        .lines()
        .filter(|line| line.starts_with("  body:"))
        .collect();
    let new_bodies: Vec<_> = new
        .lines()
        .filter(|line| line.starts_with("  body:"))
        .collect();
    assert_eq!(old_bodies.len(), 1);
    assert!(old_bodies[0].ends_with(&format!("… ({} bytes total)", body.len())));
    assert_eq!(new_bodies, vec![old_bodies[0]; 2]);
}

#[test]
fn legacy_vv_malformed_json_keeps_generic_category_retries_and_body_output() {
    let server = MockServer::start();
    let sandbox = Sandbox::new(&server.base_url());
    let reference_request = server.mock(|when, then| {
        when.method(GET).path(GET_PATH);
        then.body("{");
    });
    let date = server.mock(|when, then| {
        when.method(GET).path("/jdev/sys/date");
        then.body("{");
    });
    let time = server.mock(|when, then| {
        when.method(GET).path("/jdev/sys/time");
        then.body("{}");
    });
    let reference = sandbox.run(false, true, true);
    let migrated = sandbox.run(true, true, true);
    for output in [&reference, &migrated] {
        assert_eq!(output.status.code(), Some(1));
        let error: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(error["error"], "error");
        assert!(error.get("date").is_none());
        assert_eq!(
            String::from_utf8_lossy(&output.stderr)
                .matches("  body: {")
                .count(),
            4
        );
    }
    reference_request.assert_hits(4);
    date.assert_hits(4);
    time.assert_hits(0);
}
