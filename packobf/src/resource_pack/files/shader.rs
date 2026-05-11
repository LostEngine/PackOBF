use crate::cache::{Cache, ItemType};
use crate::options::ShaderCompression;
use crate::shader_minifier::minifier::Minifier;
use crate::LogLevel::{Info, Warning};
use crate::LogMessage;
use sha2::{Digest, Sha256};

#[derive(Clone, Debug)]
pub struct Shader {
    pub path: String,
    pub content: String,
}

impl Shader {
    pub fn new(path: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            content: content.into(),
        }
    }

    pub fn optimize(
        &mut self,
        options: &crate::options::Options,
        logger: &tokio::sync::mpsc::UnboundedSender<LogMessage>,
        cache: &Option<Cache>,
    ) {
        if options.shader_compression == ShaderCompression::None {
            return;
        }
        if let Some(cache) = cache {
            let mut sha256 = Sha256::new();
            sha256.update(self.content.as_bytes());
            let hash: [u8; 32] = sha256.finalize().into();

            if let Some(bytes) = cache
                .with_item(&hash, ItemType::Image, |it| {
                    (it.compression as u8 >= options.shader_compression.clone() as u8)
                        .then(|| it.data.clone())
                })
                .flatten()
            {
                let _ = logger.send(LogMessage {
                    level: Info,
                    message: format!("Shader '{}' was loaded from cache.", self.path),
                });
                self.content = String::from_utf8(bytes).unwrap();
                return;
            }
        }
        match Minifier::default().minify(
            &self.content,
            options.shader_compression == ShaderCompression::MinifyAndObfuscate,
        ) {
            Ok(minified_code) => {
                if !minified_code.is_empty() && minified_code != self.content {
                    if let Some(cache) = cache {
                        cache.add_item(
                            self.content.as_bytes(),
                            minified_code.as_bytes(),
                            options.shader_compression.clone() as u8,
                            ItemType::Shader,
                        )
                    }
                    self.content = minified_code;
                }
            }
            Err(e) => {
                let _ = logger.send(LogMessage {
                    level: Warning,
                    message: format!(
                        "Could not minify shader '{}'. Skipping optimization. Error: {}",
                        self.path, e
                    ),
                });
            }
        }
    }
}
