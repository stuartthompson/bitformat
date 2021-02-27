mod websocket_opcode;

use std::convert::TryInto;
use colored::{Colorize, Color};
use websocket_opcode::WebSocketOpCode;

const BITS_IN_BYTE: usize = 8;
const BYTES_IN_DWORD: usize = 4;

pub struct FormatStyle {
    pub border_color: Color,
    pub tick_mark_color: Color,
    pub title_color: Color,
    pub column_title_color: Color,
    pub dword_title_color: Color,
    pub notes_color: Color,
    pub bit_color: Color,
    pub unmasked_payload_bit_color: Color,
    pub byte_value_color: Color,
    pub data_value_color: Color,
    pub summary_title_color: Color,
    pub summary_value_color: Color,
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
            bit_color: Color::White,
            unmasked_payload_bit_color: Color::Yellow,
            byte_value_color: Color::Blue,
            data_value_color: Color::Red,
            summary_title_color: Color::Magenta,
            summary_value_color: Color::Red,
        }
    }
}

/// The length of a WebSocket data frame payload.
#[derive(Debug)]
#[derive(PartialEq)]
pub enum PayloadLength {
    Short(u8),
    Medium(u16),
    Long(u64)
}

impl std::fmt::Display for PayloadLength {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let out = match *self {
            PayloadLength::Short(length) => format!("Short ({0} bytes)", length),
            PayloadLength::Medium(length) => format!("Medium ({0} bytes)", length),
            PayloadLength::Long(length) => format!("Long ({0} bytes)", length)
        };
        write!(f, "{}", out)
    }
}

