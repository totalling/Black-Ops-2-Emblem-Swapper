pub const LAYER_SIZE: usize = 44;
pub const NUM_LAYERS: usize = 32;
pub const EMPTY_SHAPE: u16 = 65535;

#[derive(Debug, Clone)]
pub struct Layer {
    pub index: usize,
    pub shape: u16,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
    pub x: f32,
    pub y: f32,
    pub sx: f32,
    pub sy: f32,
    pub rot: f32,
    pub outlined: bool,
    pub flipped: bool,
}

pub fn strip_http(data: &[u8]) -> &[u8] {
    match find_subslice(data, b"\r\n\r\n") {
        Some(pos) => &data[pos + 4..],
        None => data,
    }
}

fn find_subslice(data: &[u8], needle: &[u8]) -> Option<usize> {
    data.windows(needle.len()).position(|w| w == needle)
}

fn read_f32_le(rec: &[u8], offset: usize) -> f32 {
    f32::from_le_bytes([rec[offset], rec[offset + 1], rec[offset + 2], rec[offset + 3]])
}

pub fn parse_slot_bytes(data: &[u8]) -> Vec<Layer> {
    let body = strip_http(data);
    let count = (body.len() / LAYER_SIZE).min(NUM_LAYERS);
    let mut layers = Vec::with_capacity(count);

    for i in 0..count {
        let rec = &body[i * LAYER_SIZE..(i + 1) * LAYER_SIZE];
        let shape = u16::from_le_bytes([rec[0], rec[1]]);
        if shape == EMPTY_SHAPE {
            continue;
        }
        layers.push(Layer {
            index: i,
            shape,
            r: read_f32_le(rec, 4),
            g: read_f32_le(rec, 8),
            b: read_f32_le(rec, 12),
            a: read_f32_le(rec, 16),
            x: read_f32_le(rec, 20),
            y: read_f32_le(rec, 24),
            sx: read_f32_le(rec, 28),
            sy: read_f32_le(rec, 32),
            rot: read_f32_le(rec, 36),
            outlined: rec[40] != 0,
            flipped: rec[41] != 0,
        });
    }
    layers
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_layer_record(shape: u16, r: f32, rot: f32, outlined: bool, flipped: bool) -> [u8; LAYER_SIZE] {
        let mut rec = [0u8; LAYER_SIZE];
        rec[0..2].copy_from_slice(&shape.to_le_bytes());
        rec[4..8].copy_from_slice(&r.to_le_bytes());
        rec[8..12].copy_from_slice(&0.5f32.to_le_bytes());
        rec[12..16].copy_from_slice(&0.25f32.to_le_bytes());
        rec[16..20].copy_from_slice(&1.0f32.to_le_bytes());
        rec[20..24].copy_from_slice(&0.1f32.to_le_bytes());
        rec[24..28].copy_from_slice(&(-0.1f32).to_le_bytes());
        rec[28..32].copy_from_slice(&0.0f32.to_le_bytes());
        rec[32..36].copy_from_slice(&0.0f32.to_le_bytes());
        rec[36..40].copy_from_slice(&rot.to_le_bytes());
        rec[40] = outlined as u8;
        rec[41] = flipped as u8;
        rec
    }

    #[test]
    fn strips_http_headers_and_parses_layers() {
        let mut body = Vec::new();
        body.extend_from_slice(&make_layer_record(0, 1.0, 45.0, true, false));
        body.extend_from_slice(&make_layer_record(EMPTY_SHAPE, 0.0, 0.0, false, false));
        body.extend_from_slice(&make_layer_record(38, 0.0, 0.0, false, true));
        for _ in 3..NUM_LAYERS {
            body.extend_from_slice(&make_layer_record(EMPTY_SHAPE, 0.0, 0.0, false, false));
        }
        assert_eq!(body.len(), LAYER_SIZE * NUM_LAYERS);

        let mut framed = b"HTTP/1.1 200 OK\r\nContent-Length: 1408\r\n\r\n".to_vec();
        framed.extend_from_slice(&body);

        let layers = parse_slot_bytes(&framed);
        assert_eq!(layers.len(), 2);
        assert_eq!(layers[0].index, 0);
        assert_eq!(layers[0].shape, 0);
        assert!(layers[0].outlined);
        assert!(!layers[0].flipped);
        assert_eq!(layers[1].index, 2);
        assert_eq!(layers[1].shape, 38);
        assert!(layers[1].flipped);
    }
}
