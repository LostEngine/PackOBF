use strum_macros::{Display, EnumString};

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

pub fn is_in_models(input: &str) -> bool {
    MODELS.contains(input)
}

pub fn is_in_textures(input: &str) -> bool {
    TEXTURES.contains(input)
}

pub fn is_in_sounds(input: &str) -> bool {
    SOUNDS.contains(input)
}

/// This only works with atlases actually usable in resource packs (currently only blocks and items)
pub fn get_atlas(input: &str) -> Option<AtlasType> {
    if input.starts_with("block/") || input.starts_with("entity/conduit/") {
        Some(AtlasType::Blocks)
    } else if input.starts_with("item/") {
        Some(AtlasType::Items)
    } else {
        None
    }
}

#[derive(Clone, Debug, Default, Display, EnumString, PartialEq)]
#[strum(serialize_all = "snake_case")]
pub enum AtlasType {
    ArmorTrims,
    BannerPatterns,
    Beds,
    Blocks,
    Celestials,
    Chests,
    DecoratedPot,
    GUI,
    #[default]
    Items,
    MapDecorations,
    Paintings,
    Particles,
    ShieldPatterns,
    ShulkerBoxes,
    Signs
}
