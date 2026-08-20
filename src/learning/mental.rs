//! Práctica de cálculo mental con temporizador (sección Matemáticas).
//!
//! Como las operaciones de "Primeros pasos" pero con **12 segundos por
//! pregunta**: si se agota el tiempo se cuenta como fallo. Sesión de 10
//! operaciones con dificultad creciente y marcador final.

use bevy::prelude::*;
use rand::seq::SliceRandom;
use rand::Rng;

use super::{screen_background, spawn_button};
use crate::audio::{play_click, play_success, Sfx};
use crate::game::GameState;
use crate::i18n::tr;

/// Número de operaciones por sesión.
const ROUNDS: usize = 10;
/// Duración (s) del feedback.
const FEEDBACK_SECONDS: f32 = 1.4;
/// Segundos para responder cada operación.
const TIME_PER_QUESTION: f32 = 12.0;

/// Una operación de la sesión, con sus 4 opciones y la correcta.
struct MentalRound {
    text: String,
    options: [String; 4],
    correct: usize,
}

/// Sesión de cálculo mental activa.
#[derive(Resource)]
pub struct MentalSession {
    rounds: Vec<MentalRound>,
    index: usize,
    correct: usize,
    wrong: usize,
    selected: Option<usize>,
    feedback: bool,
    feedback_timer: f32,
    time_left: f32,
    timed_out: bool,
    done: bool,
}

// ---- Componentes de la UI --------------------------------------------------

/// Raíz de la pantalla de cálculo mental.
#[derive(Component)]
pub struct MentalUiRoot;

/// Campo de texto etiquetado por su función.
#[derive(Component)]
pub struct MentalText(MentalField);

#[derive(Clone, Copy, PartialEq, Eq)]
enum MentalField {
    Title,
    Question,
    Timer,
    Progress,
    Feedback,
    ResultTitle,
    ResultDetail,
}

/// Texto de una opción (hijo de su botón).
#[derive(Component)]
pub struct MentalOptionText(pub usize);

/// Botón de una opción (A/B/C/D).
#[derive(Component)]
pub struct MentalOptionButton(pub usize);

/// Contenedor de resultados (oculto hasta terminar).
#[derive(Component)]
pub struct MentalResultBox;

/// Botón de volver al menú de Matemáticas.
#[derive(Component)]
pub struct MentalBackButton;

/// Plugin de la práctica de cálculo mental.
pub struct MentalPlugin;

impl Plugin for MentalPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MentalPractice), spawn_mental_ui)
            .add_systems(OnExit(GameState::MentalPractice), cleanup_mental)
            .add_systems(
                Update,
                update_mental.run_if(in_state(GameState::MentalPractice)),
            );
    }
}

/// Letras de las opciones.
const OPTION_LETTERS: [char; 4] = ['A', 'B', 'C', 'D'];

