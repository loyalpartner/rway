// End-to-end test simulating waybar's exact Subscribe flow.
// This test creates a real IPC server, connects a client, sends Subscribe,
// reads the response, and verifies the format matches what waybar expects.

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

use rway_ipc::protocol::{self, HEADER_SIZE};

fn temp_socket_path(name: &str) -> PathBuf {
    PathBuf::from(format!(
        "/tmp/rway-ipc-e2e-{}-{}.sock",
        std::process::id(),
        name
    ))
}

/// Simulate exactly what waybar does: connect, send Subscribe, read reply.
#[test]
fn subscribe_roundtrip_matches_sway_format() {
    let path = temp_socket_path("subscribe");
    let _ = std::fs::remove_file(&path);

    let server = rway_ipc::IpcServer::new(path.clone()).unwrap();

    // Client: connect and send Subscribe message (like waybar does)
    let mut client = UnixStream::connect(&path).unwrap();
    client
        .set_read_timeout(Some(std::time::Duration::from_secs(1)))
        .unwrap();

    let subscribe_payload = b"[\"workspace\"]";
    let request = protocol::encode_message(2, subscribe_payload); // type=2 = SUBSCRIBE
    client.write_all(&request).unwrap();

    // Server: accept connection, read request, send reply
    let mut server_stream = server.try_accept().unwrap();
    server_stream
        .set_read_timeout(Some(std::time::Duration::from_secs(1)))
        .unwrap();

    // Read request header
    let mut header_buf = [0u8; HEADER_SIZE];
    server_stream.read_exact(&mut header_buf).unwrap();
    let (payload_len, msg_type) = protocol::decode_header(&header_buf).unwrap();
    assert_eq!(msg_type, 2, "request type should be SUBSCRIBE");

    // Read request payload
    let mut payload = vec![0u8; payload_len as usize];
    server_stream.read_exact(&mut payload).unwrap();
    assert_eq!(payload, b"[\"workspace\"]");

    // Send reply (exactly how our ipc.rs does it)
    let reply = protocol::encode_reply(
        protocol::MessageType::Subscribe,
        &serde_json::json!({"success": true}),
    );
    eprintln!("Reply bytes ({} total): {:?}", reply.len(), &reply);
    server_stream.write_all(&reply).unwrap();
    server_stream.flush().unwrap();

    // Client: read reply header
    let mut resp_header = [0u8; HEADER_SIZE];
    client.read_exact(&mut resp_header).unwrap();
    let (resp_len, resp_type) = protocol::decode_header(&resp_header).unwrap();

    eprintln!("Response type: {}, payload_len: {}", resp_type, resp_len);

    // Verify response type matches SUBSCRIBE (2)
    assert_eq!(resp_type, 2, "response type must be SUBSCRIBE (2)");

    // Read reply payload
    let mut resp_payload = vec![0u8; resp_len as usize];
    client.read_exact(&mut resp_payload).unwrap();

    let resp_str = String::from_utf8(resp_payload).unwrap();
    eprintln!("Response payload: {}", resp_str);

    // Parse and verify JSON
    let resp_json: serde_json::Value = serde_json::from_str(&resp_str).unwrap();
    assert_eq!(resp_json["success"], true, "success must be true");

    // Verify the connection is still open (server can write events)
    let event = protocol::encode_event(
        protocol::EventType::Workspace,
        &serde_json::json!({"change": "focus"}),
    );
    server_stream.write_all(&event).unwrap();

    // Client reads the event
    let mut event_header = [0u8; HEADER_SIZE];
    client.read_exact(&mut event_header).unwrap();
    let (event_len, event_type) = protocol::decode_header(&event_header).unwrap();

    // Event type has high bit set
    assert_eq!(
        event_type, 0x80000000,
        "workspace event type should be 0x80000000"
    );
    assert!(event_len > 0);

    drop(server);
}

/// Test that the Subscribe reply payload is valid JSON that waybar can parse.
#[test]
fn subscribe_reply_payload_format() {
    let reply = protocol::encode_reply(
        protocol::MessageType::Subscribe,
        &serde_json::json!({"success": true}),
    );

    // Verify header
    assert_eq!(&reply[0..6], b"i3-ipc");

    let payload_len = u32::from_le_bytes(reply[6..10].try_into().unwrap());
    let msg_type = u32::from_le_bytes(reply[10..14].try_into().unwrap());

    assert_eq!(msg_type, 2);
    assert!(payload_len > 0);

    let payload = &reply[14..14 + payload_len as usize];
    let payload_str = std::str::from_utf8(payload).unwrap();
    eprintln!("Subscribe reply payload: '{}'", payload_str);

    // Must be parseable JSON with "success": true
    let json: serde_json::Value = serde_json::from_str(payload_str).unwrap();
    assert_eq!(json, serde_json::json!({"success": true}));
}