pub struct WebSocketFrame<'a> {
    pub frame_len: u8,
    pub is_payload_masked: bool,
    pub payload_length: PayloadLength,
    pub format_style: FormatStyle,
    fin_bit: bool,
    rsv1: bool,
    rsv2: bool,
    rsv3: bool,
    opcode_bits: u8,
    opcode: WebSocketOpCode,
    mask_bit: bool,
    payload_length_code: u8,
    payload_length_bytes: Vec<u8>,
    masking_key: [u8; 4],
    masked_payload: &'a [u8],
    unmasked_payload: Vec<u8>,
    payload_chars: Vec<char>,
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

        // Get the opcode bit values
        let opcode_bits = get_bits_from_byte(data[0], 0b00001111);

        // Check if the payload is masked
        let is_payload_masked: bool = get_bit(data[1], 0);

        // Get the payload length code (bits 9 - 15)
        let payload_length_code: u8 = get_bits_from_byte(data[1], 0b01111111);
        
        // Assemble extension data
        let mut extension_data: Vec<u8> = Vec::new();
        for ix in 0..8 {
            if data.len() > ix + 2 {
                extension_data.push(data[ix + 2]);
            }
        }

        // Calculate payload length
        let payload_length = WebSocketFrame::get_payload_length(payload_length_code, extension_data);

        // TODO: Handle larger payloads and unmasked payloads
        let payload_start_index: usize = 
            match payload_length {
                // Short payload begin at byte 6 (first 2 plus 4 for masking key)
                PayloadLength::Short(_) => 6,
                // Medium payload begin at byte 8 (first 2 plus 2 for 16-bit payload length plus 4 for masking key)                
                PayloadLength::Medium(_) => 8,
                // Long payload begin at byte 14 (first 2 plus 8 for 64-bit payload length plus 4 for masking key)                
                PayloadLength::Long(_) => 14,
            };

        // Get the byte values describing payload length
        let payload_length_bytes: Vec<u8> = 
            match payload_length {
                PayloadLength::Short(_) => vec!(payload_length_code),
                PayloadLength::Medium(_) => vec!(data[2], data[3]),
                PayloadLength::Long(_) => vec!(data[2], data[3], data[4], data[5], data[6], data[7], data[8], data[9]),
            };

        let num_payload_bytes: usize = frame_length - payload_start_index;

        // Get mask
        let masking_key: [u8; 4] = match payload_length {
            PayloadLength::Short(_) => [data[2], data[3], data[4], data[5]],
            PayloadLength::Medium(_) => [data[4], data[5], data[6], data[7]],
            PayloadLength::Long(_) => [data[10], data[11], data[12], data[13]],
        };

        // Unmask and parse payload data
        let mut unmasked_payload: Vec<u8> = Vec::new();
        let mut payload_chars: Vec<char> = Vec::new();
        for i in 0..num_payload_bytes {
            let byte: u8 = data[payload_start_index + i] ^ masking_key[i % NUM_MASK_BYTES];
            unmasked_payload.push(byte); // 32 mask bits are used repeatedly
                                         //payload.push(byte as char);
            payload_chars.push(byte as char);
        }

        WebSocketFrame {
            // Bytes in frame
            frame_len: data.len() as u8,
            // Mask bit (bit 8) indicates if the payload is masked
            is_payload_masked,
            // Payload length
            payload_length,
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
            opcode_bits,
            // Look up the opcode from the bits
            opcode: WebSocketOpCode::from_bit_value(opcode_bits),
            // Bit 8 contains mask flag
            mask_bit: is_payload_masked,
            // Bits 9 - 15 contain payload length code
            payload_length_code,
            // Payload start index used in formatting logic
            payload_length_bytes,
            // Next 4 bytes contain masking key
            masking_key,
            // Masked payload is from byte 6 to end of frame
            masked_payload: &data[payload_start_index..data.len()],
            // Unmasked payload
            unmasked_payload,
            // Vector of chars in payload
            payload_chars,
        }
    }

    /// Formats the websocket frame.
    ///
    /// # Arguments
    ///
    /// * `self` - The `WebSocketFrame` being formatted.
    pub fn format(self: &WebSocketFrame<'a>) -> String {
        let mut result = self.format_header();

        // DWORD 1
        result.push_str(&self.format_first_dword());

        // DWORD 2
        result.push_str(&self.format_second_dword());

        let payload_length: usize = 
            match self.payload_length {
                PayloadLength::Short(length) => length.into(),
                PayloadLength::Medium(length) => length.into(),
                PayloadLength::Long(length) => length.try_into().unwrap()
            };

        // The sequential dword number to start from
        let dword_from = 
            match self.payload_length {
                PayloadLength::Short(_) => 3,
                PayloadLength::Medium(_) => 3,
                PayloadLength::Long(_) => 4
            };

        // Determine how many payload bytes were formatted by initial rows
        let payload_bytes_formatted_already = match self.payload_length {
            // Short length: 2 payload bytes formatted into DWORD 1 and DWORD 2 rows
            PayloadLength::Short(_) => 2,
            // Medium length: 0 payload bytes formatted so far
            PayloadLength::Medium(_) => 0,
            // Long length: 2 payload bytes formatted into DWORD 4 row
            PayloadLength::Long(_) => 2,
        };

        // Format remaining full dwords
        let remaining_payload_dwords = (payload_length - payload_bytes_formatted_already).div_euclid(BYTES_IN_DWORD.into());
        for i in 0..remaining_payload_dwords {
            let from_byte_ix = (i * BYTES_IN_DWORD) + payload_bytes_formatted_already;
            let to_byte_ix = BYTES_IN_DWORD + from_byte_ix;
            result.push_str(&self.format_payload_dword_row(
                from_byte_ix, 
                to_byte_ix, 
                i + dword_from, 
                i + payload_bytes_formatted_already
            ));
        }
        // Format remaining bytes (formatted as partial dword)
        let remaining_bytes: usize = (payload_length - payload_bytes_formatted_already).rem_euclid(BYTES_IN_DWORD);
        if remaining_bytes > 0 {
            let from_byte_ix: usize = (remaining_payload_dwords * BYTES_IN_DWORD) + payload_bytes_formatted_already;
            let to_byte_ix: usize = from_byte_ix + remaining_bytes;
            result.push_str(&self.format_payload_dword_row(
                from_byte_ix,
                to_byte_ix,
                remaining_payload_dwords + dword_from,
                (remaining_payload_dwords * 2) + payload_bytes_formatted_already,
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
        // Start with the top border
        let mut result: String = 
            format!(
                "{0:15}{1}\n", 
                "",
                "+---------------+---------------+---------------+---------------+".color(self.format_style.border_color),
            );

        // Append column headers
        result.push_str(
            &format!(
                "{1:^15}{0}{2:^15}{0}{3:^15}{0}{4:^15}{0}{5:^15}{0}\n", 
                "|".color(self.format_style.border_color),
                "Frame Data".color(self.format_style.title_color),
                "Byte  1".color(self.format_style.column_title_color),
                "Byte  2".color(self.format_style.column_title_color),
                "Byte  3".color(self.format_style.column_title_color),
                "Byte  4".color(self.format_style.column_title_color),
            )
        );
        // Append divider (between byte headers and bit tick marks)
        result.push_str(
            &format!(
                "{0:2}{1}\n", 
                " ",
                format!(
                    "{0:^10}{1:3}{2}",
                    if self.is_payload_masked { "(Masked)".color(self.format_style.title_color) } else { "(Unmasked)".color(self.format_style.title_color) },
                    "",
                    "+---------------+---------------+---------------+---------------+".color(self.format_style.border_color),
                )
            )
        );
        // Append tens tick marks
        result.push_str(
            &format!(
                "{1:^15}{2}{3}{0:14}{2}{0:4}{4}{0:10}{2}{0:8}{5}{0:6}{2}{0:12}{6}{0:2}{2}\n",
                "",
                format!("{:?}", self.payload_length),
                "|".color(self.format_style.border_color),
                "0".color(self.format_style.tick_mark_color),
                "1".color(self.format_style.tick_mark_color),
                "2".color(self.format_style.tick_mark_color),
                "3".color(self.format_style.tick_mark_color)
            )
        );
        // Append unit tick marks
        result.push_str(
            &format!(
                "{0:15}{1}{2}{1}{3}{1}{4}{1}{5}{1}\n",
                "",
                "|".color(self.format_style.border_color),
                "0 1 2 3 4 5 6 7".color(self.format_style.tick_mark_color),
                "8 9 0 1 2 3 4 5".color(self.format_style.tick_mark_color),
                "6 7 8 9 0 1 2 3".color(self.format_style.tick_mark_color),
                "4 5 6 7 8 9 0 1".color(self.format_style.tick_mark_color)
            )
        );
        
        result
    }

    /// Formats the first dword of the data frame.
    /// 
    /// # Arguments
    /// 
    /// * `self` - The WebSocket data frame containing the dwords to format.
    fn format_first_dword(
        self: &WebSocketFrame<'a>
    ) -> String {
        // Start with the top border
        let mut result: String = 
            format!(
                "{0:7}{1}\n", 
                "",
                "+-------+---------------+---------------+---------------+---------------+".color(self.format_style.border_color),
            );
        // Line 1: DWORD 1 bit values
        result.push_str(
            &format!(
                "{0:7}{1}{2:^7}{1}{3}{1}{4}{1}{5}{1}{6}{1}{7}{1}{8}{1}{9}{1}{10}{1}{11}{1}\n",
                "",
                "|".color(self.format_style.border_color),
                "DWORD".color(self.format_style.dword_title_color),
                bit_str(self.fin_bit).color(self.format_style.bit_color),
                bit_str(self.rsv1).color(self.format_style.bit_color),
                bit_str(self.rsv2).color(self.format_style.bit_color),
                bit_str(self.rsv3).color(self.format_style.bit_color),
                &byte_str(self.opcode_bits, 4).color(self.format_style.bit_color),
                bit_str(self.mask_bit).color(self.format_style.bit_color),
                &byte_str(self.payload_length_code, 7).color(self.format_style.bit_color),
                match self.payload_length {
                    PayloadLength::Short(_) => 
                        byte_str(self.masking_key[0], 8).color(self.format_style.bit_color),
                    PayloadLength::Medium(_) | PayloadLength::Long(_) => 
                        byte_str(self.payload_length_bytes[0], 8).color(self.format_style.bit_color),
                },
                match self.payload_length {
                    PayloadLength::Short(_) => 
                        byte_str(self.masking_key[1], 8).color(self.format_style.bit_color),
                    PayloadLength::Medium(_) | PayloadLength::Long(_) => 
                        byte_str(self.payload_length_bytes[1], 8).color(self.format_style.bit_color),
                },
            )
        );
        // Line 2: Op code and first line of bit names
        result.push_str(
            &format!(
                "{0:7}{1}{2:^7}{1}{3}{1}{4}{1}{4}{1}{4}{1}{6:^7}{1}{5}{1}{7:^13}{1}{8:^31}{1}\n",
                "",
                "|".color(self.format_style.border_color),
                "1".color(self.format_style.dword_title_color),
                "F".color(self.format_style.notes_color),
                "R".color(self.format_style.notes_color),
                "M".color(self.format_style.notes_color),
                &format!("{:?}", self.opcode).color(self.format_style.data_value_color),
                match self.payload_length {
                    PayloadLength::Short(length) => format!("{} bytes", length).color(self.format_style.data_value_color),
                    PayloadLength::Medium(_) => "126: Medium".color(self.format_style.data_value_color),
                    PayloadLength::Long(_) => "127: Long".color(self.format_style.data_value_color),
                },
                match self.payload_length {
                    PayloadLength::Short(_) => format!("{}", ""),
                    PayloadLength::Medium(length)  => 
                        format!("{0:^6}{1:^19}{2:^6}", 
                            format!("({})", self.payload_length_bytes[0]).color(self.format_style.byte_value_color),
                            format!("{} bytes", length).color(self.format_style.data_value_color),
                            format!("({})", self.payload_length_bytes[1]).color(self.format_style.byte_value_color),
                        ),
                    PayloadLength::Long(length) => 
                        format!("{0:^6}{1:^19}{2:^6}", 
                            format!("({})", self.payload_length_bytes[0]).color(self.format_style.byte_value_color),
                            format!("{} bytes", length).color(self.format_style.data_value_color),
                            format!("({})", self.payload_length_bytes[1]).color(self.format_style.byte_value_color),
                        ),
                },
            )
        );
        // Append the second line of bit identifiers
        result.push_str(
            &format!(
                "{0:7}{1}{0:7}{1}{2}{1}{3}{1}{3}{1}{3}{1}{4:7}{1}{5}{1}{6:^13}{1}{7:^31}{1}\n",
                "",
                "|".color(self.format_style.border_color),
                "I".color(self.format_style.notes_color),
                "S".color(self.format_style.notes_color),
                "op code".color(self.format_style.notes_color),
                "A".color(self.format_style.notes_color),
                "Payload len".color(self.format_style.notes_color),
                match self.payload_length {
                    PayloadLength::Short(_) => "Masking-key (part 1)".color(self.format_style.notes_color),
                    PayloadLength::Medium(_) => "Payload length".color(self.format_style.notes_color),
                    PayloadLength::Long(_) => "Payload length (Part 1 of 4)".color(self.format_style.notes_color),
                }
            )
        );
        // Append the third line of bit identifiers
        result.push_str(
            &format!(
                "{0:7}{1}{0:7}{1}{2}{1}{3}{1}{3}{1}{3}{1}{4:^7}{1}{5}{1}{6:^13}{1}{7:^31}{1}\n",
                "",
                "|".color(self.format_style.border_color),
                "N".color(self.format_style.notes_color),
                "V".color(self.format_style.notes_color),
                "(4 b)".color(self.format_style.notes_color),
                "S".color(self.format_style.notes_color),
                "(7 bits)".color(self.format_style.notes_color),
                "(16 bits)".color(self.format_style.notes_color),
            )
        );
        // Append the final line of bit identifiers
        result.push_str(
            &format!(
                "{0:7}{1}{0:7}{1}{0:1}{1}{2}{1}{3}{1}{4}{1}{0:7}{1}{5}{1}{0:13}{1}{0:31}{1}\n",
                "",
                "|".color(self.format_style.border_color),
                "1".color(self.format_style.notes_color),
                "2".color(self.format_style.notes_color),
                "3".color(self.format_style.notes_color),
                "K".color(self.format_style.notes_color),
            )
        );
        // Append border separating DWORD 1 and DWORD 2
        result.push_str(
            &format!(
                "{0:7}{1}\n",
                "",
                "+-------+-+-+-+-+-------+-+-------------+-------------------------------+".color(self.format_style.border_color)
            )    
        );

        result
    }

    /// Formats the second dword of a WebSocket data frame.
    /// 
    /// # Arguments
    /// 
    /// * `self` - The WebSocket data frame being formatted.
    fn format_second_dword(
        self: &WebSocketFrame<'a>) -> String {
        // Line 1: Format the first line of DWORD 2
        let mut result: String = 
            match self.payload_length {
                PayloadLength::Short(_) => {
                    format!(
                        "{0:7}{1}{2:^7}{1}{3:^15}{1}{4:^15}{1}{5:^15}{1}{6:^15}{1}\n",
                        "",
                        "|".color(self.format_style.border_color),
                        "DWORD".color(self.format_style.dword_title_color),
                        &byte_str(self.masking_key[2], 8).color(self.format_style.bit_color),
                        &byte_str(self.masking_key[3], 8).color(self.format_style.bit_color),
                        &byte_str(self.masked_payload[0], 8).color(self.format_style.bit_color),
                        &byte_str(self.masked_payload[1], 8).color(self.format_style.bit_color),
                    )
                },
                PayloadLength::Medium(_) => {
                    format!(
                        "{0:7}{1}{2:^7}{1}{3:^15}{1}{4:^15}{1}{5:^15}{1}{6:^15}{1}\n",
                        "",
                        "|".color(self.format_style.border_color),
                        "DWORD".color(self.format_style.dword_title_color),
                        &byte_str(self.masking_key[0], 8).color(self.format_style.bit_color),
                        &byte_str(self.masking_key[1], 8).color(self.format_style.bit_color),
                        &byte_str(self.masking_key[2], 8).color(self.format_style.bit_color),
                        &byte_str(self.masking_key[3], 8).color(self.format_style.bit_color),
                    )
                },
                PayloadLength::Long(_) => {
                    format!(
                        "{0:7}{1}{2:^7}{1}{3:^15}{1}{4:^15}{1}{5:^15}{1}{6:^15}{1}\n",
                        "",
                        "|".color(self.format_style.border_color),
                        "DWORD".color(self.format_style.dword_title_color),
                        &byte_str(self.payload_length_bytes[2], 8).color(self.format_style.bit_color),
                        &byte_str(self.payload_length_bytes[3], 8).color(self.format_style.bit_color),
                        &byte_str(self.payload_length_bytes[4], 8).color(self.format_style.bit_color),
                        &byte_str(self.payload_length_bytes[5], 8).color(self.format_style.bit_color),
                    )
                }
            };

        // Line 2: Append the second line of DWORD 2
        match self.payload_length {
            PayloadLength::Short(_) => {
                result.push_str(
                    &format!(
                        "{0:7}{1}{2:^7}{1}{0:^31}{1}{0:1}{4:>5}{0:6}{3}{0:2}{5:>5}{0:6}{1}\n",
                        "",
                        "|".color(self.format_style.border_color),
                        "2".color(self.format_style.dword_title_color),
                        "MASKED".color(self.format_style.notes_color),
                        &format!("({})", self.masked_payload[0]).color(self.format_style.byte_value_color),
                        &format!("({})", self.masked_payload[1]).color(self.format_style.byte_value_color),
                    )
                );
            },
            PayloadLength::Medium(_) => {
                result.push_str(
                    &format!(
                        "{0:7}{1}{2:^7}{1}{0:^31}{1}{0:31}{1}\n",
                        "",
                        "|".color(self.format_style.border_color),
                        "2".color(self.format_style.dword_title_color),
                    )
                );
            },
            PayloadLength::Long(_) => {
                result.push_str(
                    &format!(
                        "{0:7}{1}{2:^7}{1}{0:^31}{1}{0:31}{1}\n",
                        "",
                        "|".color(self.format_style.border_color),
                        "2".color(self.format_style.dword_title_color),
                    )
                );
            }
        }
        
        // Line 3: Append the third line of DWORD 2
        match self.payload_length {
            PayloadLength::Short(_) => {
                result.push_str(
                    &format!(
                        "{0:7}{1}{0:7}{1}{2:^31}{1}{3:^15}{1}{4:^15}{1}\n",
                        "",
                        "|".color(self.format_style.border_color),
                        "Masking-key (part 2)".color(self.format_style.notes_color),
                        &byte_str(self.unmasked_payload[0], 8).color(self.format_style.unmasked_payload_bit_color),
                        &byte_str(self.unmasked_payload[1], 8).color(self.format_style.unmasked_payload_bit_color),
                    )
                );
            },
            PayloadLength::Medium(_) => {
                result.push_str(
                    &format!(
                        "{0:7}{1}{0:7}{1}{2:^31}{1}{3:^31}{1}\n",
                        "",
                        "|".color(self.format_style.border_color),
                        "Masking-key (part 1)".color(self.format_style.notes_color),
                        "Masking-key (part 2)".color(self.format_style.notes_color),
                    )
                );
            },
            PayloadLength::Long(_) => {
                result.push_str(
                    &format!(
                        "{0:7}{1}{0:7}{1}{2:^31}{1}{3:^31}{1}\n",
                        "",
                        "|".color(self.format_style.border_color),
                        "Payload length (part 2 of 4)".color(self.format_style.notes_color),
                        "(16 bits)".color(self.format_style.notes_color),
                    )
                );
            },
        }

        // Line 4: Append the fourth line of DWORD 2
        match self.payload_length {
            PayloadLength::Short(_) => {
                result.push_str(
                    &format!(
                        "{0:7}{1}{0:7}{1}{2:^31}{1}{0:1}{4:>5}{0:1}{5:3}{0:1}{3}{0:1}{6:>5}{0:1}{7:3}{0:2}{1}\n",
                        "",
                        "|".color(self.format_style.border_color),
                        "(16 bits)".color(self.format_style.notes_color),
                        "UNMASKED".color(self.format_style.notes_color),
                        &format!("({})", self.unmasked_payload[0]).color(self.format_style.byte_value_color),
                        &format!("'{0}'", &self.payload_chars[0]).color(self.format_style.data_value_color),
                        &format!("({})", self.unmasked_payload[1]).color(self.format_style.byte_value_color),
                        &format!("'{0}'", &self.payload_chars[1]).color(self.format_style.data_value_color),
                    )
                );
            },
            PayloadLength::Medium(_) => {
                result.push_str(
                    &format!(
                        "{0:7}{1}{0:7}{1}{2:^31}{1}{2:^31}{1}\n",
                        "",
                        "|".color(self.format_style.border_color),
                        "(16 bits)".color(self.format_style.notes_color),
                    )
                );   
            }
            PayloadLength::Long(_) => {
                result.push_str(
                    &format!(
                        "{0:7}{1}{0:7}{1}{2:^31}{1}{2:^31}{1}\n",
                        "",
                        "|".color(self.format_style.border_color),
                        "(16 bits)".color(self.format_style.notes_color),
                    )
                );
            }
        }

        // Line 5: Append the fifth line of DWORD 2
        match self.payload_length {
            PayloadLength::Short(_) => {
                result.push_str(
                    &format!(
                        "{0:7}{1}{0:7}{1}{0:^31}{1}{2:^31}{1}\n",
                        "",
                        "|".color(self.format_style.border_color),
                        "Payload Data (part 1)".color(self.format_style.notes_color)
                    )
                );
            }
            PayloadLength::Medium(_) | PayloadLength::Long(_) => {
                result.push_str(
                    &format!(
                        "{0:7}{1}{0:7}{1}{0:^31}{1}{0:^31}{1}\n",
                        "",
                        "|".color(self.format_style.border_color),
                    )
                );
            }
        }
        
        // Append the bottom border
        result.push_str(
            &format!(
                "{0:7}{1}\n",
                "",
                "+-------+-------------------------------+-------------------------------+".color(self.format_style.border_color),
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
        dword_number: usize,
        part_number: usize,
    ) -> String { 
        let mut result: String = String::from("");

        // Calculate number of bytes to include in this row
        let num_bytes = to_byte_ix - from_byte_ix;

        let masked_bits: &[u8] = &self.masked_payload[from_byte_ix..to_byte_ix];
        let unmasked_bits: &[u8] = &self.unmasked_payload[from_byte_ix..to_byte_ix];
        let payload_data: &[char] = &self.payload_chars[from_byte_ix..to_byte_ix];

        // Check indexes form a valid range
        if num_bytes < 1 || num_bytes > 4  {
            return String::from(
                format!("ERROR: Cannot print dword row. Illegal byte indexes provided. from_byte_ix: {} to_byte_ix: {}", 
                from_byte_ix, 
                to_byte_ix));
        }

        // Format masked bits (line 1)
        result.push_str(
            &format!(
                "{0:7}{1}{2:^7}{1}",
                "",
                "|".color(self.format_style.border_color),
                "DWORD".color(self.format_style.dword_title_color),
            )
        );
        result.push_str(
            &(0..num_bytes)
                .map(|i| format!(
                    "{1}{0}",
                    "|".color(self.format_style.border_color), 
                    &byte_str(masked_bits[i], BITS_IN_BYTE as u8).color(self.format_style.bit_color)))
                .collect::<String>()
        );
        result.push_str("\n");

        // Line 2: Masked char previews
        result.push_str(
            &format!(
                "{0:7}{1}{2:^7}{1}",
                "",
                "|".color(self.format_style.border_color),
                &dword_number.to_string().color(self.format_style.dword_title_color)
            )
        );
        match num_bytes {
            1 => result.push_str(&format!(
                "{0:1}{3:>5}{0:5}{2}{0:1}{1}",
                "",
                "|".color(self.format_style.border_color),
                "MSK".color(self.format_style.notes_color),
                &format!("({})", masked_bits[0]).color(self.format_style.byte_value_color)
            )),
            2 => result.push_str(&format!(
                "{0:1}{3:>5}{0:6}{2}{0:2}{4:>5}{0:6}{1}",
                "",
                "|".color(self.format_style.border_color),
                "MASKED".color(self.format_style.notes_color),
                &format!("({})", masked_bits[0]).color(self.format_style.byte_value_color),
                &format!("({})", masked_bits[1]).color(self.format_style.byte_value_color),
            )),
            3 => result.push_str(&format!(
                "{0:1}{4:>5}{0:6}{2}{0:2}{5:>5}{0:6}{1}{0:1}{6:>5}{0:5}{3}{0:1}{1}",
                "",
                "|".color(self.format_style.border_color),
                "MASKED".color(self.format_style.notes_color),
                "MSK".color(self.format_style.notes_color),
                &format!("({})", masked_bits[0]).color(self.format_style.byte_value_color),
                &format!("({})", masked_bits[1]).color(self.format_style.byte_value_color),
                &format!("({})", masked_bits[2]).color(self.format_style.byte_value_color),
            )),
            4 => result.push_str(&format!(
                "{0:1}{4:>5}{0:6}{2}{0:2}{5:>5}{0:6}{1}{0:1}{6:>5}{0:6}{3}{0:2}{7:>5}{0:6}{1}",
                "",
                "|".color(self.format_style.border_color),
                "MASKED".color(self.format_style.notes_color),
                "MASKED".color(self.format_style.notes_color),
                &format!("({})", masked_bits[0]).color(self.format_style.byte_value_color),
                &format!("({})", masked_bits[1]).color(self.format_style.byte_value_color),
                &format!("({})", masked_bits[2]).color(self.format_style.byte_value_color),
                &format!("({})", masked_bits[3]).color(self.format_style.byte_value_color)
            )),
            _ => {}
        }
        result.push_str("\n");

        // Line 3: Unmasked bits
        result.push_str(
            &format!(
                "{0:7}{1}{0:7}{1}",
                "",
                "|".color(self.format_style.border_color),
            )
        );
        result.push_str(
            &(0..num_bytes)
                .map(|i| format!(
                    "{1}{0}", 
                    "|".color(self.format_style.border_color), 
                    &byte_str(unmasked_bits[i], BITS_IN_BYTE as u8).color(self.format_style.unmasked_payload_bit_color)))
                .collect::<String>(),
        );
        result.push_str("\n");

        // Line 4: Unmasked char previews
        result.push_str(&format!("{0:7}{1}{0:7}{1}", "", "|".color(self.format_style.border_color)));
        match num_bytes {
            1 => result.push_str(&format!(
                "{0:1}{3:>5}{0:1}{4}{0:1}{2}{0:1}{1}",
                "",
                "|".color(self.format_style.border_color),
                "UNM".color(self.format_style.notes_color),
                &format!("({})", unmasked_bits[0]).color(self.format_style.byte_value_color),
                &format!("'{0}'", payload_data[0]).color(self.format_style.data_value_color),
            )),
            2 => result.push_str(&format!(
                "{0:1}{3:>5}{0:1}{4:3}{0:1}{2}{0:1}{5:>5}{0:1}{6:3}{0:2}{1}",
                "",
                "|".color(self.format_style.border_color),
                "UNMASKED".color(self.format_style.notes_color),
                &format!("({})", unmasked_bits[0]).color(self.format_style.byte_value_color),
                &format!("'{}'", payload_data[0]).color(self.format_style.data_value_color),
                &format!("({})", unmasked_bits[1]).color(self.format_style.byte_value_color),
                &format!("'{}'", payload_data[1]).color(self.format_style.data_value_color)
            )),
            3 => result.push_str(&format!(
                "{0:1}{4:>5}{0:1}{5:3}{0:1}{2}{0:1}{6:>5}{0:1}{7:3}{0:2}{1}{0:1}{8:>5}{0:1}{9:3}{0:1}{3}{0:1}{1}",
                "",
                "|".color(self.format_style.border_color),
                "UNMASKED".color(self.format_style.notes_color),
                "UNM".color(self.format_style.notes_color),
                &format!("({})", unmasked_bits[0]).color(self.format_style.byte_value_color),
                &format!("'{}'", payload_data[0]).color(self.format_style.data_value_color),
                &format!("({})", unmasked_bits[1]).color(self.format_style.byte_value_color),
                &format!("'{}'", payload_data[1]).color(self.format_style.data_value_color),
                &format!("({})", unmasked_bits[2]).color(self.format_style.byte_value_color),
                &format!("'{}'", payload_data[2]).color(self.format_style.data_value_color),
            )),
            4 => result.push_str(&format!(
                "{0:1}{3:>5}{0:1}{4:3}{0:1}{2}{0:1}{5:>5}{0:1}{6:3}{0:2}{1}{0:1}{7:>5}{0:1}{8:3}{0:1}{2}{0:1}{9:>5}{0:1}{10:3}{0:2}{1}",
                "",
                "|".color(self.format_style.border_color),
                "UNMASKED".color(self.format_style.notes_color),
                &format!("({})", unmasked_bits[0]).color(self.format_style.byte_value_color),
                &format!("'{}'", payload_data[0]).color(self.format_style.data_value_color),
                &format!("({})", unmasked_bits[1]).color(self.format_style.byte_value_color),
                &format!("'{}'", payload_data[1]).color(self.format_style.data_value_color),
                &format!("({})", unmasked_bits[2]).color(self.format_style.byte_value_color),
                &format!("'{}'", payload_data[2]).color(self.format_style.data_value_color),
                &format!("({})", unmasked_bits[3]).color(self.format_style.byte_value_color),
                &format!("'{}'", payload_data[3]).color(self.format_style.data_value_color),
            )),
            _ => {}
        }
        result.push_str("\n");

        // Line 5: Payload part
        result.push_str(&format!("{0:7}{1}{0:7}{1}", "", "|".color(self.format_style.border_color)));
        match num_bytes {
            1 => result.push_str(&format!(
                "{1:^15}{0}",
                "|".color(self.format_style.border_color),
                &format!("Payload pt {}", part_number).color(self.format_style.notes_color),
            )),
            2 => result.push_str(&format!(
                "{1:^31}{0}",
                "|".color(self.format_style.border_color),
                &format!("Payload Data (part {})", part_number).color(self.format_style.notes_color),
            )),
            3 => result.push_str(&format!(
                "{1:^47}{0}",
                "|".color(self.format_style.border_color),
                &format!("Payload Data (part {})", part_number).color(self.format_style.notes_color),
            )),
            4 => result.push_str(&format!(
                "{1:^63}{0}",
                "|".color(self.format_style.border_color),
                &format!("Payload Data (part {})", part_number).color(self.format_style.notes_color),
            )),
            _ => {}
        }
        result.push_str("\n");

        // Format bottom border
        result.push_str(&format!("{0:7}{1}", "", "+-------+".color(self.format_style.border_color)));
        result.push_str(
            &(0..num_bytes)
                .map(|_| "---------------+".color(self.format_style.border_color).to_string())
                .collect::<String>(),
        );
        result.push_str("\n");

        result
    }

    /// Derives a WebSocket payload length from its payload length code and extension bytes.
    /// 
    /// Per RFC 6455 Section 5.2: https://tools.ietf.org/html/rfc6455#section-5.2
    /// 
    /// # Arguments
    /// 
    /// * `code` - The payload length code.
    /// * `ext_bytes` - The extension bytes.
    fn get_payload_length(
        code: u8, 
        ext_bytes: Vec<u8>
    ) -> PayloadLength {
        // Code <= 125: The code *is* the payload length
        if code <= 125 {
            return PayloadLength::Short(code);
        }
        // Code 126: The first 2 extension bytes contain the payload length
        if code == 126 {
            return PayloadLength::Medium(u16::from_be_bytes([ext_bytes[0], ext_bytes[1]]));
        }
        // Code 127: The 8 extension bytes contain the payload length
        if code == 127 {
            return PayloadLength::Long(u64::from_be_bytes([ext_bytes[0], ext_bytes[1], ext_bytes[2], ext_bytes[3], ext_bytes[4], ext_bytes[5], ext_bytes[6], ext_bytes[7]]));
        }
        // Code must have been an 8-bit value
        panic!("ERROR: Unable to determine payload length from code: {}", code);
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

    /// Tests that a short length frame with a masked text payload is formatted correctly.
    #[test]
    fn test_short_masked_text_frame() {
        let bytes = base64::decode("gYR7q0rdD845qQ==").unwrap();

        let frame = WebSocketFrame::from_bytes(&bytes);
        // let expected = "               \u{1b}[36m+---------------+---------------+---------------+---------------+\u{1b}[0m\n  \u{1b}[37mFrame Data\u{1b}[0m   \u{1b}[36m|\u{1b}[0m\u{1b}[32m    Byte  1    \u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[32m    Byte  2    \u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[32m    Byte  3    \u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[32m    Byte  4    \u{1b}[0m\u{1b}[36m|\u{1b}[0m\n  \u{1b}[37m (Masked) \u{1b}[0m   \u{1b}[36m+---------------+---------------+---------------+---------------+\u{1b}[0m\n   Short(4)    \u{1b}[36m|\u{1b}[0m\u{1b}[32m0\u{1b}[0m              \u{1b}[36m|\u{1b}[0m    \u{1b}[32m1\u{1b}[0m          \u{1b}[36m|\u{1b}[0m        \u{1b}[32m2\u{1b}[0m      \u{1b}[36m|\u{1b}[0m            \u{1b}[32m3\u{1b}[0m  \u{1b}[36m|\u{1b}[0m\n               \u{1b}[36m|\u{1b}[0m\u{1b}[32m0 1 2 3 4 5 6 7\u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[32m8 9 0 1 2 3 4 5\u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[32m6 7 8 9 0 1 2 3\u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[32m4 5 6 7 8 9 0 1\u{1b}[0m\u{1b}[36m|\u{1b}[0m\n       \u{1b}[36m+-------+---------------+---------------+---------------+---------------+\u{1b}[0m\n       \u{1b}[36m|\u{1b}[0m\u{1b}[32m DWORD \u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[37m1\u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[37m0\u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[37m0\u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[37m0\u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[37m0 0 0 1\u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[37m1\u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[37m0 0 0 0 1 0 0\u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[37m0 1 1 1 1 0 1 1\u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[37m1 0 1 0 1 0 1 1\u{1b}[0m\u{1b}[36m|\u{1b}[0m\n       \u{1b}[36m|\u{1b}[0m\u{1b}[32m   1   \u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[35mF\u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[35mR\u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[35mR\u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[35mR\u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[31m Text  \u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[35mM\u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[31m  Short(4)   \u{1b}[0m\u{1b}[36m|\u{1b}[0m                               \u{1b}[36m|\u{1b}[0m\n       \u{1b}[36m|\u{1b}[0m       \u{1b}[36m|\u{1b}[0m\u{1b}[35mI\u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[35mS\u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[35mS\u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[35mS\u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[35mop code\u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[35mA\u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[35m Payload len \u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[35m     Masking-key (part 1)      \u{1b}[0m\u{1b}[36m|\u{1b}[0m\n       \u{1b}[36m|\u{1b}[0m       \u{1b}[36m|\u{1b}[0m\u{1b}[35mN\u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[35mV\u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[35mV\u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[35mV\u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[35m (4 b) \u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[35mS\u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[35m  (7 bits)   \u{1b}[0m\u{1b}[36m|\u{1b}[0m                               \u{1b}[36m|\u{1b}[0m\n       \u{1b}[36m|\u{1b}[0m       \u{1b}[36m|\u{1b}[0m \u{1b}[36m|\u{1b}[0m\u{1b}[35m1\u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[35m2\u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[35m3\u{1b}[0m\u{1b}[36m|\u{1b}[0m       \u{1b}[36m|\u{1b}[0m\u{1b}[35mK\u{1b}[0m\u{1b}[36m|\u{1b}[0m             \u{1b}[36m|\u{1b}[0m                               \u{1b}[36m|\u{1b}[0m\n       \u{1b}[36m+-------+-+-+-+-+-------+-+-------------+-------------------------------+\u{1b}[0m\n       \u{1b}[36m|\u{1b}[0m\u{1b}[32m DWORD \u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[37m0 1 0 0 1 0 1 0\u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[37m1 1 0 1 1 1 0 1\u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[37m0 0 0 0 1 1 1 1\u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[37m1 1 0 0 1 1 1 0\u{1b}[0m\u{1b}[36m|\u{1b}[0m\n       \u{1b}[36m|\u{1b}[0m\u{1b}[32m   2   \u{1b}[0m\u{1b}[36m|\u{1b}[0m                               \u{1b}[36m|\u{1b}[0m \u{1b}[34m (15)\u{1b}[0m      \u{1b}[35mMASKED\u{1b}[0m  \u{1b}[34m(206)\u{1b}[0m      \u{1b}[36m|\u{1b}[0m\n       \u{1b}[36m|\u{1b}[0m       \u{1b}[36m|\u{1b}[0m\u{1b}[35m     Masking-key (part 2)      \u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[33m0 1 1 1 0 1 0 0\u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[33m0 1 1 0 0 1 0 1\u{1b}[0m\u{1b}[36m|\u{1b}[0m\n       \u{1b}[36m|\u{1b}[0m       \u{1b}[36m|\u{1b}[0m\u{1b}[35m           (16 bits)           \u{1b}[0m\u{1b}[36m|\u{1b}[0m \u{1b}[34m(116)\u{1b}[0m \u{1b}[31m\'t\'\u{1b}[0m \u{1b}[35mUNMASKED\u{1b}[0m \u{1b}[34m(101)\u{1b}[0m \u{1b}[31m\'e\'\u{1b}[0m  \u{1b}[36m|\u{1b}[0m\n       \u{1b}[36m|\u{1b}[0m       \u{1b}[36m|\u{1b}[0m                               \u{1b}[36m|\u{1b}[0m\u{1b}[35m     Payload Data (part 1)     \u{1b}[0m\u{1b}[36m|\u{1b}[0m\n       \u{1b}[36m+-------+-------------------------------+-------------------------------+\u{1b}[0m\n       \u{1b}[36m|\u{1b}[0m\u{1b}[32m DWORD \u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[37m0 0 1 1 1 0 0 1\u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[37m1 0 1 0 1 0 0 1\u{1b}[0m\u{1b}[36m|\u{1b}[0m\n       \u{1b}[36m|\u{1b}[0m\u{1b}[32m   3   \u{1b}[0m\u{1b}[36m|\u{1b}[0m \u{1b}[34m (57)\u{1b}[0m      \u{1b}[35mMASKED\u{1b}[0m  \u{1b}[34m(169)\u{1b}[0m      \u{1b}[36m|\u{1b}[0m\n       \u{1b}[36m|\u{1b}[0m       \u{1b}[36m|\u{1b}[0m\u{1b}[33m0 1 1 1 0 0 1 1\u{1b}[0m\u{1b}[36m|\u{1b}[0m\u{1b}[33m0 1 1 1 0 1 0 0\u{1b}[0m\u{1b}[36m|\u{1b}[0m\n       \u{1b}[36m|\u{1b}[0m       \u{1b}[36m|\u{1b}[0m \u{1b}[34m(115)\u{1b}[0m \u{1b}[31m\'s\'\u{1b}[0m \u{1b}[35mUNMASKED\u{1b}[0m \u{1b}[34m(116)\u{1b}[0m \u{1b}[31m\'t\'\u{1b}[0m  \u{1b}[36m|\u{1b}[0m\n       \u{1b}[36m|\u{1b}[0m       \u{1b}[36m|\u{1b}[0m\u{1b}[35m     Payload Data (part 2)     \u{1b}[0m\u{1b}[36m|\u{1b}[0m\n       \u{1b}[36m+-------+\u{1b}[0m\u{1b}[36m---------------+\u{1b}[0m\u{1b}[36m---------------+\u{1b}[0m\n";
        
        println!("{}", frame.format());

        //assert_eq!(frame.format(), expected);
    }

    /// Tests that a medium length frame with a masked text payload is formatted correctly.
    #[test]
    fn test_medium_masked_text_frame() {
        // Medium length
        let medium_bytes = base64::decode("gf4Ago6okLi/mqOMu56ngLeYoYq9nKWOuZCpiL+ao4y7nqeAt5ihir2cpY65kKmIv5qjjLuep4C3mKGKvZyljrmQqYi/mqOMu56ngLeYoYq9nKWOuZCpiL+ao4y7nqeAt5ihir2cpY65kKmIv5qjjLuep4C3mKGKvZyljrmQqYi/mqOMu56ngLeY").unwrap();
        let medium_frame = WebSocketFrame::from_bytes(&medium_bytes);

        println!("{}", medium_frame.format());

    }
}

// #endregion WebSocket Frame Unit Tests
