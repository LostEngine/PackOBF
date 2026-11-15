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
}
