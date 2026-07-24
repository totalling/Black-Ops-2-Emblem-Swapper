use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use image::imageops::FilterType;
use image::{GrayImage, Luma, Rgba, RgbaImage};

use crate::error::AppError;
use crate::emblem::format::Layer;
use crate::emblem::shape_map::shape_file_stem;

pub struct ShapeCache {
    shapes_dir: PathBuf,
    cache: Mutex<HashMap<u16, Option<GrayImage>>>,
}

impl ShapeCache {
    pub fn new(shapes_dir: PathBuf) -> Self {
        ShapeCache {
            shapes_dir,
            cache: Mutex::new(HashMap::new()),
        }
    }

    fn load_alpha(&self, shape_id: u16) -> Option<GrayImage> {
        {
            let cache = self.cache.lock().unwrap();
            if let Some(entry) = cache.get(&shape_id) {
                return entry.clone();
            }
        }

        let result = (|| -> Option<GrayImage> {
            let stem = shape_file_stem(shape_id)?;
            let path: PathBuf = self.shapes_dir.join(format!("{stem}.png"));
            if !path.is_file() {
                return None;
            }
            let img = image::open(&path).ok()?.to_rgba8();
            let (w, h) = img.dimensions();
            let mut alpha = GrayImage::new(w, h);
            for (x, y, px) in img.enumerate_pixels() {
                alpha.put_pixel(x, y, Luma([px[3]]));
            }
            Some(alpha)
        })();

        self.cache.lock().unwrap().insert(shape_id, result.clone());
        result
    }
}

fn tinted_shape(cache: &ShapeCache, shape_id: u16, r: f32, g: f32, b: f32, a: f32) -> (RgbaImage, bool) {
    let a_clamped = a.clamp(0.0, 1.0);
    let rc = (r * 255.0) as u8;
    let gc = (g * 255.0) as u8;
    let bc = (b * 255.0) as u8;

    if let Some(alpha) = cache.load_alpha(shape_id) {
        let (w, h) = alpha.dimensions();
        let mut out = RgbaImage::new(w, h);
        for (x, y, px) in alpha.enumerate_pixels() {
            let out_alpha = (px[0] as f32 * a_clamped) as u8;
            out.put_pixel(x, y, Rgba([rc, gc, bc, out_alpha]));
        }
        (out, false)
    } else {
        let alpha_byte = (a_clamped * 255.0) as u8;
        let img = RgbaImage::from_pixel(256, 256, Rgba([rc, gc, bc, alpha_byte]));
        (img, true)
    }
}

fn outline_rgba(img: &RgbaImage, stroke_px: f64) -> RgbaImage {
    let radius = ((stroke_px / 2.0).round() as i64).max(1) as u32;
    let k = 2 * radius + 1;
    let (w, h) = img.dimensions();

    let mut alpha_channel = GrayImage::new(w, h);
    for (x, y, px) in img.enumerate_pixels() {
        alpha_channel.put_pixel(x, y, Luma([px[3]]));
    }

    let dilated = box_rank_filter(&alpha_channel, k, true);
    let eroded = box_rank_filter(&alpha_channel, k, false);

    let mut out = RgbaImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let src = img.get_pixel(x, y);
            let dv = dilated.get_pixel(x, y)[0] as i16;
            let ev = eroded.get_pixel(x, y)[0] as i16;
            let edge = (dv - ev).clamp(0, 255) as u8;
            out.put_pixel(x, y, Rgba([src[0], src[1], src[2], edge]));
        }
    }
    out
}

