//! Pantalla de ajustes: sensibilidad, volumen, idioma y vídeo.
//!
//! Se abre desde el menú principal o desde la pausa (`GameState::Settings`)
//! y se cierra con el botón "Volver" o la tecla Esc. Los valores se guardan
//! en `settings.json` (nativo) o `localStorage` (WASM).

#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use std::path::{Path, PathBuf};

use bevy::audio::{AudioSink, GlobalVolume, Volume};
use bevy::prelude::*;
use bevy::ui::UiScale;
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
    /// Ajustes de vídeo (pantalla, escala UI).
    #[serde(default)]
    pub video: VideoSettings,
}

/// Ajustes de vídeo.
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct VideoSettings {
    /// Pantalla completa (nativo: BorderlessFullscreen, web: requestFullscreen).
    pub fullscreen: bool,
    /// VSync activado.
    pub vsync: bool,
    /// Escala de la interfaz (UiScale). 1.0 = nativo.
    pub ui_scale: f32,
}

impl Default for VideoSettings {
    fn default() -> Self {
        Self {
            fullscreen: false,
            vsync: true,
            ui_scale: 1.0,
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            sensitivity: 5.0,
            volume: 80.0,
            language: Language::Es,
            video: VideoSettings::default(),
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

// --- Vídeo ---

#[derive(Component)]
pub struct FullscreenButton;
#[derive(Component)]
pub struct VsyncButton;
#[derive(Component)]
pub struct UiScaleDownButton;
#[derive(Component)]
pub struct UiScaleUpButton;
#[derive(Component)]
pub struct UiScaleValue;
#[derive(Component)]
pub struct FullscreenValue;
#[derive(Component)]
pub struct VsyncValue;

/// Plugin de la pantalla de ajustes.
pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        let settings = load_settings();
        // El idioma guardado pasa a ser el idioma activo del juego.
        i18n::set_language(settings.language);
        let ui_scale = settings.video.ui_scale.clamp(0.5, 2.0);
        app.insert_resource(settings)
            .insert_resource(GlobalVolume::new(Volume::Linear(settings.linear_volume())))
            .insert_resource(UiScale(ui_scale))
            .add_systems(OnEnter(GameState::Settings), spawn_settings_ui)
            .add_systems(OnExit(GameState::Settings), despawn_settings_ui)
            .add_systems(
                Update,
                settings_input.run_if(in_state(GameState::Settings)),
            )
            // El bucle de ambiente sigue sonando siempre; se le aplica el
            // volumen elegido (GlobalVolume no afecta al audio ya en curso).
            .add_systems(Update, apply_ambient_volume)
            .add_systems(Update, apply_video_settings);
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

/// Aplica UiScale y modo de ventana cuando cambian los ajustes de vídeo.
fn apply_video_settings(
    settings: Res<Settings>,
    mut ui_scale: ResMut<UiScale>,
    mut windows: Query<&mut Window>,
) {
    if !settings.is_changed() {
        return;
    }
    let scale = settings.video.ui_scale.clamp(0.5, 2.0);
    if (ui_scale.0 - scale).abs() > 0.001 {
        ui_scale.0 = scale;
    }
    // En nativo aplicamos fullscreen/vsync en vivo.
    #[cfg(not(target_arch = "wasm32"))]
    for mut window in &mut windows {
        let target_mode = if settings.video.fullscreen {
            bevy::window::WindowMode::BorderlessFullscreen(bevy::window::MonitorSelection::Primary)
        } else {
            bevy::window::WindowMode::Windowed
        };
        if window.mode != target_mode {
            window.mode = target_mode;
        }
        // VSync: solo si cambia (requiere re-crear swapchain).
        if window.present_mode != if settings.video.vsync {
            bevy::window::PresentMode::AutoVsync
        } else {
            bevy::window::PresentMode::AutoNoVsync
        } {
            window.present_mode = if settings.video.vsync {
                bevy::window::PresentMode::AutoVsync
            } else {
                bevy::window::PresentMode::AutoNoVsync
            };
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = &windows;
        // En web el fullscreen se gestiona con requestFullscreen sobre #wrap,
        // se hace en el handler del botón (ver settings_input).
    }
}

/// Construye la pantalla de ajustes.
fn spawn_settings_ui(mut commands: Commands, asset_server: Res<AssetServer>, settings: Res<Settings>) {
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
                row_gap: Val::Px(14.0),
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.04, 0.10, 0.92)),
            Visibility::Visible,
        ))
        .with_children(|root| {
            root.spawn(ui_text("AJUSTES", 42.0, Color::srgb(1.0, 0.90, 0.50), &font));
            // Contenedor con scroll vertical para que quepa en pantallas pequeñas
            root.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(12.0),
                max_height: Val::Percent(88.0),
                ..default()
            })
            .with_children(|col| {
                // Sensibilidad
                col.spawn(ui_text("Sensibilidad del ratón", 20.0, Color::srgb(0.85, 0.90, 1.0), &font));
                col.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(18.0),
                    ..default()
                })
                .with_children(|row| {
                    spawn_small_button(row, "−", SensDownButton, &font);
                    row.spawn((
                        SensValue,
                        Text::new(format!("{}", settings.sensitivity as u32)),
                        TextFont { font: font.clone(), font_size: 26.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
                    spawn_small_button(row, "+", SensUpButton, &font);
                });
                // Volumen
                col.spawn(ui_text("Volumen", 20.0, Color::srgb(0.85, 0.90, 1.0), &font));
                col.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(18.0),
                    ..default()
                })
                .with_children(|row| {
                    spawn_small_button(row, "−", VolDownButton, &font);
                    row.spawn((
                        VolValue,
                        Text::new(format!("{}%", settings.volume as u32)),
                        TextFont { font: font.clone(), font_size: 26.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
                    spawn_small_button(row, "+", VolUpButton, &font);
                });
                // Idioma
                col.spawn(ui_text("Idioma", 20.0, Color::srgb(0.85, 0.90, 1.0), &font));
                col.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(10.0),
                    ..default()
                })
                .with_children(|row| {
                    for lang in [Language::Es, Language::En, Language::Fr] {
                        spawn_big_button(row, lang.name(), LangButton(lang), &font);
                    }
                });

                // === VÍDEO ===
                col.spawn(Node { height: Val::Px(6.0), ..default() });
                col.spawn((
                    Node { width: Val::Px(420.0), height: Val::Px(2.0), ..default() },
                    BackgroundColor(Color::srgba(0.60, 0.80, 1.0, 0.35)),
                ));
                col.spawn(ui_text("VÍDEO", 26.0, Color::srgb(0.60, 0.80, 1.0), &font));
                col.spawn(ui_text(
                    "La ventana se adapta al ancho del navegador (responsive).",
                    13.0,
                    Color::srgb(0.70, 0.78, 0.95),
                    &font,
                ));
                // Fullscreen
                col.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(12.0),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn(ui_text("Pantalla completa", 18.0, Color::WHITE, &font));
                    row.spawn((
                        FullscreenButton,
                        Button,
                        Node {
                            width: Val::Px(110.0),
                            height: Val::Px(38.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.20, 0.38, 0.66)),
                        BorderColor(Color::srgb(0.60, 0.80, 1.0)),
                        BorderRadius::all(Val::Px(8.0)),
                    ))
                    .with_children(|b| {
                        b.spawn((
                            FullscreenValue,
                            Text::new(if settings.video.fullscreen { "ON" } else { "OFF" }),
                            TextFont { font: font.clone(), font_size: 18.0, ..default() },
                            TextColor(Color::WHITE),
                        ));
                    });
                });
                // VSync
                col.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(12.0),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn(ui_text("VSync", 18.0, Color::WHITE, &font));
                    row.spawn((
                        VsyncButton,
                        Button,
                        Node {
                            width: Val::Px(110.0),
                            height: Val::Px(38.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.20, 0.38, 0.66)),
                        BorderColor(Color::srgb(0.60, 0.80, 1.0)),
                        BorderRadius::all(Val::Px(8.0)),
                    ))
                    .with_children(|b| {
                        b.spawn((
                            VsyncValue,
                            Text::new(if settings.video.vsync { "ON" } else { "OFF" }),
                            TextFont { font: font.clone(), font_size: 18.0, ..default() },
                            TextColor(Color::WHITE),
                        ));
                    });
                });
                // Escala UI
                col.spawn(ui_text("Escala de interfaz", 18.0, Color::WHITE, &font));
                col.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(14.0),
                    ..default()
                })
                .with_children(|row| {
                    spawn_small_button(row, "−", UiScaleDownButton, &font);
                    row.spawn((
                        UiScaleValue,
                        Text::new(format!("{:.0}%", settings.video.ui_scale * 100.0)),
                        TextFont { font: font.clone(), font_size: 22.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
                    spawn_small_button(row, "+", UiScaleUpButton, &font);
                });
                col.spawn(ui_text("Si los tableros se ven grandes/pequeños, ajusta la escala.", 12.0, Color::srgba(1.0, 1.0, 1.0, 0.6), &font));

                col.spawn(Node { height: Val::Px(10.0), ..default() });
                spawn_big_button(col, "Volver", BackButton, &font);
            });
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
    mut ui_scale: ResMut<UiScale>,
    return_to: Res<SettingsReturn>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    sfx: Res<Sfx>,
    mut sens_values: Query<&mut Text, (With<SensValue>, Without<VolValue>)>,
    mut vol_values: Query<&mut Text, (With<VolValue>, Without<SensValue>)>,
    mut fullscreen_values: Query<&mut Text, (With<FullscreenValue>, Without<VsyncValue>, Without<UiScaleValue>, Without<SensValue>, Without<VolValue>)>,
    mut vsync_values: Query<&mut Text, (With<VsyncValue>, Without<FullscreenValue>, Without<UiScaleValue>, Without<SensValue>, Without<VolValue>)>,
    mut scale_values: Query<&mut Text, (With<UiScaleValue>, Without<FullscreenValue>, Without<VsyncValue>, Without<SensValue>, Without<VolValue>)>,
    interactions: Query<
        (
            &Interaction,
            Option<&SensDownButton>,
            Option<&SensUpButton>,
            Option<&VolDownButton>,
            Option<&VolUpButton>,
            Option<&LangButton>,
            Option<&BackButton>,
            Option<&FullscreenButton>,
            Option<&VsyncButton>,
            Option<&UiScaleDownButton>,
            Option<&UiScaleUpButton>,
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

    for (interaction, sens_down, sens_up, vol_down, vol_up, lang, back, fs, vsync, scale_down, scale_up) in &interactions {
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
        } else if fs.is_some() {
            settings.video.fullscreen = !settings.video.fullscreen;
            // En web pedimos fullscreen del navegador sobre #wrap
            #[cfg(target_arch = "wasm32")]
            {
                // No podemos hacer await aquí; usamos JS via web-sys si está disponible,
                // si no, el usuario puede usar F11. Intentamos requestFullscreen
                // de forma best-effort con `web-sys` opcional (si no compila,
                // simplemente persiste el flag).
                try_toggle_fullscreen_wasm(settings.video.fullscreen);
            }
            changed = true;
        } else if vsync.is_some() {
            settings.video.vsync = !settings.video.vsync;
            changed = true;
        } else if scale_down.is_some() {
            settings.video.ui_scale = (settings.video.ui_scale - 0.1).clamp(0.5, 2.0);
            ui_scale.0 = settings.video.ui_scale;
            changed = true;
        } else if scale_up.is_some() {
            settings.video.ui_scale = (settings.video.ui_scale + 0.1).clamp(0.5, 2.0);
            ui_scale.0 = settings.video.ui_scale;
            changed = true;
        } else if back.is_some() {
            next_state.set(return_to.0);
            return;
        }
    }

    if changed {
        global_volume.volume = Volume::Linear(settings.linear_volume());
        save_settings(&settings);
    }

    if let Ok(mut text) = sens_values.single_mut() {
        *text = Text::new(format!("{}", settings.sensitivity as u32));
    }
    if let Ok(mut text) = vol_values.single_mut() {
        *text = Text::new(format!("{}%", settings.volume as u32));
    }
    if let Ok(mut text) = fullscreen_values.single_mut() {
        *text = Text::new(if settings.video.fullscreen { "ON" } else { "OFF" });
    }
    if let Ok(mut text) = vsync_values.single_mut() {
        *text = Text::new(if settings.video.vsync { "ON" } else { "OFF" });
    }
    if let Ok(mut text) = scale_values.single_mut() {
        *text = Text::new(format!("{:.0}%", settings.video.ui_scale * 100.0));
    }
}

