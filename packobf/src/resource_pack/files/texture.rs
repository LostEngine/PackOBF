use crate::cache::Cache;
use crate::options::Options;
use crate::resource_pack::files::unknowntexture::UnknownTexture;
use crate::resource_pack::identifier::Identifier;
use crate::LogMessage;

// textures are referenced in `models`, `items` and `font`
#[derive(Clone, Debug)]
pub struct Texture {
    pub overlay: String,
    pub identifier: Identifier,
    pub unknown_texture: UnknownTexture
}

impl Texture {
    pub fn new(
        overlay: impl Into<String>,
        identifier: impl Into<Identifier>,
        bytes: Vec<u8>,
    ) -> Self {
        let overlay = overlay.into();
        let prefix = if overlay.is_empty() {
            "".to_string()
        } else {
            format!("{}/", overlay)
        };
        let identifier = identifier.into();
        let path = format!(
            "{}assets/{}/textures/{}.png",
            prefix, identifier.namespace, identifier.path
        );
        let unknown_texture = UnknownTexture::new(path, bytes);
        Self {
            overlay: overlay.into(),
            identifier: identifier.into(),
            unknown_texture,
        }
    }

    pub fn optimize(
        &mut self,
        options: &Options,
        logger: &tokio::sync::mpsc::UnboundedSender<LogMessage>,
        cache: &Option<Cache>,
    ) {
        self.unknown_texture.optimize(options, logger, cache);
    }

    pub fn path(&self) -> String {
        self.unknown_texture.path.clone()
    }
}

