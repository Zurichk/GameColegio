//! Fracciones (Matemáticas) — compara o calcula fracciones.
//!
//! 10 rondas: mitad compara (¿qué es mayor?) y mitad suma/resta simple
//! de fracciones con mismo denominador. 4 opciones, trilingüe.

use bevy::prelude::*;
use rand::seq::SliceRandom;
use rand::Rng;

use super::{screen_background, spawn_button};
use crate::audio::{play_click, play_success, Sfx};
use crate::game::GameState;
use crate::i18n::tr;

const ROUNDS: usize = 10;
const FEEDBACK_SECONDS: f32 = 1.4;

struct FractionRound {
    text: String,
    options: [String; 4],
    correct: usize,
}

#[derive(Resource)]
pub struct FractionsSession {
    rounds: Vec<FractionRound>,
    index: usize,
    correct: usize,
    wrong: usize,
    selected: Option<usize>,
    feedback: bool,
    feedback_timer: f32,
    done: bool,
}

#[derive(Component)]
pub struct FractionsUiRoot;
#[derive(Component)]
pub struct FractionsText(FractionsField);
#[derive(Clone, Copy, PartialEq, Eq)]
enum FractionsField {
    Title,
    Question,
    Progress,
    Feedback,
    ResultTitle,
    ResultDetail,
}
#[derive(Component)]
pub struct FractionsOptionText(pub usize);
#[derive(Component)]
pub struct FractionsOptionButton(pub usize);
#[derive(Component)]
pub struct FractionsResultBox;
#[derive(Component)]
pub struct FractionsBackButton;

pub struct FractionsPlugin;
impl Plugin for FractionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::FractionsPractice), spawn_fractions_ui)
            .add_systems(OnExit(GameState::FractionsPractice), cleanup_fractions)
            .add_systems(Update, update_fractions.run_if(in_state(GameState::FractionsPractice)));
    }
}
const OPTION_LETTERS: [char; 4] = ['A', 'B', 'C', 'D'];

fn fractions_text(parent: &mut ChildSpawnerCommands, field: FractionsField, text: &str, size: f32, font: &Handle<Font>) {
    parent.spawn((
        FractionsText(field),
        Text::new(text.to_string()),
        TextFont { font: font.clone(), font_size: size, ..default() },
        TextColor(Color::WHITE),
        TextLayout { linebreak: LineBreak::WordBoundary, ..default() },
        Node { max_width: Val::Px(700.0), ..default() },
    ));
}
fn fractions_option_text(parent: &mut ChildSpawnerCommands, index: usize, size: f32, font: &Handle<Font>) {
    parent.spawn((
        FractionsOptionText(index),
        Text::new(String::new()),
        TextFont { font: font.clone(), font_size: size, ..default() },
        TextColor(Color::WHITE),
    ));
}

fn spawn_fractions_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands.insert_resource(FractionsSession {
        rounds: build_rounds(),
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
            FractionsUiRoot,
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
                        width: Val::Px(680.0),
                        padding: UiRect::axes(Val::Px(28.0), Val::Px(24.0)),
                        row_gap: Val::Px(14.0),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.07, 0.09, 0.18, 0.96)),
                    BorderRadius::all(Val::Px(16.0)),
                ))
                .with_children(|panel| {
                    fractions_text(panel, FractionsField::Title, "", 28.0, &font);
                    fractions_text(panel, FractionsField::Question, "", 30.0, &font);
                    for index in 0..4 {
                        panel
                            .spawn((
                                Button,
                                FractionsOptionButton(index),
                                Node {
                                    width: Val::Px(600.0),
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
                                fractions_option_text(option, index, 21.0, &font);
                            });
                    }
                    fractions_text(panel, FractionsField::Progress, "", 17.0, &font);
                    fractions_text(panel, FractionsField::Feedback, "", 22.0, &font);
                    panel
                        .spawn((
                            FractionsResultBox,
                            Node {
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                row_gap: Val::Px(10.0),
                                ..default()
                            },
                            Visibility::Hidden,
                        ))
                        .with_children(|results| {
                            fractions_text(results, FractionsField::ResultTitle, "", 26.0, &font);
                            fractions_text(results, FractionsField::ResultDetail, "", 20.0, &font);
                            spawn_button(results, "Volver a Matemáticas", FractionsBackButton, &font);
                        });
                });
        });
}

