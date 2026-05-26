use dashmap::DashMap;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io;
use std::io::{BufReader, BufWriter, Read, Write};

const MAGIC_NUMBER: [u8; 8] = *b"PACKOBF1";
pub const VERSION: u16 = 1;

pub struct CachedItem {
    pub compression: Compression,
    pub version: u16,
    pub data: Vec<u8>,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Compression {
    Fastest = 0,
    Normal = 1,
    Best = 2,
}

impl Compression {
    fn from_u8(value: u8) -> Self {
        match value {
            0 => Compression::Fastest,
            2 => Compression::Best,
            _ => Compression::Normal,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CachedItemKey {
    hash: [u8; 32],
    item_type: ItemType,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemType {
    Generic = 0,
    Image = 1,
    Sound = 2,
}

impl ItemType {
    fn from_u8(value: u8) -> Self {
        match value {
            1 => ItemType::Image,
            2 => ItemType::Sound,
            _ => ItemType::Generic,
        }
    }
}

pub struct Cache {
    pub items: DashMap<CachedItemKey, CachedItem>,
}

impl Cache {
    pub fn save_to_file(&self, path: &str) -> io::Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        writer.write_all(&MAGIC_NUMBER)?;

        // Write the number of entries (u64)
        writer.write_all(&(self.items.len() as u64).to_le_bytes())?;

        for x in &self.items {
            let cached_item_key = x.key();
            let item = x.value();

            // Write the Key (32 bytes)
            writer.write_all(cached_item_key.hash.as_slice())?;

            // Write the Type (1 byte)
            writer.write_all(&[cached_item_key.item_type as u8])?;

            // Write Compression (1 byte)
            writer.write_all(&[item.compression as u8])?;

            // Write Version (2 bytes)
            writer.write_all(&item.version.to_le_bytes())?;

            // Write Data Length (u64) and Data
            writer.write_all(&(item.data.len() as u64).to_le_bytes())?;
            writer.write_all(&item.data)?;
        }

        writer.flush()?;
        Ok(())
    }

    pub fn load_from_file(path: &str) -> io::Result<Self> {
        let file = File::open(path);
        if let Err(e) = file {
            return if e.kind() == io::ErrorKind::NotFound {
                Ok(Cache {
                    items: DashMap::new(),
                })
            } else {
                Err(e)
            };
        }
        let file = file?;
        let mut reader = BufReader::new(file);
        let items = DashMap::new();

        let mut magic = [0u8; 8];
        reader.read_exact(&mut magic)?;

        if magic != MAGIC_NUMBER {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Not a valid cache file: Magic number mismatch",
            ));
        }

        // Read the number of entries
        let mut len_bytes = [0u8; 8];
        reader.read_exact(&mut len_bytes)?;
        let count = u64::from_le_bytes(len_bytes);

        for _ in 0..count {
            // Read Key
            let mut hash = [0u8; 32];
            reader.read_exact(&mut hash)?;

            // Read Type
            let mut type_bytes = [0u8; 1];
            reader.read_exact(&mut type_bytes)?;
            let item_type = ItemType::from_u8(type_bytes[0]);

            // Read Compression
            let mut comp_byte = [0u8; 1];
            reader.read_exact(&mut comp_byte)?;
            let compression = Compression::from_u8(comp_byte[0]);

            // Read Version
            let mut ver_bytes = [0u8; 2];
            reader.read_exact(&mut ver_bytes)?;
            let version = u16::from_le_bytes(ver_bytes);

            // Read Data Length and then the Data
            let mut data_len_bytes = [0u8; 8];
            reader.read_exact(&mut data_len_bytes)?;
            let data_len = u64::from_le_bytes(data_len_bytes) as usize;

            let mut data = vec![0u8; data_len];
            reader.read_exact(&mut data)?;

            if version == VERSION {
                items.insert(
                    CachedItemKey { hash, item_type },
                    CachedItem {
                        compression,
                        version,
                        data,
                    },
                );
            }
        }

        Ok(Cache { items })
    }

    pub fn with_item<F, R>(&self, hash: &[u8; 32], item_type: ItemType, f: F) -> Option<R>
    where
        F: FnOnce(&CachedItem) -> R,
    {
        self.items
            .get(&CachedItemKey {
                hash: hash.to_owned(),
                item_type,
            })
            .map(|item| f(item.value()))
    }

    pub fn add_item(
        &self,
        old_bytes: &[u8],
        data: impl Into<Vec<u8>>,
        compression: u8,
        item_type: ItemType,
    ) {
        let mut sha256 = Sha256::new();
        sha256.update(old_bytes);
        let hash: [u8; 32] = sha256.finalize().into();

        self.items.insert(
            CachedItemKey { hash, item_type },
            CachedItem {
                compression: Compression::from_u8(compression),
                version: VERSION,
                data: data.into(),
            },
        );
    }

    pub fn add_item_hash(
        &self,
        hash: &[u8; 32],
        data: impl Into<Vec<u8>>,
        compression: u8,
        item_type: ItemType,
    ) {
        self.items.insert(
            CachedItemKey {
                hash: *hash,
                item_type,
            },
            CachedItem {
                compression: Compression::from_u8(compression),
                version: VERSION,
                data: data.into(),
            },
        );
    }
}
