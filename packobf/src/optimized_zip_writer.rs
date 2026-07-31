use crate::cache::{Cache, ItemType};
use crate::options::{analyze_and_get_zopfli_config_best, analyze_and_get_zopfli_config_normal, Compression, Options, PreCheckResult, ULTRA_ZOPFLI_OPTIONS};
use crate::profile_scope;
use byteorder::{LittleEndian, WriteBytesExt};
use dashmap::DashMap;
use libdeflater::CompressionLvl;
use sha2::{Digest, Sha256};
use std::io::{self, Error, Seek, Write};
use std::sync::{Arc, Mutex};
use crate::options::PreCheckResult::{CompressWithZopfli, Skip};

#[derive(Clone, Debug)]
pub struct CachedFileData {
    pub header_offset: u32,
    pub compression_method: u16,
    pub compressed_size: u32,
}

struct CentralDirectoryEntry {
    filename: String,
    compression_method: u16,
    compressed_size: u32,
    header_offset: u32,
}

struct Inner<W: Write + Seek> {
    writer: W,
    cd_entries: Vec<CentralDirectoryEntry>,
}

pub struct OptimizedZipWriter<W: Write + Seek> {
    content_cache: DashMap<[u8; 32], CachedFileData>,
    inner: Arc<Mutex<Inner<W>>>,
}

impl<W: Write + Seek> OptimizedZipWriter<W> {
    pub fn new(writer: W) -> Self {
        Self {
            content_cache: DashMap::default(),
            inner: Arc::new(Mutex::new(Inner {
                writer,
                cd_entries: Vec::new(),
            })),
        }
    }

    pub fn add_file(
        &self,
        filename: &str,
        data: &[u8],
        options: &Options,
        cache: &Option<Cache>,
    ) -> io::Result<()> {
        profile_scope!("add_file::zip");
        let mut sha256 = Sha256::new();
        sha256.update(data);
        let hash: [u8; 32] = sha256.finalize().into();

        let existing_match = self.content_cache.get(&hash).map(|r| r.clone());
        if let Some(cached_data) = existing_match {
            let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());

            return self.record_entry(&mut inner, filename, cached_data);
        }
        let uncompressed_size = data.len() as u32;

