//! Menús de la zona de aprendizaje.
//!
//! - **Zona de aprendizaje** (`GameState::LearningMenu`): centro con 4
//!   secciones: Lengua, Matemáticas, Ciencias y Memoria.
//! - **Lengua** (`GameState::LanguageMenu`): Leer y escribir, Ortografía y
//!   Ahorcado.
//! - **Matemáticas** (`GameState::MathMenu`): Sumar, Restar, Multiplicar,
//!   Dividir y Cálculo mental.
//! - **Ciencias** (`GameState::ScienceMenu`): Ciencias naturales y Geografía
//!   de España.
//! - **Juegos de memoria** (`GameState::MemoryMenu`): Parejas de letras,
//!   números, mixtas, formas o palabras, y memoria de secuencia.

use bevy::prelude::*;

use super::math::MathOperation;
use super::memory::{MemoryConfig, MemoryKind};
use super::trivia::TriviaKind;
use super::{screen_background, spawn_button, ui_text};
use crate::game::GameState;

// ---- Componentes: centro de la zona de aprendizaje --------------------------

/// Raíz del centro "Zona de aprendizaje".
#[derive(Component)]
pub struct LearningMenuUi;

/// Botón de la sección de Lengua.
#[derive(Component)]
pub struct LanguageSectionButton;

/// Botón de la sección de Matemáticas.
#[derive(Component)]
pub struct MathSectionButton;

/// Botón de la sección de Ciencias.
#[derive(Component)]
pub struct ScienceSectionButton;

/// Botón de la sección de Memoria.
#[derive(Component)]
pub struct MemorySectionButton;

/// Botón de volver al menú principal.
#[derive(Component)]
pub struct LearningBackButton;

// ---- Componentes: sección Lengua -------------------------------------------

/// Raíz del menú "Lengua".
#[derive(Component)]
pub struct LanguageMenuUi;

/// Botón de empezar la práctica de leer y escribir.
#[derive(Component)]
pub struct ReadingButton;

/// Botón de práctica de ortografía.
#[derive(Component)]
pub struct SpellingButton;

/// Botón de jugar al ahorcado.
#[derive(Component)]
pub struct HangmanButton;

/// Botón de sinónimos.
#[derive(Component)]
pub struct SynonymsButton;

/// Botón de anagramas.
#[derive(Component)]
pub struct AnagramButton;

/// Botón de volver a la zona de aprendizaje.
#[derive(Component)]
pub struct LanguageBackButton;

// ---- Componentes: sección Matemáticas --------------------------------------

/// Raíz del menú "Matemáticas".
#[derive(Component)]
pub struct MathMenuUi;

/// Botón de práctica de sumar.
#[derive(Component)]
pub struct AddButton;

/// Botón de práctica de restar.
#[derive(Component)]
pub struct SubButton;

/// Botón de práctica de multiplicar.
#[derive(Component)]
pub struct MulButton;

/// Botón de práctica de dividir.
#[derive(Component)]
pub struct DivButton;

/// Botón de práctica de cálculo mental.
#[derive(Component)]
pub struct MentalButton;

/// Botón del juego "Mayor, menor o igual".
#[derive(Component)]
pub struct CompareButton;

/// Botón de fracciones.
#[derive(Component)]
pub struct FractionsButton;

/// Botón de geometría.
#[derive(Component)]
pub struct GeometryButton;

/// Botón de problemas.
#[derive(Component)]
pub struct WordProblemsButton;

/// Botón de volver a la zona de aprendizaje.
#[derive(Component)]
pub struct MathBackButton;

// ---- Componentes: sección Ciencias -----------------------------------------

/// Raíz del menú "Ciencias".
#[derive(Component)]
pub struct ScienceMenuUi;

/// Botón de cuestionario de ciencias naturales.
#[derive(Component)]
pub struct ScienceButton;

/// Botón de cuestionario de geografía de España.
#[derive(Component)]
pub struct GeographyButton;

/// Botón de cuerpo humano.
#[derive(Component)]
pub struct HumanBodyButton;

/// Botón de universo.
#[derive(Component)]
pub struct SpaceButton;

/// Botón de volver a la zona de aprendizaje.
#[derive(Component)]
pub struct ScienceBackButton;

// ---- Componentes: Juegos de memoria ----------------------------------------

/// Raíz del menú "Juegos de memoria".
#[derive(Component)]
pub struct MemoryMenuUi;

