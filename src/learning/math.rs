//! Práctica de sumar, restar, multiplicar y dividir (sección Matemáticas).
//!
//! Se elige la operación en el menú de Matemáticas y se genera una sesión
//! de 10 operaciones con dificultad creciente: números pequeños al principio
//! y más grandes al final. Cada respuesta da un feedback de 1,4 s y al final
//! se muestra el marcador con la opción de volver.

use bevy::prelude::*;
use rand::Rng;
use rand::seq::SliceRandom;

use super::{screen_background, spawn_button};
use crate::audio::{play_click, play_success, Sfx};
use crate::game::GameState;
use crate::i18n::tr;

/// Número de operaciones por sesión.
const ROUNDS: usize = 10;
/// Duración (s) del feedback.
const FEEDBACK_SECONDS: f32 = 1.4;

/// Operación que se va a practicar.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MathOperation {
    Add,
    Sub,
    Mul,
    Div,
}

impl MathOperation {
    /// Nombre legible de la operación.
    pub fn title(self) -> &'static str {
        match self {
            MathOperation::Add => "Sumar",
            MathOperation::Sub => "Restar",
            MathOperation::Mul => "Multiplicar",
            MathOperation::Div => "Dividir",
        }
    }

    /// Símbolo de la operación.
    pub fn symbol(self) -> &'static str {
        match self {
            MathOperation::Add => "+",
            MathOperation::Sub => "−",
            MathOperation::Mul => "×",
            MathOperation::Div => "÷",
        }
    }
}

/// Una operación de la sesión, con sus 4 opciones y la correcta.
struct MathRound {
    text: String,
    options: [String; 4],
    correct: usize,
}

/// Sesión de práctica de operaciones activa.
#[derive(Resource)]
pub struct MathSession {
    operation: MathOperation,
    rounds: Vec<MathRound>,
    index: usize,
    correct: usize,
    wrong: usize,
    selected: Option<usize>,
    feedback: bool,
    feedback_timer: f32,
    done: bool,
}

// ---- Componentes de la UI --------------------------------------------------

/// Raíz de la pantalla de operaciones.
#[derive(Component)]
pub struct MathUiRoot;

/// Campo de texto etiquetado por su función.
#[derive(Component)]
pub struct MathText(MathField);

#[derive(Clone, Copy, PartialEq, Eq)]
enum MathField {
    Title,
    Question,
    Progress,
    Feedback,
    ResultTitle,
    ResultDetail,
}

/// Texto de una opción (hijo de su botón).
#[derive(Component)]
pub struct MathOptionText(pub usize);

/// Botón de una opción (A/B/C/D).
#[derive(Component)]
pub struct MathOptionButton(pub usize);

/// Contenedor de resultados (oculto hasta terminar).
#[derive(Component)]
pub struct MathResultBox;

/// Botón de volver al menú de Matemáticas.
#[derive(Component)]
pub struct MathBackButton;

/// Plugin de la práctica de operaciones.
pub struct MathPlugin;

impl Plugin for MathPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MathPractice), spawn_math_ui)
            .add_systems(OnExit(GameState::MathPractice), cleanup_math)
            .add_systems(
                Update,
                update_math.run_if(in_state(GameState::MathPractice)),
            );
    }
}

/// Letras de las opciones.
const OPTION_LETTERS: [char; 4] = ['A', 'B', 'C', 'D'];

/// Crea un texto del campo indicado.
fn math_text(
    parent: &mut ChildSpawnerCommands,
    field: MathField,
    text: &str,
    size: f32,
    font: &Handle<Font>,
) {
    parent.spawn((
        MathText(field),
        Text::new(text.to_string()),
        TextFont {
            font: font.clone(),
            font_size: size,
            ..default()
        },
        TextColor(Color::WHITE),
        TextLayout {
            linebreak: LineBreak::WordBoundary,
            ..default()
        },
        Node {
            max_width: Val::Px(700.0),
            ..default()
        },
    ));
}