        // Compress the data using DEFLATE
        let compressed_data = Self::compress(data, options, &hash, cache)?;
        let mut compressed_size = compressed_data.len() as u32;
        let using_store = compressed_size == 0; // Skip compression if it's larger when compressed
        if using_store {
            compressed_size = uncompressed_size;
        }
        let compression_method = if using_store {
            0 // Store
        } else {
            8 // Deflate
        };

        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());

        // Check the cache again after we get the lock
        let re_check = self.content_cache.get(&hash).map(|r| r.clone());
        if let Some(cached_data) = re_check {
            return self.record_entry(&mut inner, filename, cached_data);
        }

        // Record the precise start offset for the Local File Header
        let header_offset = inner.writer.stream_position()? as u32;

        // Write the Minimized Local File Header (LFH)
        // We zero out the metadata, filename length, and omit the filename string to save space.
        inner.writer.write_u32::<LittleEndian>(0x04034b50)?; // LFH Signature
        inner.writer.write_u16::<LittleEndian>(0)?; // Version needed to extract (Zeroed)
        inner.writer.write_u16::<LittleEndian>(0)?; // General purpose bit flag
        inner.writer.write_u16::<LittleEndian>(0)?; // Compression method (Zeroed)
        inner.writer.write_u32::<LittleEndian>(0)?; // Last mod file time and date (Zeroed)
        inner.writer.write_u32::<LittleEndian>(0)?; // CRC-32 (Zeroed)
        inner.writer.write_u32::<LittleEndian>(0)?; // Compressed size (Zeroed)
        inner.writer.write_u32::<LittleEndian>(0)?; // Uncompressed size (Zeroed)
        inner.writer.write_u16::<LittleEndian>(0)?; // File name length (Zeroed)
        inner.writer.write_u16::<LittleEndian>(0)?; // Extra field length (Zeroed)

        // Write the actual compressed payload
        if using_store {
            inner.writer.write_all(data)?; // When using Store
        } else {
            inner.writer.write_all(&compressed_data)?; // When using Deflate
        }

        let new_cache = CachedFileData {
            header_offset,
            compression_method,
            compressed_size
        };

        // Insert into the hashmap so future identical files point here
        self.content_cache.insert(hash, new_cache.clone());
        inner.cd_entries.push(CentralDirectoryEntry {
            filename: filename.to_string(),
            compression_method: new_cache.compression_method,
            compressed_size: new_cache.compressed_size,
            header_offset: new_cache.header_offset,
        });

        Ok(())
    }

    fn compress(
        data: &[u8],
        options: &Options,
        hash: &[u8; 32],
        cache: &Option<Cache>,
    ) -> Result<Vec<u8>, Error> {
        if let Some(cache) = cache {
            if let Some(bytes) = cache
                .with_item(hash, ItemType::Generic, |it| {
                    (it.compression as u8 >= options.compression.clone() as u8)
                        .then(|| it.data.clone())
                })
                .flatten()
            {
                return Ok(bytes);
            }
        }
        let input_size = data.len();
        Ok(match options.compression {
            Compression::Fastest => {
                let mut compressor = libdeflater::Compressor::default();
                let mut out = vec![0u8; compressor.deflate_compress_bound(data.len())];
                let size = compressor
                    .deflate_compress(data, &mut out)
                    .map_err(|_| Error::other("Compression failed"))?;
                out.truncate(size);
                let out_size = out.len();
                if out_size > input_size {
                    out = vec![];
                }
                if let Some(cache) = cache {
                    cache.add_item_hash(
                        hash,
                        &*out,
                        crate::cache::Compression::Fastest as u8,
                        ItemType::Generic,
                    )
                }
                out
            }
            Compression::Fast => {
                let mut compressor = libdeflater::Compressor::new(CompressionLvl::best());
                let mut out = vec![0u8; compressor.deflate_compress_bound(data.len())];
                let size = compressor
                    .deflate_compress(data, &mut out)
                    .map_err(|_| Error::other("Compression failed"))?;
                out.truncate(size);
                let out_size = out.len();
                if out_size > input_size {
                    out = vec![];
                }
                if let Some(cache) = cache {
                    cache.add_item_hash(
                        hash,
                        &*out,
                        crate::cache::Compression::Fast as u8,
                        ItemType::Generic,
                    )
                }
                out
            }
            Compression::Normal => {
                let pre_check_result = analyze_and_get_zopfli_config_normal(&data);
                Self::compress_with_pre_check(data, hash, cache, input_size, pre_check_result)?
            }
            Compression::Best => {
                let pre_check_result = analyze_and_get_zopfli_config_best(&data);
                Self::compress_with_pre_check(data, hash, cache, input_size, pre_check_result)?
            }
            Compression::Ultra => {
                let mut encoder = zopfli::DeflateEncoder::new(
                    ULTRA_ZOPFLI_OPTIONS.to_owned(),
                    zopfli::BlockType::Dynamic,
                    Vec::new(),
                );
                encoder.write_all(data)?;
                let mut out = encoder.finish()?;
                let out_size = out.len();
                if out_size > input_size {
                    out = vec![];
                }
                if let Some(cache) = cache {
                    cache.add_item_hash(
                        hash,
                        &*out,
                        crate::cache::Compression::Ultra as u8,
                        ItemType::Generic,
                    )
                }
                out
            }
        })
    }

    fn compress_with_pre_check(data: &[u8], hash: &[u8; 32], cache: &Option<Cache>, input_size: usize, pre_check_result: PreCheckResult) -> Result<Vec<u8>, Error> {
        Ok(match pre_check_result {
            CompressWithZopfli(options) => {
                let mut encoder = zopfli::DeflateEncoder::new(
                    options,
                    zopfli::BlockType::Dynamic,
                    Vec::new(),
                );
                encoder.write_all(data)?;
                let mut out = encoder.finish()?;
                let out_size = out.len();
                if out_size > input_size {
                    out = vec![];
                }
                if let Some(cache) = cache {
                    cache.add_item_hash(
                        hash,
                        &*out,
                        crate::cache::Compression::Best as u8,
                        ItemType::Generic,
                    )
                }
                out
            }
            Skip => {
                let out = vec![];
                if let Some(cache) = cache {
                    cache.add_item_hash(
                        hash,
                        &*out,
                        crate::cache::Compression::Best as u8,
                        ItemType::Generic,
                    )
                }
                out
            }
        })
    }

    fn record_entry(
        &self,
        inner: &mut Inner<W>,
        filename: &str,
        data: CachedFileData,
    ) -> io::Result<()> {
        inner.cd_entries.push(CentralDirectoryEntry {
            filename: filename.to_string(),
            compression_method: data.compression_method,
            compressed_size: data.compressed_size,
            header_offset: data.header_offset,
        });
        Ok(())
    }

    /// Writes the Central Directory and End of Central Directory (EOCD) records.
    /// This finalizes the ZIP file, making it valid for CD-parsing tools.
    pub fn finish(&self) -> io::Result<()> {
        profile_scope!("finish::zip");
        let mut inner_guard = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let Inner {
            ref mut writer,
            ref cd_entries,
        } = *inner_guard;

        let cd_start_offset = writer.stream_position()? as u32;

        // Write all Central Directory entries
        for entry in cd_entries {
            let filename_bytes = entry.filename.as_bytes();

            writer.write_u32::<LittleEndian>(0x02014b50)?; // CD Signature
            writer.write_u16::<LittleEndian>(0)?; // Version made by
            writer.write_u16::<LittleEndian>(0)?; // Version needed to extract
            writer.write_u16::<LittleEndian>(0)?; // General purpose bit flag
            writer.write_u16::<LittleEndian>(entry.compression_method)?; // Compression method
            writer.write_u32::<LittleEndian>(0)?; // Last mod file time and date
            writer.write_u32::<LittleEndian>(0)?; // Actual CRC-32
            writer.write_u32::<LittleEndian>(entry.compressed_size)?; // Actual Compressed size
            writer.write_u32::<LittleEndian>(0)?; // Actual Uncompressed size
            writer.write_u16::<LittleEndian>(filename_bytes.len() as u16)?; // File name length
            writer.write_u16::<LittleEndian>(0)?; // Extra field length
            writer.write_u16::<LittleEndian>(0)?; // File comment length
            writer.write_u16::<LittleEndian>(0)?; // Disk number start
            writer.write_u16::<LittleEndian>(0)?; // Internal file attributes
            writer.write_u32::<LittleEndian>(0)?; // External file attributes
            writer.write_u32::<LittleEndian>(entry.header_offset)?; // Relative offset of LFH

            writer.write_all(filename_bytes)?;
        }

        let cd_end_offset = writer.stream_position()? as u32;
        let cd_size = cd_end_offset - cd_start_offset;
        // let total_entries = cd_entries.len() as u16;

        // Write End of Central Directory (EOCD) Record
        writer.write_u32::<LittleEndian>(0x06054b50)?; // EOCD Signature
        writer.write_u16::<LittleEndian>(u16::MAX)?; // Number of this disk
        writer.write_u16::<LittleEndian>(0)?; // Disk where CD starts
        writer.write_u16::<LittleEndian>(0)?; // Number of CD records on this disk
        writer.write_u16::<LittleEndian>(0)?; // Total number of CD records
        writer.write_u32::<LittleEndian>(cd_size)?; // Size of central directory
        writer.write_u32::<LittleEndian>(cd_start_offset)?; // Offset of start of CD
        writer.write_u16::<LittleEndian>(0)?; // ZIP file comment length

        writer.flush()?;
        Ok(())
    }
}