fn cleanup_fractions(mut commands: Commands, roots: Query<Entity, With<FractionsUiRoot>>) {
    for root in &roots {
        commands.entity(root).despawn();
    }
    commands.remove_resource::<FractionsSession>();
}

fn build_rounds() -> Vec<FractionRound> {
    let mut rng = rand::thread_rng();
    (0..ROUNDS).map(|_| make_round(&mut rng)).collect()
}

fn make_round(rng: &mut impl Rng) -> FractionRound {
    // 50% comparar, 50% sumar
    if rng.gen_bool(0.5) {
        // Comparar fracciones con mismo denominador o numerador
        let denom = rng.gen_range(2..10);
        let a = rng.gen_range(1..denom);
        let b = rng.gen_range(1..denom);
        if a == b {
            return make_round(rng);
        }
        let text = format!("¿Qué es mayor? {a}/{denom}  o  {b}/{denom}");
        let correct_answer = if a > b { format!("{a}/{denom}") } else { format!("{b}/{denom}") };
        let mut options = vec![correct_answer.clone()];
        // distractores
        let mut distractors = vec![format!("{}/{}", a.min(b), denom + 1), format!("{}/{}", (a + b) / 2 + 1, denom), "Iguales".to_string()];
        distractors.retain(|d| d != &correct_answer);
        while options.len() < 4 && !distractors.is_empty() {
            options.push(distractors.remove(0));
        }
        while options.len() < 4 {
            options.push(format!("{}/{}", rng.gen_range(1..9), rng.gen_range(2..10)));
        }
        options.shuffle(rng);
        let correct = options.iter().position(|o| o == &correct_answer).unwrap_or(0);
        FractionRound { text, options: options.try_into().unwrap(), correct }
    } else {
        // Suma con mismo denominador
        let denom = rng.gen_range(2..8);
        let a = rng.gen_range(1..denom);
        let b = rng.gen_range(1..denom);
        let sum_num = a + b;
        let text = format!("{a}/{denom} + {b}/{denom} = ?");
        // simplificar si se puede no necesario para juego, dejamos como fracción
        let correct_answer = if sum_num % denom == 0 { format!("{}", sum_num / denom) } else { format!("{sum_num}/{denom}") };
        let mut options = vec![correct_answer.clone()];
        let mut distractors = vec![
            format!("{}/{}", sum_num + 1, denom),
            format!("{}/{}", sum_num, denom + 1),
            format!("{}/{}", a + b + 1, denom),
        ];
        distractors.retain(|d| d != &correct_answer);
        distractors.shuffle(rng);
        for d in distractors.into_iter().take(3) {
            options.push(d);
        }
        options.shuffle(rng);
        let correct = options.iter().position(|o| o == &correct_answer).unwrap_or(0);
        FractionRound { text, options: options.try_into().unwrap(), correct }
    }
}

const OPTION_NEUTRAL: Color = Color::srgb(0.15, 0.18, 0.28);
const OPTION_DIM: Color = Color::srgb(0.10, 0.12, 0.20);
const OPTION_CORRECT: Color = Color::srgb(0.15, 0.42, 0.25);
const OPTION_WRONG: Color = Color::srgb(0.50, 0.20, 0.20);

