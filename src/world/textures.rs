//! Generación de texturas procedurales (sin archivos externos).
//!
//! Se crean imágenes RGBA con ruido determinista para dar variedad visual
//! al colegio: césped, baldosas, yeso, pizarra, madera y piedra.
//! v2 fotorealista low-poly: 256px + variación de roughness simulada via color.

use bevy::image::{ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{AddressMode, Extent3d, TextureDimension, TextureFormat};

/// Tamaño de las texturas generadas — 256 para definición fotorealista sin
/// pesar en WASM (256*256*4 = 256KB por textura).
const TEX_SIZE: u32 = 256;

/// Ruido determinista en `[0, 1)` a partir de la posición del píxel.
fn noise(x: u32, y: u32, seed: u32) -> f32 {
    let mut h = x
        .wrapping_mul(374_761_393)
        .wrapping_add(y.wrapping_mul(668_265_263))
        .wrapping_add(seed);
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    h ^= h >> 16;
    (h & 0xffff) as f32 / 65_535.0
}

/// Ruido fractal simple para variación orgánica.
fn fbm(x: u32, y: u32, seed: u32) -> f32 {
    noise(x, y, seed) * 0.5 + noise(x * 2, y * 2, seed + 1) * 0.25 + noise(x * 4, y * 4, seed + 2) * 0.125
}

/// Convierte un color `[0,1]` a bytes RGBA.
fn rgba(r: f32, g: f32, b: f32) -> [u8; 4] {
    let c = |v: f32| (v.clamp(0.0, 1.0) * 255.0) as u8;
    [c(r), c(g), c(b), 255]
}

/// Crea una imagen con repetición de textura y el patrón dado.
fn textured_image(pixels: impl Fn(u32, u32) -> [u8; 4]) -> Image {
    let mut data = Vec::with_capacity((TEX_SIZE * TEX_SIZE * 4) as usize);
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            data.extend_from_slice(&pixels(x, y));
        }
    }
    let mut image = Image::new(
        Extent3d {
            width: TEX_SIZE,
            height: TEX_SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: AddressMode::Repeat.into(),
        address_mode_v: AddressMode::Repeat.into(),
        ..default()
    });
    image
}

/// Césped fotorealista: verde con parches secos, mechones y variación de altura.
pub fn grass() -> Image {
    textured_image(|x, y| {
        let v = fbm(x, y, 7);
        let patch = noise(x / 4, y / 4, 9);
        let blade = noise(x, y, 31);
        let mut r = 0.26 + (v - 0.5) * 0.12;
        let mut g = 0.50 + (v - 0.5) * 0.22;
        let mut b = 0.24 + (v - 0.5) * 0.10;
        if patch > 0.68 {
            // Parche seco/soleado
            g += 0.08;
            r += 0.07;
            b -= 0.02;
        }
        if blade > 0.88 {
            (r, g, b) = (0.42, 0.68, 0.30);
        }
        if patch < 0.22 {
            // Sombra húmeda
            r *= 0.85;
            g *= 0.88;
            b *= 0.92;
        }
        rgba(r, g, b)
    })
}

/// Baldosas de suelo interior: porcelánico crema 60x60 con juntas finas y bisel.
pub fn floor_tiles() -> Image {
    const TILE: u32 = 64; // 4x4 baldosas por textura a 256px (cada baldosa 64px)
    textured_image(|x, y| {
        let tx = x % TILE;
        let ty = y % TILE;
        // Junta de 2px con bisel oscuro + claro
        if tx <= 1 || ty <= 1 {
            return rgba(0.38, 0.36, 0.34);
        }
        if tx == 2 || ty == 2 {
            return rgba(0.52, 0.50, 0.48);
        }
        let cell = noise(x / TILE, y / TILE, 3);
        let shade = 0.90 + (cell - 0.5) * 0.06;
        // Veteado diagonal sutil + moteado
        let vein = ((x as f32 * 0.35 + y as f32 * 0.22).sin() * 0.5 + 0.5) * 0.025;
        let speckle = (noise(x, y, 53) - 0.5) * 0.015;
        let s = shade + vein + speckle;
        rgba(s, s * 0.97, s * 0.92)
    })
}

