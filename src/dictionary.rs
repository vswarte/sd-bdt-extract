use std::fs;
use std::path::Path;
use std::num::Wrapping;
use std::collections::HashMap;

use crate::config;

pub(crate) fn build_dictionary() -> HashMap<u64, String> {
    let dictionary_file_contents = {
        if Path::new(config::DICTIONARY_OVERRIDE_PATH).exists() {
            fs::read_to_string(config::DICTIONARY_OVERRIDE_PATH)
                .expect("Could not read ./dictionary.txt")
        } else {
            config::STANDARD_DICTIONARY.to_string()
        }
    };

    let mut result = HashMap::new();

    for line in dictionary_file_contents.lines() {
        result.insert(hash_path(line), line.to_string());
    }

    result
}

const HASH_PRIME: Wrapping<u64> = Wrapping(0x85);

pub(crate) fn hash_path(input: &str) -> u64 {
    let mut result = Wrapping(0);

    for character in input.as_bytes() {
        result = result * HASH_PRIME + Wrapping(*character as u64);
    }

    result.0
}