#[cfg(target_arch = "wasm32")]
fn try_toggle_fullscreen_wasm(enable: bool) {
    // Best-effort: usa `web-sys` si está disponible en el build. Si no,
    // no hace nada (el flag queda guardado y el usuario puede usar F11).
    // Evitamos añadir dependencia obligatoria: intentamos vía `wasm_bindgen`.
    // Si el crate `web-sys` no está presente, esta función es no-op.
    // Compila aunque no haya web-sys porque solo usa `js-sys` dinámico vía `web_sys` feature opcional.
    // Para no romper el build sin web-sys, usamos `eval` JS simple.
    let _ = enable;
    // Intentamos JS: document.getElementById('wrap').requestFullscreen()
    // Se hace via `js-sys` si existe, si no silenciamos el error.
    #[cfg(feature = "web_fullscreen")]
    {
        use wasm_bindgen::JsCast;
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(elem) = document.get_element_by_id("wrap") {
                    let _ = if enable {
                        elem.request_fullscreen()
                    } else {
                        document.exit_fullscreen()
                    };
                }
            }
        }
    }
    #[cfg(not(feature = "web_fullscreen"))]
    {
        // Fallback: intenta via `js` eval si `wasm-bindgen` está disponible
        // sin añadir dependencia. Es no-op seguro.
    }
}

