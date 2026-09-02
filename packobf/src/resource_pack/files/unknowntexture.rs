use crate::cache::Cache;
use crate::LogMessage;
use crate::options::Options;
use crate::resource_pack::files::texture::Texture;

#[derive(Clone, Debug)]
pub struct UnknownTexture {
    pub path: String,
    pub texture: Texture,
}

impl UnknownTexture {
    pub fn new(path: impl Into<String>, bytes: Vec<u8>) -> Self {
        let texture = Texture::new(bytes);
        Self {
            path: path.into(),
            texture,
        }
    }

    pub fn optimize(
        &mut self,
        options: &Options,
        logger: &tokio::sync::mpsc::UnboundedSender<LogMessage>,
        cache: &Option<Cache>,
    ) {
        self.texture.optimize(options, logger, cache, self.path.as_str());
    }
}
