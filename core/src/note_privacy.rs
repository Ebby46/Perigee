pub struct NoteEnvelope {
    pub padded_data: Vec<u8>,
    pub fixed_size: usize,
}

impl NoteEnvelope {
    pub fn new(payload: &[u8], target_size: usize) -> Self {
        let padded = Self::pad_to_fixed_size(payload, target_size);
        Self {
            padded_data: padded,
            fixed_size: target_size,
        }
    }

    pub fn from_string(text: &str, target_size: usize) -> Self {
        Self::new(text.as_bytes(), target_size)
    }

    pub fn pad_to_fixed_size(data: &[u8], target_size: usize) -> Vec<u8> {
        let mut padded = data.to_vec();
        if padded.len() < target_size {
            let padding_len = target_size - padded.len();
            for i in 0..padding_len {
                padded.push((i % 256) as u8);
            }
        } else {
            padded.truncate(target_size);
        }
        padded
    }

    pub fn reveal_size_hint(&self) -> usize {
        self.fixed_size
    }
}

pub struct NoteSanitizer;

impl NoteSanitizer {
    pub fn sanitize_metadata(metadata: &str) -> String {
        metadata
            .chars()
            .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
            .collect()
    }

    pub fn redact_counterparties(metadata: &str) -> String {
        metadata
            .replace("from:", "from: [REDACTED]")
            .replace("to:", "to: [REDACTED]")
            .replace("counterparty:", "counterparty: [REDACTED]")
    }

    pub fn hash_identifier(id: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        id.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }
}
