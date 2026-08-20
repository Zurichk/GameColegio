//! Zona de aprendizaje: centro con 4 secciones y sus juegos.
//!
//! - **Centro** (`menu`): hub con las secciones Lengua, Matemáticas,
//!   Ciencias y Memoria.
//! - **Lengua** (`reading`, `spelling`, `hangman`): leer y escribir,
//!   ortografía y ahorcado.
//! - **Matemáticas** (`math`, `mental`): operaciones y cálculo mental.
//! - **Ciencias** (`trivia`): ciencias naturales y geografía de España.
//! - **Memoria** (`memory`, `sequence`): emparejar tarjetas (letras,
//!   números, mixtas, formas o palabras) y repetir la secuencia de colores.
//!
//! Todos los estados usan overlays de UI de pantalla completa (como el modo
//! tablero), por lo que no interfieren con la exploración 3D: los sistemas
//! del mundo solo corren en `GameState::Playing`.

pub mod compare;
pub mod hangman;
pub mod math;
pub mod memory;
pub mod mental;
pub mod menu;
pub mod reading;
pub mod sequence;
pub mod spelling;
pub mod trivia;

use bevy::prelude::*;

use crate::i18n::tr;

/// Plugin de la zona de aprendizaje: agrupa los menús y todos los juegos.
pub struct LearningPlugin;

impl Plugin for LearningPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            menu::LearningMenuPlugin,
            reading::ReadingPlugin,
            spelling::SpellingPlugin,
            hangman::HangmanPlugin,
            math::MathPlugin,
            mental::MentalPlugin,
            compare::ComparePlugin,
            trivia::TriviaPlugin,
            memory::MemoryPlugin,
            sequence::SequencePlugin,
        ));
    }
}

/// Crea un texto con la fuente de interfaz (misma que el resto de la UI).
/// La etiqueta se traduce al idioma activo (si existe traducción).
pub fn ui_text(
    label: &str,
    size: f32,
    color: Color,
    font: &Handle<Font>,
) -> (Text, TextFont, TextColor) {
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

/// Crea un botón grande con texto centrado dentro de `parent`.
pub fn spawn_button(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    marker: impl Bundle,
    font: &Handle<Font>,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(320.0),
                height: Val::Px(52.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.20, 0.38, 0.66)),
            BorderColor(Color::srgb(0.60, 0.80, 1.0)),
            BorderRadius::all(Val::Px(10.0)),
            // Inherited: en Bevy 0.16 `Visible` se muestra aunque el padre esté
            // oculto (rompería los botones dentro de los paneles ocultos).
            Visibility::Inherited,
            marker,
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(tr(label)),
                TextFont {
                    font: font.clone(),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                // Centra las líneas del texto dentro de su propia caja (si el
                // texto se rompe en varias líneas, quedan centradas y no
                // alineadas a la izquierda como por defecto).
                TextLayout {
                    justify: JustifyText::Center,
                    ..default()
                },
                Node {
                    width: Val::Percent(100.0),
                    ..default()
                },
            ));
        });
}

/// Fondo oscuro semitransparente de las pantallas de aprendizaje.
pub fn screen_background() -> BackgroundColor {
    BackgroundColor(Color::srgba(0.02, 0.03, 0.08, 0.86))
}