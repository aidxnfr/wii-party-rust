// port of engine/src/platform/png.cpp

// port of engine/src/platform/png.cpp crc32
fn crc32(data: &[u8], mut crc: u32) -> u32 {
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xEDB88320u32 & 0u32.wrapping_sub(crc & 1));
        }
    }
    crc
}

// port of engine/src/platform/png.cpp put32
fn put32(out: &mut Vec<u8>, value: u32) {
    for shift in (0..=24).rev().step_by(8) {
        out.push((value >> shift) as u8);
    }
}

// port of engine/src/platform/png.cpp put_chunk
fn put_chunk(out: &mut Vec<u8>, chunk_type: &[u8; 4], body: &[u8]) {
    put32(out, body.len() as u32);
    let mut tagged = Vec::with_capacity(4 + body.len());
    tagged.extend_from_slice(chunk_type);
    tagged.extend_from_slice(body);
    out.extend_from_slice(&tagged);
    put32(out, !crc32(&tagged, 0xFFFFFFFFu32));
}

// port of engine/src/platform/png.cpp encode_raw_png
fn encode_raw_png(raw: &[u8], width: u32, height: u32, color_type: u8) -> Vec<u8> {
    let mut deflated = vec![0x78, 0x01];
    let mut position = 0usize;
    while position < raw.len() {
        let length = std::cmp::min(65535usize, raw.len() - position);
        deflated.push(if position + length >= raw.len() { 1 } else { 0 });
        deflated.push(length as u8);
        deflated.push((length >> 8) as u8);
        let inverted = !(length as u16);
        deflated.push(inverted as u8);
        deflated.push((inverted >> 8) as u8);
        deflated.extend_from_slice(&raw[position..position + length]);
        position += length;
    }
    let mut a = 1u32;
    let mut b = 0u32;
    for &byte in raw {
        a = (a + byte as u32) % 65521;
        b = (b + a) % 65521;
    }
    put32(&mut deflated, (b << 16) | a);
    let mut file = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    let mut header = Vec::new();
    put32(&mut header, width);
    put32(&mut header, height);
    header.extend_from_slice(&[8, color_type, 0, 0, 0]);
    put_chunk(&mut file, b"IHDR", &header);
    put_chunk(&mut file, b"IDAT", &deflated);
    put_chunk(&mut file, b"IEND", &[]);
    file
}

// port of engine/src/platform/png.cpp write_file
fn write_file(path: &str, file: &[u8]) {
    let _ = std::fs::write(path, file);
}

// port of engine/src/platform/png.cpp encode_png
pub fn encode_png(pixels: &[u32], width: u32, height: u32) -> Vec<u8> {
    let mut raw = Vec::with_capacity((width as usize * 3 + 1) * height as usize);
    for y in 0..height {
        raw.push(0);
        for x in 0..width {
            let p = pixels[y as usize * width as usize + x as usize];
            raw.push((p >> 16) as u8);
            raw.push((p >> 8) as u8);
            raw.push(p as u8);
        }
    }
    encode_raw_png(&raw, width, height, 2)
}

// port of engine/src/platform/png.cpp save_png
pub fn save_png(path: &str, pixels: &[u32], width: u32, height: u32) {
    write_file(path, &encode_png(pixels, width, height));
}

// port of engine/src/platform/png.cpp save_png_rgba
pub fn save_png_rgba(path: &str, pixels: &[u32], width: u32, height: u32) {
    let mut raw = Vec::with_capacity((width as usize * 4 + 1) * height as usize);
    for y in 0..height {
        raw.push(0);
        for x in 0..width {
            let p = pixels[y as usize * width as usize + x as usize];
            raw.push(p as u8);
            raw.push((p >> 8) as u8);
            raw.push((p >> 16) as u8);
            raw.push((p >> 24) as u8);
        }
    }
    write_file(path, &encode_raw_png(&raw, width, height, 6));
}
