use std::io::{Error, Write};
use libdeflater::adler32;
use crate::profile_scope;

pub fn zopfli_zlib(input: &[u8], options: zopfli::Options, checksum: bool) -> Result<Vec<u8>, Error> {
    profile_scope!(std::any::type_name_of_val(&zopfli_zlib));
    let mut compressed_data = zopfli_deflate(input, options)?;

    let checksum = if checksum {
        adler32(input).to_be_bytes()
    } else {
        [0; 4]
    };

    let mut out = Vec::with_capacity(2 + compressed_data.len() + 4);
    out.extend_from_slice(&[0x78, 0x9C]); // zlib header (fixed value) https://en.wikipedia.org/wiki/Zlib#Data_format
    out.append(&mut compressed_data);
    out.extend_from_slice(&checksum);

    Ok(out)
}

pub fn zopfli_deflate(input: &[u8], options: zopfli::Options) -> Result<Vec<u8>, Error> {
    profile_scope!(std::any::type_name_of_val(&zopfli_deflate));
    let mut encoder = zopfli::DeflateEncoder::new(options, zopfli::BlockType::Dynamic, Vec::new());
    encoder.write_all(input)?;
    encoder.finish()
}