fn box_rank_filter(img: &GrayImage, k: u32, is_max: bool) -> GrayImage {
    let (w, h) = img.dimensions();
    let k = k as usize;

    let mut tmp = vec![0u8; (w * h) as usize];
    let mut row = vec![0u8; w as usize];
    for y in 0..h {
        for x in 0..w {
            row[x as usize] = img.get_pixel(x, y)[0];
        }
        let filtered = sliding_extreme_1d(&row, k, is_max);
        for x in 0..w {
            tmp[(y * w + x) as usize] = filtered[x as usize];
        }
    }

    let mut out = GrayImage::new(w, h);
    let mut col = vec![0u8; h as usize];
    for x in 0..w {
        for y in 0..h {
            col[y as usize] = tmp[(y * w + x) as usize];
        }
        let filtered = sliding_extreme_1d(&col, k, is_max);
        for y in 0..h {
            out.put_pixel(x, y, Luma([filtered[y as usize]]));
        }
    }
    out
}

fn sliding_extreme_1d(input: &[u8], k: usize, is_max: bool) -> Vec<u8> {
    let n = input.len();
    if n == 0 {
        return Vec::new();
    }
    let radius = k / 2;

    let mut padded = Vec::with_capacity(n + 2 * radius);
    padded.resize(radius, input[0]);
    padded.extend_from_slice(input);
    padded.resize(padded.len() + radius, input[n - 1]);

    let mut out = vec![0u8; n];
    let mut deque: std::collections::VecDeque<usize> = std::collections::VecDeque::new();

    for i in 0..padded.len() {
        let v = padded[i];
        while let Some(&back) = deque.back() {
            let should_pop = if is_max {
                padded[back] <= v
            } else {
                padded[back] >= v
            };
            if should_pop {
                deque.pop_back();
            } else {
                break;
            }
        }
        deque.push_back(i);
        if *deque.front().unwrap() + k <= i {
            deque.pop_front();
        }
        if i >= k - 1 {
            out[i - (k - 1)] = padded[*deque.front().unwrap()];
        }
    }
    out
}

fn round15(x: f64) -> f64 {
    (x * 1e15).round() / 1e15
}

fn cubic_weight(x: f64) -> f64 {
    const A: f64 = -0.5;
    let x = x.abs();
    if x <= 1.0 {
        (A + 2.0) * x * x * x - (A + 3.0) * x * x + 1.0
    } else if x < 2.0 {
        A * x * x * x - 5.0 * A * x * x + 8.0 * A * x - 4.0 * A
    } else {
        0.0
    }
}

fn sample_bicubic(img: &RgbaImage, sx: f64, sy: f64) -> Rgba<u8> {
    let (w, h) = img.dimensions();
    let x0 = sx.floor() as i64;
    let y0 = sy.floor() as i64;
    let fx = sx - x0 as f64;
    let fy = sy - y0 as f64;

    let mut acc = [0f64; 4];
    for m in -1i64..=2 {
        let wy = cubic_weight(fy - m as f64);
        if wy == 0.0 {
            continue;
        }
        for n in -1i64..=2 {
            let wx = cubic_weight(fx - n as f64);
            if wx == 0.0 {
                continue;
            }
            let weight = wx * wy;
            let px = x0.saturating_add(n);
            let py = y0.saturating_add(m);
            let pixel = if px >= 0 && px < w as i64 && py >= 0 && py < h as i64 {
                *img.get_pixel(px as u32, py as u32)
            } else {
                Rgba([0, 0, 0, 0])
            };
            for c in 0..4 {
                acc[c] += pixel[c] as f64 * weight;
            }
        }
    }

    Rgba([
        acc[0].round().clamp(0.0, 255.0) as u8,
        acc[1].round().clamp(0.0, 255.0) as u8,
        acc[2].round().clamp(0.0, 255.0) as u8,
        acc[3].round().clamp(0.0, 255.0) as u8,
    ])
}

