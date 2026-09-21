use crate::resource_pack::files::atlas::Atlas;
use crate::resource_pack::files::blockstate::Blockstate;
use crate::resource_pack::files::font::Font;
use crate::resource_pack::files::item::Item;
use crate::resource_pack::files::json::Json;
use crate::resource_pack::files::model::Model;
use crate::resource_pack::files::pack_mcmeta::PackMcmeta;
use crate::resource_pack::files::resource_pack_file::ResourcePackFile;
use crate::resource_pack::files::shader::Shader;
use crate::resource_pack::files::sound::Sound;
use crate::resource_pack::files::sound_definitions::SoundDefinitions;
use crate::resource_pack::files::asset_texture::AssetTexture;
use crate::resource_pack::files::unknowntexture::UnknownTexture;
use dashmap::DashMap;
use std::sync::{Arc, Mutex};
use fxhash::FxBuildHasher;

pub(crate) type FastDashMap<K, V> = DashMap<K, V, FxBuildHasher>;

#[derive(Clone, Debug, Default)]
pub struct ResourcePack {
    pub pack_mcmeta: Arc<Mutex<Option<PackMcmeta>>>,
    pub models: FastDashMap<String, Model>,
    pub json_files: FastDashMap<String, Json>,
    pub textures: FastDashMap<String, AssetTexture>,
    pub unknown_textures: FastDashMap<String, UnknownTexture>,
    pub shaders: FastDashMap<String, Shader>,
    pub unknown_files: FastDashMap<String, ResourcePackFile>,
    pub blockstates: FastDashMap<String, Blockstate>,
    pub fonts: FastDashMap<String, Font>,
    pub items: FastDashMap<String, Item>,
    pub sounds: FastDashMap<String, Sound>,
    pub sound_definitions: FastDashMap<String, SoundDefinitions>,
    pub atlases: FastDashMap<String, Atlas>,
}

impl ResourcePack {
    pub fn pack_mcmeta(&self, pack_mcmeta: PackMcmeta) {
        self.pack_mcmeta.lock().unwrap().replace(pack_mcmeta);
    }

    pub fn model(&self, model: Model) {
        self.models.insert(model.path(), model);
    }

    pub fn json_file(&self, json: Json) {
        self.json_files.insert(json.path.to_string(), json);
    }

    pub fn texture(&self, texture: AssetTexture) {
        self.textures.insert(texture.path(), texture);
    }

    pub fn unknown_texture(&self, unknown_texture: UnknownTexture) {
        self.unknown_textures.insert(unknown_texture.path.clone(), unknown_texture);
    }

    pub fn shader(&self, shader: Shader) {
        self.shaders.insert(shader.path.to_string(), shader);
    }

    pub fn unknown_file(&self, file: ResourcePackFile) {
        self.unknown_files.insert(file.path.to_string(), file);
    }

    pub fn blockstate(&self, file: Blockstate) {
        self.blockstates.insert(file.path(), file);
    }

    pub fn font(&self, file: Font) {
        self.fonts.insert(file.path(), file);
    }

    pub fn item(&self, file: Item) {
        self.items.insert(file.path(), file);
    }

    pub fn sound(&self, sound: Sound) {
        self.sounds.insert(sound.path(), sound);
    }

    pub fn sound_definitions(&self, sound_definitions: SoundDefinitions) {
        self.sound_definitions.insert(sound_definitions.path(), sound_definitions);
    }

    pub fn atlas(&self, atlas: Atlas) {
        self.atlases.insert(atlas.path(), atlas);
    }

    /// Freezes the resource pack, converting DashMaps into contiguous Vecs or FxHashMaps.
    pub fn freeze(self) -> FrozenResourcePack {
        // Option A: Convert to sorted Vecs (Optimal for cache locality & continuous Rayon streams)
        let pack_mcmeta = self.pack_mcmeta.lock().unwrap().take();

        let mut models: Vec<_> = self.models.into_iter().collect();
        models.sort_unstable_by(|a, b| a.0.cmp(&b.0));

        let mut json_files: Vec<_> = self.json_files.into_iter().collect();
        json_files.sort_unstable_by(|a, b| a.0.cmp(&b.0));

        let mut textures: Vec<_> = self.textures.into_iter().collect();
        textures.sort_unstable_by(|a, b| a.0.cmp(&b.0));

        let mut unknown_textures: Vec<_> = self.unknown_textures.into_iter().collect();
        unknown_textures.sort_unstable_by(|a, b| a.0.cmp(&b.0));

        let mut shaders: Vec<_> = self.shaders.into_iter().collect();
        shaders.sort_unstable_by(|a, b| a.0.cmp(&b.0));

        let mut unknown_files: Vec<_> = self.unknown_files.into_iter().collect();
        unknown_files.sort_unstable_by(|a, b| a.0.cmp(&b.0));

        let mut blockstates: Vec<_> = self.blockstates.into_iter().collect();
        blockstates.sort_unstable_by(|a, b| a.0.cmp(&b.0));

        let mut fonts: Vec<_> = self.fonts.into_iter().collect();
        fonts.sort_unstable_by(|a, b| a.0.cmp(&b.0));

        let mut items: Vec<_> = self.items.into_iter().collect();
        items.sort_unstable_by(|a, b| a.0.cmp(&b.0));

        let mut sounds: Vec<_> = self.sounds.into_iter().collect();
        sounds.sort_unstable_by(|a, b| a.0.cmp(&b.0));

        let mut sound_definitions: Vec<_> = self.sound_definitions.into_iter().collect();
        sound_definitions.sort_unstable_by(|a, b| a.0.cmp(&b.0));

        let mut atlases: Vec<_> = self.atlases.into_iter().collect();
        atlases.sort_unstable_by(|a, b| a.0.cmp(&b.0));

        FrozenResourcePack {
            pack_mcmeta,
            models,
            json_files,
            textures,
            unknown_textures,
            shaders,
            unknown_files,
            blockstates,
            fonts,
            items,
            sounds,
            sound_definitions,
            atlases,
        }
    }
}

#[derive(Debug, Default)]
pub struct FrozenResourcePack {
    pub pack_mcmeta: Option<PackMcmeta>,
    pub models: Vec<(String, Model)>,
    pub json_files: Vec<(String, Json)>,
    pub textures: Vec<(String, AssetTexture)>,
    pub unknown_textures: Vec<(String, UnknownTexture)>,
    pub shaders: Vec<(String, Shader)>,
    pub unknown_files: Vec<(String, ResourcePackFile)>,
    pub blockstates: Vec<(String, Blockstate)>,
    pub fonts: Vec<(String, Font)>,
    pub items: Vec<(String, Item)>,
    pub sounds: Vec<(String, Sound)>,
    pub sound_definitions: Vec<(String, SoundDefinitions)>,
    pub atlases: Vec<(String, Atlas)>,
}

impl FrozenResourcePack {
    /// Total count of all contained items across all collections.
    pub const fn len(&self) -> usize {
        self.models.len()
            + self.json_files.len()
            + self.textures.len()
            + self.unknown_textures.len()
            + self.shaders.len()
            + self.unknown_files.len()
            + self.blockstates.len()
            + self.fonts.len()
            + self.items.len()
            + self.sounds.len()
            + self.sound_definitions.len()
            + self.atlases.len()
            + if self.pack_mcmeta.is_some() { 1 } else { 0 }
    }

    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
