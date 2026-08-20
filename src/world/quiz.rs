//! Cuestionarios (Fase 5): pantalla de pregunta con opciones A/B/C/D, tres
//! preguntas por asignatura, nota final y animación de retroalimentación.
//!
//! Con la tecla **Q** cerca de un profesor se abre un cuestionario de su
//! asignatura: 1 pregunta Fácil, 1 Media y 1 Difícil elegidas al azar. Se
//! responde con **clic** o con las teclas **1-4**. Tras cada respuesta se
//! muestra el feedback (verde si aciertas, rojo si fallas) durante 1,4 s y
//! se pasa a la siguiente. Al terminar aparece la nota y si la asignatura
//! está superada (se supera acertando las **tres** preguntas).

use bevy::prelude::*;

use crate::board::questions::{random_closed_question, Category, Difficulty, Question};
use crate::game::GameState;
use crate::i18n::tr;
use crate::player::Player;
use crate::world::teacher::Teacher;

/// Número de preguntas por cuestionario.
const QUIZ_LENGTH: usize = 3;
/// Duración (s) del feedback antes de pasar a la siguiente pregunta.
const FEEDBACK_SECONDS: f32 = 1.4;
/// Distancia máxima (metros) para iniciar un cuestionario con un profesor.
const QUIZ_DISTANCE: f32 = 2.6;

/// Sesión de cuestionario activa: mientras exista, el overlay está visible.
#[derive(Resource)]
pub struct QuizSession {
    /// Nombre de la asignatura.
    pub subject: &'static str,
    /// Color de la asignatura (título).
    pub accent: Color,
    /// Preguntas del cuestionario (Fácil, Media y Difícil).
    pub questions: Vec<Question>,
    /// Pregunta actual (0..3).
    pub index: usize,
    /// Aciertos.
    pub correct: usize,
    /// Fallos.
    pub wrong: usize,
    /// Opción elegida en la pregunta actual (durante el feedback).
    pub selected: Option<usize>,
    /// `true` mientras se muestra el feedback de la pregunta actual.
    pub feedback: bool,
    /// Cuenta atrás del feedback (s).
    pub feedback_timer: f32,
    /// `true` cuando las 3 preguntas están respondidas (pantalla de notas).
    pub done: bool,
}

impl QuizSession {
    /// Nota según los aciertos.
    fn nota(&self) -> String {
        match self.correct {
            3 => tr("10 · Sobresaliente"),
            2 => tr("6,7 · Notable"),
            1 => tr("3,3 · Suspenso"),
            _ => tr("0 · Suspenso"),
        }
    }

    /// La asignatura se supera acertando las tres preguntas.
    pub fn passed(&self) -> bool {
        self.correct == QUIZ_LENGTH
    }
}

// ---- Componentes de la UI -------------------------------------------------

/// Raíz del overlay del cuestionario (oculto hasta que hay sesión).
#[derive(Component)]
pub struct QuizOverlay;

/// Campo de texto del cuestionario (título, pregunta, progreso, feedback y
/// resultados), distinguido por `QuizField`.
#[derive(Component)]
pub struct QuizText(pub QuizField);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum QuizField {
    Title,
    Question,
    Progress,
    Feedback,
    ResultTitle,
    ResultDetail,
}

/// Texto de una opción (hijo de su botón).
#[derive(Component)]
pub struct QuizOptionText(pub usize);

/// Botón de una opción (A/B/C/D).
#[derive(Component)]
pub struct QuizOptionButton(pub usize);

/// Contenedor de los resultados (se muestra al terminar).
#[derive(Component)]
pub struct QuizResultBox;

/// Botón de cerrar el cuestionario.
#[derive(Component)]
pub struct QuizCloseButton;

/// Plugin de cuestionarios.
pub struct QuizPlugin;

impl Plugin for QuizPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_quiz_overlay).add_systems(
            Update,
            update_quiz.run_if(in_state(GameState::Playing)),
        );
    }
}

/// Letras de las opciones.
const OPTION_LETTERS: [char; 4] = ['A', 'B', 'C', 'D'];

/// Mapea la asignatura del profesor a su categoría del banco de preguntas.
fn category_of(subject: &str) -> Option<Category> {
    match subject {
        "Matemáticas" => Some(Category::Math),
        "Historia" => Some(Category::History),
        "Informática" => Some(Category::Cs),
        _ => None,
    }
}

/// Crea un texto del cuestionario (con su campo), ajustado a la anchura.
fn quiz_text(
    parent: &mut ChildSpawnerCommands,
    field: QuizField,
    text: &str,
    size: f32,
    font: &Handle<Font>,
) {
    parent.spawn((
        QuizText(field),
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
            max_width: Val::Px(580.0),
            ..default()
        },
    ));
}

