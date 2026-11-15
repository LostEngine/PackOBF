use crate::LogMessage;
use crate::cache::Cache;
use crate::options::Options;
use crate::resource_pack::files::texture::Texture;
use crate::resource_pack::identifier::Identifier;
use crate::resource_pack::mapping::get_mappings;

// textures are referenced in `models`, `items` and `font`
#[derive(Clone, Debug)]
pub struct AssetTexture {
    pub overlay: String,
    pub identifier: Identifier,
    pub texture: Texture,
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
        self,
        options: &Options,
        logger: &tokio::sync::mpsc::UnboundedSender<LogMessage>,
        cache: &Option<Cache>,
    ) -> Vec<u8> {
        let path = self.path();
        self.texture.optimize(options, logger, cache, &path)
    }

    pub fn path(&self) -> String {
        Self::path_for(&self.overlay, &self.identifier)
    }

    pub(crate) fn path_for(overlay: &str, identifier: &Identifier) -> String {
        let mappings = get_mappings();
        let mapped = mappings.texture_mappings.asset_identifier(identifier);
        let prefix = match overlay {
            "" => "".to_string(),
            x => format!("{x}/"),
        };
        format!(
            "{prefix}assets/{}/textures/{}.png",
            mapped.namespace, mapped.path
        )
    }
}
