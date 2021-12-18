use crate::{
    models::{DataChunks, TopicArgument, WORD_LENGTH},
    numbers::from_hex_string,
};
use std::ops::Div;

impl DataChunks {
    fn value_of_dyn_type(&self, start_index: usize) -> String {
        let offset_bytes = from_hex_string(self.get(start_index)).expect(&format!(
            "Dynamic type at index {} has unexpected offset",
            start_index
        )) as usize;

        let word_length_bytes = (WORD_LENGTH / 2) as usize;
        let offset_index = offset_bytes / word_length_bytes;

        let value_size_hex = from_hex_string(self.get(offset_index)).expect(&format!(
            "Dynamic type at index {} has unexpected data size",
            start_index
        )) * 2; // we are interested in hex

        let mut current_data_index = offset_index + 1;
        let mut output = String::new();
        output.push_str("0x");
        let data_chunks = value_size_hex.div(WORD_LENGTH as u64);
        let last_chunk_carry = value_size_hex % (WORD_LENGTH as u64);

        if data_chunks > 0 {
            for _index in 0..data_chunks {
                output.push_str(self.get(current_data_index));
                current_data_index += 1;
            }
        }
        if last_chunk_carry > 0 {
            output.push_str(&self.get(current_data_index)[0..(last_chunk_carry as usize)]);
        }
        output
    }
}

impl TopicArgument {
    fn parse(&self, index: usize, chunks: &DataChunks) -> String {
        let output = match self {
            TopicArgument::Address => format!("0x{}", chunks.as_slice()[index][24..64].to_string()),
            TopicArgument::Uint8 => from_hex_string(&chunks.as_slice()[index][56..64])
                .expect("Uint8 parse error")
                .to_string(),
            TopicArgument::Uint256 => from_hex_string(&chunks.as_slice()[index])
                .expect("Uint256 parse error")
                .to_string(),
            TopicArgument::Bytes32 => format!("0x{}", chunks.as_slice()[index]),
            TopicArgument::Bytes => chunks.value_of_dyn_type(index),
        };
        output
    }
}

#[cfg(test)]
mod tests {
    use crate::models::{DataChunks, TopicArgument};

    #[test]
    fn static_type() {
        let topic_argument = TopicArgument::Address;
        let chunks = DataChunks::new(vec![
            "00000000000000000000000026a7ecdb60d38b06fffeba426713aa191cffc2ed".to_string(),
        ]);
        let expected = "0x26a7ecdb60d38b06fffeba426713aa191cffc2ed";

        let actual = topic_argument.parse(0, &chunks);
        assert_eq!(actual, expected);
    }

    #[test]
    fn dynamic_type() {
        let topic_argument = TopicArgument::Bytes;
        let chunks = DataChunks::new(vec![
            "00000000000000000000000026a7ecdb60d38b06fffeba426713aa191cffc2ed".to_string(),
            "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            "0000000000000000000000000000000000000000000000000000000000000160".to_string(), // offset of bytes
            "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            "0000000000000000000000000000000000000000000000000000000000011ef3".to_string(),
            "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            "00000000000000000000000000000000000000000000000000000000000001e0".to_string(),
            "0000000000000000000000000000000000000000000000000000000000000260".to_string(),
            "0000000000000000000000000000000000000000000000000000000000000044".to_string(), // size of bytes
            "0d582f13000000000000000000000000be8c10dbf4c6148f9834c56c3331f819".to_string(),
            "1f35555200000000000000000000000000000000000000000000000000000000".to_string(),
            "0000000100000000000000000000000000000000000000000000000000000000".to_string(),
        ]);

        let expected = "0x0d582f13000000000000000000000000be8c10dbf4c6148f9834c56c3331f819\
        1f35555200000000000000000000000000000000000000000000000000000000\
        00000001";

        let actual = topic_argument.parse(2, &chunks);
        assert_eq!(actual, expected);
    }

    #[test]
    fn full_tx_log_parsing() {
        let topic_argument = vec![
            TopicArgument::Address,
            TopicArgument::Uint256,
            TopicArgument::Bytes,
            // ignoring arguments after bytes. Index > 2
        ];
        let chunks = DataChunks::new(vec![
            "00000000000000000000000026a7ecdb60d38b06fffeba426713aa191cffc2ed".to_string(),
            "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            "0000000000000000000000000000000000000000000000000000000000000160".to_string(), // offset of bytes
            "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            "0000000000000000000000000000000000000000000000000000000000011ef3".to_string(),
            "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            "00000000000000000000000000000000000000000000000000000000000001e0".to_string(),
            "0000000000000000000000000000000000000000000000000000000000000260".to_string(),
            "0000000000000000000000000000000000000000000000000000000000000044".to_string(), // size of bytes
            "0d582f13000000000000000000000000be8c10dbf4c6148f9834c56c3331f819".to_string(),
            "1f35555200000000000000000000000000000000000000000000000000000000".to_string(),
            "0000000100000000000000000000000000000000000000000000000000000000".to_string(),
        ]);

        let expected = [
            "0x26a7ecdb60d38b06fffeba426713aa191cffc2ed",
            "0",
            "0x0d582f13000000000000000000000000be8c10dbf4c6148f9834c56c3331f819\
            1f35555200000000000000000000000000000000000000000000000000000000\
            00000001",
        ];

        for (index, topic_argument) in topic_argument.iter().enumerate() {
            let actual = topic_argument.parse(index, &chunks);
            assert_eq!(actual, expected[index]);
        }
    }
}
