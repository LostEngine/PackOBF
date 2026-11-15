use crate::LogLevel::Warning;
use crate::cache::{Cache, ItemType};
use crate::options::{Options, SoundQuality};
use crate::resource_pack::identifier::Identifier;
use crate::resource_pack::mapping::get_mappings;
use crate::{LogMessage, profile_scope};
use std::io::{Cursor, Error};
use std::num::NonZeroU8;
use vorbis_rs::{VorbisBitrateManagementStrategy, VorbisDecoder, VorbisEncoderBuilder};

#[derive(Clone, Debug)]
pub struct Sound {
    pub overlay: String,
    pub identifier: Identifier,
    pub bytes: Vec<u8>,
}

impl Sound {
    pub fn new(
        overlay: impl Into<String>,
        identifier: impl Into<Identifier>,
        bytes: Vec<u8>,
    ) -> Self {
        Self {
            overlay: overlay.into(),
            identifier: identifier.into(),
            bytes,
        }
    }

    pub fn optimize(
        &self,
        option: &Options,
        logger: &tokio::sync::mpsc::UnboundedSender<LogMessage>,
        cache: &Option<Cache>,
    ) -> Vec<u8> {
        profile_scope!(std::any::type_name_of_val(&Sound::optimize));
        match convert_ogg(&self.bytes, 1, &option.sound_quality) {
            Ok(bytes) => {
                if let Some(cache) = cache {
                    cache.add_item(&self.bytes, &*bytes, 0, ItemType::Sound);
                }
                bytes
            }
            Err(e) => {
                let _ = logger.send(LogMessage {
                    level: Warning,
                    message: format!(
                        "Could not optimize sound '{}'. Skipping optimization. Error: {}",
                        self.path(),
                        e
                    ),
                });
                self.bytes.clone()
            }
        }
    }

    pub fn path(&self) -> String {
        let mapped = get_mappings().sound_mappings
            .get(&self.identifier.to_string())
            .map_or_else(|| self.identifier.clone(), |id| Identifier::parse(id));
        let prefix = match self.overlay.as_str() {
            "" => "".to_string(),
            x => format!("{x}/"),
        };
        format!(
            "{}assets/{}/sounds/{}.ogg",
            prefix, mapped.namespace, mapped.path
        )
    }
}

pub fn convert_ogg(
    input: &[u8],
    channels: u8,
    quality: &SoundQuality,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    assert!((1..=2).contains(&channels));

    let mut decoder = VorbisDecoder::new(Cursor::new(input))?;
    let sample_rate = decoder.sampling_frequency();

    let quality = match quality.value() {
        Some(quality) => quality,
        None => {
            let mut total_samples = 0_u64;

            while let Some(block) = decoder.decode_audio_block()? {
                total_samples += block.samples()[0].len() as u64;
            }

            let duration = total_samples as f64 / sample_rate.get() as f64;

            decoder = VorbisDecoder::new(Cursor::new(input))?;

            SoundQuality::dynamic_quality(duration)
        }
    };

    let mut output = Vec::new();

    let mut builder = VorbisEncoderBuilder::new_with_serial(
        sample_rate,
        NonZeroU8::new(channels).unwrap(),
        &mut output,
        0,
    );

    builder.bitrate_management_strategy(
        VorbisBitrateManagementStrategy::QualityVbr {
            target_quality: quality,
        },
    );

    let mut encoder = builder.build()?;

    while let Some(block) = decoder.decode_audio_block()? {
        let samples = block.samples();

        let converted: Vec<Vec<f32>> = match (samples.len(), channels) {
            // Mono to Mono or Stereo to Stereo (copies the data)
            (1, 1) | (2, 2) => {
                samples.iter().map(|s| s.to_vec()).collect()
            }

            // Stereo to Mono by averaging their samples
            (2, 1) => {
                let mono = samples[0]
                    .iter()
                    .zip(samples[1].iter())
                    .map(|(l, r)| (l + r) * 0.5)
                    .collect();

                vec![mono]
            }

            // Mono to Stereo (duplicates the samples)
            (1, 2) => {
                vec![samples[0].to_vec(), samples[0].to_vec()]
            }

            _ => Err(Error::other(format!("Unsupported number of channels: {}", samples.len())))?,
        };

        encoder.encode_audio_block(&converted)?;
    }

    encoder.finish()?;
    Ok(output)
}
