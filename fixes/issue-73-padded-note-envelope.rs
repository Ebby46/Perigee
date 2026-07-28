//! Fix for #73: pad shielded notes to a fixed-size envelope so size and
//! pattern metadata can't be used for timing/size correlation.

pub const NOTE_ENVELOPE_SIZE: usize = 512;

pub fn pad_note(mut note: Vec<u8>) -> Result<Vec<u8>, &'static str> {
    if note.len() > NOTE_ENVELOPE_SIZE {
        return Err("note exceeds fixed envelope size");
    }
    note.resize(NOTE_ENVELOPE_SIZE, 0);
    Ok(note)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_notes_produce_uniform_size_envelope() {
        let small = pad_note(vec![1, 2, 3]).unwrap();
        let large = pad_note(vec![9; 500]).unwrap();
        assert_eq!(small.len(), NOTE_ENVELOPE_SIZE);
        assert_eq!(large.len(), NOTE_ENVELOPE_SIZE);
    }

    #[test]
    fn oversized_note_is_rejected() {
        assert!(pad_note(vec![0; 600]).is_err());
    }
}
