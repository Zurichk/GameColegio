//! Pantalla de ajustes (Fase 8): sensibilidad del ratón y volumen.
//!
//! Se abre desde el menú principal o desde la pausa (`GameState::Settings`)
//! y se cierra con el botón "Volver" o la tecla Esc. Los valores se guardan
//! en `settings.json` junto al guardado de partida.

#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use std::path::{Path, PathBuf};

use bevy::audio::{AudioSink, GlobalVolume, Volume};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::audio::{play_click, AmbientMusic, Sfx};
use crate::game::GameState;
use crate::i18n::{self, tr, Language};

/// Ajustes del jugador.
#[derive(Resource, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Settings {
    /// Sensibilidad del ratón (1..=10; 5 = la predeterminada).
    pub sensitivity: f32,
    /// Volumen maestro en % (0..=100).
    pub volume: f32,
    /// Idioma de la interfaz y de las preguntas.
    pub language: Language,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            sensitivity: 5.0,
            volume: 80.0,
            language: Language::Es,
        }
    }
}

impl Settings {
    /// Factor multiplicador de la sensibilidad base de la cámara.
    pub fn sensitivity_multiplier(&self) -> f32 {
        (self.sensitivity / 5.0).clamp(0.2, 2.0)
    }

    /// Volumen en escala lineal (0..=1).
    pub fn linear_volume(&self) -> f32 {
        (self.volume / 100.0).clamp(0.0, 1.0)
    }
}

/// Estado al que volver al cerrar los ajustes (menú principal o pausa).
#[derive(Resource)]
pub struct SettingsReturn(pub GameState);

/// Raíz de la pantalla de ajustes.
#[derive(Component)]
pub struct SettingsUi;

/// Botón "−" de la sensibilidad.
#[derive(Component)]
pub struct SensDownButton;

/// Botón "+" de la sensibilidad.
#[derive(Component)]
pub struct SensUpButton;

/// Botón "−" del volumen.
#[derive(Component)]
pub struct VolDownButton;

/// Botón "+" del volumen.
#[derive(Component)]
pub struct VolUpButton;

/// Botón de idioma (Español/English/Français).
#[derive(Component)]
pub struct LangButton(pub Language);

/// Botón de volver.
#[derive(Component)]
pub struct BackButton;

/// Texto del valor de sensibilidad.
#[derive(Component)]
pub struct SensValue;

/// Texto del valor de volumen.
#[derive(Component)]
pub struct VolValue;

/// Plugin de la pantalla de ajustes.
pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        let settings = load_settings();
        // El idioma guardado pasa a ser el idioma activo del juego.
        i18n::set_language(settings.language);
        app.insert_resource(settings)
            .insert_resource(GlobalVolume::new(Volume::Linear(settings.linear_volume())))
            .add_systems(OnEnter(GameState::Settings), spawn_settings_ui)
            .add_systems(OnExit(GameState::Settings), despawn_settings_ui)
            .add_systems(
                Update,
                settings_input.run_if(in_state(GameState::Settings)),
            )
            // El bucle de ambiente sigue sonando siempre; se le aplica el
            // volumen elegido (GlobalVolume no afecta al audio ya en curso).
            .add_systems(Update, apply_ambient_volume);
    }
}

/// Factor de escala del ambiente respecto al volumen maestro.
const AMBIENT_VOLUME_SCALE: f32 = 0.4;

/// Ajusta en vivo el volumen del bucle de ambiente (`AudioSink`).
fn apply_ambient_volume(
    settings: Res<Settings>,
    mut sinks: Query<&mut AudioSink, With<AmbientMusic>>,
) {
    let volume = Volume::Linear(settings.linear_volume() * AMBIENT_VOLUME_SCALE);
    for mut sink in &mut sinks {
        sink.set_volume(volume);
    }
}