/// Crea un texto del campo indicado.
fn mental_text(
    parent: &mut ChildSpawnerCommands,
    field: MentalField,
    text: &str,
    size: f32,
    font: &Handle<Font>,
) {
    parent.spawn((
        MentalText(field),
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
fn mental_option_text(
    parent: &mut ChildSpawnerCommands,
    index: usize,
    size: f32,
    font: &Handle<Font>,
) {
    parent.spawn((
        MentalOptionText(index),
        Text::new(String::new()),
        TextFont {
            font: font.clone(),
            font_size: size,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

/// Construye la pantalla de cálculo mental.
fn spawn_mental_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands.insert_resource(MentalSession {
        rounds: build_rounds(),
        index: 0,
        correct: 0,
        wrong: 0,
        selected: None,
        feedback: false,
        feedback_timer: 0.0,
        time_left: TIME_PER_QUESTION,
        timed_out: false,
        done: false,
    });
    commands
        .spawn((
            MentalUiRoot,
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
                    mental_text(panel, MentalField::Title, "", 28.0, &font);
                    mental_text(panel, MentalField::Question, "", 40.0, &font);

                    // Opciones A/B/C/D.
                    for index in 0..4 {
                        panel
                            .spawn((
                                Button,
                                MentalOptionButton(index),
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
                                mental_option_text(option, index, 22.0, &font);
                            });
                    }

                    mental_text(panel, MentalField::Timer, "", 20.0, &font);
                    mental_text(panel, MentalField::Progress, "", 17.0, &font);
                    mental_text(panel, MentalField::Feedback, "", 22.0, &font);

                    // Resultados (ocultos hasta terminar).
                    panel
                        .spawn((
                            MentalResultBox,
                            Node {
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                row_gap: Val::Px(10.0),
                                ..default()
                            },
                            Visibility::Hidden,
                        ))
                        .with_children(|results| {
                            mental_text(results, MentalField::ResultTitle, "", 26.0, &font);
                            mental_text(results, MentalField::ResultDetail, "", 20.0, &font);
                            spawn_button(results, "Volver a Matemáticas", MentalBackButton, &font);
                        });
                });
        });
}

/// Destruye la pantalla y la sesión al salir.
fn cleanup_mental(mut commands: Commands, roots: Query<Entity, With<MentalUiRoot>>) {
    for root in &roots {
        commands.entity(root).despawn();
    }
    commands.remove_resource::<MentalSession>();
}

/// Genera las 10 operaciones de la sesión (dificultad creciente).
fn build_rounds() -> Vec<MentalRound> {
    let mut rng = rand::thread_rng();
    (0..ROUNDS).map(|i| make_round(i, &mut rng)).collect()
}

/// Crea una operación mental con sus 4 opciones (1 correcta + 3 cercanas).
fn make_round(index: usize, rng: &mut impl Rng) -> MentalRound {
    // Mezcla de operaciones: 0 suma, 1 resta, 2 multiplicación, 3 división.
    let op = index % 4;
    let step = index / 4; // dificultad creciente
    let (text, answer) = match op {
        0 => {
            let limit = 10 + step * 8;
            let a = rng.gen_range(1..=limit);
            let b = rng.gen_range(1..=limit);
            (format!("{a} + {b}"), a + b)
        }
        1 => {
            let max = 20 + step * 8;
            let a = rng.gen_range(2..=max);
            let b = rng.gen_range(1..a);
            (format!("{a} − {b}"), a - b)
        }
        2 => {
            let limit = if step == 0 { 5 } else { 9 };
            let a = rng.gen_range(2..=limit);
            let b = rng.gen_range(2..=limit);
            (format!("{a} × {b}"), a * b)
        }
        _ => {
            let limit = if step == 0 { 5 } else { 9 };
            let b = rng.gen_range(2..=limit);
            let q = rng.gen_range(2..=limit);
            (format!("{} ÷ {b}", b * q), q)
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

    MentalRound {
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

/// Gestiona la sesión de cálculo mental: respuesta, temporizador y resultados.
fn update_mental(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut commands: Commands,
    sfx: Res<Sfx>,
    session: Option<ResMut<MentalSession>>,
    mut texts: Query<
        (&MentalText, &mut Text, &mut TextColor, &mut Visibility),
        (
            Without<MentalOptionText>,
            Without<MentalOptionButton>,
            Without<MentalResultBox>,
        ),
    >,
    mut option_texts: Query<
        (&MentalOptionText, &mut Text),
        (
            Without<MentalText>,
            Without<MentalOptionButton>,
            Without<MentalResultBox>,
        ),
    >,
    mut option_colors: Query<(&MentalOptionButton, &mut BackgroundColor), Without<MentalText>>,
    option_clicks: Query<
        (&Interaction, &MentalOptionButton),
        (Changed<Interaction>, Without<MentalText>),
    >,
    mut result_box: Query<
        &mut Visibility,
        (
            With<MentalResultBox>,
            Without<MentalText>,
            Without<MentalOptionButton>,
        ),
    >,
    close_clicks: Query<&Interaction, (Changed<Interaction>, With<MentalBackButton>)>,
) {
    let dt = time.delta_secs();
    let mut session = match session {
        Some(session) => session,
        None => {
            commands.insert_resource(MentalSession {
                rounds: build_rounds(),
                index: 0,
                correct: 0,
                wrong: 0,
                selected: None,
                feedback: false,
                feedback_timer: 0.0,
                time_left: TIME_PER_QUESTION,
                timed_out: false,
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
                MentalField::ResultTitle => {
                    *text = Text::new(tr(if session.correct >= ROUNDS / 2 { "¡Muy bien!" } else { "¡Sigue practicando!" }));
                    *color = TextColor(if session.correct >= ROUNDS / 2 {
                        Color::srgb(0.40, 0.90, 0.50)
                    } else {
                        Color::srgb(0.95, 0.55, 0.30)
                    });
                    *vis = Visibility::Visible;
                }
                MentalField::ResultDetail => {
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

    // 2) Feedback (el temporizador se congela mientras se muestra).
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
            session.timed_out = false;
            session.index += 1;
            if session.index >= ROUNDS {
                session.done = true;
                return;
            }
            session.time_left = TIME_PER_QUESTION;
        }
    } else {
        // 3) Temporizador: se acaba el tiempo → fallo.
        session.time_left -= dt;
        if session.time_left <= 0.0 {
            session.wrong += 1;
            session.selected = None;
            session.timed_out = true;
            session.feedback = true;
            session.feedback_timer = FEEDBACK_SECONDS;
            session.time_left = 0.0;
        }

        // 4) Responder: clic en una opción o teclas 1-4.
        if session.time_left > 0.0 {
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
                session.timed_out = false;
                session.feedback = true;
                session.feedback_timer = FEEDBACK_SECONDS;
            }
        }
        for (_button, mut bg) in &mut option_colors {
            *bg = BackgroundColor(OPTION_NEUTRAL);
        }
    }

    // 5) Textos de la operación actual.
    let question = &session.rounds[session.index];
    for (field, mut text, mut color, mut vis) in &mut texts {
        match field.0 {
            MentalField::Title => {
                *text = Text::new(tr("CÁLCULO MENTAL"));
                *color = TextColor(Color::srgb(1.0, 0.90, 0.50));
                *vis = Visibility::Visible;
            }
            MentalField::Question => {
                *text = Text::new(format!("{} = ?", question.text));
                *vis = Visibility::Visible;
            }
            MentalField::Timer => {
                if session.feedback {
                    *text = Text::new(String::new());
                    *vis = Visibility::Hidden;
                } else {
                    let seconds = session.time_left.max(0.0);
                    let urgent = seconds <= 4.0;
                    *text = Text::new(tr("Tiempo: {} s").replace("{}", &format!("{:.1}", seconds)));
                    *color = TextColor(if urgent {
                        Color::srgb(1.0, 0.45, 0.40)
                    } else {
                        Color::srgb(0.60, 0.90, 1.0)
                    });
                    *vis = Visibility::Visible;
                }
            }
            MentalField::Progress => {
                *text = Text::new(tr("Operación {}/{}  ·  Aciertos: {}  ·  Fallos: {}").replace("{}", &(session.index + 1).to_string()).replace("{}", &ROUNDS.to_string()).replace("{}", &session.correct.to_string()).replace("{}", &session.wrong.to_string()));
                *vis = Visibility::Visible;
            }
            MentalField::Feedback => {
                if session.feedback {
                    if session.timed_out {
                        *text = Text::new(tr("Se acabó el tiempo — era {}) {}").replace("{}", &OPTION_LETTERS[question.correct].to_string()).replace("{}", &question.options[question.correct]));
                        *color = TextColor(Color::srgb(0.95, 0.40, 0.40));
                    } else if session.selected == Some(question.correct) {
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