/// Yeso de pared: crema con grano fino y micro-sombras de enlucido.
pub fn plaster() -> Image {
    textured_image(|x, y| {
        let v = fbm(x, y, 11);
        let speckle = noise(x / 2, y / 2, 17);
        let trowel = ((x as f32 * 0.08 + y as f32 * 0.05).sin() * 0.5 + 0.5) * 0.015;
        let base = 0.94 + (v - 0.5) * 0.04 + (speckle - 0.5) * 0.03 + trowel;
        rgba(base, base * 0.985, base * 0.94)
    })
}

/// Pizarra verde pizarra con vetas, polvo de tiza y marco desgastado.
pub fn blackboard() -> Image {
    textured_image(|x, y| {
        let v = noise(x, y, 5);
        let stripe = if (y / 6) % 2 == 0 { 0.012 } else { -0.012 };
        let chalk_dust = if noise(x, y, 23) > 0.982 { 0.18 } else { 0.0 };
        let chalk_stroke = if noise(x / 3, y / 8, 29) > 0.88 { 0.04 } else { 0.0 };
        let shade = 0.14 + (v - 0.5) * 0.04 + stripe + chalk_dust + chalk_stroke;
        rgba(shade * 0.80, shade * 1.95, shade * 1.02)
    })
}

/// Madera roble con vetas anchas, nudos y variación de tono por tabla.
pub fn wood() -> Image {
    textured_image(|x, y| {
        let board = y / 32; // cada tabla 32px de alto
        let board_tint = (noise(board, 0, 41) - 0.5) * 0.08;
        let grain = ((y as f32 / 9.0).sin() * 0.5 + 0.5) * 0.20;
        let grain2 = ((y as f32 / 3.5 + x as f32 * 0.02).sin() * 0.5 + 0.5) * 0.06;
        let v = noise(x, y, 13);
        let knot = if noise(x, y, 19) > 0.973 { 0.12 } else { 0.0 };
        let base = 0.46 + grain + grain2 + (v - 0.5) * 0.08 + knot + board_tint;
        rgba(base * 1.16, base * 0.82, base * 0.50)
    })
}

/// Piedra para zócalo/bordillo: gris cálido con juntas y variación de bloque.
pub fn stone() -> Image {
    const BLOCK_W: u32 = 64;
    const BLOCK_H: u32 = 32;
    textured_image(|x, y| {
        let bx = x % BLOCK_W;
        let by = y % BLOCK_H;
        if bx <= 1 || by <= 1 {
            return rgba(0.42, 0.41, 0.40); // junta
        }
        let cell = noise(x / BLOCK_W, y / BLOCK_H, 43);
        let grain = (noise(x, y, 47) - 0.5) * 0.06;
        let base = 0.62 + (cell - 0.5) * 0.10 + grain;
        rgba(base, base * 0.98, base * 0.96)
    })
}

/// Teja cerámica terracota para tejado (vista superior).
pub fn roof_tiles() -> Image {
    const TILE_W: u32 = 32;
    const TILE_H: u32 = 16;
    textured_image(|x, y| {
        let tx = x % TILE_W;
        let ty = y % TILE_H;
        // Solape de teja
        let edge = if ty <= 1 || tx <= 1 { -0.08 } else { 0.0 };
        let highlight = if ty == 2 { 0.06 } else { 0.0 };
        let cell = noise(x / TILE_W, y / TILE_H, 51);
        let base = 0.52 + (cell - 0.5) * 0.10 + edge + highlight;
        rgba(base * 1.15, base * 0.55, base * 0.42)
    })
}
