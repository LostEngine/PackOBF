use std::io::Error;
use libdeflater::{adler32, CompressionLvl};
use crate::profile_scope;

pub fn libdeflater_zlib(input: &[u8], level: u8, checksum: bool) -> Result<Vec<u8>, Error> {
    profile_scope!(std::any::type_name_of_val(&libdeflater_zlib));
    let mut compressed_data = libdeflater_deflate(input, level)?;

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

pub fn libdeflater_deflate(input: &[u8], level: u8) -> Result<Vec<u8>, Error> {
    profile_scope!(std::any::type_name_of_val(&libdeflater_deflate));
    let mut compressor = libdeflater::Compressor::new(CompressionLvl::new(level as i32).unwrap());
    let mut out = vec![0_u8; compressor.deflate_compress_bound(input.len())];
    let size = compressor
        .deflate_compress(input, &mut out)
        .map_err(|_| Error::other("Compression failed"))?;
    out.truncate(size);
    Ok(out)
}