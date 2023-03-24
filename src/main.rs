use std::fs;
use std::io::{Cursor, SeekFrom, Seek, Read, Write};
use std::path::PathBuf;

use rayon::prelude::*;
use indicatif::{ProgressIterator, ProgressStyle};
use util::decrypt_header;

use crate::dictionary::build_dictionary;
use crate::bhd::{Header, Bucket, FileHeader};
use crate::util::{HeaderParseError, decrypt_with_file_header};

mod bhd;
mod util;
mod config;
mod dictionary;

fn main() {
    let dictionary = build_dictionary();

    let file_headers = {
        let mut header_file = fs::File::open(config::SD_BHD_PATH)
            .expect("Could not open the file");

        get_file_headers(&mut header_file)
            .expect("Could not retrieve file headers")
    };

    // Read all of the data file into mem. Will most likely not work for bigger bdts, probably need
    // some queue to feed the threads files and a procedural scanner as Church Guard suggested.
    let data_buffer = {
        let mut reader = fs::File::open(config::SD_BDT_PATH)
            .expect("Could not open the file");

        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).unwrap();

        buffer
    };

    // Create a vector of data buffer cursor and file header info
    let extract_tasks = file_headers.into_iter()
        .map(|h| (Cursor::new(&data_buffer), h))
        .collect::<Vec<(Cursor<&Vec<u8>>, FileHeader)>>();

    let progress_style = ProgressStyle::with_template(config::PROGRESS_BAR_TEMPLATE).unwrap();

    extract_tasks.into_iter()
        .progress_with_style(progress_style)
        .for_each(|(mut r, h)| {
            // Determine the output path
            let output_path = PathBuf::from(match dictionary.get(&h.file_path_hash) {
                Some(a) => format!("sound/{}", a.clone()).to_string(),
                None => format!("sound/unknown/{}.bin", h.file_path_hash).to_string(),
            });

            // Create the parent directory if it doesn't exist
            {
                let mut output_dir = output_path.clone();
                output_dir.pop();
                fs::create_dir_all(&output_dir).expect("Could not create output directory");
            }

            // Copy the encrypted file from the BDT stream
            let file_buffer = {
                let mut b = Vec::new();

                // Seek to the offset
                r.seek(SeekFrom::Start(h.file_offset)).expect("Could not seek in BDT stream");

                // Copy the encrypted bytes into the bufer
                r.take(h.padded_file_size as u64)
                    .read_to_end(&mut b)
                    .expect("Could not copy data from BDT stream");

                b
            };

            // Decrypt the contents
            let decrypted_contents = decrypt_with_file_header(&file_buffer, &h)
                .expect("Could not decrypt BDT segment");

            // Create the file and write the decrypted contents
            let mut output_file = fs::File::create(&output_path)
                .expect("Could not open output file");

            output_file.write_all(&decrypted_contents)
                .expect("Could not write to output file");
        });
}

fn get_file_headers(reader: &mut impl Read) -> Result<Vec<FileHeader>, HeaderParseError> {
    let mut file_buffer = Vec::new();
    reader.read_to_end(&mut file_buffer).map_err(|e| HeaderParseError::IO(e))?;

    let decrypted_header = decrypt_header(file_buffer.as_slice(), config::SD_KEY)?;

    let mut bhd_reader = Cursor::new(decrypted_header);
    let header = Header::from_reader(&mut bhd_reader)
        .map_err(|e| HeaderParseError::IO(e))?;

    let mut buckets = Vec::new();
    for _ in 0..header.bucket_count {
        let bucket = Bucket::from_reader(&mut bhd_reader)
            .map_err(|e| HeaderParseError::IO(e))?;

        buckets.push(bucket);
    }

    let mut file_headers = Vec::new();
    for bucket in buckets.iter() {
        bhd_reader.seek(SeekFrom::Start(bucket.file_header_offset as u64))
            .map_err(|e| HeaderParseError::IO(e))?;

        for _ in 0..bucket.file_header_count {
            let file_header = FileHeader::from_reader(&mut bhd_reader)
                .map_err(|e| HeaderParseError::IO(e))?;
            file_headers.push(file_header);
        }
    }

    Ok(file_headers)
}
