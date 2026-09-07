#![cfg(feature = "ssr")]

use std::{
    io::{Read, Write},
    net::TcpStream,
    thread,
    time::Duration,
};

use web::mail::{MailConfig, Mailer};

fn mailpit_get(path: &str) -> std::io::Result<String> {
    let mut stream = TcpStream::connect("127.0.0.1:8025")?;
    stream.write_all(
        format!(
            "GET {path} HTTP/1.1\r\nHost: 127.0.0.1:8025\r\nAccept-Encoding: identity\r\nConnection: close\r\n\r\n"
        )
        .as_bytes(),
    )?;
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    assert!(response.starts_with("HTTP/1.1 200"));
    let (headers, body) = response.split_once("\r\n\r\n").unwrap_or_default();
    if headers
        .to_ascii_lowercase()
        .contains("transfer-encoding: chunked")
    {
        decode_chunked(body)
    } else {
        Ok(body.to_owned())
    }
}

fn decode_chunked(mut encoded: &str) -> std::io::Result<String> {
    let mut decoded = String::new();
    loop {
        let (size, rest) = encoded.split_once("\r\n").ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "missing chunk size")
        })?;
        let size = usize::from_str_radix(size.split(';').next().unwrap_or_default(), 16)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
        if size == 0 {
            return Ok(decoded);
        }
        if rest.len() < size + 2 || !rest.is_char_boundary(size) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid chunk body",
            ));
        }
        decoded.push_str(&rest[..size]);
        encoded = &rest[size + 2..];
    }
}

fn messages_for(recipient: &str) -> std::io::Result<String> {
    let list: serde_json::Value = serde_json::from_str(&mailpit_get("/api/v1/messages")?)?;
    let mut messages = String::new();
    for summary in list["messages"].as_array().into_iter().flatten() {
        let belongs_to_recipient = summary["To"]
            .as_array()
            .into_iter()
            .flatten()
            .any(|address| address["Address"].as_str() == Some(recipient));
        if belongs_to_recipient && let Some(id) = summary["ID"].as_str() {
            messages.push_str(&mailpit_get(&format!("/api/v1/message/{id}"))?);
        }
    }
    Ok(messages)
}

#[tokio::test]
async fn verification_and_reset_messages_reach_local_mailpit() {
    let marker = format!("mailpit-{}@example.test", std::process::id());
    let mailer = Mailer::new(MailConfig {
        smtp_host: "127.0.0.1".into(),
        smtp_port: 1025,
        from: "DAC2 <no-reply@dac2.local>".into(),
        base_url: "http://127.0.0.1:3000".into(),
    });

    mailer
        .send_verification(&marker, "verification-marker")
        .await
        .expect("verification message reaches Mailpit");
    mailer
        .send_password_reset(&marker, "reset-marker")
        .await
        .expect("reset message reaches Mailpit");

    let mut response = String::new();
    for _ in 0..20 {
        response = messages_for(&marker).expect("Mailpit API and message bodies are reachable");
        if response.contains("verification-marker") && response.contains("reset-marker") {
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }

    assert!(response.contains(&marker));
    assert!(response.contains("verification-marker"));
    assert!(response.contains("reset-marker"));
}
