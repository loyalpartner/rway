// Sway IPC 二进制协议：i3-ipc magic + length + type + JSON payload
//
// 协议格式:
//   [6 字节: "i3-ipc"] [4 字节: payload_length (u32 LE)] [4 字节: message_type (u32 LE)] [payload 字节: JSON]
//
// 与 Sway/i3 的 wire 格式完全兼容，确保 swaymsg 和 waybar 可以直接使用。

use thiserror::Error;

/// Sway IPC 协议魔数，固定 6 字节
pub const MAGIC: &[u8; 6] = b"i3-ipc";

/// IPC 消息头大小（字节）：6(magic) + 4(length) + 4(type)
pub const HEADER_SIZE: usize = 14;

/// 协议错误类型
#[derive(Debug, Error, PartialEq)]
pub enum ProtocolError {
    #[error("魔数无效：期望 'i3-ipc'，实际收到 {0:?}")]
    InvalidMagic([u8; 6]),

    #[error("消息头长度不足：需要 14 字节，实际 {0} 字节")]
    HeaderTooShort(usize),
}

/// 客户端发往服务端的消息类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum MessageType {
    RunCommand = 0,
    GetWorkspaces = 1,
    Subscribe = 2,
    GetOutputs = 3,
    GetTree = 4,
    GetMarks = 5,
    GetBarConfig = 6,
    GetVersion = 7,
    GetBindingModes = 8,
    GetConfig = 9,
    SendTick = 10,
    Sync = 11,
    GetBindingState = 12,
    GetInputs = 100,
    GetSeats = 101,
}

/// 服务端推送的事件类型（最高位置 1 表示事件）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum EventType {
    Workspace = 0x80000000,
    Output = 0x80000001,
    Mode = 0x80000002,
    Window = 0x80000003,
    BarConfigUpdate = 0x80000004,
    Binding = 0x80000005,
    Shutdown = 0x80000006,
    Tick = 0x80000007,
    BarStateUpdate = 0x80000014,
    Input = 0x80000015,
}

/// 将消息编码为 Sway IPC wire 格式字节序列
///
/// # 参数
/// - `msg_type`: 消息类型编号（u32）
/// - `payload`: JSON payload 字节切片
///
/// # 返回
/// 完整的 IPC 消息字节序列
pub fn encode_message(msg_type: u32, payload: &[u8]) -> Vec<u8> {
    let payload_len = payload.len() as u32;
    let mut buf = Vec::with_capacity(HEADER_SIZE + payload.len());

    // 6 字节魔数
    buf.extend_from_slice(MAGIC);
    // 4 字节 payload 长度（小端序）
    buf.extend_from_slice(&payload_len.to_le_bytes());
    // 4 字节消息类型（小端序）
    buf.extend_from_slice(&msg_type.to_le_bytes());
    // payload 字节
    buf.extend_from_slice(payload);

    buf
}

/// 解析 14 字节的 IPC 消息头
///
/// # 参数
/// - `header`: 恰好 14 字节的消息头
///
/// # 返回
/// `Ok((payload_len, msg_type))` 或 `Err(ProtocolError)`
pub fn decode_header(header: &[u8; 14]) -> Result<(u32, u32), ProtocolError> {
    // 验证魔数
    let magic: [u8; 6] = header[0..6].try_into().unwrap();
    if &magic != MAGIC {
        return Err(ProtocolError::InvalidMagic(magic));
    }

    let payload_len = u32::from_le_bytes(header[6..10].try_into().unwrap());
    let msg_type = u32::from_le_bytes(header[10..14].try_into().unwrap());

    Ok((payload_len, msg_type))
}

/// 将服务端应答编码为 IPC wire 格式
///
/// # 参数
/// - `msg_type`: 应答类型（与请求类型相同编号）
/// - `payload`: serde_json::Value，将被序列化为 JSON
pub fn encode_reply(msg_type: MessageType, payload: &serde_json::Value) -> Vec<u8> {
    let json = serde_json::to_vec(payload).expect("serde_json 序列化不应失败");
    encode_message(msg_type as u32, &json)
}

/// 将服务端事件编码为 IPC wire 格式
///
/// # 参数
/// - `event_type`: 事件类型（最高位为 1）
/// - `payload`: serde_json::Value，将被序列化为 JSON
pub fn encode_event(event_type: EventType, payload: &serde_json::Value) -> Vec<u8> {
    let json = serde_json::to_vec(payload).expect("serde_json 序列化不应失败");
    encode_message(event_type as u32, &json)
}

