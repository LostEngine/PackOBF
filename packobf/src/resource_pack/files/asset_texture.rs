use crate::cache::Cache;
use crate::options::Options;
use crate::resource_pack::files::texture::Texture;
use crate::resource_pack::identifier::Identifier;
use crate::LogMessage;

// textures are referenced in `models`, `items` and `font`
#[derive(Clone, Debug)]
pub struct AssetTexture {
    pub overlay: String,
    pub identifier: Identifier,
    pub texture: Texture
}

impl AssetTexture {
    pub fn new(
        overlay: impl Into<String>,
        identifier: impl Into<Identifier>,
        bytes: Vec<u8>,
    ) -> Self {
        let overlay = overlay.into();
        let identifier = identifier.into();
        let texture = Texture::new(bytes);
        Self {
            overlay,
            identifier,
            texture,
        }
    }

    pub fn optimize(
        &mut self,
        options: &Options,
        logger: &tokio::sync::mpsc::UnboundedSender<LogMessage>,
        cache: &Option<Cache>,
    ) {
        self.texture.optimize(options, logger, cache, self.path().as_str());
    }

    pub fn path(&self) -> String {
        let prefix = match self.overlay.as_str() {
            "" => "".to_string(),
            x => format!("{}/", x),
        };
        format!(
            "{}assets/{}/textures/{}.png",
            prefix, self.identifier.namespace, self.identifier.path
        )
    }
}