/// Botón de parejas de letras.
#[derive(Component)]
pub struct MemoryLettersButton;

/// Botón de parejas de números.
#[derive(Component)]
pub struct MemoryNumbersButton;

/// Botón de parejas mixtas (letras y números).
#[derive(Component)]
pub struct MemoryMixedButton;

/// Botón de parejas de formas geométricas.
#[derive(Component)]
pub struct MemoryShapesButton;

/// Botón de parejas de palabras para leer.
#[derive(Component)]
pub struct MemoryWordsButton;

/// Botón de parejas de colores.
#[derive(Component)]
pub struct MemoryColorsButton;

/// Botón de parejas de banderas.
#[derive(Component)]
pub struct MemoryFlagsButton;

/// Botón de jugar a la memoria de secuencia (repite la secuencia de colores).
#[derive(Component)]
pub struct MemorySequenceButton;

/// Botón de volver a la zona de aprendizaje.
#[derive(Component)]
pub struct MemoryBackButton;

/// Plugin de los menús de la zona de aprendizaje.
pub struct LearningMenuPlugin;

impl Plugin for LearningMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::LearningMenu), spawn_learning_menu)
            .add_systems(OnExit(GameState::LearningMenu), despawn_learning_menu)
            .add_systems(
                Update,
                learning_menu_input.run_if(in_state(GameState::LearningMenu)),
            )
            .add_systems(OnEnter(GameState::LanguageMenu), spawn_language_menu)
            .add_systems(OnExit(GameState::LanguageMenu), despawn_language_menu)
            .add_systems(
                Update,
                language_menu_input.run_if(in_state(GameState::LanguageMenu)),
            )
            .add_systems(OnEnter(GameState::MathMenu), spawn_math_menu)
            .add_systems(OnExit(GameState::MathMenu), despawn_math_menu)
            .add_systems(
                Update,
                math_menu_input.run_if(in_state(GameState::MathMenu)),
            )
            .add_systems(OnEnter(GameState::ScienceMenu), spawn_science_menu)
            .add_systems(OnExit(GameState::ScienceMenu), despawn_science_menu)
            .add_systems(
                Update,
                science_menu_input.run_if(in_state(GameState::ScienceMenu)),
            )
            .add_systems(OnEnter(GameState::MemoryMenu), spawn_memory_menu)
            .add_systems(OnExit(GameState::MemoryMenu), despawn_memory_menu)
            .add_systems(
                Update,
                memory_menu_input.run_if(in_state(GameState::MemoryMenu)),
            );
    }
}

/// Construye el centro "Zona de aprendizaje".
fn spawn_learning_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands
        .spawn((
            LearningMenuUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(12.0),
                ..default()
            },
            screen_background(),
            Visibility::Visible,
        ))
        .with_children(|root| {
            root.spawn(ui_text(
                "ZONA DE APRENDIZAJE",
                56.0,
                Color::srgb(1.0, 0.90, 0.50),
                &font,
            ));
            root.spawn(ui_text(
                "Elige una sección",
                24.0,
                Color::srgb(0.85, 0.90, 1.0),
                &font,
            ));
            root.spawn(Node {
                height: Val::Px(14.0),
                ..default()
            });
            spawn_button(root, "Lengua", LanguageSectionButton, &font);
            spawn_button(root, "Matemáticas", MathSectionButton, &font);
            spawn_button(root, "Ciencias", ScienceSectionButton, &font);
            spawn_button(root, "Memoria", MemorySectionButton, &font);
            root.spawn(Node {
                height: Val::Px(14.0),
                ..default()
            });
            spawn_button(root, "Volver al menú principal", LearningBackButton, &font);
        });
}

/// Destruye el centro "Zona de aprendizaje".
fn despawn_learning_menu(mut commands: Commands, roots: Query<Entity, With<LearningMenuUi>>) {
    for root in &roots {
        commands.entity(root).despawn();
    }
}