/// Crea el texto de una opción (hijo de su botón).
fn math_option_text(
    parent: &mut ChildSpawnerCommands,
    index: usize,
    size: f32,
    font: &Handle<Font>,
) {
    parent.spawn((
        MathOptionText(index),
        Text::new(String::new()),
        TextFont {
            font: font.clone(),
            font_size: size,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

/// Construye la pantalla de operaciones.
fn spawn_math_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    operation: Option<Res<MathOperation>>,
) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    let operation = operation.map(|o| *o).unwrap_or(MathOperation::Add);
    commands.insert_resource(MathSession {
        operation,
        rounds: build_rounds(operation),
        index: 0,
        correct: 0,
        wrong: 0,
        selected: None,
        feedback: false,
        feedback_timer: 0.0,
        done: false,
    });
    commands
        .spawn((
            MathUiRoot,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            screen_background(),
            Visibility::Visible,
            ZIndex(30),
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        width: Val::Px(640.0),
                        padding: UiRect::axes(Val::Px(28.0), Val::Px(24.0)),
                        row_gap: Val::Px(14.0),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.07, 0.09, 0.18, 0.96)),
                    BorderRadius::all(Val::Px(16.0)),
                ))
                .with_children(|panel| {
                    math_text(panel, MathField::Title, "", 28.0, &font);
                    math_text(panel, MathField::Question, "", 40.0, &font);

                    // Opciones A/B/C/D.
                    for index in 0..4 {
                        panel
                            .spawn((
                                Button,
                                MathOptionButton(index),
                                Node {
                                    width: Val::Px(560.0),
                                    height: Val::Px(46.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border: UiRect::all(Val::Px(2.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.15, 0.18, 0.28)),
                                BorderColor(Color::srgb(0.50, 0.55, 0.70)),
                                BorderRadius::all(Val::Px(8.0)),
                            ))
                            .with_children(|option| {
                                math_option_text(option, index, 22.0, &font);
                            });
                    }

                    math_text(panel, MathField::Progress, "", 17.0, &font);
                    math_text(panel, MathField::Feedback, "", 22.0, &font);

                    // Resultados (ocultos hasta terminar).
                    panel
                        .spawn((
                            MathResultBox,
                            Node {
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                row_gap: Val::Px(10.0),
                                ..default()
                            },
                            Visibility::Hidden,
                        ))
                        .with_children(|results| {
                            math_text(results, MathField::ResultTitle, "", 26.0, &font);
                            math_text(results, MathField::ResultDetail, "", 20.0, &font);
                            spawn_button(results, "Volver a Matemáticas", MathBackButton, &font);
                        });
                });
        });
}

/// Destruye la pantalla y la sesión al salir.
fn cleanup_math(mut commands: Commands, roots: Query<Entity, With<MathUiRoot>>) {
    for root in &roots {
        commands.entity(root).despawn();
    }
    commands.remove_resource::<MathSession>();
}

/// Genera las 10 operaciones de la sesión (dificultad creciente).
fn build_rounds(operation: MathOperation) -> Vec<MathRound> {
    let mut rng = rand::thread_rng();
    (0..ROUNDS).map(|i| make_round(operation, i, &mut rng)).collect()
}

/// Crea una operación con sus 4 opciones (1 correcta + 3 cercanas).
fn make_round(operation: MathOperation, index: usize, rng: &mut impl Rng) -> MathRound {
    let step = index / 3; // 0, 1 o 2 → dificultad creciente.
    let (text, answer) = match operation {
        MathOperation::Add => {
            let limit = 10 + step * 10;
            let a = rng.gen_range(1..=limit);
            let b = rng.gen_range(1..=limit);
            (format!("{a} {} {b}", operation.symbol()), a + b)
        }
        MathOperation::Sub => {
            let max = 20 + step * 10;
            let a = rng.gen_range(2..=max);
            let b = rng.gen_range(1..a);
            (format!("{a} {} {b}", operation.symbol()), a - b)
        }
        MathOperation::Mul => {
            let limit = if step == 0 { 5 } else { 9 };
            let a = rng.gen_range(2..=limit);
            let b = rng.gen_range(2..=limit);
            (format!("{a} {} {b}", operation.symbol()), a * b)
        }
        MathOperation::Div => {
            let limit = if step == 0 { 5 } else { 9 };
            let b = rng.gen_range(2..=limit);
            let q = rng.gen_range(2..=limit);
            (format!("{} {} {b}", b * q, operation.symbol()), q)
        }
    };

    // Distractores: valores cercanos a la respuesta, siempre positivos.
    let offsets = [1, -1, 2, -2, 3, -3, 5, -5, 10, -10];
    let mut candidates: Vec<i32> = offsets
        .iter()
        .map(|o| answer as i32 + o)
        .filter(|v| *v >= 0 && *v != answer as i32)
        .collect();
    candidates.shuffle(rng);
    candidates.truncate(3);

    let mut options = vec![answer.to_string()];
    options.extend(candidates.into_iter().map(|v| v.to_string()));
    options.shuffle(rng);
    let correct = options.iter().position(|o| o == &answer.to_string()).unwrap_or(0);

    MathRound {
        text,
        options: options.try_into().unwrap_or_else(|_| {
            [String::new(), String::new(), String::new(), String::new()]
        }),
        correct,
    }
}

