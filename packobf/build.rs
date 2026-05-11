use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

fn main() {
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen.rs");
    let mut file = BufWriter::new(File::create(&path).unwrap());

    let models_file = File::open("src/minecraft/models.txt").unwrap();
    let models_reader = BufReader::new(models_file);

    writeln!(
        &mut file,
        "static MODELS: phf::Set<&'static str> = "
    ).unwrap();

    let mut builder = phf_codegen::Set::new();

    for line in models_reader.lines() {
        builder.entry(line.unwrap().trim().to_string());
    }

    writeln!(
        &mut file,
        "{};",
        builder.build()
    ).unwrap();

    let textures_file = File::open("src/minecraft/textures.txt").unwrap();
    let textures_reader = BufReader::new(textures_file);

    writeln!(
        &mut file,
        "static TEXTURES: phf::Set<&'static str> = "
    ).unwrap();

    let mut builder = phf_codegen::Set::new();

    for line in textures_reader.lines() {
        builder.entry(line.unwrap().trim().to_string());
    }

    writeln!(
        &mut file,
        "{};",
        builder.build()
    ).unwrap();

    let sounds_file = File::open("src/minecraft/sounds.txt").unwrap();
    let sounds_reader = BufReader::new(sounds_file);

    writeln!(
        &mut file,
        "static SOUNDS: phf::Set<&'static str> = "
    ).unwrap();

    let mut builder = phf_codegen::Set::new();

    for line in sounds_reader.lines() {
        builder.entry(line.unwrap().trim().to_string());
    }

    writeln!(
        &mut file,
        "{};",
        builder.build()
    ).unwrap();
}