/// Procesa los clics del centro de aprendizaje (y la tecla Escape).
fn learning_menu_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    interactions: Query<
        (
            &Interaction,
            Option<&LanguageSectionButton>,
            Option<&MathSectionButton>,
            Option<&ScienceSectionButton>,
            Option<&MemorySectionButton>,
            Option<&LearningBackButton>,
        ),
        Changed<Interaction>,
    >,
) {
    if keys.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::MainMenu);
        return;
    }
    for (interaction, language, math, science, memory, back) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if language.is_some() {
            next_state.set(GameState::LanguageMenu);
        } else if math.is_some() {
            next_state.set(GameState::MathMenu);
        } else if science.is_some() {
            next_state.set(GameState::ScienceMenu);
        } else if memory.is_some() {
            next_state.set(GameState::MemoryMenu);
        } else if back.is_some() {
            next_state.set(GameState::MainMenu);
        }
    }
}

/// Construye el menú "Lengua".
fn spawn_language_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands
        .spawn((
            LanguageMenuUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(12.0),
                ..default()
            },
            screen_background(),
            Visibility::Visible,
        ))
        .with_children(|root| {
            root.spawn(ui_text(
                "LENGUA",
                56.0,
                Color::srgb(0.80, 0.75, 1.0),
                &font,
            ));
            root.spawn(ui_text(
                "Leer, escribir y jugar con las palabras",
                24.0,
                Color::srgb(0.85, 0.90, 1.0),
                &font,
            ));
            root.spawn(Node {
                height: Val::Px(14.0),
                ..default()
            });
            spawn_button(root, "Leer y escribir", ReadingButton, &font);
            spawn_button(root, "Ortografía", SpellingButton, &font);
            spawn_button(root, "Ahorcado", HangmanButton, &font);
            spawn_button(root, "Sinónimos", SynonymsButton, &font);
            spawn_button(root, "Anagramas", AnagramButton, &font);
            root.spawn(Node {
                height: Val::Px(14.0),
                ..default()
            });
            spawn_button(root, "Volver a la zona de aprendizaje", LanguageBackButton, &font);
        });
}

/// Destruye el menú "Lengua".
fn despawn_language_menu(mut commands: Commands, roots: Query<Entity, With<LanguageMenuUi>>) {
    for root in &roots {
        commands.entity(root).despawn();
    }
}

/// Procesa los clics del menú "Lengua" (y la tecla Escape).
fn language_menu_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    interactions: Query<
        (
            &Interaction,
            Option<&ReadingButton>,
            Option<&SpellingButton>,
            Option<&HangmanButton>,
            Option<&SynonymsButton>,
            Option<&AnagramButton>,
            Option<&LanguageBackButton>,
        ),
        Changed<Interaction>,
    >,
) {
    if keys.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::LearningMenu);
        return;
    }
    for (interaction, reading, spelling, hangman, synonyms, anagram, back) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if reading.is_some() {
            next_state.set(GameState::ReadingPractice);
        } else if spelling.is_some() {
            next_state.set(GameState::SpellingPractice);
        } else if hangman.is_some() {
            next_state.set(GameState::HangmanGame);
        } else if synonyms.is_some() {
            next_state.set(GameState::SynonymsPractice);
        } else if anagram.is_some() {
            next_state.set(GameState::AnagramPractice);
        } else if back.is_some() {
            next_state.set(GameState::LearningMenu);
        }
    }
}

/// Construye el menú "Matemáticas".
fn spawn_math_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands
        .spawn((
            MathMenuUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(12.0),
                ..default()
            },
            screen_background(),
            Visibility::Visible,
        ))
        .with_children(|root| {
            root.spawn(ui_text(
                "MATEMÁTICAS",
                56.0,
                Color::srgb(1.0, 0.90, 0.50),
                &font,
            ));
            root.spawn(ui_text(
                "Operaciones y cálculo mental",
                24.0,
                Color::srgb(0.85, 0.90, 1.0),
                &font,
            ));
            root.spawn(Node {
                height: Val::Px(14.0),
                ..default()
            });
            spawn_button(root, "Sumar", AddButton, &font);
            spawn_button(root, "Restar", SubButton, &font);
            spawn_button(root, "Multiplicar", MulButton, &font);
            spawn_button(root, "Dividir", DivButton, &font);
            spawn_button(root, "Cálculo mental", MentalButton, &font);
            spawn_button(root, "Mayor, menor o igual", CompareButton, &font);
            spawn_button(root, "Fracciones", FractionsButton, &font);
            spawn_button(root, "Geometría", GeometryButton, &font);
            spawn_button(root, "Problemas", WordProblemsButton, &font);
            root.spawn(Node {
                height: Val::Px(14.0),
                ..default()
            });
            spawn_button(root, "Volver a la zona de aprendizaje", MathBackButton, &font);
        });
}