fn rotate_expand_bicubic(img: &RgbaImage, angle_degrees_arg: f64) -> RgbaImage {
    let w = img.width() as f64;
    let h = img.height() as f64;

    let angle = -(angle_degrees_arg.rem_euclid(360.0)).to_radians();
    let cos_a = round15(angle.cos());
    let sin_a = round15(angle.sin());
    let (a, b, d, e) = (cos_a, sin_a, -sin_a, cos_a);
    let (mut c, mut f) = (0.0f64, 0.0f64);

    let transform =
        |x: f64, y: f64, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64| (a * x + b * y + c, d * x + e * y + f);

    let rotn_center = (w / 2.0, h / 2.0);
    let (t0, t1) = transform(-rotn_center.0, -rotn_center.1, a, b, c, d, e, f);
    c = t0 + rotn_center.0;
    f = t1 + rotn_center.1;

    let corners = [(0.0, 0.0), (w, 0.0), (w, h), (0.0, h)];
    let mut xs = Vec::with_capacity(4);
    let mut ys = Vec::with_capacity(4);
    for (cx0, cy0) in corners {
        let (tx, ty) = transform(cx0, cy0, a, b, c, d, e, f);
        xs.push(tx);
        ys.push(ty);
    }
    let max_x = xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let min_x = xs.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_y = ys.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let min_y = ys.iter().cloned().fold(f64::INFINITY, f64::min);
    let nw = max_x.ceil() - min_x.floor();
    let nh = max_y.ceil() - min_y.floor();

    let (t2, t3) = transform(-(nw - w) / 2.0, -(nh - h) / 2.0, a, b, c, d, e, f);
    c = t2;
    f = t3;

    let out_w = if nw.is_finite() { (nw.max(1.0)) as u32 } else { MAX_LAYER_DIM * 2 };
    let out_h = if nh.is_finite() { (nh.max(1.0)) as u32 } else { MAX_LAYER_DIM * 2 };
    let out_w = out_w.clamp(1, MAX_LAYER_DIM * 2);
    let out_h = out_h.clamp(1, MAX_LAYER_DIM * 2);

    let mut out = RgbaImage::new(out_w, out_h);
    for oy in 0..out.height() {
        for ox in 0..out.width() {
            let sx = a * (ox as f64 + 0.5) + b * (oy as f64 + 0.5) + c - 0.5;
            let sy = d * (ox as f64 + 0.5) + e * (oy as f64 + 0.5) + f - 0.5;
            out.put_pixel(ox, oy, sample_bicubic(img, sx, sy));
        }
    }
    out
}

fn alpha_composite(canvas: &mut RgbaImage, top: &RgbaImage, x: i64, y: i64) {
    let (cw, ch) = (canvas.width() as i64, canvas.height() as i64);
    let (tw, th) = (top.width() as i64, top.height() as i64);

    for ty in 0..th {
        let cy = y.saturating_add(ty);
        if cy < 0 || cy >= ch {
            continue;
        }
        for tx in 0..tw {
            let cx = x.saturating_add(tx);
            if cx < 0 || cx >= cw {
                continue;
            }
            let tp = top.get_pixel(tx as u32, ty as u32);
            let ta = tp[3] as f64 / 255.0;
            if ta <= 0.0 {
                continue;
            }
            let bp = *canvas.get_pixel(cx as u32, cy as u32);
            let ba = bp[3] as f64 / 255.0;
            let out_a = ta + ba * (1.0 - ta);
            if out_a <= 0.0 {
                canvas.put_pixel(cx as u32, cy as u32, Rgba([0, 0, 0, 0]));
                continue;
            }
            let mut out = [0u8; 4];
            for c in 0..3 {
                let tc = tp[c] as f64;
                let bc = bp[c] as f64;
                let outc = (tc * ta + bc * ba * (1.0 - ta)) / out_a;
                out[c] = outc.round().clamp(0.0, 255.0) as u8;
            }
            out[3] = (out_a * 255.0).round().clamp(0.0, 255.0) as u8;
            canvas.put_pixel(cx as u32, cy as u32, Rgba(out));
        }
    }
}

const MAX_LAYER_DIM: u32 = 2048;

fn scaled_dim(raw_scale: f32, base_px: f64) -> u32 {
    let value = 2f64.powf(raw_scale as f64) * base_px;
    if !value.is_finite() {
        return MAX_LAYER_DIM;
    }
    (value as u32).clamp(1, MAX_LAYER_DIM)
}

