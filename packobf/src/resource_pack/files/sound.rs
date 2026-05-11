use crate::cache::{Cache, ItemType};
use crate::resource_pack::identifier::Identifier;
use crate::LogLevel::{Info, Warning};
use crate::LogMessage;
use optivorbis::remuxer::ogg_to_ogg::{OggVorbisStreamPassthroughMangler, Settings};
use optivorbis::VorbisCommentFieldsAction::Delete;
use optivorbis::VorbisVendorStringAction::Empty;
use optivorbis::{OggToOgg, Remuxer, VorbisOptimizerSettings};
use sha2::{Digest, Sha256};
use std::io::Cursor;

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

    pub fn optimize<L>(&mut self, logger: &L, cache: &Option<Cache>)
    where
        L: Fn(LogMessage),
    {
        if let Some(cache) = cache {
            let mut sha256 = Sha256::new();
            sha256.update(self.bytes.as_slice());
            let hash: [u8; 32] = sha256.finalize().into();

            if let Some(bytes) = cache.with_item(&hash, ItemType::Sound, |it| it.data.clone()) {
                logger(LogMessage {
                    level: Info,
                    message: format!("Sound '{}' was loaded from cache.", self.path()),
                });
                self.bytes = bytes;
                return;
            }
        }
        let mut source = Cursor::new(self.bytes.clone());
        let sink = Vec::new();

        let remuxer_settings = Settings {
            randomize_stream_serials: false,
            first_stream_serial_offset: 0,
            ignore_start_sample_offset: true,
            error_on_no_vorbis_streams: true,
            verify_ogg_page_checksums: true,
            vorbis_stream_mangler: OggVorbisStreamPassthroughMangler,
        };
        let mut optimizer_settings = VorbisOptimizerSettings::default();
        optimizer_settings.vendor_string_action = Empty;
        optimizer_settings.comment_fields_action = Delete;

        let remuxer = OggToOgg::<OggVorbisStreamPassthroughMangler>::new(
            remuxer_settings,
            optimizer_settings,
        );

        match remuxer.remux(&mut source, sink) {
            Ok(bytes) => {
                if let Some(cache) = cache {
                    cache.add_item(&self.bytes, &*bytes, 0, ItemType::Sound)
                }
                self.bytes = bytes;
            }
            Err(e) => {
                logger(LogMessage {
                    level: Warning,
                    message: format!(
                        "Could not optimize sound '{}'. Skipping optimization. Error: {}",
                        self.path(),
                        e
                    ),
                });
            }
        }
    }

    pub fn path(&self) -> String {
        let prefix = if self.overlay.is_empty() {
            "".to_string()
        } else {
            format!("{}/", self.overlay)
        };
        format!(
            "{}assets/{}/sounds/{}.ogg",
            prefix, self.identifier.namespace, self.identifier.path
        )
    }
}
