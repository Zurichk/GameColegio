//! # GameColegio
//!
//! Videojuego educativo 3D ambientado en un colegio, desarrollado con
//! [Bevy](https://bevyengine.org/).
//!
//! Punto de entrada de la aplicación. La lógica del juego está organizada
//! en módulos (`game`, `world`, `player`, `camera`), cada uno con su propio
//! plugin de Bevy.

pub mod board;
mod audio;
mod camera;
mod classic;
mod fx;
mod game;
mod hud;
mod i18n;
mod learning;
mod menu;
mod pause;
mod player;
mod save;
mod settings;
mod world;

use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;

use crate::game::GamePlugin;

/// Localiza la raíz del proyecto para que Bevy resuelva la carpeta `assets`.
///
/// Bevy busca los assets relativos a `BEVY_ASSET_ROOT`, `CARGO_MANIFEST_DIR`
/// o, en último caso, el directorio del ejecutable. Al lanzar
/// `gamecolegio.exe` directamente (doble clic) no existe ninguna de esas
/// variables, así que subimos desde el ejecutable hasta encontrar la carpeta
/// `assets` (con `cargo run` la misma ruta también es válida).
///
/// En la web (WASM) los assets se sirven por HTTP y no hay sistema de
/// archivos: esta función es un no-op.
#[cfg(not(target_arch = "wasm32"))]
fn ensure_asset_root() {
    if std::env::var_os("BEVY_ASSET_ROOT").is_some() {
        return;
    }
    let start = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(std::path::Path::to_path_buf));
    let mut candidate = start.as_deref();
    while let Some(dir) = candidate {
        if dir.join("assets").is_dir() {
            // `set_var` es unsafe en edition 2024; aquí es seguro porque
            // se llama en el hilo principal antes de arrancar Bevy.
            unsafe { std::env::set_var("BEVY_ASSET_ROOT", dir) };
            return;
        }
        candidate = dir.parent();
    }
}

#[cfg(target_arch = "wasm32")]
fn ensure_asset_root() {}

fn main() {
    ensure_asset_root();
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Game Colegio".to_string(),
                        resolution: (1600.0_f32, 900.0_f32).into(),
                        ..default()
                    }),
                    ..default()
                })
                // Silencia el ruido del cargador de Vulkan (capas de overlay
                // de Steam ausentes) y los avisos internos de ventana.
                .set(LogPlugin {
                    filter: "info,wgpu_hal::vulkan=off,bevy_diagnostic=warn,bevy_winit=warn,bevy_window=warn"
                        .to_string(),
                    level: Level::INFO,
                    ..default()
                }),
        )
        .add_plugins(GamePlugin)
        .run();
}