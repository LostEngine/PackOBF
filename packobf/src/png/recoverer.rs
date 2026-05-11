use std::io::Cursor;
use zune_png::zune_core::options::DecoderOptions;
use zune_png::PngDecoder;

pub fn recover_png(input: &[u8]) -> Result<Vec<u8>, String> {
    let cursor = Cursor::new(input);

    let options = DecoderOptions::default()
        .png_set_confirm_crc(false)
        .set_strict_mode(false);

    let mut decoder = PngDecoder::new_with_options(cursor, options);

    let pixels = decoder
        .decode()
        .map_err(|e| format!("Zune-png failed to recover pixels: {:?}", e))?;

    let info = decoder.info().ok_or("Could not get image info")?;
    let (width, height) = (info.width as u32, info.height as u32);

    let colorspace = decoder.colorspace().ok_or("Unknown colorspace")?;

    use image::codecs::png::PngEncoder;
    use image::{ColorType, ImageEncoder};

    let mut output = Vec::new();
    let encoder = PngEncoder::new(&mut output);

    let color_type = match colorspace {
        zune_png::zune_core::colorspace::ColorSpace::RGB => ColorType::Rgb8,
        zune_png::zune_core::colorspace::ColorSpace::RGBA => ColorType::Rgba8,
        zune_png::zune_core::colorspace::ColorSpace::Luma => ColorType::L8,
        zune_png::zune_core::colorspace::ColorSpace::LumaA => ColorType::La8,
        _ => return Err(format!("Unsupported colorspace: {:?}", colorspace)),
    };

    encoder
        .write_image(&pixels.u8().ok_or("Could not get pixel bytes")?, width, height, color_type.into())
        .map_err(|e| e.to_string())?;

    Ok(output)
}
