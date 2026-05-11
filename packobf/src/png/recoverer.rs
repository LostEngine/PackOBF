use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::{ExtendedColorType, ImageEncoder};

pub fn recover_png(input: &[u8]) -> Result<Vec<u8>, String> {
    unsafe {
        let mut width: i32 = 0;
        let mut height: i32 = 0;
        let mut channels: i32 = 0;

        let pixel_ptr = stb_image::stb_image::stbi_load_from_memory(
            input.as_ptr(),
            input.len() as i32,
            &mut width,
            &mut height,
            &mut channels,
            0,
        );

        if pixel_ptr.is_null() {
            return Err("stb_image could not recover any pixels from this file.".to_string());
        }

        let byte_count = (width * height * channels) as usize;
        let raw_pixels = std::slice::from_raw_parts(pixel_ptr, byte_count).to_vec();

        stb_image::stb_image::stbi_image_free(pixel_ptr as *mut _);

        rebuild_png(raw_pixels, width as u32, height as u32, channels)
    }
}

fn rebuild_png(pixels: Vec<u8>, w: u32, h: u32, channels: i32) -> Result<Vec<u8>, String> {
    let mut output = Vec::new();
    let color_type = match channels {
        1 => ExtendedColorType::L8,
        2 => ExtendedColorType::La8,
        3 => ExtendedColorType::Rgb8,
        4 => ExtendedColorType::Rgba8,
        _ => return Err("Unsupported channel count".to_string()),
    };

    let encoder = PngEncoder::new_with_quality(&mut output, CompressionType::Uncompressed, FilterType::Adaptive);
    encoder
        .write_image(&pixels, w, h, color_type)
        .map_err(|e| e.to_string())?;

    Ok(output)
}
