//! Juego "Mayor, menor o igual" (sección Matemáticas).
//!
//! Se muestran dos números y el jugador elige si el primero es mayor (>),
//! menor (<) o igual (=) que el segundo. Sesión de 10 rondas con dificultad
//! creciente, feedback y marcador final.

use bevy::prelude::*;
use rand::Rng;

use super::{screen_background, spawn_button};
use crate::audio::{play_click, play_success, Sfx};
use crate::game::GameState;
use crate::i18n::tr;

/// Número de rondas por sesión.
const ROUNDS: usize = 10;
/// Duración (s) del feedback.
const FEEDBACK_SECONDS: f32 = 1.2;

/// Una ronda del juego: los dos números y la respuesta esperada.
struct CompareRound {
    a: i64,
    b: i64,
    /// 0 = mayor (>), 1 = menor (<), 2 = igual (=).
    answer: usize,
}

/// Sesión activa del juego.
#[derive(Resource)]
pub struct CompareSession {
    rounds: Vec<CompareRound>,
    index: usize,
    correct: usize,
    wrong: usize,
    selected: Option<usize>,
    feedback: bool,
    feedback_timer: f32,
    done: bool,
}

// ---- Componentes de la UI --------------------------------------------------

/// Raíz de la pantalla.
#[derive(Component)]
pub struct CompareUiRoot;

/// Campo de texto etiquetado por su función.
#[derive(Component)]
pub struct CompareText(CompareField);

#[derive(Clone, Copy, PartialEq, Eq)]
enum CompareField {
    Title,
    Question,
    Progress,
    Feedback,
    ResultTitle,
    ResultDetail,
}

/// Botón de comparación (>, < o =).
#[derive(Component)]
pub struct CompareButton(pub usize);

/// Contenedor de resultados (oculto hasta terminar).
#[derive(Component)]
pub struct CompareResultBox;

/// Botón de volver al menú de Matemáticas.
#[derive(Component)]
pub struct CompareBackButton;

/// Plugin del juego de comparación.
pub struct ComparePlugin;

impl Plugin for ComparePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::ComparePractice), spawn_compare_ui)
            .add_systems(OnExit(GameState::ComparePractice), cleanup_compare)
            .add_systems(
                Update,
                update_compare.run_if(in_state(GameState::ComparePractice)),
            );
    }
}

/// Símbolos de las respuestas.
const SYMBOLS: [&str; 3] = [">", "<", "="];

/// Devuelve un texto del campo indicado (listo para `spawn`).
fn compare_text(
    field: CompareField,
    text: &str,
    size: f32,
    font: &Handle<Font>,
) -> (CompareText, Text, TextFont, TextColor, TextLayout, Node) {
    (
        CompareText(field),
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
    )
}

/// Construye la pantalla del juego.
fn spawn_compare_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands
        .spawn((
            CompareUiRoot,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(16.0),
                ..default()
            },
            screen_background(),
        ))
        .with_children(|root| {
            root.spawn(compare_text(
                CompareField::Title,
                "¿QUÉ SIGNO VA ENTRE LOS DOS NÚMEROS?",
                40.0,
                &font,
            ));
            root.spawn(compare_text(CompareField::Question, "", 72.0, &font));
            root.spawn(compare_text(CompareField::Progress, "", 22.0, &font));
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(24.0),
                ..default()
            })
            .with_children(|row| {
                for (i, symbol) in SYMBOLS.iter().enumerate() {
                    row.spawn((
                        Button,
                        Node {
                            width: Val::Px(110.0),
                            height: Val::Px(90.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.20, 0.38, 0.66)),
                        BorderColor(Color::srgb(0.60, 0.80, 1.0)),
                        BorderRadius::all(Val::Px(12.0)),
                        Visibility::Inherited,
                        CompareButton(i),
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Text::new(symbol.to_string()),
                            TextFont {
                                font: font.clone(),
                                font_size: 52.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
                }
            });
            root.spawn(compare_text(
                CompareField::Feedback,
                "",
                26.0,
                &font,
            ));
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(10.0),
                    align_items: AlignItems::Center,
                    ..default()
                },
                CompareResultBox,
                Visibility::Hidden,
            ))
            .with_children(|box_root| {
                box_root.spawn(compare_text(
                    CompareField::ResultTitle,
                    "¡Bien hecho!",
                    42.0,
                    &font,
                ));
                box_root.spawn(compare_text(
                    CompareField::ResultDetail,
                    "",
                    24.0,
                    &font,
                ));
                spawn_button(box_root, "Jugar otra vez", CompareBackButton, &font);
                spawn_button(
                    box_root,
                    "Volver a la zona de aprendizaje",
                    CompareBackButton,
                    &font,
                );
            });
        });
}

