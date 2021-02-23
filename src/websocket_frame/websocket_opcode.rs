#[derive(Debug)]
#[derive(PartialEq)]
pub enum WebSocketOpCode {
    Continuation,
    Text,
    Binary,
    CloseConnection,
    Ping,
    Pong,
    Unrecognized,
    ReservedFuture,
}

impl WebSocketOpCode {
    /// Gets an opcode from a 4-bit value
    pub fn from_bit_value(opcode_bits: u8) -> WebSocketOpCode {
        match opcode_bits {
            0 => WebSocketOpCode::Continuation,
            1 => WebSocketOpCode::Text,
            2 => WebSocketOpCode::Binary,
            8 => WebSocketOpCode::CloseConnection,
            9 => WebSocketOpCode::Ping,
            10 => WebSocketOpCode::Pong,
            3 | 4 | 5 | 6 | 7 | 
            11 | 12 | 13 | 14 | 15 => WebSocketOpCode::ReservedFuture,
            _ => WebSocketOpCode::Unrecognized,
        }
    }
}

// #region Unit tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_opcode_continuation() {
        // Continuation
        assert_eq!(WebSocketOpCode::Continuation, WebSocketOpCode::from_bit_value(0b00000000));
    }

    #[test]
    fn text_get_opcode_text() {
        // Text
        assert_eq!(WebSocketOpCode::Text, WebSocketOpCode::from_bit_value(0b00000001));
    }

    #[test]
    fn text_get_opcode_binary() {
        // Binary
        assert_eq!(WebSocketOpCode::Binary, WebSocketOpCode::from_bit_value(0b00000010));
    }

    #[test]
    fn text_get_opcode_close_connection() {
        // Close Connection
        assert_eq!(WebSocketOpCode::CloseConnection, WebSocketOpCode::from_bit_value(0b00001000));
    }

    #[test]
    fn text_get_opcode_ping() {
        // Ping
        assert_eq!(WebSocketOpCode::Ping, WebSocketOpCode::from_bit_value(0b00001001));
    }

    #[test]
    fn text_get_opcode_pong() {
        // Pong
        assert_eq!(WebSocketOpCode::Pong, WebSocketOpCode::from_bit_value(0b00001010));
    }

    #[test]
    fn text_get_opcode_reserved_future() {
        // Pong
        assert_eq!(WebSocketOpCode::ReservedFuture, WebSocketOpCode::from_bit_value(0b00000011));
        assert_eq!(WebSocketOpCode::ReservedFuture, WebSocketOpCode::from_bit_value(0b00000100));
        assert_eq!(WebSocketOpCode::ReservedFuture, WebSocketOpCode::from_bit_value(0b00000101));
        assert_eq!(WebSocketOpCode::ReservedFuture, WebSocketOpCode::from_bit_value(0b00000110));
        assert_eq!(WebSocketOpCode::ReservedFuture, WebSocketOpCode::from_bit_value(0b00000111));
        assert_eq!(WebSocketOpCode::ReservedFuture, WebSocketOpCode::from_bit_value(0b00001011));
        assert_eq!(WebSocketOpCode::ReservedFuture, WebSocketOpCode::from_bit_value(0b00001100));
        assert_eq!(WebSocketOpCode::ReservedFuture, WebSocketOpCode::from_bit_value(0b00001101));
        assert_eq!(WebSocketOpCode::ReservedFuture, WebSocketOpCode::from_bit_value(0b00001111));
    }

    #[test]
    fn text_get_opcode_unrecognized() {
        // Pong
        assert_eq!(WebSocketOpCode::Unrecognized, WebSocketOpCode::from_bit_value(0b01000000));
    }
}

// #endregion Unit tests