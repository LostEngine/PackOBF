use crate::cache::{Cache, ItemType};
pub(crate) use crate::options::ZOPFLI_OPTIONS;
use crate::options::{Compression, Options};
use byteorder::{LittleEndian, WriteBytesExt};
use crc32fast::Hasher as Crc32Hasher;
use dashmap::DashMap;
use libdeflater::CompressionLvl;
use sha2::{Digest, Sha256};
use std::io::{self, Error, ErrorKind, Seek, Write};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct CachedFileData {
    pub header_offset: u32,
    pub crc32: u32,
    pub compressed_size: u32,
    pub uncompressed_size: u32,
}

struct CentralDirectoryEntry {
    filename: String,
    crc32: u32,
    compressed_size: u32,
    uncompressed_size: u32,
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
            content_cache: DashMap::default().into(),
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
        let mut sha256 = Sha256::new();
        sha256.update(data);
        let hash: [u8; 32] = sha256.finalize().into();

        let existing_match = self.content_cache.get(&hash).map(|r| r.clone());
        if let Some(cached_data) = existing_match {
            return self.record_entry(&mut self.inner.lock().unwrap(), filename, cached_data);
        }

        // Calculate CRC32 (Required for Central Directory)
        let mut crc32_hasher = Crc32Hasher::new();
        crc32_hasher.update(data);
        let crc32 = crc32_hasher.finalize();
        let uncompressed_size = data.len() as u32;

        // Compress the data using DEFLATE
        let compressed_data: Vec<u8> = Self::compress(data, options, &hash, cache)?;
        let compressed_size = compressed_data.len() as u32;

        let mut inner = self.inner.lock().unwrap();

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
        inner.writer.write_u16::<LittleEndian>(10)?; // Version needed to extract (1.0)
        inner.writer.write_u16::<LittleEndian>(0)?; // General purpose bit flag
        inner.writer.write_u16::<LittleEndian>(8)?; // Compression method (8 = Deflate)
        inner.writer.write_u32::<LittleEndian>(0)?; // Last mod file time and date (Zeroed)
        inner.writer.write_u32::<LittleEndian>(0)?; // CRC-32 (Zeroed)
        inner.writer.write_u32::<LittleEndian>(0)?; // Compressed size (Zeroed)
        inner.writer.write_u32::<LittleEndian>(0)?; // Uncompressed size (Zeroed)
        inner.writer.write_u16::<LittleEndian>(0)?; // File name length (Zeroed)
        inner.writer.write_u16::<LittleEndian>(0)?; // Extra field length (Zeroed)

        // Write the actual compressed payload
        inner.writer.write_all(&compressed_data)?;

        let new_cache = CachedFileData {
            header_offset,
            crc32,
            compressed_size,
            uncompressed_size,
        };

        // Insert into the hashmap so future identical files point here
        self.content_cache.insert(hash, new_cache.clone());
        inner.cd_entries.push(CentralDirectoryEntry {
            filename: filename.to_string(),
            crc32: new_cache.crc32,
            compressed_size: new_cache.compressed_size,
            uncompressed_size: new_cache.uncompressed_size,
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
                .with_item(&hash, ItemType::Generic, |it| {
                    (it.compression as u8 >= options.compression.clone() as u8)
                        .then(|| it.data.clone())
                })
                .flatten()
            {
                return Ok(bytes)
            }
        }
        Ok(match options.compression {
            Compression::Simplest => {
                if cache.is_some() {}
                let mut compressor = libdeflater::Compressor::default();
                let mut out = vec![0u8; compressor.deflate_compress_bound(data.len())];
                let size = compressor
                    .deflate_compress(data, &mut out)
                    .map_err(|_| Error::new(ErrorKind::Other, "Compression failed"))?;
                out.truncate(size);
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
            Compression::Normal => {
                let mut compressor = libdeflater::Compressor::new(CompressionLvl::best());
                let mut out = vec![0u8; compressor.deflate_compress_bound(data.len())];
                let size = compressor
                    .deflate_compress(data, &mut out)
                    .map_err(|_| Error::new(ErrorKind::Other, "Compression failed"))?;
                out.truncate(size);
                if let Some(cache) = cache {
                    cache.add_item_hash(
                        hash,
                        &*out,
                        crate::cache::Compression::Normal as u8,
                        ItemType::Generic,
                    )
                }
                out
            }
            Compression::Max => {
                let mut encoder = zopfli::DeflateEncoder::new(
                    ZOPFLI_OPTIONS.to_owned(),
                    zopfli::BlockType::Dynamic,
                    Vec::new(),
                );
                encoder.write_all(data)?;
                let out = encoder.finish()?;
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
            crc32: data.crc32,
            compressed_size: data.compressed_size,
            uncompressed_size: data.uncompressed_size,
            header_offset: data.header_offset,
        });
        Ok(())
    }

    /// Writes the Central Directory and End of Central Directory (EOCD) records.
    /// This finalizes the ZIP file, making it valid for CD-parsing tools.
    pub fn finish(&self) -> io::Result<()> {
        let mut inner_guard = self.inner.lock().unwrap();
        let Inner {
            ref mut writer,
            ref cd_entries,
        } = *inner_guard;

        let cd_start_offset = writer.stream_position()? as u32;

        // Write all Central Directory entries
        for entry in cd_entries {
            let filename_bytes = entry.filename.as_bytes();

            writer.write_u32::<LittleEndian>(0x02014b50)?; // CD Signature
            writer.write_u16::<LittleEndian>(10)?; // Version made by
            writer.write_u16::<LittleEndian>(10)?; // Version needed to extract
            writer.write_u16::<LittleEndian>(0)?; // General purpose bit flag
            writer.write_u16::<LittleEndian>(8)?; // Compression method (8 = Deflate)
            writer.write_u32::<LittleEndian>(0)?; // Last mod file time and date
            writer.write_u32::<LittleEndian>(entry.crc32)?; // Actual CRC-32
            writer.write_u32::<LittleEndian>(entry.compressed_size)?; // Actual Compressed size
            writer.write_u32::<LittleEndian>(entry.uncompressed_size)?; // Actual Uncompressed size
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
