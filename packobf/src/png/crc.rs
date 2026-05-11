/// This function modifies the PNG CRCs to make them invalid and removes the IEND CRC.
/// Doing this will break most PNG file readers, but Minecraft doesn't care about CRC.
pub fn modify_png_crcs(input: &[u8]) -> Result<Vec<u8>, &'static str> {
    const PNG_SIGNATURE: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

    if input.len() < 8 || input[0..8] != PNG_SIGNATURE {
        return Err("Invalid PNG signature");
    }

    let mut output = Vec::with_capacity(input.len() - 4); // 4 bytes of CRC are removed from the IEND chunk, we don't need to allocate them.
    output.extend_from_slice(&PNG_SIGNATURE);

    let mut offset = 8;

    while offset < input.len() {
        if offset + 8 > input.len() {
            output.extend_from_slice(&input[offset..]);
            break;
        }

        let length_bytes: [u8; 4] = input[offset..offset + 4].try_into().unwrap();
        let length = u32::from_be_bytes(length_bytes) as usize;

        let chunk_type = &input[offset + 4..offset + 8];

        let chunk_end = offset + 8 + length;
        let crc_end = chunk_end + 4;

        if crc_end > input.len() {
            output.extend_from_slice(&input[offset..]);
            break;
        }

        output.extend_from_slice(&input[offset..chunk_end]);

        match chunk_type {
            b"IHDR" | b"PLTE" | b"IDAT" => {
                output.extend_from_slice(&[0, 0, 0, 0]);
            }
            b"IEND" => {
                if crc_end < input.len() {
                    output.extend_from_slice(&input[crc_end..]);
                }
                break;
            }
            _ => {
                output.extend_from_slice(&input[chunk_end..crc_end]);
            }
        }

        offset = crc_end;
    }

    Ok(output)
}
