use colored::Colorize;
use super::color::Color;

const BITS_IN_BYTE: u8 = 8;
const BYTES_IN_DWORD: u8 = 4;

pub struct FormatStyle {
    pub border_color: Color,
    pub tick_mark_color: Color,
    pub title_color: Color,
    pub column_title_color: Color,
    pub dword_title_color: Color,
    pub notes_color: Color,
}

impl FormatStyle {
    pub fn new() -> FormatStyle {
        FormatStyle {
            border_color: Color::Cyan,
            tick_mark_color: Color::Green,
            title_color: Color::White,
            column_title_color: Color::Green,
            dword_title_color: Color::Green,
            notes_color: Color::Magenta,
        }
    }
}

pub struct WebSocketFrame<'a> {
    pub frame_len: u8,
    pub is_payload_masked: bool,
    pub is_short_payload: bool,
    pub format_style: FormatStyle,
    fin_bit: bool,
    rsv1: bool,
    rsv2: bool,
    rsv3: bool,
    opcode: u8,
    mask_bit: bool,
    payload_len: u8,
    masking_key: [u8; 4],
    masked_payload: &'a [u8],
    unmasked_payload: Vec<u8>,
    payload: Vec<char>,
}

impl<'a> WebSocketFrame<'a> {
    /// Builds a websocket frame from a byte array
    ///
    /// # Arguments
    ///
    /// * `data` - The byte array to convert to a `WebSocketFrame`.
    pub fn from_bytes(data: &Vec<u8>) -> WebSocketFrame {
        const NUM_MASK_BYTES: usize = 4;

        // Get frame length
        let frame_length: usize = data.len();

        // Check if the payload is masked
        let is_payload_masked: bool = get_bit(data[1], 0);

        // Get the payload length (bits 9 - 15)
        let payload_len: u8 = get_bits_from_byte(data[1], 0b01111111);

        // TODO: Handle larger payloads and unmasked payloads
        let payload_start_index = 6;

        let num_payload_bytes: usize = frame_length - payload_start_index;

        // Get mask
        let masking_key: [u8; 4] = [data[2], data[3], data[4], data[5]];

        // Unmask and parse payload data
        let mut unmasked_payload: Vec<u8> = Vec::new();
        let mut payload: Vec<char> = Vec::new();
        for i in 0..num_payload_bytes {
            let byte: u8 = data[payload_start_index + i] ^ masking_key[i % NUM_MASK_BYTES];
            unmasked_payload.push(byte); // 32 mask bits are used repeatedly
                                         //payload.push(byte as char);
            payload.push(byte as char);
        }

        WebSocketFrame {
            // Bytes in frame
            frame_len: data.len() as u8,
            // Mask bit (bit 8) indicates if the payload is masked
            is_payload_masked,
            // Short payloads are <= 126 chars in length
            is_short_payload: payload_len <= 126,
            // Use default format style
            format_style: FormatStyle::new(),
            // Bit 0 contains fin bit
            fin_bit: get_bit(data[0], 0),
            // Bit 1 contains rsv1
            rsv1: get_bit(data[0], 1),
            // Bit 2 contains rsv2
            rsv2: get_bit(data[0], 2),
            // Bit 3 contains rsv3
            rsv3: get_bit(data[0], 3),
            // Bits 4 - 7 contain the opcode
            opcode: get_bits_from_byte(data[0], 0b00001111),
            // Bit 8 contains mask flag
            mask_bit: is_payload_masked,
            // Bits 9 - 15 contain payload length
            payload_len,
            // Next 4 bytes contain masking key
            masking_key,
            // Masked payload is from byte 6 to end of frame
            masked_payload: &data[6..data.len()],
            // Unmasked payload
            unmasked_payload,
            payload,
        }
    }

    /// Formats the websocket frame.
    ///
    /// # Arguments
    ///
    /// * `self` - The `WebSocketFrame` being formatted.
    pub fn format(self: &WebSocketFrame<'a>) -> String {
        let mut result = self.format_header();

        result.push_str(&self.format_first_two_dwords());

        // Format remaining full dwords
        let remaining_payload_dwords: u8 = (self.payload_len - 2).div_euclid(BYTES_IN_DWORD);
        for i in 0..remaining_payload_dwords {
            let from_byte_ix: usize = ((i * BYTES_IN_DWORD) + 2) as usize;
            let to_byte_ix: usize = from_byte_ix + BYTES_IN_DWORD as usize;
            result.push_str(&self.format_payload_dword_row(from_byte_ix, to_byte_ix, i + 3, i + 2));
        }

        // Format remaining bytes (formatted as partial dword)
        let remaining_bytes: u8 = (self.payload_len - 2).rem_euclid(BYTES_IN_DWORD);
        if remaining_bytes > 0 {
            let from_byte_ix: usize = ((remaining_payload_dwords * BYTES_IN_DWORD) + 2) as usize;
            let to_byte_ix: usize = from_byte_ix + remaining_bytes as usize;
            result.push_str(&self.format_payload_dword_row(
                from_byte_ix,
                to_byte_ix,
                remaining_payload_dwords + 3,
                (remaining_payload_dwords * 2) + 2,
            ));
        }

        result
    }

