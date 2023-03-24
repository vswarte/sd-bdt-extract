use std::io;
use crate::util::read_as_type;

#[derive(Debug)]
pub(crate) struct Header {
    pub magic: [u8; 4],
    pub endianness: i8,
    pub unk1: u8,
    pub unk2: u8,
    pub unk3: u8,
    pub unk4: u32,
    pub file_size: u32,
    pub bucket_count: u32,
    pub bucket_offset: u32,
    pub salt_length: u32,
    pub salt: Vec<u8>,
}

impl Header {
    pub fn from_reader(reader: &mut impl io::Read) -> Result<Self, io::Error> {
        let magic = read_as_type::<[u8; 4]>(reader)?;
        let endianness = read_as_type::<i8>(reader)?;
        let unk1 = read_as_type::<u8>(reader)?;
        let unk2 = read_as_type::<u8>(reader)?;
        let unk3 = read_as_type::<u8>(reader)?;
        let unk4 = read_as_type::<u32>(reader)?;
        let file_size = read_as_type::<u32>(reader)?;
        let bucket_count = read_as_type::<u32>(reader)?;
        let bucket_offset = read_as_type::<u32>(reader)?;
        let salt_length = read_as_type::<u32>(reader)?;

        let mut salt = vec![0u8; salt_length as usize];
        reader.read_exact(salt.as_mut_slice())?;

        Ok(Self {
            magic,
            endianness,
            unk1,
            unk2,
            unk3,
            unk4,
            file_size,
            bucket_count,
            bucket_offset,
            salt_length,
            salt,
        })
    }
}

#[derive(Debug)]
pub(crate) struct Bucket {
    pub file_header_count: i32,
    pub file_header_offset: i32,
}

impl Bucket {
    pub fn from_reader(reader: &mut impl io::Read) -> Result<Self, io::Error> {
        Ok(Self {
            file_header_count: read_as_type::<i32>(reader)?,
            file_header_offset: read_as_type::<i32>(reader)?,
        })
    }
}

#[derive(Debug)]
pub(crate) struct FileHeader {
    pub file_path_hash: u64,
    pub padded_file_size: u32,
    pub file_size: u32,
    pub file_offset: u64,
    pub aes_key: [u8; 16],
    pub aes_ranges: Vec<(i64, i64)>,
}

impl FileHeader {
    pub fn from_reader(reader: &mut (impl io::Read + io::Seek)) -> Result<Self, io::Error> {
        let file_path_hash = read_as_type::<u64>(reader)?;
        let padded_file_size = read_as_type::<u32>(reader)?;
        let file_size = read_as_type::<u32>(reader)?;
        let file_offset = read_as_type::<u64>(reader)?;

        // Sha offset
        let _ = read_as_type::<u64>(reader)?;

        let aes_key_offset = read_as_type::<u64>(reader)?;
        let mut aes_ranges = Vec::new();

        // Read AES key, the ranges and get the fuck back
        let current_position = reader.seek(io::SeekFrom::Current(0))
            .expect("Could not get current position");

        reader.seek(io::SeekFrom::Start(aes_key_offset))
            .expect("Could not seek to AES key");

        let mut aes_key = [0u8; 16];
        reader.read_exact(&mut aes_key).expect("Could not read AES key");

        let aes_range_count = read_as_type::<u32>(reader)?;
        for _ in 0..aes_range_count {
            aes_ranges.push((read_as_type::<i64>(reader)?, read_as_type::<i64>(reader)?));
        }

        reader.seek(io::SeekFrom::Start(current_position))
            .expect("Could not seek back after reading AES key");

        Ok(Self {
            file_path_hash,
            padded_file_size,
            file_size,
            file_offset,
            aes_key,
            aes_ranges,
        })
    }
}