/// Destruye la pantalla y la sesión al salir.
fn cleanup_compare(mut commands: Commands, roots: Query<Entity, With<CompareUiRoot>>) {
    for root in &roots {
        commands.entity(root).despawn();
    }
    commands.remove_resource::<CompareSession>();
}

/// Genera las rondas de la sesión (dificultad creciente).
fn build_rounds() -> Vec<CompareRound> {
    let mut rng = rand::thread_rng();
    (0..ROUNDS)
        .map(|i| {
            let step = i / 3;
            let limit: i64 = 9 + step as i64 * 8;
            let a: i64 = rng.gen_range(1..=limit);
            let b: i64 = rng.gen_range(1..=limit);
            // Mezcla de casos: ~30 % de igualdades para que no sea trivial.
            let answer = if i % 3 == 2 && a >= 2 && b >= 2 {
                2
            } else if a > b {
                0
            } else if a < b {
                1
            } else {
                2
            };
            CompareRound { a, b, answer }
        })
        .collect()
}

/// Colores de los botones según el resultado.
const OPTION_NEUTRAL: Color = Color::srgb(0.15, 0.18, 0.28);
const OPTION_CORRECT: Color = Color::srgb(0.15, 0.42, 0.25);
const OPTION_WRONG: Color = Color::srgb(0.50, 0.20, 0.20);