/// Destruye el menú "Matemáticas".
fn despawn_math_menu(mut commands: Commands, roots: Query<Entity, With<MathMenuUi>>) {
    for root in &roots {
        commands.entity(root).despawn();
    }
}

/// Procesa los clics del menú "Matemáticas" (y la tecla Escape).
fn math_menu_input(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    interactions: Query<
        (
            &Interaction,
            Option<&AddButton>,
            Option<&SubButton>,
            Option<&MulButton>,
            Option<&DivButton>,
            Option<&MentalButton>,
            Option<&CompareButton>,
            Option<&FractionsButton>,
            Option<&GeometryButton>,
            Option<&WordProblemsButton>,
            Option<&MathBackButton>,
        ),
        Changed<Interaction>,
    >,
) {
    if keys.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::LearningMenu);
        return;
    }
    for (interaction, add, sub, mul, div, mental, compare, fractions, geometry, problems, back) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if add.is_some() {
            commands.insert_resource(MathOperation::Add);
            next_state.set(GameState::MathPractice);
        } else if sub.is_some() {
            commands.insert_resource(MathOperation::Sub);
            next_state.set(GameState::MathPractice);
        } else if mul.is_some() {
            commands.insert_resource(MathOperation::Mul);
            next_state.set(GameState::MathPractice);
        } else if div.is_some() {
            commands.insert_resource(MathOperation::Div);
            next_state.set(GameState::MathPractice);
        } else if mental.is_some() {
            next_state.set(GameState::MentalPractice);
        } else if compare.is_some() {
            next_state.set(GameState::ComparePractice);
        } else if fractions.is_some() {
            next_state.set(GameState::FractionsPractice);
        } else if geometry.is_some() {
            next_state.set(GameState::GeometryPractice);
        } else if problems.is_some() {
            next_state.set(GameState::WordProblemsPractice);
        } else if back.is_some() {
            next_state.set(GameState::LearningMenu);
        }
    }
}

/// Construye el menú "Ciencias".
fn spawn_science_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands
        .spawn((
            ScienceMenuUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(12.0),
                ..default()
            },
            screen_background(),
            Visibility::Visible,
        ))
        .with_children(|root| {
            root.spawn(ui_text(
                "CIENCIAS",
                56.0,
                Color::srgb(0.70, 0.95, 1.0),
                &font,
            ));
            root.spawn(ui_text(
                "Naturaleza, cuerpo humano y geografía",
                24.0,
                Color::srgb(0.85, 0.90, 1.0),
                &font,
            ));
            root.spawn(Node {
                height: Val::Px(14.0),
                ..default()
            });
            spawn_button(root, "Ciencias naturales", ScienceButton, &font);
            spawn_button(root, "Cuerpo humano", HumanBodyButton, &font);
            spawn_button(root, "Universo", SpaceButton, &font);
            spawn_button(root, "Geografía de España", GeographyButton, &font);
            root.spawn(Node {
                height: Val::Px(14.0),
                ..default()
            });
            spawn_button(root, "Volver a la zona de aprendizaje", ScienceBackButton, &font);
        });
}

/// Destruye el menú "Ciencias".
fn despawn_science_menu(mut commands: Commands, roots: Query<Entity, With<ScienceMenuUi>>) {
    for root in &roots {
        commands.entity(root).despawn();
    }
}

/// Procesa los clics del menú "Ciencias" (y la tecla Escape).
fn science_menu_input(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    interactions: Query<
        (
            &Interaction,
            Option<&ScienceButton>,
            Option<&GeographyButton>,
            Option<&HumanBodyButton>,
            Option<&SpaceButton>,
            Option<&ScienceBackButton>,
        ),
        Changed<Interaction>,
    >,
) {
    if keys.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::LearningMenu);
        return;
    }
    for (interaction, science, geography, human, space, back) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if science.is_some() {
            commands.insert_resource(TriviaKind::Science);
            next_state.set(GameState::SciencePractice);
        } else if geography.is_some() {
            commands.insert_resource(TriviaKind::Geography);
            next_state.set(GameState::GeographyPractice);
        } else if human.is_some() {
            commands.insert_resource(TriviaKind::HumanBody);
            next_state.set(GameState::SciencePractice);
        } else if space.is_some() {
            commands.insert_resource(TriviaKind::Space);
            next_state.set(GameState::SciencePractice);
        } else if back.is_some() {
            next_state.set(GameState::LearningMenu);
        }
    }
}

