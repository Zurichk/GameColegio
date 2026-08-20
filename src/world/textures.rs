//! Generación de texturas procedurales (sin archivos externos).
//!
//! Se crean imágenes RGBA con ruido determinista para dar variedad visual
//! al colegio: césped, baldosas, yeso, pizarra y madera.

use bevy::image::{ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{AddressMode, Extent3d, TextureDimension, TextureFormat};

/// Tamaño de las texturas generadas (potencia de dos, económica).
const TEX_SIZE: u32 = 128;

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

/// Césped: verde con mechones y parches más claros.
pub fn grass() -> Image {
    textured_image(|x, y| {
        let v = noise(x, y, 7);
        let patch = noise(x / 2, y / 2, 9);
        let mut r = 0.30 + (v - 0.5) * 0.08;
        let mut g = 0.52 + (v - 0.5) * 0.18;
        let mut b = 0.28 + (v - 0.5) * 0.06;
        if patch > 0.72 {
            g += 0.10;
            r += 0.06;
            b += 0.05;
        }
        if v > 0.90 {
            // Mechón de hierba clara.
            (r, g, b) = (0.46, 0.74, 0.34);
        }
        rgba(r, g, b)
    })
}

/// Baldosas de suelo interior: cuadrícula crema con juntas y veteado.
pub fn floor_tiles() -> Image {
    const TILE: u32 = 32; // 4x4 baldosas por textura.
    textured_image(|x, y| {
        let tx = x % TILE;
        let ty = y % TILE;
        // Junta de 1 px.
        if tx == 0 || ty == 0 {
            return rgba(0.45, 0.43, 0.40);
        }
        let cell = noise(x / TILE, y / TILE, 3);
        let shade = 0.88 + (cell - 0.5) * 0.08;
        // Veteado sutil dentro de cada baldosa.
        let vein = ((x as f32 * 0.5 + y as f32 * 0.3).sin() * 0.5 + 0.5) * 0.03;
        let s = shade + vein;
        rgba(s, s * 0.97, s * 0.90)
    })
}

/// Yeso de pared: crema con grano muy sutil.
pub fn plaster() -> Image {
    textured_image(|x, y| {
        let v = noise(x, y, 11);
        let speckle = noise(x / 3, y / 3, 17);
        let base = 0.93 + (v - 0.5) * 0.05 + (speckle - 0.5) * 0.03;
        rgba(base, base * 0.985, base * 0.94)
    })
}

/// Pizarra: verde oscuro con rayas tenues y polvo de tiza.
pub fn blackboard() -> Image {
    textured_image(|x, y| {
        let v = noise(x, y, 5);
        let stripe = if (y / 4) % 2 == 0 { 0.015 } else { -0.015 };
        let chalk = if noise(x, y, 23) > 0.985 { 0.20 } else { 0.0 };
        let shade = 0.13 + (v - 0.5) * 0.05 + stripe + chalk;
        rgba(shade * 0.85, shade * 2.2, shade * 1.05)
    })
}

/// Madera con vetas: para puertas, escritorios y marcos.
pub fn wood() -> Image {
    textured_image(|x, y| {
        let grain = ((y as f32 / 7.0).sin() * 0.5 + 0.5) * 0.22;
        let v = noise(x, y, 13);
        let knot = if noise(x, y, 19) > 0.97 { 0.10 } else { 0.0 };
        let base = 0.44 + grain + (v - 0.5) * 0.10 + knot;
        rgba(base * 1.18, base * 0.85, base * 0.52)
    })
}