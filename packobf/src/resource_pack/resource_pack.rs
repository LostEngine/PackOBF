use crate::resource_pack::files::model::Model;
use crate::resource_pack::files::resource_pack_file::ResourcePackFile;
use crate::resource_pack::files::texture::Texture;
use dashmap::DashMap;
use crate::resource_pack::files::atlas::Atlas;
use crate::resource_pack::files::blockstate::Blockstate;
use crate::resource_pack::files::font::Font;
use crate::resource_pack::files::item::Item;
use crate::resource_pack::files::json::Json;
use crate::resource_pack::files::shader::Shader;
use crate::resource_pack::files::sound::Sound;
use crate::resource_pack::files::sound_definitions::SoundDefinitions;

#[derive(Clone, Debug, Default)]
pub struct ResourcePack {
    pub models: DashMap<String, Model>,
    pub json_files: DashMap<String, Json>,
    pub textures: DashMap<String, Texture>,
    pub shaders: DashMap<String, Shader>,
    pub unknown_files: DashMap<String, ResourcePackFile>,
    pub blockstates: DashMap<String, Blockstate>,
    pub fonts: DashMap<String, Font>,
    pub items: DashMap<String, Item>,
    pub sounds: DashMap<String, Sound>,
    pub sound_definitions: DashMap<String, SoundDefinitions>,
    pub atlases: DashMap<String, Atlas>,
}

impl ResourcePack {
    pub fn model(&self, model: Model) {
        self.models.insert(model.path(), model);
    }

    pub fn json_file(&self, json: Json) {
        self.json_files.insert(json.path.to_string(), json);
    }

    pub fn texture(&self, texture: Texture) {
        self.textures.insert(texture.path(), texture);
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
}