/// Construye la pantalla de ajustes.
fn spawn_settings_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands
        .spawn((
            SettingsUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(18.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.04, 0.10, 0.85)),
            Visibility::Visible,
        ))
        .with_children(|root| {
            root.spawn(ui_text("AJUSTES", 54.0, Color::srgb(1.0, 0.90, 0.50), &font));
            root.spawn(Node {
                height: Val::Px(10.0),
                ..default()
            });

            // Sensibilidad del ratón.
            root.spawn(ui_text(
                "Sensibilidad del ratón",
                24.0,
                Color::srgb(0.85, 0.90, 1.0),
                &font,
            ));
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(18.0),
                ..default()
            })
            .with_children(|row| {
                spawn_small_button(row, "−", SensDownButton, &font);
                row.spawn((
                    SensValue,
                    Text::new(""),
                    TextFont {
                        font: font.clone(),
                        font_size: 30.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
                spawn_small_button(row, "+", SensUpButton, &font);
            });

            // Volumen.
            root.spawn(ui_text(
                "Volumen",
                24.0,
                Color::srgb(0.85, 0.90, 1.0),
                &font,
            ));
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(18.0),
                ..default()
            })
            .with_children(|row| {
                spawn_small_button(row, "−", VolDownButton, &font);
                row.spawn((
                    VolValue,
                    Text::new(""),
                    TextFont {
                        font: font.clone(),
                        font_size: 30.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
                spawn_small_button(row, "+", VolUpButton, &font);
            });

            // Idioma.
            root.spawn(ui_text(
                "Idioma",
                24.0,
                Color::srgb(0.85, 0.90, 1.0),
                &font,
            ));
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(14.0),
                ..default()
            })
            .with_children(|row| {
                for lang in [Language::Es, Language::En, Language::Fr] {
                    spawn_big_button(row, lang.name(), LangButton(lang), &font);
                }
            });

            root.spawn(Node {
                height: Val::Px(14.0),
                ..default()
            });
            spawn_big_button(root, "Volver", BackButton, &font);
        });
}

/// Destruye la pantalla de ajustes.
fn despawn_settings_ui(mut commands: Commands, roots: Query<Entity, With<SettingsUi>>) {
    for root in &roots {
        commands.entity(root).despawn();
    }
}

/// Procesa los botones y teclas de los ajustes.
fn settings_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<Settings>,
    mut global_volume: ResMut<GlobalVolume>,
    return_to: Res<SettingsReturn>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    sfx: Res<Sfx>,
    mut sens_values: Query<&mut Text, (With<SensValue>, Without<VolValue>)>,
    mut vol_values: Query<&mut Text, (With<VolValue>, Without<SensValue>)>,
    interactions: Query<
        (
            &Interaction,
            Option<&SensDownButton>,
            Option<&SensUpButton>,
            Option<&VolDownButton>,
            Option<&VolUpButton>,
            Option<&LangButton>,
            Option<&BackButton>,
        ),
        Changed<Interaction>,
    >,
) {
    let mut changed = false;

    if keys.just_pressed(KeyCode::Escape) {
        play_click(&mut commands, &sfx);
        next_state.set(return_to.0);
        return;
    }

    for (interaction, sens_down, sens_up, vol_down, vol_up, lang, back) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        play_click(&mut commands, &sfx);
        if sens_down.is_some() {
            settings.sensitivity = (settings.sensitivity - 1.0).max(1.0);
            changed = true;
        } else if sens_up.is_some() {
            settings.sensitivity = (settings.sensitivity + 1.0).min(10.0);
            changed = true;
        } else if vol_down.is_some() {
            settings.volume = (settings.volume - 10.0).max(0.0);
            changed = true;
        } else if vol_up.is_some() {
            settings.volume = (settings.volume + 10.0).min(100.0);
            changed = true;
        } else if let Some(lang) = lang {
            if settings.language != lang.0 {
                settings.language = lang.0;
                i18n::set_language(lang.0);
                changed = true;
            }
        } else if back.is_some() {
            next_state.set(return_to.0);
            return;
        }
    }

    if changed {
        // Aplica el volumen al instante y persiste los ajustes.
        global_volume.volume = Volume::Linear(settings.linear_volume());
        save_settings(&settings);
    }

    // Actualiza los valores mostrados.
    if let Ok(mut text) = sens_values.single_mut() {
        *text = Text::new(format!("{}", settings.sensitivity as u32));
    }
    if let Ok(mut text) = vol_values.single_mut() {
        *text = Text::new(format!("{}%", settings.volume as u32));
    }
}