/// Gestiona la sesión: respuesta, feedback y resultados.
fn update_compare(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut commands: Commands,
    sfx: Res<Sfx>,
    session: Option<ResMut<CompareSession>>,
    mut texts: Query<
        (&CompareText, &mut Text, &mut TextColor, &mut Visibility),
        (
            Without<CompareButton>,
            Without<CompareResultBox>,
        ),
    >,
    mut button_colors: Query<(&CompareButton, &mut BackgroundColor), Without<CompareText>>,
    button_clicks: Query<
        (&Interaction, &CompareButton),
        (Changed<Interaction>, Without<CompareText>),
    >,
    mut result_box: Query<
        &mut Visibility,
        (With<CompareResultBox>, Without<CompareText>, Without<CompareButton>),
    >,
    close_clicks: Query<&Interaction, (Changed<Interaction>, With<CompareBackButton>)>,
) {
    let dt = time.delta_secs();
    let mut session = match session {
        Some(session) => session,
        None => {
            commands.insert_resource(CompareSession {
                rounds: build_rounds(),
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

    // Actualiza los textos según el estado de la sesión.
    for (field, mut text, mut color, mut visibility) in &mut texts {
        if session.feedback {
            *visibility = Visibility::Visible;
            match field.0 {
                CompareField::Progress => {
                    *text = Text::new(tr("Ronda {}/{} · Aciertos: {} · Fallos: {}")
                        .replace("{}", &(session.index + 1).to_string())
                        .replace("{}", &ROUNDS.to_string())
                        .replace("{}", &session.correct.to_string())
                        .replace("{}", &session.wrong.to_string()));
                }
                CompareField::Feedback => {
                    let correct = session.selected == Some(session.rounds[session.index].answer);
                    *text = Text::new(if correct {
                        tr("¡Correcto!")
                    } else {
                        tr("Incorrecto — el signo correcto era: {}")
                            .replace(
                                "{}",
                                SYMBOLS[session.rounds[session.index].answer],
                            )
                    });
                    *color = TextColor(if correct {
                        Color::srgb(0.40, 0.95, 0.55)
                    } else {
                        Color::srgb(0.95, 0.45, 0.45)
                    });
                }
                _ => {}
            }
        } else if session.done {
            *visibility = Visibility::Hidden;
        } else {
            let round = &session.rounds[session.index];
            match field.0 {
                CompareField::Question => {
                    *text = Text::new(format!("{} ? {}", round.a, round.b));
                }
                CompareField::Progress => {
                    *text = Text::new(tr("Ronda {}/{} · Aciertos: {} · Fallos: {}")
                        .replace("{}", &(session.index + 1).to_string())
                        .replace("{}", &ROUNDS.to_string())
                        .replace("{}", &session.correct.to_string())
                        .replace("{}", &session.wrong.to_string()));
                }
                CompareField::Feedback => {
                    *text = Text::new("");
                }
                _ => {}
            }
        }
    }

    // Feedback: espera y pasa a la siguiente ronda.
    if session.feedback {
        session.feedback_timer -= dt;
        if session.feedback_timer <= 0.0 {
            session.feedback = false;
            session.selected = None;
            session.index += 1;
            if session.index >= session.rounds.len() {
                session.done = true;
                // Muestra el marcador final.
for (field, mut text, mut color, _) in &mut texts {
                    if let CompareField::ResultTitle = field.0 {
                        let title = if session.correct >= 7 {
                            tr("¡Asignatura superada!")
                        } else {
                            tr("Asignatura no superada")
                        };
                        *text = Text::new(title);
                        *color = TextColor(if session.correct >= 7 {
                            Color::srgb(0.40, 0.95, 0.55)
                        } else {
                            Color::srgb(0.95, 0.45, 0.45)
                        });
                    }
                    if let CompareField::ResultDetail = field.0 {
                        *text = Text::new(tr("Aciertos: {} · Fallos: {} — Nota: {}")
                            .replace("{}", &session.correct.to_string())
                            .replace("{}", &session.wrong.to_string())
                            .replace(
                                "{}",
                                match session.correct {
                                    10 => "10 · Sobresaliente",
                                    7..=9 => "6,7 · Notable",
                                    5..=6 => "3,3 · Suspenso",
                                    _ => "0 · Suspenso",
                                },
                            ));
                    }
                }
                if let Ok(mut visibility) = result_box.single_mut() {
                    *visibility = Visibility::Visible;
                }
                play_success(&mut commands, &sfx);
            }
        }
        return;
    }

    // Resultados: botón de volver a jugar o salir.
    if session.done {
        for interaction in &close_clicks {
            if *interaction == Interaction::Pressed {
                commands.set_state(GameState::MathMenu);
                return;
            }
        }
        return;
    }

    // Clic en un símbolo.
    for (interaction, button) in &button_clicks {
        if *interaction != Interaction::Pressed {
            continue;
        }
        play_click(&mut commands, &sfx);
        session.selected = Some(button.0);
        let correct = button.0 == session.rounds[session.index].answer;
        if correct {
            session.correct += 1;
        } else {
            session.wrong += 1;
        }
        session.feedback = true;
        session.feedback_timer = FEEDBACK_SECONDS;

        // Pinta los botones: verde el correcto, rojo el pulsado si falló.
        for (b, mut bg) in &mut button_colors {
            if b.0 == session.rounds[session.index].answer {
                *bg = BackgroundColor(OPTION_CORRECT);
            } else if b.0 == button.0 {
                *bg = BackgroundColor(OPTION_WRONG);
            } else {
                *bg = BackgroundColor(OPTION_NEUTRAL);
            }
        }
        return;
    }

    // Restaura el color neutro de los botones entre rondas.
    for (_, mut bg) in &mut button_colors {
        *bg = BackgroundColor(OPTION_NEUTRAL);
    }
}