    /// Formats the WebSocket frame header.
    ///
    /// # Arguments
    ///
    /// * `self` The WebSocket frame being formatted.
    fn format_header(self: &WebSocketFrame<'a>) -> String {
        // Closures to help apply style colors
        let border_color = |s: &str| s.color(self.format_style.border_color.to_string());
        let tick_color = |s: &str| s.color(self.format_style.tick_mark_color.to_string());
        let title_color = |s: &str| s.color(self.format_style.title_color.to_string());
        let column_title_color = |s: &str| s.color(self.format_style.column_title_color.to_string());

        // Start with the top border
        let mut result: String = 
            format!(
                "{0:15}{1}\n", 
                "",
                border_color("+---------------+---------------+---------------+---------------+")
            );

        // Append column headers
        result.push_str(
            &format!(
                "{0:2}{2}{0:3}{1}{3:^15}{1}{4:^15}{1}{5:^15}{1}{6:^15}{1}\n", 
                "", 
                border_color("|"),
                title_color("Frame Data"),
                column_title_color("Byte  1"),
                column_title_color("Byte  2"),
                column_title_color("Byte  3"),
                column_title_color("Byte  4")
            )
        );
        // Append divider (between byte headers and bit tick marks)
        result.push_str(
            &format!(
                "{0:2}{1}\n", 
                " ",
                format!(
                    "{0:^10}{1:3}{2}",
                    if self.is_payload_masked { title_color("(Masked)") } else { title_color("(Unmasked)") },
                    "",
                    border_color("+---------------+---------------+---------------+---------------+"),
                )
            )
        );
        // Append tens tick marks
        result.push_str(
            &format!(
                "{0:2}{1:^10}{0:3}{2}{3}{0:14}{2}{0:4}{4}{0:10}{2}{0:8}{5}{0:6}{2}{0:12}{6}{0:2}{2}\n",
                "",
                if self.is_short_payload { title_color("(Short)") } else { title_color("(Long)") },
                border_color("|"),
                tick_color("0"),
                tick_color("1"),
                tick_color("2"),
                tick_color("3")
            )
        );
        // Append unit tick marks
        result.push_str(
            &format!(
                "{0:15}{1}{2}{1}{3}{1}{4}{1}{5}{1}\n",
                "",
                border_color("|"),
                tick_color("0 1 2 3 4 5 6 7"),
                tick_color("8 9 0 1 2 3 4 5"),
                tick_color("6 7 8 9 0 1 2 3"),
                tick_color("4 5 6 7 8 9 0 1")
            )
        );
        
        result
    }