/// Construye el menú "Juegos de memoria".
fn spawn_memory_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands
        .spawn((
            MemoryMenuUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(12.0),
                ..default()
            },
            screen_background(),
            Visibility::Visible,
        ))
        .with_children(|root| {
            root.spawn(ui_text(
                "JUEGOS DE MEMORIA",
                56.0,
                Color::srgb(0.95, 0.75, 1.0),
                &font,
            ));
            root.spawn(ui_text(
                "Encuentra las parejas iguales",
                24.0,
                Color::srgb(0.85, 0.90, 1.0),
                &font,
            ));
            root.spawn(Node {
                height: Val::Px(14.0),
                ..default()
            });
            spawn_button(root, "Parejas de letras (6)", MemoryLettersButton, &font);
            spawn_button(root, "Parejas de números (8)", MemoryNumbersButton, &font);
            spawn_button(root, "Parejas mixtas (10)", MemoryMixedButton, &font);
            spawn_button(root, "Parejas de formas (8)", MemoryShapesButton, &font);
            spawn_button(root, "Parejas de palabras (6)", MemoryWordsButton, &font);
            spawn_button(root, "Parejas de colores (6)", MemoryColorsButton, &font);
            spawn_button(root, "Parejas de banderas (8)", MemoryFlagsButton, &font);
            spawn_button(root, "Memoria de secuencia", MemorySequenceButton, &font);
            root.spawn(Node {
                height: Val::Px(14.0),
                ..default()
            });
            spawn_button(root, "Volver a la zona de aprendizaje", MemoryBackButton, &font);
        });
}

/// Destruye el menú "Juegos de memoria".
fn despawn_memory_menu(mut commands: Commands, roots: Query<Entity, With<MemoryMenuUi>>) {
    for root in &roots {
        commands.entity(root).despawn();
    }
}

/// Procesa los clics del menú de memoria (y la tecla Escape).
fn memory_menu_input(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    interactions: Query<
        (
            &Interaction,
            Option<&MemoryLettersButton>,
            Option<&MemoryNumbersButton>,
            Option<&MemoryMixedButton>,
            Option<&MemoryShapesButton>,
            Option<&MemoryWordsButton>,
            Option<&MemoryColorsButton>,
            Option<&MemoryFlagsButton>,
            Option<&MemorySequenceButton>,
            Option<&MemoryBackButton>,
        ),
        Changed<Interaction>,
    >,
) {
    if keys.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::LearningMenu);
        return;
    }
    for (
        interaction,
        letters,
        numbers,
        mixed,
        shapes,
        words,
        colors,
        flags,
        sequence,
        back,
    ) in &interactions
    {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if letters.is_some() {
            commands.insert_resource(MemoryConfig {
                kind: MemoryKind::Letters,
                pairs: 6,
            });
            next_state.set(GameState::MemoryGame);
        } else if numbers.is_some() {
            commands.insert_resource(MemoryConfig {
                kind: MemoryKind::Numbers,
                pairs: 8,
            });
            next_state.set(GameState::MemoryGame);
        } else if mixed.is_some() {
            commands.insert_resource(MemoryConfig {
                kind: MemoryKind::Mixed,
                pairs: 10,
            });
            next_state.set(GameState::MemoryGame);
        } else if shapes.is_some() {
            commands.insert_resource(MemoryConfig {
                kind: MemoryKind::Shapes,
                pairs: 8,
            });
            next_state.set(GameState::MemoryGame);
        } else if words.is_some() {
            commands.insert_resource(MemoryConfig {
                kind: MemoryKind::Words,
                pairs: 6,
            });
            next_state.set(GameState::MemoryGame);
        } else if colors.is_some() {
            commands.insert_resource(MemoryConfig {
                kind: MemoryKind::Colors,
                pairs: 6,
            });
            next_state.set(GameState::MemoryGame);
        } else if flags.is_some() {
            commands.insert_resource(MemoryConfig {
                kind: MemoryKind::Flags,
                pairs: 8,
            });
            next_state.set(GameState::MemoryGame);
        } else if sequence.is_some() {
            next_state.set(GameState::MemorySequence);
        } else if back.is_some() {
            next_state.set(GameState::LearningMenu);
        }
    }
}