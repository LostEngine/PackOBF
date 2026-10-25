/* AI Slop Notice
Code in this module (mcdoc2rust), which is not part of the PackOBF cargo workspace, was highly generated using LLMs.
All the code hasn't been reviewed by a real developer, and it may be terrible. However, the code that it generates is
manually verified and tested and is included manually in the PackOBF codebase. This code is provided for transparency,
but it does not reflect the rest of the codebase.
See more in mcdoc2rust/README.md
*/

mod export;
#[cfg(debug_assertions)]
mod generated;
mod generator;
#[cfg(debug_assertions)]
mod resource_pack;
mod version;

use crate::export::McdocFile;
use crate::generator::Generator;

const LARGE_TYPE_CLOSURE_WARNING_THRESHOLD: usize = 100;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let schema_bytes = match std::fs::read("symbols.json") {
        Ok(bytes) => bytes,
        Err(_) => reqwest::blocking::get("https://raw.githubusercontent.com/SpyglassMC/vanilla-mcdoc/refs/heads/generated/symbols.json")?.bytes()?.to_vec(),
    };
    let schema: McdocFile = serde_json::from_slice(&schema_bytes)?;

    //let rust_code = Generator::new(&schema).generate_all();
    //std::fs::write("src/generated.rs", rust_code)?;

    macro_rules! generate_code {
        ($path:expr, $name:expr, $value_overrides:expr, $specialized_identifier_types:expr) => {
            let (code, type_closures) = Generator::new(&schema)
                .with_value_overrides($value_overrides)
                .with_specialized_identifier_types($specialized_identifier_types)
                .generate_for_path_with_type_closures($path);
            let mut type_closures = type_closures;
            type_closures.sort_by(|(left_name, left_count), (right_name, right_count)| {
                right_count
                    .cmp(left_count)
                    .then_with(|| left_name.cmp(right_name))
            });
            for (type_name, dependency_count) in type_closures {
                if dependency_count > LARGE_TYPE_CLOSURE_WARNING_THRESHOLD {
                    eprintln!(
                        "warning: generated class {} requires {} generated types, including itself (threshold: {})",
                        type_name, dependency_count, LARGE_TYPE_CLOSURE_WARNING_THRESHOLD
                    );
                }
            }
            std::fs::create_dir_all("src/generated")?;
            std::fs::write(format!("src/generated/{}.rs", $name), code)?;
        };
    }
    generate_code!("::java::assets::atlas::Atlas", "atlas", &[], false);
    generate_code!("::java::assets::font::Font", "font", &[], true);
    generate_code!(
        "::java::assets::item_definition::ItemDefinition",
        "item_definition",
        // Blacklisting NBT/Component types that would take thousands of more lines
        &[
            "JavaDataAdvancementPredicateBlockPredicateNbt",
            "JavaAssetsItemDefinitionSelectPropertyTypeComponentCases",
            "JavaAssetsItemDefinitionConditionalPropertyTypeComponentValue"
        ],
        true
    );
    generate_code!(
        "::java::assets::block_state_definition::BlockStateDefinition",
        "block_state_definition",
        &[],
        true
    );
    generate_code!("::java::assets::model::Model", "model", &[], true);
    generate_code!("::java::assets::sounds::Sounds", "sounds", &[], true);
    generate_code!(
        "::java::assets::equipment::Equipment",
        "equipment",
        &[],
        true
    );
    generate_code!("::java::assets::lang::Lang", "lang", &[], true);

    //println!("{}", serde_json::to_string(&schema)?);
    Ok(())
}