    fn format_first_two_dwords(
        self: &WebSocketFrame<'a>
    ) -> String {
        // Closures to help apply style colors
        let border_color = |s: &str| s.color(self.format_style.border_color.to_string());
        let tick_color = |s: &str| s.color(self.format_style.tick_mark_color.to_string());
        let title_color = |s: &str| s.color(self.format_style.title_color.to_string());
        let column_title_color = |s: &str| s.color(self.format_style.column_title_color.to_string());
        let dword_title_color = |s: &str| s.color(self.format_style.dword_title_color.to_string());
        let notes_color = |s: &str| s.color(self.format_style.notes_color.to_string());

        // Start with the top border
        let mut result: String = 
            format!(
                "{0:7}{1}\n", 
                "",
                border_color("+-------+---------------+---------------+---------------+---------------+")
            );
        // Append the bit values line
        result.push_str(
            &format!(
                "{0:7}{1}{2:^7}{1}{3}{1}{4}{1}{5}{1}{6}{1}{7}{1}{8}{1}{9}{1}{10}{1}{11}{1}\n",
                "",
                border_color("|"),
                dword_title_color("DWORD"),
                bit_str(self.fin_bit),
                bit_str(self.rsv1),
                bit_str(self.rsv2),
                bit_str(self.rsv3),
                byte_str(self.opcode, 4),
                bit_str(self.mask_bit),
                byte_str(self.payload_len, 7),
                byte_str(self.masking_key[0], 8),
                byte_str(self.masking_key[1], 8),
            )
        );
        // Append the first line of bit identifiers
        result.push_str(
            &format!(
                "{0:7}{1}{2:^7}{1}{3}{1}{4}{1}{4}{1}{4}{1}{0:7}{1}{5}{1}{0:13}{1}{0:31}{1}\n",
                "",
                border_color("|"),
                dword_title_color("1"),
                notes_color("F"),
                notes_color("R"),
                notes_color("M")
            )
        );
        // Append the second line of bit identifiers
        result.push_str(
            &format!(
                "{0:7}{1}{0:7}{1}{2}{1}{3}{1}{3}{1}{3}{1}{4:7}{1}{5}{1}{6:^13}{1}{7:^31}{1}\n",
                "",
                border_color("|"),
                notes_color("I"),
                notes_color("S"),
                notes_color("op code"),
                notes_color("A"),
                notes_color("Payload len"),
                notes_color("Masking-key (part 1)"),
            )
        );
        // Append the third line of bit identifiers
        result.push_str(
            &format!(
                "{0:7}{1}{0:7}{1}{2}{1}{3}{1}{3}{1}{3}{1}{4:^7}{1}{5}{1}{6:^13}{1}{0:31}{1}\n",
                "",
                border_color("|"),
                notes_color("N"),
                notes_color("V"),
                notes_color("(4 b)"),
                notes_color("S"),
                notes_color("(7 bits)"),
            )
        );
        // Append the final line of bit identifiers
        result.push_str(
            &format!(
                "{0:7}{1}{0:7}{1}{0:1}{1}{2}{1}{3}{1}{4}{1}{0:7}{1}{5}{1}{0:13}{1}{0:31}{1}\n",
                "",
                border_color("|"),
                notes_color("1"),
                notes_color("2"),
                notes_color("3"),
                notes_color("K"),
            )
        );
        // Append border separating DWORD 1 and DWORD 2
        result.push_str(
            &format!(
                "{0:7}{1}\n",
                "",
                border_color("+-------+-+-+-+-+-------+-+-------------+-------------------------------+")
            )    
        );
        // Append the first line of DWORD 2
        result.push_str(
            &format!(
                "{0:7}{1}{2:^7}{1}{3:^15}{1}{4:^15}{1}{5:^15}{1}{6:^15}{1}\n",
                "",
                border_color("|"),
                dword_title_color("DWORD"),
                byte_str(self.masking_key[2], 8),
                byte_str(self.masking_key[3], 8),
                byte_str(self.masked_payload[0], 8),
                byte_str(self.masked_payload[1], 8),
            )
        );
        // Append the second line of DWORD 2
        result.push_str(
            &format!(
                "{0:7}{1}{2:^7}{1}{0:^31}{1}{0:1}{4:>5}{0:6}{3}{0:2}{5:>5}{0:6}{1}\n",
                "",
                border_color("|"),
                dword_title_color("2"),
                notes_color("MASKED"),
                format!("({})", self.masked_payload[0]),
                format!("({})", self.masked_payload[1]),
            )
        );
        // Append the third line of DWORD 2
        result.push_str(
            &format!(
                "{0:7}{1}{0:7}{1}{2:^31}{1}{3:^15}{1}{4:^15}{1}\n",
                "",
                border_color("|"),
                notes_color("Masking-key (part 2)"),
                byte_str(self.unmasked_payload[0], 8),
                byte_str(self.unmasked_payload[1], 8),
            )
        );
        // Append the fourth line of DWORD 2
        result.push_str(
            &format!(
                "{0:7}{1}{0:7}{1}{3:^31}{1}{0:1}{5:>5}{0:1}{2}{6}{2}{0:1}{4}{0:1}{7:>5}{0:1}{2}{8}{2}{0:2}{1}\n",
                "",
                border_color("|"),
                "'",
                notes_color("(16 bits)"),
                notes_color("UNMASKED"),
                format!("({})", self.unmasked_payload[0]),
                self.payload[0],
                format!("({})", self.unmasked_payload[1]),
                self.payload[1]
            )
        );
        // Append the fifth line of DWORD 2
        result.push_str(
            &format!(
                "{0:7}{1}{0:7}{1}{0:^31}{1}{2:^31}{1}\n",
                "",
                border_color("|"),
                notes_color("Payload Data (part 1)")
            )
        );
        // Append the bottom border
        result.push_str(
            &format!(
                "{0:7}{1}",
                "",
                border_color("+-------+-------------------------------+-------------------------------+"),
            )
        );

        result
    }