pub fn render_png(layers: &[Layer], size: u32, cache: &ShapeCache) -> RgbaImage {
    const BG: [u8; 4] = [24, 24, 24, 255];
    let mut canvas = RgbaImage::from_pixel(size, size, Rgba(BG));
    let cx = size as f64 / 2.0;
    let cy = cx;
    let unit = size as f64;
    let base_px = size as f64;

    let mut sorted: Vec<&Layer> = layers.iter().collect();
    sorted.sort_by_key(|l| l.index);

    for l in sorted {
        let w = scaled_dim(l.sx, base_px);
        let h = scaled_dim(l.sy, base_px);

        let (tinted, placeholder) = tinted_shape(cache, l.shape, l.r, l.g, l.b, l.a);
        let tinted = if l.flipped {
            image::imageops::flip_horizontal(&tinted)
        } else {
            tinted
        };

        let mut resized = image::imageops::resize(&tinted, w, h, FilterType::Lanczos3);
        if l.outlined && !placeholder {
            resized = outline_rgba(&resized, size as f64 / 118.0);
        }

        let rotated = rotate_expand_bicubic(&resized, -(l.rot as f64));

        let x = cx + (l.x as f64) * unit - (rotated.width() as f64) / 2.0;
        let y = cy + (l.y as f64) * unit - (rotated.height() as f64) / 2.0;

        alpha_composite(&mut canvas, &rotated, x.trunc() as i64, y.trunc() as i64);
    }

    canvas
}

pub fn render_file_png_bytes(path: &Path, size: u32, cache: &ShapeCache) -> Result<Vec<u8>, AppError> {
    let data = std::fs::read(path)?;
    let layers = crate::emblem::format::parse_slot_bytes(&data);
    let img = render_png(&layers, size, cache);

    let mut buf = std::io::Cursor::new(Vec::new());
    img.write_to(&mut buf, image::ImageFormat::Png)?;
    Ok(buf.into_inner())
}

