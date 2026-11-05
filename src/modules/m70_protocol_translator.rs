//! Adaptador de payloads para um formato interno JSON. Não implementa
//! transporte de rede nem fala diretamente com gRPC, WebSocket ou CAN.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkProtocol {
    GrpcStream { payload: Vec<u8> },
    WebSocketJson { payload: String },
    CanBusFrame { id: u32, data: [u8; 8] },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NormalizedMessage {
    pub source_protocol: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TranslationError {
    InvalidWebSocketJson,
    InvalidGrpcJson,
    CanIdOutOfRange(u32),
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ProtocolTranslator;

impl ProtocolTranslator {
    pub fn translate_to_internal(input: NetworkProtocol) -> Result<NormalizedMessage, TranslationError> {
        match input {
            NetworkProtocol::GrpcStream { payload } => {
                let value = serde_json::from_slice(&payload).map_err(|_| TranslationError::InvalidGrpcJson)?;
                Ok(NormalizedMessage { source_protocol: "gRPC".to_owned(), payload: value })
            }
            NetworkProtocol::WebSocketJson { payload } => {
                let value = serde_json::from_str(&payload).map_err(|_| TranslationError::InvalidWebSocketJson)?;
                Ok(NormalizedMessage { source_protocol: "WebSocket".to_owned(), payload: value })
            }
            NetworkProtocol::CanBusFrame { id, data } => {
                if id > 0x1FFF_FFFF {
                    return Err(TranslationError::CanIdOutOfRange(id));
                }
                let raw_hex = data.iter().map(|byte| format!("{byte:02x}")).collect::<String>();
                Ok(NormalizedMessage {
                    source_protocol: "CAN".to_owned(),
                    payload: serde_json::json!({"can_id": id, "raw_hex": raw_hex, "data_length": data.len()}),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_json_payloads_without_replacing_their_content() {
        let input = NetworkProtocol::WebSocketJson { payload: r#"{"event":"USER_PROMPT","value":7}"#.to_owned() };
        let normalized = ProtocolTranslator::translate_to_internal(input).unwrap();
        assert_eq!(normalized.payload["event"], "USER_PROMPT");
        assert_eq!(normalized.payload["value"], 7);
    }

    #[test]
    fn represents_can_frame_bytes_as_unambiguous_hex() {
        let normalized = ProtocolTranslator::translate_to_internal(NetworkProtocol::CanBusFrame {
            id: 0x1AC,
            data: [0x01, 0x20, 0xFF, 0, 0, 0, 0, 0],
        }).unwrap();
        assert_eq!(normalized.payload["raw_hex"], "0120ff0000000000");
    }

    #[test]
    fn rejects_invalid_json_and_out_of_range_can_ids() {
        assert_eq!(ProtocolTranslator::translate_to_internal(NetworkProtocol::WebSocketJson { payload: "{".into() }), Err(TranslationError::InvalidWebSocketJson));
        assert_eq!(ProtocolTranslator::translate_to_internal(NetworkProtocol::CanBusFrame { id: 0x2000_0000, data: [0; 8] }), Err(TranslationError::CanIdOutOfRange(0x2000_0000)));
    }
}