    /// Formats a dword table row displaying part of a websocket frame payload.
    /// # Arguments
    ///
    /// * `self` The WebSocket frame being formatted.
    fn format_payload_dword_row(
        self: &WebSocketFrame<'a>,
        from_byte_ix: usize,
        to_byte_ix: usize,
        dword_number: u8,
        part_number: u8,
    ) -> String {
        let mut result: String = String::from("");

        // Calculate number of bytes to include in this row
        let num_bytes = to_byte_ix - from_byte_ix;

        let masked_bits: &[u8] = &self.masked_payload[from_byte_ix..to_byte_ix];
        let unmasked_bits: &[u8] = &self.unmasked_payload[from_byte_ix..to_byte_ix];
        let payload_data: &[char] = &self.payload[from_byte_ix..to_byte_ix];

        // Check indexes form a valid range
        if num_bytes < 1 || num_bytes > 4  {
            return String::from(
                format!("ERROR: Cannot print dword row. Illegal byte indexes provided. from_byte_ix: {} to_byte_ix: {}", 
                from_byte_ix, 
                to_byte_ix));
        }

        let delim = |d: u8| format!("({})", d);
        let indent = |s: &str| format!("       {}", s);

        // Format masked bits
        result.push_str("\n       | DWORD |");
        result.push_str(
            &(0..num_bytes)
                .map(|i| format!("{}|", byte_str(masked_bits[i], BITS_IN_BYTE)))
                .collect::<String>(),
        );
        result.push_str("\n");

        // Format masked char previews
        result.push_str(&indent(&format!("| {:^5} |", dword_number)));
        match num_bytes {
            1 => result.push_str(&format!(
                " {:>5}     MSK |",
                format!("({})", masked_bits[0])
            )),
            2 => result.push_str(&format!(
                " {0:>5}      MASKED  {1:>5}      |",
                delim(masked_bits[0]),
                delim(masked_bits[1])
            )),
            3 => result.push_str(&format!(
                " {0:>5}      MASKED  {1:>5}      | {2:>5}     MSK |",
                delim(masked_bits[0]),
                delim(masked_bits[1]),
                delim(masked_bits[2])
            )),
            4 => result.push_str(&format!(
                " {0:>5}      MASKED  {1:>5}      | {2:>5}      MASKED  {3:>5}      |",
                delim(masked_bits[0]),
                delim(masked_bits[1]),
                delim(masked_bits[2]),
                delim(masked_bits[3])
            )),
            _ => {}
        }
        result.push_str("\n");

        // Format unmasked bits
        result.push_str(&indent("|       |"));
        result.push_str(
            &(0..num_bytes)
                .map(|i| format!("{}|", byte_str(unmasked_bits[i], BITS_IN_BYTE)))
                .collect::<String>(),
        );
        result.push_str("\n");

        // Format unmasked char previews
        result.push_str(&indent("|       |"));
        match num_bytes {
            1 => result.push_str(&format!(
                " {0:>5} '{1}' UNM |",
                delim(unmasked_bits[0]),
                payload_data[0]
            )),
            2 => result.push_str(&format!(
                " {0:>5} '{1}' UNMASKED {2:>5} '{3}'  |",
                delim(unmasked_bits[0]),
                payload_data[0],
                delim(unmasked_bits[1]),
                payload_data[1]
            )),
            3 => result.push_str(&format!(
                " {0:>5} '{1}' UNMASKED {2:>5} '{3}'  | {4:>5} '{5}' UNM |",
                delim(unmasked_bits[0]),
                payload_data[0],
                delim(unmasked_bits[1]),
                payload_data[1],
                delim(unmasked_bits[2]),
                payload_data[2],
            )),
            4 => result.push_str(&format!(
                " {0:>5} '{1}' UNMASKED {2:>5} '{3}'  | {4:>5} '{5}' UNMASKED {6:>5} '{7}'  |",
                delim(unmasked_bits[0]),
                payload_data[0],
                delim(unmasked_bits[1]),
                payload_data[1],
                delim(unmasked_bits[2]),
                payload_data[2],
                delim(unmasked_bits[3]),
                payload_data[3],
            )),
            _ => {}
        }
        result.push_str("\n");

        // Format payload part text
        result.push_str(&indent("|       |"));
        match num_bytes {
            1 => result.push_str(&format!(
                "{:^15}|",
                format!("Payload pt {}", part_number)
            )),
            2 => result.push_str(&format!(
                "{:^31}|",
                format!("Payload Data (part {})", part_number)
            )),
            3 => result.push_str(&format!(
                "{0:^47}|",
                format!("Payload Data (part {})", part_number),
            )),
            4 => result.push_str(&format!(
                "{0:^63}|",
                format!("Payload Data (part {})", part_number),
            )),
            _ => {}
        }
        result.push_str("\n");

        // Format bottom border
        result.push_str(&indent("+-------+"));
        result.push_str(
            &(0..num_bytes)
                .map(|_| "---------------+")
                .collect::<String>(),
        );
        result.push_str("\n");

        result
    }
}

