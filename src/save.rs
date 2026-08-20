//! Persistencia de la partida (Fase 7): guardado y carga en JSON.
//!
//! Se guarda el **progreso** (asignaturas superadas), la **posición del
//! jugador** y el **estado de las puertas**. El archivo `savegame.json` se
//! escribe en la raíz del proyecto (la misma carpeta `assets`, resuelta por
//! `BEVY_ASSET_ROOT`).
//!
//! Guardado automático: al superar una asignatura y al volver al menú
//! principal. Guardado manual: botón "Guardar partida" del menú de pausa.
//! Carga: botón "Continuar" del menú principal.

#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::game::{GameState, RestartWorld, SaveGameRequested};
use crate::player::{Player, PLAYER_SPAWN};
use crate::world::quiz::QuizSession;
use crate::world::Door;

/// Versión del formato de guardado (incrementar al cambiar `SaveData`).
#[cfg(not(target_arch = "wasm32"))]
const SAVE_VERSION: u32 = 1;

/// Progreso actual de la partida (recurso en memoria).
#[derive(Resource, Debug)]
pub struct Progress {
    /// Asignaturas superadas (acertar las 3 preguntas de su cuestionario).
    pub passed: Vec<String>,
    /// Última posición del jugador (se sincroniza mientras se explora).
    pub player_pos: Vec3,
    /// Estado de cada puerta, por id (para restaurar al cargar).
    pub doors: Vec<SavedDoor>,
}

/// Estado persistido de una puerta.
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct SavedDoor {
    pub id: u8,
    pub open: bool,
    pub x: f32,
}

impl Default for Progress {
    fn default() -> Self {
        Self {
            passed: Vec::new(),
            player_pos: PLAYER_SPAWN,
            doors: Vec::new(),
        }
    }
}

impl Progress {
    /// `true` si la asignatura ya está superada.
    pub fn has_passed(&self, subject: &str) -> bool {
        self.passed.iter().any(|p| p == subject)
    }
}

/// Plugin de persistencia.
pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Progress::default())
            .add_systems(
                Update,
                (
                    sync_player_pos.run_if(in_state(GameState::Playing)),
                    update_progress.run_if(in_state(GameState::Playing)),
                    save_system.run_if(on_event::<SaveGameRequested>),
                    reset_progress.run_if(on_event::<RestartWorld>),
                ),
            )
            .add_systems(OnEnter(GameState::MainMenu), auto_save_on_menu);
    }
}

// ---- Sistemas --------------------------------------------------------------

/// Mantiene la posición del jugador en el progreso mientras se explora.
fn sync_player_pos(player_q: Query<&Transform, With<Player>>, mut progress: ResMut<Progress>) {
    if let Ok(tf) = player_q.single() {
        progress.player_pos = tf.translation;
    }
}

/// Registra la asignatura como superada cuando acaba un cuestionario con
/// las 3 respuestas correctas y dispara el guardado automático.
fn update_progress(
    quiz: Option<Res<QuizSession>>,
    mut progress: ResMut<Progress>,
    mut save: EventWriter<SaveGameRequested>,
) {
    let Some(session) = quiz else {
        return;
    };
    if session.done && session.passed() && !progress.has_passed(session.subject) {
        progress.passed.push(session.subject.to_string());
        save.write(SaveGameRequested);
    }
}

/// Escribe `savegame.json` con el estado actual cuando llega un evento
/// `SaveGameRequested`.
#[cfg(not(target_arch = "wasm32"))]
fn save_system(
    progress: Res<Progress>,
    doors: Query<(&Door, &Transform)>,
    mut save: EventReader<SaveGameRequested>,
) {
    if save.read().next().is_none() {
        return;
    }
    let data = SaveData {
        version: SAVE_VERSION,
        passed: progress.passed.clone(),
        player_pos: progress.player_pos.to_array(),
        doors: doors
            .iter()
            .map(|(door, tf)| SavedDoor {
                id: door.id,
                open: door.open,
                x: tf.translation.x,
            })
            .collect(),
    };
    let json = match serde_json::to_string_pretty(&data) {
        Ok(json) => json,
        Err(err) => {
            bevy::log::warn!("no se pudo serializar la partida: {err}");
            return;
        }
    };
    if let Some(path) = save_path() {
        if let Err(err) = fs::write(&path, json) {
            bevy::log::warn!("no se pudo guardar la partida en {}: {err}", path.display());
        }
    }
}

