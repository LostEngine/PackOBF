use crate::deflate::libdeflater::libdeflater_deflate;
use crate::options::{
    FASTEST_ZOPFLI_OPTIONS, FAST_ZOPFLI_OPTIONS, NORMAL_ZOPFLI_OPTIONS, SLOWEST_ZOPFLI_OPTIONS,
    SLOW_ZOPFLI_OPTIONS,
};
use clap::ValueEnum;
use libdeflater::adler32;
use std::io::Error;
use crate::deflate::zopfli::zopfli_deflate;
use crate::profile_scope;

pub fn dynamic_zlib(input: &[u8], level: Level, checksum: bool) -> Result<Vec<u8>, Error> {
    profile_scope!(std::any::type_name_of_val(&dynamic_zlib));
    let mut compressed_data = dynamic_deflate(input, level, true)?;

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

pub fn dynamic_deflate(input: &[u8], level: Level, compress_anyway: bool) -> Result<Vec<u8>, Error> {
    profile_scope!(std::any::type_name_of_val(&dynamic_deflate));
    let (fast_compressed_data, savings_ratio) = analyze(input, level as u8)?;
    let result = level.get_precheck_result(savings_ratio, input.len());
    match result {
        PreCheckResult::Skip => if compress_anyway {
            Ok(fast_compressed_data)
        } else {
            Ok(vec![])
        },
        PreCheckResult::CompressWithZopfli(options) => {
            Ok(zopfli_deflate(input, options)?)
        }
        PreCheckResult::LibDeflater => {
            Ok(libdeflater_deflate(input, 12)?)
        }
    }
}

fn analyze(input: &[u8], level: u8) -> Result<(Vec<u8>, f64), Error> {
    profile_scope!(std::any::type_name_of_val(&analyze));
    let original_size = input.len();
    if original_size == 0 {
        return Err(Error::other("Compression failed"));
    }

    let fast_compressed_data = libdeflater_deflate(&input, level)?;
    let fast_compressed_size = fast_compressed_data.len();

    let bytes_saved = original_size.saturating_sub(fast_compressed_size);
    let savings_ratio = bytes_saved as f64 / original_size as f64;
    Ok((fast_compressed_data, savings_ratio))
}

pub enum PreCheckResult {
    /// Skip Compression entirely
    Skip,
    /// Use Zopfli with dynamically assigned options
    CompressWithZopfli(zopfli::Options),
    /// Use Libdeflater Level 12
    LibDeflater,
}

#[repr(u8)] // libdeflate level for analysis
#[derive(ValueEnum, Clone, Debug, Copy)]
pub enum Level {
    Normal = 6,
    Best = 9,
}

impl Level {
    fn get_precheck_result(&self, savings_ratio: f64, original_size: usize) -> PreCheckResult {
        // Less than 1% savings
        if savings_ratio < 0.01 {
            return PreCheckResult::Skip; // Don't waste CPU time on Zopfli
        }

        // 1% to 8% savings
        if savings_ratio < 0.08 {
            return match self {
                Level::Normal => PreCheckResult::LibDeflater,
                Level::Best => PreCheckResult::CompressWithZopfli(*FAST_ZOPFLI_OPTIONS),
            };
        }

        // > 8% savings
        PreCheckResult::CompressWithZopfli(match original_size {
            0..=51_200 => match self {
                Level::Normal => *NORMAL_ZOPFLI_OPTIONS,
                Level::Best => *SLOWEST_ZOPFLI_OPTIONS,
            },

            51_201..=512_000 => match self {
                Level::Normal => *FAST_ZOPFLI_OPTIONS,
                Level::Best => *SLOW_ZOPFLI_OPTIONS,
            },

            _ => match self {
                Level::Normal => *FASTEST_ZOPFLI_OPTIONS,
                Level::Best => *NORMAL_ZOPFLI_OPTIONS,
            },
        })
    }
}
