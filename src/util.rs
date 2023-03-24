use std::io;
use std::mem;
use aes::Aes128;
use aes::Aes128Dec;
use aes::cipher::{BlockDecrypt, KeyInit};
use aes::cipher::generic_array::GenericArray;
use openssl::rsa::{Padding, Rsa};

use crate::bhd::FileHeader;

#[derive(Debug)]
pub(crate) enum HeaderParseError {
    IO(io::Error),
    Decrypt,
    InvalidKey,
}

pub(crate) fn decrypt_header(file: &[u8], key: &[u8]) -> Result<Vec<u8>, HeaderParseError> {
    let public_key = Rsa::public_key_from_pem_pkcs1(key)
        .map_err(|_| HeaderParseError::InvalidKey)?;

    let key_size = public_key.size() as usize;
    let mut len = 0;
    let mut decrypted_data = Vec::new();

    while len < file.len() {
        let mut decrypted_block = vec![0; key_size];
        let next_block = len + key_size;
        let block_data = &file[len..next_block];

        len += public_key.public_decrypt(block_data, &mut decrypted_block, Padding::NONE)
            .map_err(|_| HeaderParseError::Decrypt)?;

        decrypted_data.extend_from_slice(&decrypted_block[1..]);
    }

    decrypted_data.truncate(len as usize);

    Ok(decrypted_data)
}

#[derive(Debug)]
pub(crate) enum FileExtractionError {
    InvalidKey,
    Decrypt,
}

pub(crate) fn decrypt_with_file_header(encrypted_data: &[u8], header: &FileHeader) -> Result<Vec<u8>, FileExtractionError> {
    let key = GenericArray::from_slice(&header.aes_key);
    let cipher = Aes128::new(&key);

    // Copy the data so we can decrypt it in-place
    let mut data_copy = encrypted_data.to_vec();

    // Apparently they only encrypt parts of the files instead of the whole file
    for (start, end) in header.aes_ranges.iter() {
        if *start == -1 {
            continue;
        }

        let encrypted_range = &mut data_copy[*start as usize..*end as usize];

        // Decrypt by chunks of key size. This is done serially explicitly because we're already
        // threading on a per-file basis.
        encrypted_range.chunks_mut(16)
            .map(|c| GenericArray::from_mut_slice(c))
            .for_each(|b| cipher.decrypt_block(b));
    }

    data_copy.truncate(header.file_size as usize);

    Ok(data_copy)
}

pub(crate) fn read_as_type<T: Sized + Default>(reader: &mut impl io::Read) -> Result<T, io::Error> {
    let result = T::default();

    unsafe {
        let buffer: &mut [u8] = std::slice::from_raw_parts_mut(
            &result as *const T as *const u8 as *mut u8,
            mem::size_of::<T>(),
        );

        reader.read_exact(buffer)?;
    }

    Ok(result)
}