pub fn render_file_png_bytes_cached(
    path: &Path,
    cache_path: &Path,
    size: u32,
    cache: &ShapeCache,
) -> Result<Vec<u8>, AppError> {
    let source_modified = std::fs::metadata(path)?.modified().ok();

    if let Some(source_modified) = source_modified {
        if let Ok(cache_meta) = std::fs::metadata(cache_path) {
            if let Ok(cache_modified) = cache_meta.modified() {
                if cache_modified >= source_modified {
                    if let Ok(bytes) = std::fs::read(cache_path) {
                        return Ok(bytes);
                    }
                }
            }
        }
    }

    let bytes = render_file_png_bytes(path, size, cache)?;
    let _ = std::fs::write(cache_path, &bytes);
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shapes_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/reference_shapes")
    }

    fn layer(index: usize, shape: u16, sx: f32, sy: f32, rot: f32, x: f32, y: f32, outlined: bool, flipped: bool) -> Layer {
        Layer {
            index,
            shape,
            r: 1.0,
            g: 0.4,
            b: 0.1,
            a: 1.0,
            x,
            y,
            sx,
            sy,
            rot,
            outlined,
            flipped,
        }
    }

    #[test]
    fn renders_full_pipeline_without_panicking() {
        let cache = ShapeCache::new(shapes_dir());
        let layers = vec![
            layer(0, 0, 0.0, 0.0, 0.0, 0.0, 0.0, false, false),
            layer(1, 38, -1.0, -1.0, 45.0, 0.3, -0.2, true, true),
            layer(2, 65000, -2.0, -2.0, 190.0, -0.4, 0.4, true, false),
        ];

        let img = render_png(&layers, 220, &cache);
        assert_eq!(img.width(), 220);
        assert_eq!(img.height(), 220);

        for px in img.pixels() {
            assert_eq!(px[3], 255);
        }
    }

    #[test]
    fn extreme_scale_values_from_corrupt_data_do_not_blow_up_allocation() {
        let cache = ShapeCache::new(shapes_dir());
        let layers = vec![
            layer(0, 0, 1000.0, 0.0, 0.0, 0.0, 0.0, false, false),
            layer(1, 0, f32::INFINITY, f32::NEG_INFINITY, 0.0, 0.0, 0.0, false, false),
            layer(2, 0, f32::NAN, f32::NAN, 0.0, 0.0, 0.0, false, false),
        ];

        let img = render_png(&layers, 220, &cache);
        assert_eq!(img.width(), 220);
        assert_eq!(img.height(), 220);
    }

    #[test]
    fn extreme_position_values_from_corrupt_data_do_not_overflow_compositing() {
        let cache = ShapeCache::new(shapes_dir());
        let layers = vec![
            layer(0, 0, 0.0, 0.0, 0.0, 1e30, 1e30, false, false),
            layer(1, 0, 0.0, 0.0, 0.0, f32::INFINITY, f32::NEG_INFINITY, false, false),
            layer(2, 0, 0.0, 0.0, 0.0, f32::NAN, f32::NAN, false, false),
        ];

        let img = render_png(&layers, 220, &cache);
        assert_eq!(img.width(), 220);
        assert_eq!(img.height(), 220);
    }

    #[test]
    fn scaled_dim_clamps_absurd_and_non_finite_values() {
        assert_eq!(scaled_dim(0.0, 220.0), 220);
        assert_eq!(scaled_dim(1000.0, 220.0), MAX_LAYER_DIM);
        assert_eq!(scaled_dim(f32::INFINITY, 220.0), MAX_LAYER_DIM);
        assert_eq!(scaled_dim(f32::NAN, 220.0), MAX_LAYER_DIM);
        assert_eq!(scaled_dim(-1000.0, 220.0), 1);
    }

    #[test]
    fn rotation_bbox_matches_pillow_expand_semantics() {
        let src = RgbaImage::from_pixel(100, 100, Rgba([255, 255, 255, 255]));
        let rotated = rotate_expand_bicubic(&src, 45.0);
        assert_eq!(rotated.width(), 142);
        assert_eq!(rotated.height(), 142);

        let unrotated = rotate_expand_bicubic(&src, 0.0);
        assert_eq!(unrotated.width(), 100);
        assert_eq!(unrotated.height(), 100);
    }

    #[test]
    fn outline_extracts_a_ring_not_a_fill() {
        let mut solid = RgbaImage::from_pixel(40, 40, Rgba([255, 0, 0, 255]));
        for y in 0..40u32 {
            for x in 0..40u32 {
                if x < 5 || x >= 35 || y < 5 || y >= 35 {
                    solid.put_pixel(x, y, Rgba([255, 0, 0, 0]));
                }
            }
        }
        let outlined = outline_rgba(&solid, 3.0);
        assert_eq!(outlined.get_pixel(20, 20)[3], 0);
        assert!(outlined.get_pixel(5, 20)[3] > 0);
    }

    #[test]
    fn cached_render_reuses_disk_cache_until_source_changes() {
        let cache = ShapeCache::new(shapes_dir());
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("slot_1.bin");
        let cache_path = dir.path().join("thumb_1_220.png");
        let layer_bytes = crate::emblem::format::LAYER_SIZE * crate::emblem::format::NUM_LAYERS;
        std::fs::write(&source_path, vec![0u8; layer_bytes]).unwrap();

        let bytes = render_file_png_bytes_cached(&source_path, &cache_path, 220, &cache).unwrap();
        assert!(cache_path.exists());
        assert_eq!(std::fs::read(&cache_path).unwrap(), bytes);

        std::fs::write(&cache_path, b"stale-but-should-be-served").unwrap();
        let served = render_file_png_bytes_cached(&source_path, &cache_path, 220, &cache).unwrap();
        assert_eq!(served, b"stale-but-should-be-served");

        std::thread::sleep(std::time::Duration::from_millis(1100));
        std::fs::write(&source_path, vec![1u8; layer_bytes]).unwrap();
        let refreshed = render_file_png_bytes_cached(&source_path, &cache_path, 220, &cache).unwrap();
        assert_ne!(refreshed, b"stale-but-should-be-served".to_vec());
    }
}