// ─────────────────────────────────────────────────────────────────────────────
// 单元测试（TDD：先写测试，后写实现）
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ── encode_message ────────────────────────────────────────────────────────

    /// 测试基本编码：空 payload
    #[test]
    fn test_encode_message_empty_payload() {
        let msg = encode_message(0, b"");

        // 总长度 = 14 字节头（无 payload）
        assert_eq!(msg.len(), HEADER_SIZE);

        // 魔数正确
        assert_eq!(&msg[0..6], b"i3-ipc");

        // payload 长度为 0（小端序）
        assert_eq!(&msg[6..10], &0u32.to_le_bytes());

        // 消息类型为 0（小端序）
        assert_eq!(&msg[10..14], &0u32.to_le_bytes());
    }

    /// 测试带 payload 的编码
    #[test]
    fn test_encode_message_with_payload() {
        let payload = b"hello";
        let msg = encode_message(7, payload);

        assert_eq!(msg.len(), HEADER_SIZE + 5);
        assert_eq!(&msg[0..6], b"i3-ipc");
        assert_eq!(&msg[6..10], &5u32.to_le_bytes());
        assert_eq!(&msg[10..14], &7u32.to_le_bytes());
        assert_eq!(&msg[14..], b"hello");
    }

    /// 测试消息类型编号写入小端序正确
    #[test]
    fn test_encode_message_type_little_endian() {
        // 消息类型 0x0100_0000（大数值）
        let msg = encode_message(0x01000000, b"");
        let stored_type = u32::from_le_bytes(msg[10..14].try_into().unwrap());
        assert_eq!(stored_type, 0x01000000);
    }

    /// 测试事件类型（最高位为 1 的大数值）
    #[test]
    fn test_encode_message_event_type_high_bit() {
        let msg = encode_message(EventType::Workspace as u32, b"{}");
        let stored_type = u32::from_le_bytes(msg[10..14].try_into().unwrap());
        assert_eq!(stored_type, 0x80000000);
    }

    /// 测试大 payload（性能/边界测试）
    #[test]
    fn test_encode_message_large_payload() {
        let payload: Vec<u8> = vec![0x41u8; 100_000]; // 100KB 'A'
        let msg = encode_message(4, &payload);

        assert_eq!(msg.len(), HEADER_SIZE + 100_000);
        let stored_len = u32::from_le_bytes(msg[6..10].try_into().unwrap());
        assert_eq!(stored_len, 100_000);
    }

    // ── decode_header ─────────────────────────────────────────────────────────

    /// 测试合法消息头解析
    #[test]
    fn test_decode_header_valid() {
        let mut header = [0u8; 14];
        header[0..6].copy_from_slice(b"i3-ipc");
        header[6..10].copy_from_slice(&42u32.to_le_bytes()); // payload_len = 42
        header[10..14].copy_from_slice(&7u32.to_le_bytes()); // msg_type = 7

        let result = decode_header(&header);
        assert_eq!(result, Ok((42, 7)));
    }

    /// 测试零长度 payload 的消息头
    #[test]
    fn test_decode_header_zero_payload_len() {
        let mut header = [0u8; 14];
        header[0..6].copy_from_slice(b"i3-ipc");
        // payload_len = 0, msg_type = 0（已默认为 0）

        let result = decode_header(&header);
        assert_eq!(result, Ok((0, 0)));
    }

    /// 测试魔数错误时返回正确错误
    #[test]
    fn test_decode_header_invalid_magic() {
        let mut header = [0u8; 14];
        header[0..6].copy_from_slice(b"swayxx"); // 错误魔数
        header[6..10].copy_from_slice(&0u32.to_le_bytes());
        header[10..14].copy_from_slice(&0u32.to_le_bytes());

        let result = decode_header(&header);
        assert!(matches!(result, Err(ProtocolError::InvalidMagic(_))));
        if let Err(ProtocolError::InvalidMagic(magic)) = result {
            assert_eq!(&magic, b"swayxx");
        }
    }

    /// 测试魔数全零时返回错误
    #[test]
    fn test_decode_header_all_zero_magic() {
        let header = [0u8; 14];
        let result = decode_header(&header);
        assert!(matches!(result, Err(ProtocolError::InvalidMagic(_))));
    }

    /// 测试大端序 payload_len 不被接受（Sway 使用小端序）
    #[test]
    fn test_decode_header_large_msg_type() {
        // 事件类型 0x80000000（Workspace 事件）
        let mut header = [0u8; 14];
        header[0..6].copy_from_slice(b"i3-ipc");
        header[6..10].copy_from_slice(&2u32.to_le_bytes());
        header[10..14].copy_from_slice(&0x80000000u32.to_le_bytes());

        let result = decode_header(&header);
        assert_eq!(result, Ok((2, 0x80000000)));
    }

    // ── encode/decode 往返测试 ────────────────────────────────────────────────

    /// 测试编码后能正确解码（往返一致性）
    #[test]
    fn test_encode_decode_roundtrip() {
        let original_type: u32 = MessageType::GetVersion as u32;
        let original_payload = b"{\"test\":true}";

        let encoded = encode_message(original_type, original_payload);

        // 提取头部并解码
        let header: [u8; 14] = encoded[0..14].try_into().unwrap();
        let (payload_len, msg_type) = decode_header(&header).unwrap();

        assert_eq!(msg_type, original_type);
        assert_eq!(payload_len as usize, original_payload.len());
        assert_eq!(&encoded[14..14 + payload_len as usize], original_payload);
    }

    /// 测试所有 MessageType 的编码往返
    #[test]
    fn test_encode_decode_all_message_types() {
        let types = [
            MessageType::RunCommand,
            MessageType::GetWorkspaces,
            MessageType::Subscribe,
            MessageType::GetOutputs,
            MessageType::GetTree,
            MessageType::GetMarks,
            MessageType::GetBarConfig,
            MessageType::GetVersion,
            MessageType::GetBindingModes,
            MessageType::GetConfig,
            MessageType::SendTick,
            MessageType::Sync,
            MessageType::GetBindingState,
            MessageType::GetInputs,
            MessageType::GetSeats,
        ];

        for mt in types {
            let encoded = encode_message(mt as u32, b"{}");
            let header: [u8; 14] = encoded[0..14].try_into().unwrap();
            let (_, decoded_type) = decode_header(&header).unwrap();
            assert_eq!(decoded_type, mt as u32, "MessageType {:?} 往返失败", mt);
        }
    }

    // ── encode_reply ──────────────────────────────────────────────────────────

    /// 测试 encode_reply 生成正确格式的应答消息
    #[test]
    fn test_encode_reply_get_version() {
        let payload = json!({"major": 1, "minor": 0, "patch": 0});
        let encoded = encode_reply(MessageType::GetVersion, &payload);

        let header: [u8; 14] = encoded[0..14].try_into().unwrap();
        let (payload_len, msg_type) = decode_header(&header).unwrap();

        assert_eq!(msg_type, MessageType::GetVersion as u32);
        let json_bytes = &encoded[14..14 + payload_len as usize];
        let decoded: serde_json::Value = serde_json::from_slice(json_bytes).unwrap();
        assert_eq!(decoded["major"], 1);
    }

    /// 测试 encode_reply 使用空 JSON 对象
    #[test]
    fn test_encode_reply_empty_object() {
        let payload = json!({});
        let encoded = encode_reply(MessageType::RunCommand, &payload);

        assert_eq!(&encoded[0..6], b"i3-ipc");
        assert_eq!(&encoded[10..14], &0u32.to_le_bytes()); // RunCommand = 0
    }

    // ── encode_event ──────────────────────────────────────────────────────────

    /// 测试事件编码正确设置高位标志
    #[test]
    fn test_encode_event_workspace() {
        let payload = json!({"change": "focus"});
        let encoded = encode_event(EventType::Workspace, &payload);

        let header: [u8; 14] = encoded[0..14].try_into().unwrap();
        let (_, msg_type) = decode_header(&header).unwrap();

        assert_eq!(msg_type, 0x80000000);
    }

    /// 测试所有事件类型的编码
    #[test]
    fn test_encode_event_all_types() {
        let events = [
            (EventType::Workspace, 0x80000000u32),
            (EventType::Output, 0x80000001),
            (EventType::Mode, 0x80000002),
            (EventType::Window, 0x80000003),
            (EventType::BarConfigUpdate, 0x80000004),
            (EventType::Binding, 0x80000005),
            (EventType::Shutdown, 0x80000006),
            (EventType::Tick, 0x80000007),
            (EventType::BarStateUpdate, 0x80000014),
            (EventType::Input, 0x80000015),
        ];

        for (event_type, expected_id) in events {
            let encoded = encode_event(event_type, &json!({}));
            let header: [u8; 14] = encoded[0..14].try_into().unwrap();
            let (_, msg_type) = decode_header(&header).unwrap();
            assert_eq!(
                msg_type, expected_id,
                "EventType {:?} 编号不匹配",
                event_type
            );
        }
    }

    // ── 边界情况测试 ───────────────────────────────────────────────────────────

    /// 测试 Unicode payload（特殊字符、Emoji）
    #[test]
    fn test_encode_message_unicode_payload() {
        let payload = "{\"name\":\"工作区🚀\"}".as_bytes();
        let msg = encode_message(1, payload);

        let header: [u8; 14] = msg[0..14].try_into().unwrap();
        let (payload_len, _) = decode_header(&header).unwrap();
        assert_eq!(payload_len as usize, payload.len());
        assert_eq!(&msg[14..], payload);
    }

    /// 测试 payload 长度字段为 u32 最大值附近不会 panic（仅验证编码）
    #[test]
    fn test_encode_message_max_u32_type() {
        let msg = encode_message(u32::MAX, b"");
        let stored = u32::from_le_bytes(msg[10..14].try_into().unwrap());
        assert_eq!(stored, u32::MAX);
    }

    /// 测试 MAGIC 常量的字节内容
    #[test]
    fn test_magic_bytes_content() {
        assert_eq!(MAGIC, b"i3-ipc");
        assert_eq!(MAGIC.len(), 6);
    }

    /// 测试 HEADER_SIZE 常量值
    #[test]
    fn test_header_size_constant() {
        assert_eq!(HEADER_SIZE, 14);
        assert_eq!(HEADER_SIZE, MAGIC.len() + 4 + 4);
    }
}