/// En la web (WASM) no hay sistema de archivos: el guardado es un no-op.
#[cfg(target_arch = "wasm32")]
fn save_system(
    progress: Res<Progress>,
    doors: Query<(&Door, &Transform)>,
    mut save: EventReader<SaveGameRequested>,
) {
    if save.read().next().is_none() {
        return;
    }
    // Sin soporte de archivos: solo se registra el progreso en memoria.
    let _ = (&progress, &doors);
}

/// Restablece el progreso al reiniciar la partida desde la pausa.
fn reset_progress(
    mut progress: ResMut<Progress>,
    mut restart: EventReader<RestartWorld>,
) {
    if restart.read().next().is_some() {
        *progress = Progress::default();
    }
}

/// Guardado automático al entrar en el menú principal. Solo crea el archivo
/// si ya existe una partida o hay progreso real (asignatura superada); así
/// el botón "Continuar" no aparece en un primer arranque.
fn auto_save_on_menu(progress: Res<Progress>, mut save: EventWriter<SaveGameRequested>) {
    if save_exists() || !progress.passed.is_empty() {
        save.write(SaveGameRequested);
    }
}

// ---- Formato de guardado ---------------------------------------------------

/// Contenido del archivo `savegame.json`.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Serialize, Deserialize)]
struct SaveData {
    version: u32,
    passed: Vec<String>,
    player_pos: [f32; 3],
    doors: Vec<SavedDoor>,
}

/// Ruta del archivo de guardado (junto a la carpeta `assets`).
/// En la web (WASM) no hay sistema de archivos: se devuelve `None` y el
/// guardado se convierte en un no-op (el botón "Continuar" no aparece).
#[cfg(not(target_arch = "wasm32"))]
fn save_path() -> Option<PathBuf> {
    if let Some(root) = std::env::var_os("BEVY_ASSET_ROOT") {
        return Some(Path::new(&root).join("savegame.json"));
    }
    std::env::current_dir().ok().map(|dir| dir.join("savegame.json"))
}

#[cfg(target_arch = "wasm32")]
#[allow(dead_code)]
fn save_path() -> Option<std::path::PathBuf> {
    None
}

/// `true` si existe una partida guardada (para el botón "Continuar").
/// En la web no hay guardado: siempre `false`.
#[cfg(not(target_arch = "wasm32"))]
pub fn save_exists() -> bool {
    save_path().map_or(false, |path| path.exists())
}

#[cfg(target_arch = "wasm32")]
pub fn save_exists() -> bool {
    false
}

/// Carga la partida guardada, si existe y el formato es válido.
/// En la web no hay guardado: siempre `None`.
#[cfg(not(target_arch = "wasm32"))]
pub fn try_load_save() -> Option<Progress> {
    let path = save_path()?;
    let text = fs::read_to_string(path).ok()?;
    let data: SaveData = serde_json::from_str(&text).ok()?;
    if data.version != SAVE_VERSION {
        return None;
    }
    Some(Progress {
        passed: data.passed,
        player_pos: Vec3::from(data.player_pos),
        doors: data.doors,
    })
}

#[cfg(target_arch = "wasm32")]
pub fn try_load_save() -> Option<Progress> {
    None
}

// ---- Tests -----------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_data_round_trip() {
        let data = SaveData {
            version: SAVE_VERSION,
            passed: vec!["Matemáticas".to_string(), "Historia".to_string()],
            player_pos: [1.5, 2.0, 16.25],
            doors: vec![SavedDoor { id: 0, open: false, x: 0.0 }],
        };
        let json = serde_json::to_string(&data).expect("serializa");
        let back: SaveData = serde_json::from_str(&json).expect("deserializa");
        assert_eq!(back.version, SAVE_VERSION);
        assert_eq!(back.passed, data.passed);
        assert_eq!(back.player_pos, data.player_pos);
        assert_eq!(back.doors.len(), 1);
        assert_eq!(back.doors[0].id, 0);
        assert_eq!(back.doors[0].open, false);
    }

    #[test]
    fn progress_default_uses_spawn_point() {
        let progress = Progress::default();
        assert!(progress.passed.is_empty());
        assert!(progress.doors.is_empty());
        assert_eq!(progress.player_pos, PLAYER_SPAWN);
        assert!(!progress.has_passed("Informática"));
    }
}