fn update_fractions(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut commands: Commands,
    sfx: Res<Sfx>,
    session: Option<ResMut<FractionsSession>>,
    mut texts: Query<(&FractionsText, &mut Text, &mut TextColor, &mut Visibility), (Without<FractionsOptionText>, Without<FractionsOptionButton>, Without<FractionsResultBox>)>,
    mut option_texts: Query<(&FractionsOptionText, &mut Text), (Without<FractionsText>, Without<FractionsOptionButton>, Without<FractionsResultBox>)>,
    mut option_colors: Query<(&FractionsOptionButton, &mut BackgroundColor), Without<FractionsText>>,
    option_clicks: Query<(&Interaction, &FractionsOptionButton), (Changed<Interaction>, Without<FractionsText>)>,
    mut result_box: Query<&mut Visibility, (With<FractionsResultBox>, Without<FractionsText>, Without<FractionsOptionButton>)>,
    close_clicks: Query<&Interaction, (Changed<Interaction>, With<FractionsBackButton>)>,
) {
    let dt = time.delta_secs();
    let mut session = match session {
        Some(s) => s,
        None => {
            commands.insert_resource(FractionsSession { rounds: build_rounds(), index: 0, correct: 0, wrong: 0, selected: None, feedback: false, feedback_timer: 0.0, done: false });
            return;
        }
    };
    if keys.just_pressed(KeyCode::Escape) {
        commands.set_state(GameState::MathMenu);
        return;
    }
    if session.done {
        let close = close_clicks.single().map_or(false, |i| *i == Interaction::Pressed) || keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::KeyQ);
        if close {
            commands.set_state(GameState::MathMenu);
            return;
        }
        for (field, mut text, mut color, mut vis) in &mut texts {
            match field.0 {
                FractionsField::ResultTitle => {
                    *text = Text::new(tr(if session.correct >= ROUNDS / 2 { "¡Muy bien!" } else { "¡Sigue practicando!" }));
                    *color = TextColor(if session.correct >= ROUNDS / 2 { Color::srgb(0.40, 0.90, 0.50) } else { Color::srgb(0.95, 0.55, 0.30) });
                    *vis = Visibility::Visible;
                }
                FractionsField::ResultDetail => {
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
    if session.feedback {
        session.feedback_timer -= dt;
        for (button, mut bg) in &mut option_colors {
            let q = &session.rounds[session.index];
            *bg = BackgroundColor(if button.0 == q.correct { OPTION_CORRECT } else if Some(button.0) == session.selected { OPTION_WRONG } else { OPTION_DIM });
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
        let mut chosen: Option<usize> = None;
        for (interaction, button) in &option_clicks {
            if *interaction == Interaction::Pressed {
                chosen = Some(button.0);
                break;
            }
        }
        if chosen.is_none() {
            for (index, code) in [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4].iter().enumerate() {
                if keys.just_pressed(*code) {
                    chosen = Some(index);
                    break;
                }
            }
        }
        if let Some(index) = chosen {
            let q = &session.rounds[session.index];
            if index == q.correct {
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
    let question = &session.rounds[session.index];
    for (field, mut text, mut color, mut vis) in &mut texts {
        match field.0 {
            FractionsField::Title => {
                *text = Text::new(tr("FRACCIONES"));
                *color = TextColor(Color::srgb(1.0, 0.90, 0.50));
                *vis = Visibility::Visible;
            }
            FractionsField::Question => {
                *text = Text::new(question.text.clone());
                *vis = Visibility::Visible;
            }
            FractionsField::Progress => {
                *text = Text::new(tr("Operación {}/{}  ·  Aciertos: {}  ·  Fallos: {}").replace("{}", &(session.index + 1).to_string()).replace("{}", &ROUNDS.to_string()).replace("{}", &session.correct.to_string()).replace("{}", &session.wrong.to_string()));
                *vis = Visibility::Visible;
            }
            FractionsField::Feedback => {
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
        *text = Text::new(format!("{}) {}", OPTION_LETTERS[field.0], question.options[field.0]));
    }
    for (interaction, _button) in &option_clicks {
        if *interaction == Interaction::Pressed {
            play_click(&mut commands, &sfx);
            break;
        }
    }
}
