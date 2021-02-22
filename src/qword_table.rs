const BITS_IN_BYTE: u8 = 8;

pub struct QwordTable<'a> {
    data: &'a Vec<u8>
}

impl<'a> QwordTable<'a> {
    pub fn from_bytes(data: &Vec<u8>) -> QwordTable {
        QwordTable {
            data
        }
    }

    /// Formats a vector of bytes as a qword table.
    /// 
    /// # Arguments
    /// 
    /// * `data` - The bytes to format.
    pub fn format(self: &QwordTable<'a>) -> String {
        let mut result = self.format_qword_table_header();
        let num_qwords = self.data.len().div_euclid(BITS_IN_BYTE as usize);
        // Append full qwords
        for i in 0..num_qwords {
            let from_byte_ix = i * BITS_IN_BYTE as usize;
            let to_byte_ix = from_byte_ix + BITS_IN_BYTE as usize;
            let qword_number: usize = i + 1;
            result.push_str(&self.format_qword_row(qword_number, &self.data[from_byte_ix..to_byte_ix], BITS_IN_BYTE as usize));
        }
        // Append final bytes
        let remaining_bytes = self.data.len().rem_euclid(BITS_IN_BYTE as usize);
        let from_byte_ix: usize = num_qwords * BITS_IN_BYTE as usize;
        let to_byte_ix: usize = from_byte_ix + remaining_bytes as usize;
        let qword_number: usize = num_qwords + 1;
        result.push_str(&self.format_qword_row(
            qword_number,
            &self.data[from_byte_ix..to_byte_ix],
            remaining_bytes,
        ));
        result
    }

    /// Formats the header for a qword table.
    fn format_qword_table_header(self: &QwordTable<'a>) -> String {
        // Top border
        let mut result = String::from("       +");
        result.push_str(
            &(0..BITS_IN_BYTE)
                .map(|_| "--------+")
                .collect::<String>(),
        );
        // Append table label
        result.push_str("\n Bytes |");
        // Append column labels
        result.push_str(
            &(0..BITS_IN_BYTE)
                .map(|i| format!(" Byte {} |", i))
                .collect::<String>(),
        );
        // Append bottom border
        result.push_str("\n+------+");
        result.push_str(
            &(0..BITS_IN_BYTE)
                .map(|_| "--------+")
                .collect::<String>(),
        );
        result.push_str("\n");
        result
    }

    /// Formats a row of bytes in a qword table.
    /// 
    /// # Arguments
    /// 
    /// * `qword_number` - The sequence number of this qword.
    /// * `data` - The bytes within the qword to format.
    /// * `num_bytes` - The number of bytes to format.
    fn format_qword_row(self: &QwordTable<'a>, qword_number: usize, data: &[u8], num_bytes: usize) -> String {
        if data.len() != num_bytes {
            return format!("ERROR: Data must contain exactly {} bytes. QWORD: {}\n", num_bytes, qword_number);
        }

        // Row header
        let mut result = String::from("|QWORD |");
        // Append byte values
        result.push_str(
            &(0..num_bytes)
                .map(|i| format!("{:0>8b}|", data[i]))
                .collect::<String>(),
        );
        // Append qword number
        result.push_str(&format!("\n|{:^6}|", qword_number));
        // Append byte value
        result.push_str(
            &(0..num_bytes)
                .map(|i| format!("{:>8}|", format!("({})", data[i])))
                .collect::<String>(),
        );
        // Append bottom border
        result.push_str("\n+------+");
        result.push_str(
            &(0..num_bytes)
                .map(|_| "--------+")
                .collect::<String>(),
        );
        result.push_str("\n");
        result
    }
}

#[cfg(test)]
mod tests {
    // #[test]
    // fn test_qword_one_byte_basic() {
    //     use super::*;
    //     let result = qword_row_one_byte(2, &[129]);
    //     assert_eq!(
    //         "\n|QWORD |10000001|\n|  2   |   (129)|\n+------+--------+",
    //         result
    //     );
    // }
}