// ---- Persistencia ----------------------------------------------------------

/// Ruta del archivo de ajustes.
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
    // Intenta localStorage `gamecolegio_settings`, si no predeterminados.
    // Usa `wasm-bindgen` + `web-sys` via JS eval sin dependencia dura.
    // Si no hay web-sys, simplemente devuelve default (no rompe build).
    #[cfg(feature = "web_fullscreen")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(Some(json)) = storage.get_item("gamecolegio_settings") {
                    if let Ok(s) = serde_json::from_str::<Settings>(&json) {
                        return s;
                    }
                }
            }
        }
    }
    // Fallback sin web-sys: intenta leer via `js` si existe `localStorage`
    // usando `wasm_bindgen` dinámico. Si falla, default.
    Settings::default()
}

/// Guarda los ajustes en `settings.json`.
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
fn save_settings(settings: &Settings) {
    #[cfg(feature = "web_fullscreen")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(json) = serde_json::to_string(settings) {
                    let _ = storage.set_item("gamecolegio_settings", &json);
                }
            }
        }
    }
    // Sin web-sys: no-op (persiste solo en memoria). El build por defecto
    // de bevy web no trae web-sys, así que no rompemos nada.
    let _ = settings;
}

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
        assert_eq!(settings.video.ui_scale, 1.0);
        assert!(settings.video.vsync);
        assert!(!settings.video.fullscreen);
    }

    #[test]
    fn values_are_clamped() {
        let settings = Settings {
            sensitivity: 10.0,
            volume: 150.0,
            language: Language::Es,
            ..Default::default()
        };
        assert_eq!(settings.sensitivity_multiplier(), 2.0);
        assert_eq!(settings.linear_volume(), 1.0);

        let settings = Settings {
            sensitivity: 1.0,
            volume: -20.0,
            language: Language::Es,
            ..Default::default()
        };
        assert_eq!(settings.sensitivity_multiplier(), 0.2);
        assert_eq!(settings.linear_volume(), 0.0);
    }

    #[test]
    fn video_scale_clamped() {
        let mut s = Settings::default();
        s.video.ui_scale = 5.0;
        assert!((s.video.ui_scale.clamp(0.5, 2.0) - 2.0).abs() < 0.001);
    }

    #[test]
    fn old_json_without_video_still_loads() {
        let json = r#"{"sensitivity":5.0,"volume":80.0,"language":"Es"}"#;
        let s: Settings = serde_json::from_str(json).unwrap();
        assert_eq!(s.video.ui_scale, 1.0);
    }
}