fn get_bits_from_byte(byte: u8, mask: u8) -> u8 {
    byte & mask
}

/// Formats a byte or partial byte.
///
/// # Arguments
///
/// * `byte` - The byte to format.
/// * `num_bits` - The number of bits to format.
fn byte_str<'a>(byte: u8, num_bits: u8) -> String {
    let mut result: String = String::from("");
    result.push_str(
        &(8 - num_bits..8)
            .map(|i| format!("{} ", bit_str(get_bit(byte, i))))
            .collect::<String>(),
    );
    result.trim().to_string()
}

fn bit_str<'a>(bit: bool) -> &'a str {
    if bit == true {
        "1"
    } else {
        "0"
    }
}

fn get_bit(byte: u8, bit_position: u8) -> bool {
    match bit_position {
        0 => byte & 0b10000000 != 0,
        1 => byte & 0b01000000 != 0,
        2 => byte & 0b00100000 != 0,
        3 => byte & 0b00010000 != 0,
        4 => byte & 0b00001000 != 0,
        5 => byte & 0b00000100 != 0,
        6 => byte & 0b00000010 != 0,
        7 => byte & 0b00000001 != 0,
        _ => false,
    }
}

// #region WebSocket Frame Unit Tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_masked_frame() {
        let bytes = base64::decode("gYNaDpE2O2zy").unwrap();

        let frame = WebSocketFrame::from_bytes(&bytes);
        let expected = "               +---------------+---------------+---------------+---------------+\n  Frame Data   |    Byte  0    |    Byte  1    |    Byte  2    |    Byte  3    |\n   (Masked)    +---------------+---------------+---------------+---------------+\n   (Short)     |0              |    1          |        2      |            3  |\n               |0 1 2 3 4 5 6 7|8 9 0 1 2 3 4 5|6 7 8 9 0 1 2 3|4 5 6 7 8 9 0 1|\n       +-------+-+-+-+-+-------+-+-------------+-------------------------------+\n       | DWORD |1|0|0|0|0 0 0 1|1|0 0 0 0 0 1 1|0 1 0 1 1 0 1 0|0 0 0 0 1 1 1 0|\n       |   1   |F|R|R|R|       |M|             |                               |\n       |       |I|S|S|S|op code|A| Payload len |     Masking-key (part 1)      |\n       |       |N|V|V|V| (4 b) |S|  (7 bits)   |           (16 bits)           |\n       |       | |1|2|3|       |K|             |                               |\n       +-------+-+-+-+-+-------+-+-------------+-------------------------------+\n       | DWORD |1 0 0 1 0 0 0 1|0 0 1 1 0 1 1 0|0 0 1 1 1 0 1 1|0 1 1 0 1 1 0 0|\n       |   2   |                               |  (59)      MASKED  (108)      |\n       |       |     Masking-key (part 2)      |0 1 1 0 0 0 0 1|0 1 1 0 0 0 1 0|\n       |       |           (16 bits)           |  (97) \'a\' UNMASKED  (98) \'b\'  |\n       |       |                               |     Payload Data (part 1)     |       \n       +-------+-------------------------------+-------------------------------+\n       | DWORD |1 1 1 1 0 0 1 0|\n       |   3   | (242)     MSK |\n       |       |0 1 1 0 0 0 1 1|\n       |       |  (99) \'c\' UNM |\n       |       | Payload pt 2  |\n       +-------+---------------+\n";
        
        println!("1-------10--------20--------30--------40--------50--------60--------70--------80");
        println!("{}", frame.format());

        assert_eq!(frame.format(), expected);
    }
}

// #endregion WebSocket Frame Unit Tests
