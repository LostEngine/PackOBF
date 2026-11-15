use crate::resource_pack::files::asset_texture::AssetTexture;
use crate::resource_pack::files::atlas::Atlas;
use crate::resource_pack::files::blockstate::Blockstate;
use crate::resource_pack::files::font::Font;
use crate::resource_pack::files::item::Item;
use crate::resource_pack::files::json::Json;
use crate::resource_pack::files::lang::Lang;
use crate::resource_pack::files::model::Model;
use crate::resource_pack::files::pack_mcmeta::PackMcmeta;
use crate::resource_pack::files::shader::Shader;
use crate::resource_pack::files::sound::Sound;
use crate::resource_pack::files::sounds::SoundDefinitions;
use crate::resource_pack::files::unknown_file::UnknownFile;
use crate::resource_pack::files::unknowntexture::UnknownTexture;

#[derive(Clone, Debug)]
pub struct ResourcePack {
    pub pack_mcmeta: Option<PackMcmeta>,
    pub assets: Vec<Asset>,
}

#[derive(Clone, Debug)]
pub enum Asset {
    Model(Model),
    Json(Json),
    AssetTexture(AssetTexture),
    UnknownTexture(UnknownTexture),
    Shader(Shader),
    UnknownFile(UnknownFile),
    Blockstate(Blockstate),
    Font(Font),
    Item(Item),
    Sound(Sound),
    SoundDefinitions(SoundDefinitions),
    Atlas(Atlas),
    Lang(Lang),
}

impl ResourcePack {
    pub const fn new(pack_mcmeta: Option<PackMcmeta>, assets: Vec<Asset>) -> Self {
        Self {
            pack_mcmeta,
            assets,
        }
    }
}