/// Colores de los botones de opción.
const OPTION_NEUTRAL: Color = Color::srgb(0.15, 0.18, 0.28);
const OPTION_DIM: Color = Color::srgb(0.10, 0.12, 0.20);
const OPTION_CORRECT: Color = Color::srgb(0.15, 0.42, 0.25);
const OPTION_WRONG: Color = Color::srgb(0.50, 0.20, 0.20);

/// Gestiona la sesión de operaciones: respuesta, feedback y resultados.
fn update_math(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut commands: Commands,
    sfx: Res<Sfx>,
    session: Option<ResMut<MathSession>>,
    mut texts: Query<
        (&MathText, &mut Text, &mut TextColor, &mut Visibility),
        (
            Without<MathOptionText>,
            Without<MathOptionButton>,
            Without<MathResultBox>,
        ),
    >,
    mut option_texts: Query<
        (&MathOptionText, &mut Text),
        (
            Without<MathText>,
            Without<MathOptionButton>,
            Without<MathResultBox>,
        ),
    >,
    mut option_colors: Query<(&MathOptionButton, &mut BackgroundColor), Without<MathText>>,
    option_clicks: Query<
        (&Interaction, &MathOptionButton),
        (Changed<Interaction>, Without<MathText>),
    >,
    mut result_box: Query<
        &mut Visibility,
        (
            With<MathResultBox>,
            Without<MathText>,
            Without<MathOptionButton>,
        ),
    >,
    close_clicks: Query<&Interaction, (Changed<Interaction>, With<MathBackButton>)>,
    operation: Option<Res<MathOperation>>,
) {
    let dt = time.delta_secs();
    let mut session = match session {
        Some(session) => session,
        None => {
            let operation = operation.map(|o| *o).unwrap_or(MathOperation::Add);
            commands.insert_resource(MathSession {
                operation,
                rounds: build_rounds(operation),
                index: 0,
                correct: 0,
                wrong: 0,
                selected: None,
                feedback: false,
                feedback_timer: 0.0,
                done: false,
            });
            return;
        }
    };

    // Escape: volver al menú de Matemáticas.
    if keys.just_pressed(KeyCode::Escape) {
        commands.set_state(GameState::MathMenu);
        return;
    }

    // 1) Resultados.
    if session.done {
        let close = close_clicks.single().map_or(false, |i| *i == Interaction::Pressed)
            || keys.just_pressed(KeyCode::Enter)
            || keys.just_pressed(KeyCode::KeyQ);
        if close {
            commands.set_state(GameState::MathMenu);
            return;
        }
        for (field, mut text, mut color, mut vis) in &mut texts {
            match field.0 {
                MathField::ResultTitle => {
                    *text = Text::new(tr(if session.correct >= ROUNDS / 2 { "¡Muy bien!" } else { "¡Sigue practicando!" }));
                    *color = TextColor(if session.correct >= ROUNDS / 2 {
                        Color::srgb(0.40, 0.90, 0.50)
                    } else {
                        Color::srgb(0.95, 0.55, 0.30)
                    });
                    *vis = Visibility::Visible;
                }
                MathField::ResultDetail => {
                    *text = Text::new(tr("Aciertos: {} · Fallos: {}  de {} operaciones").replace("{}", &session.correct.to_string()).replace("{}", &session.wrong.to_string()).replace("{}", &ROUNDS.to_string()));
                    *vis = Visibility::Visible;
                }
                _ => {}
            }
        }
        if let Ok(mut vis) = result_box.single_mut() {
            *vis = Visibility::Visible;
        }
        return;
    }

    // 2) Feedback.
    if session.feedback {
        session.feedback_timer -= dt;
        for (button, mut bg) in &mut option_colors {
            let question = &session.rounds[session.index];
            *bg = BackgroundColor(if button.0 == question.correct {
                OPTION_CORRECT
            } else if Some(button.0) == session.selected {
                OPTION_WRONG
            } else {
                OPTION_DIM
            });
        }
        if session.feedback_timer <= 0.0 {
            session.feedback = false;
            session.selected = None;
            session.index += 1;
            if session.index >= ROUNDS {
                session.done = true;
                return;
            }
        }
    } else {
        // 3) Responder: clic en una opción o teclas 1-4.
        let mut chosen: Option<usize> = None;
        for (interaction, button) in &option_clicks {
            if *interaction == Interaction::Pressed {
                chosen = Some(button.0);
                break;
            }
        }
        if chosen.is_none() {
            for (index, code) in [
                KeyCode::Digit1,
                KeyCode::Digit2,
                KeyCode::Digit3,
                KeyCode::Digit4,
            ]
            .iter()
            .enumerate()
            {
                if keys.just_pressed(*code) {
                    chosen = Some(index);
                    break;
                }
            }
        }
        if let Some(index) = chosen {
            let question = &session.rounds[session.index];
            if index == question.correct {
                session.correct += 1;
                play_success(&mut commands, &sfx);
            } else {
                session.wrong += 1;
            }
            session.selected = Some(index);
            session.feedback = true;
            session.feedback_timer = FEEDBACK_SECONDS;
        }
        for (_button, mut bg) in &mut option_colors {
            *bg = BackgroundColor(OPTION_NEUTRAL);
        }
    }

    // 4) Textos de la operación actual.
    let question = &session.rounds[session.index];
    for (field, mut text, mut color, mut vis) in &mut texts {
        match field.0 {
            MathField::Title => {
                *text = Text::new(tr(session.operation.title()));
                *color = TextColor(Color::srgb(0.85, 0.95, 1.0));
                *vis = Visibility::Visible;
            }
            MathField::Question => {
                *text = Text::new(format!("{} = ?", question.text));
                *vis = Visibility::Visible;
            }
            MathField::Progress => {
                *text = Text::new(tr("Operación {}/{}  ·  Aciertos: {}  ·  Fallos: {}").replace("{}", &(session.index + 1).to_string()).replace("{}", &ROUNDS.to_string()).replace("{}", &session.correct.to_string()).replace("{}", &session.wrong.to_string()));
                *vis = Visibility::Visible;
            }
            MathField::Feedback => {
                if session.feedback {
                    let ok = session.selected == Some(question.correct);
                    if ok {
                        *text = Text::new(tr("¡Correcto!"));
                        *color = TextColor(Color::srgb(0.40, 0.90, 0.50));
                    } else {
                        *text = Text::new(tr("Incorrecto — era {}) {}").replace("{}", &OPTION_LETTERS[question.correct].to_string()).replace("{}", &question.options[question.correct]));
                        *color = TextColor(Color::srgb(0.95, 0.40, 0.40));
                    }
                    *vis = Visibility::Visible;
                } else {
                    *vis = Visibility::Hidden;
                }
            }
            _ => {}
        }
    }
    for (field, mut text) in &mut option_texts {
        *text = Text::new(format!(
            "{}) {}",
            OPTION_LETTERS[field.0],
            question.options[field.0]
        ));
    }
    // Clic sonoro al pulsar una opción.
    for (interaction, _button) in &option_clicks {
        if *interaction == Interaction::Pressed {
            play_click(&mut commands, &sfx);
            break;
        }
    }
}