// ---- Persistencia ----------------------------------------------------------

/// Ruta del archivo de ajustes.
/// En la web (WASM) no hay sistema de archivos: se devuelve `None` y los
/// ajustes se mantienen solo en memoria (los predeterminados al recargar).
#[cfg(not(target_arch = "wasm32"))]
fn settings_path() -> Option<PathBuf> {
    if let Some(root) = std::env::var_os("BEVY_ASSET_ROOT") {
        return Some(Path::new(&root).join("settings.json"));
    }
    std::env::current_dir().ok().map(|dir| dir.join("settings.json"))
}

#[cfg(target_arch = "wasm32")]
#[allow(dead_code)]
fn settings_path() -> Option<std::path::PathBuf> {
    None
}

/// Carga los ajustes guardados (o los predeterminados si no existen).
/// En la web no hay persistencia: siempre los predeterminados.
#[cfg(not(target_arch = "wasm32"))]
fn load_settings() -> Settings {
    let Some(path) = settings_path() else {
        return Settings::default();
    };
    let Ok(text) = fs::read_to_string(path) else {
        return Settings::default();
    };
    serde_json::from_str(&text).unwrap_or_default()
}

#[cfg(target_arch = "wasm32")]
fn load_settings() -> Settings {
    Settings::default()
}

/// Guarda los ajustes en `settings.json`.
/// En la web no hay persistencia: no-op.
#[cfg(not(target_arch = "wasm32"))]
fn save_settings(settings: &Settings) {
    let Some(path) = settings_path() else {
        return;
    };
    if let Ok(json) = serde_json::to_string_pretty(settings) {
        if let Err(err) = fs::write(&path, json) {
            bevy::log::warn!("no se pudo guardar los ajustes: {err}");
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn save_settings(_settings: &Settings) {}

// ---- Helpers de UI ---------------------------------------------------------

/// Botón pequeño "−"/"+" con texto centrado.
fn spawn_small_button(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    marker: impl Bundle,
    font: &Handle<Font>,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(52.0),
                height: Val::Px(52.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.20, 0.38, 0.66)),
            BorderColor(Color::srgb(0.60, 0.80, 1.0)),
            BorderRadius::all(Val::Px(10.0)),
            Visibility::Visible,
            marker,
        ))
        .with_children(|button| {
            button.spawn(ui_text(label, 26.0, Color::WHITE, font));
        });
}

/// Botón grande "Volver".
fn spawn_big_button(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    marker: impl Bundle,
    font: &Handle<Font>,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(220.0),
                height: Val::Px(54.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.20, 0.38, 0.66)),
            BorderColor(Color::srgb(0.60, 0.80, 1.0)),
            BorderRadius::all(Val::Px(10.0)),
            Visibility::Visible,
            marker,
        ))
        .with_children(|button| {
            button.spawn(ui_text(label, 24.0, Color::WHITE, font));
        });
}

/// Texto con la fuente de interfaz.
fn ui_text(label: &str, size: f32, color: Color, font: &Handle<Font>) -> (Text, TextFont, TextColor) {
    (
        Text::new(tr(label)),
        TextFont {
            font: font.clone(),
            font_size: size,
            ..default()
        },
        TextColor(color),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_sane() {
        let settings = Settings::default();
        assert_eq!(settings.sensitivity, 5.0);
        assert_eq!(settings.volume, 80.0);
        assert_eq!(settings.language, Language::Es);
        assert_eq!(settings.sensitivity_multiplier(), 1.0);
        assert_eq!(settings.linear_volume(), 0.8);
    }

    #[test]
    fn values_are_clamped() {
        let settings = Settings {
            sensitivity: 10.0,
            volume: 150.0,
            language: Language::Es,
        };
        assert_eq!(settings.sensitivity_multiplier(), 2.0);
        assert_eq!(settings.linear_volume(), 1.0);

        let settings = Settings {
            sensitivity: 1.0,
            volume: -20.0,
            language: Language::Es,
        };
        assert_eq!(settings.sensitivity_multiplier(), 0.2);
        assert_eq!(settings.linear_volume(), 0.0);
    }
}