/// Crea el texto de una opción (hijo de su botón).
fn quiz_option_text(
    parent: &mut ChildSpawnerCommands,
    index: usize,
    size: f32,
    font: &Handle<Font>,
) {
    parent.spawn((
        QuizOptionText(index),
        Text::new(String::new()),
        TextFont {
            font: font.clone(),
            font_size: size,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

/// Construye el overlay del cuestionario (oculto hasta que haya sesión).
fn spawn_quiz_overlay(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");

    commands
        .spawn((
            QuizOverlay,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.03, 0.08, 0.72)),
            Visibility::Hidden,
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
                    quiz_text(panel, QuizField::Title, "Cuestionario", 26.0, &font);
                    quiz_text(panel, QuizField::Question, "", 22.0, &font);

                    // Opciones A/B/C/D (botones clicables).
                    for index in 0..4 {
                        panel
                            .spawn((
                                Button,
                                QuizOptionButton(index),
                                Node {
                                    width: Val::Px(580.0),
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
                                quiz_option_text(option, index, 19.0, &font);
                            });
                    }

                    quiz_text(panel, QuizField::Progress, "", 17.0, &font);
                    quiz_text(panel, QuizField::Feedback, "", 22.0, &font);

                    // Resultados (ocultos hasta terminar las 3 preguntas).
                    panel
                        .spawn((
                            QuizResultBox,
                            Node {
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                row_gap: Val::Px(10.0),
                                ..default()
                            },
                            Visibility::Hidden,
                        ))
                        .with_children(|results| {
                            quiz_text(results, QuizField::ResultTitle, "", 26.0, &font);
                            quiz_text(results, QuizField::ResultDetail, "", 20.0, &font);
                            results
                                .spawn((
                                    Button,
                                    QuizCloseButton,
                                    Node {
                                        width: Val::Px(220.0),
                                        height: Val::Px(44.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        border: UiRect::all(Val::Px(2.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.20, 0.40, 0.30)),
                                    BorderColor(Color::srgb(0.45, 0.75, 0.55)),
                                    BorderRadius::all(Val::Px(8.0)),
                                ))
                                .with_children(|button| {
                                    button.spawn((
                                        Text::new(tr("Cerrar")),
                                        TextFont {
                                            font: font.clone(),
                                            font_size: 20.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                });
                        });
                });
        });
}

/// Colores de fondo de los botones de opción.
const OPTION_NEUTRAL: Color = Color::srgb(0.15, 0.18, 0.28);
const OPTION_DIM: Color = Color::srgb(0.10, 0.12, 0.20);
const OPTION_CORRECT: Color = Color::srgb(0.15, 0.42, 0.25);
const OPTION_WRONG: Color = Color::srgb(0.50, 0.20, 0.20);

/// Gestiona el cuestionario: lo abre con Q, responde con clic/teclas 1-4,
/// muestra el feedback y al final la nota con la opción de cerrar.
fn update_quiz(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    dialog: Option<Res<crate::world::dialog::DialogSession>>,
    mut commands: Commands,
    player_q: Query<&Transform, (With<Player>, Without<Teacher>)>,
    teachers: Query<(Entity, &Teacher, &Transform), Without<Player>>,
    session: Option<ResMut<QuizSession>>,
    mut overlay: Query<
        &mut Visibility,
        (
            With<QuizOverlay>,
            Without<QuizText>,
            Without<QuizResultBox>,
            Without<QuizOptionButton>,
        ),
    >,
    mut texts: Query<
        (&QuizText, &mut Text, &mut TextColor, &mut Visibility),
        (
            Without<QuizOverlay>,
            Without<QuizResultBox>,
            Without<QuizOptionButton>,
            Without<QuizOptionText>,
        ),
    >,
    mut option_texts: Query<
        (&QuizOptionText, &mut Text),
        (
            Without<QuizText>,
            Without<QuizOverlay>,
            Without<QuizResultBox>,
            Without<QuizOptionButton>,
        ),
    >,
    mut option_colors: Query<(&QuizOptionButton, &mut BackgroundColor), Without<QuizText>>,
    option_clicks: Query<
        (&Interaction, &QuizOptionButton),
        (Changed<Interaction>, Without<QuizText>),
    >,
    mut result_box: Query<
        &mut Visibility,
        (
            With<QuizResultBox>,
            Without<QuizText>,
            Without<QuizOverlay>,
            Without<QuizOptionButton>,
        ),
    >,
    close_clicks: Query<&Interaction, (Changed<Interaction>, With<QuizCloseButton>)>,
) {
    let dt = time.delta_secs();
    let Ok(player_tf) = player_q.single() else {
        return;
    };

    // 1) Sin sesión: overlay oculto y, con Q cerca de un profesor (y sin
    //    diálogo abierto), se empieza el cuestionario.
    let Some(mut session) = session else {
        if let Ok(mut vis) = overlay.single_mut() {
            *vis = Visibility::Hidden;
        }
        if dialog.is_some() || !keys.just_pressed(KeyCode::KeyQ) {
            return;
        }
        let mut nearest: Option<(f32, Entity)> = None;
        for (entity, _teacher, tf) in &teachers {
            let dx = tf.translation.x - player_tf.translation.x;
            let dz = tf.translation.z - player_tf.translation.z;
            let dist = Vec2::new(dx, dz).length();
            if dist < QUIZ_DISTANCE && nearest.map_or(true, |(d, _)| dist < d) {
                nearest = Some((dist, entity));
            }
        }
        let Some((_, entity)) = nearest else {
            return;
        };
        let Ok((_, teacher, _)) = teachers.get(entity) else {
            return;
        };
        let Some(category) = category_of(teacher.subject) else {
            return;
        };
        commands.insert_resource(QuizSession {
            subject: teacher.subject,
            accent: teacher.accent,
            questions: vec![
                random_closed_question(category, Difficulty::Easy),
                random_closed_question(category, Difficulty::Medium),
                random_closed_question(category, Difficulty::Hard),
            ],
            index: 0,
            correct: 0,
            wrong: 0,
            selected: None,
            feedback: false,
            feedback_timer: 0.0,
            done: false,
        });
        return;
    };

    // 2) Sesión activa: mostrar el overlay.
    if let Ok(mut vis) = overlay.single_mut() {
        *vis = Visibility::Visible;
    }

    // 3) Pantalla de resultados: nota + cerrar.
    if session.done {
        let close = close_clicks.single().map_or(false, |i| *i == Interaction::Pressed)
            || keys.just_pressed(KeyCode::KeyQ)
            || keys.just_pressed(KeyCode::Enter)
            || keys.just_pressed(KeyCode::Escape);
        if close {
            commands.remove_resource::<QuizSession>();
            return;
        }
        // Resultado: título verde/rojo y detalle con la nota.
        for (field, mut text, mut color, mut vis) in &mut texts {
            match field.0 {
                QuizField::ResultTitle => {
                    *text = Text::new(tr(if session.passed() { "¡Asignatura superada!" } else { "Asignatura no superada" }));
                    *color = TextColor(if session.passed() {
                        Color::srgb(0.40, 0.90, 0.50)
                    } else {
                        Color::srgb(0.95, 0.40, 0.40)
                    });
                    *vis = Visibility::Visible;
                }
                QuizField::ResultDetail => {
                    *text = Text::new(
                        tr("Aciertos: {} · Fallos: {}  —  Nota: {}")
                            .replace("{}", &session.correct.to_string())
                            .replace("{}", &session.wrong.to_string())
                            .replace("{}", &session.nota()),
                    );
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

    // 4) Feedback: cuenta atrás y colores de las opciones.
    if session.feedback {
        session.feedback_timer -= dt;
        for (button, mut bg) in &mut option_colors {
            let question = session.questions[session.index];
            *bg = BackgroundColor(if button.0 == question.correct {
                OPTION_CORRECT
            } else if Some(button.0) == session.selected {
                OPTION_WRONG
            } else {
                OPTION_DIM
            });
        }
        if session.feedback_timer <= 0.0 {
            // Siguiente pregunta o pantalla de resultados.
            session.feedback = false;
            session.selected = None;
            session.index += 1;
            if session.index >= QUIZ_LENGTH {
                session.done = true;
                return;
            }
        }
    } else {
        // 5) Responder: clic en una opción o teclas 1-4.
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
            let question = session.questions[session.index];
            if index == question.correct {
                session.correct += 1;
            } else {
                session.wrong += 1;
            }
            session.selected = Some(index);
            session.feedback = true;
            session.feedback_timer = FEEDBACK_SECONDS;
        }
        // Opciones en color neutro mientras se responde.
        for (_button, mut bg) in &mut option_colors {
            *bg = BackgroundColor(OPTION_NEUTRAL);
        }
    }

    // 6) Textos de la pregunta actual.
    let question = session.questions[session.index];
    for (field, mut text, mut color, mut vis) in &mut texts {
        match field.0 {
            QuizField::Title => {
                *text = Text::new(tr("Cuestionario de {}").replace("{}", session.subject));
                *color = TextColor(session.accent);
                *vis = Visibility::Visible;
            }
            QuizField::Question => {
                *text = Text::new(question.text);
                *vis = Visibility::Visible;
            }
            QuizField::Progress => {
                *text = Text::new(
                    tr("Pregunta {}/{}  ·  Aciertos: {}  ·  Fallos: {}")
                        .replace("{}", &(session.index + 1).to_string())
                        .replace("{}", &QUIZ_LENGTH.to_string())
                        .replace("{}", &session.correct.to_string())
                        .replace("{}", &session.wrong.to_string()),
                );
                *vis = Visibility::Visible;
            }
            QuizField::Feedback => {
                if session.feedback {
                    if session.selected == Some(question.correct) {
                        *text = Text::new(tr("¡Correcto!"));
                        *color = TextColor(Color::srgb(0.40, 0.90, 0.50));
                    } else {
                        *text = Text::new(
                            tr("Incorrecto — era {}) {}")
                                .replace("{}", &OPTION_LETTERS[question.correct].to_string())
                                .replace("{}", &question.options[question.correct]),
                        );
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
        *text = Text::new(format!("{}) {}", OPTION_LETTERS[field.0], question.options[field.0]));
    }
}