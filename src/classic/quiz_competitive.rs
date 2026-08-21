//! Quiz Competitivo 2-4 jugadores — misma pregunta, primero en pulsar responde.

use bevy::prelude::*;
use rand::seq::SliceRandom;

use crate::board::questions::{random_closed_question, Category, Difficulty};
use crate::game::GameState;
use crate::learning::screen_background;
use crate::i18n::tr;

const WIN_POINTS: u32 = 5;

#[derive(Resource, Clone)]
struct QuizCompSession {
    question: String,
    options: [String; 4],
    correct: usize,
    category: Category,
    scores: [u32; 4],
    buzzed: Option<usize>, // player index 0..3 que pulsó primero
    players: usize,
    winner: Option<usize>,
}

impl QuizCompSession {
    fn new(players: usize) -> Self {
        let mut s = Self { question: String::new(), options: [String::new(), String::new(), String::new(), String::new()], correct: 0, category: Category::Math, scores: [0;4], buzzed: None, players, winner: None };
        s.next_question();
        s
    }
    fn next_question(&mut self) {
        let cat = *Category::colored().choose(&mut rand::thread_rng()).unwrap();
        let diff = [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard].choose(&mut rand::thread_rng()).copied().unwrap();
        let q = random_closed_question(cat, diff);
        self.question = q.text.to_string();
        self.category = cat;
        self.options = q.options.map(|s| s.to_string());
        self.correct = q.correct;
        self.buzzed = None;
    }
}

#[derive(Component)]
struct QuizCompUiRoot;
#[derive(Component)]
struct QuizCompText(QuizCompField);
#[derive(Clone, Copy, PartialEq, Eq)]
enum QuizCompField { Title, Question, Status, Scores }
#[derive(Component)]
struct QuizCompBuzzButton(usize);
#[derive(Component)]
struct QuizCompOptionButton(usize);
#[derive(Component)]
struct QuizCompBackButton;

pub struct QuizCompPlugin;
impl Plugin for QuizCompPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::QuizCompetitiveGame), spawn_quiz_comp)
            .add_systems(OnExit(GameState::QuizCompetitiveGame), cleanup_quiz_comp)
            .add_systems(Update, update_quiz_comp.run_if(in_state(GameState::QuizCompetitiveGame)));
    }
}

fn spawn_quiz_comp(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands.insert_resource(QuizCompSession::new(2));
    commands
        .spawn((QuizCompUiRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, screen_background(), Visibility::Visible, ZIndex(30)))
        .with_children(|overlay| {
            overlay.spawn((Node { flex_direction: FlexDirection::Column, width: Val::Px(720.0), padding: UiRect::axes(Val::Px(20.0), Val::Px(16.0)), row_gap: Val::Px(10.0), align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.07, 0.09, 0.18, 0.96)), BorderRadius::all(Val::Px(16.0)))).with_children(|panel| {
                panel.spawn((QuizCompText(QuizCompField::Title), Text::new("QUIZ COMPETITIVO 2-4"), TextFont { font: font.clone(), font_size: 28.0, ..default() }, TextColor(Color::srgb(0.95, 0.85, 0.40))));
                panel.spawn((QuizCompText(QuizCompField::Scores), Text::new("Puntos: J1 0 - J2 0"), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE)));
                panel.spawn((QuizCompText(QuizCompField::Question), Text::new(""), TextFont { font: font.clone(), font_size: 22.0, ..default() }, TextColor(Color::WHITE), TextLayout { linebreak: LineBreak::WordBoundary, ..default() }, Node { max_width: Val::Px(680.0), ..default() }));
                panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(12.0), ..default() }).with_children(|row| {
                    for i in 0..4 {
                        row.spawn((Button, QuizCompBuzzButton(i), Node { width: Val::Px(120.0), height: Val::Px(50.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.60, 0.30, 0.20)), BorderColor(Color::srgb(0.80, 0.50, 0.50)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(format!("J{} ¡Ya!", i+1)), TextFont { font: font.clone(), font_size: 16.0, ..default() }, TextColor(Color::WHITE))); });
                    }
                });
                for i in 0..4 {
                    panel.spawn((Button, QuizCompOptionButton(i), Node { width: Val::Px(600.0), height: Val::Px(46.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.15, 0.18, 0.28)), BorderColor(Color::srgb(0.50, 0.55, 0.70)), BorderRadius::all(Val::Px(8.0)))).with_children(|b| { b.spawn((Text::new("".to_string()), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE))); });
                }
                panel.spawn((QuizCompText(QuizCompField::Status), Text::new("¡Pulsad para reservar turno!"), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::srgb(0.80, 0.95, 1.0))));
                panel.spawn((Button, QuizCompBackButton, Node { width: Val::Px(200.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Volver")), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE))); });
            });
        });
}

fn cleanup_quiz_comp(mut commands: Commands, roots: Query<Entity, With<QuizCompUiRoot>>) {
    for root in &roots { commands.entity(root).despawn(); }
    commands.remove_resource::<QuizCompSession>();
}

fn update_quiz_comp(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut session: ResMut<QuizCompSession>,
    buzz_clicks: Query<(&Interaction, &QuizCompBuzzButton), (Changed<Interaction>, Without<QuizCompBackButton>)>,
    option_clicks: Query<(&Interaction, &QuizCompOptionButton), (Changed<Interaction>, Without<QuizCompBackButton>)>,
    back_clicks: Query<&Interaction, (Changed<Interaction>, With<QuizCompBackButton>)>,
    mut texts: Query<(&QuizCompText, &mut Text)>,
    mut option_texts: Query<(&QuizCompOptionButton, &Children)>,
    mut button_texts: Query<&mut Text, Without<QuizCompText>>,
) {
    if keys.just_pressed(KeyCode::Escape) || back_clicks.single().map_or(false, |i| *i == Interaction::Pressed) { commands.set_state(GameState::ClassicMenu); return; }
    if session.winner.is_some() {
        // esperar volver
        for (field, mut text) in &mut texts {
            if field.0 == QuizCompField::Status {
                *text = Text::new(format!("¡Gana Jugador {} con {} puntos!", session.winner.unwrap()+1, WIN_POINTS));
            }
        }
        return;
    }
    // buzz con teclas Q W E R para J1-4 o botones
    let mut buzz_by_key: Option<usize> = None;
    if keys.just_pressed(KeyCode::KeyQ) { buzz_by_key = Some(0); }
    if keys.just_pressed(KeyCode::KeyP) { buzz_by_key = Some(1); }
    if keys.just_pressed(KeyCode::KeyZ) { buzz_by_key = Some(2); }
    if keys.just_pressed(KeyCode::KeyM) { buzz_by_key = Some(3); }
    if buzz_by_key.is_some() && session.buzzed.is_none() { session.buzzed = buzz_by_key; }
    for (interaction, btn) in &buzz_clicks {
        if *interaction == Interaction::Pressed && session.buzzed.is_none() {
            session.buzzed = Some(btn.0);
        }
    }
    // responder
    let mut chosen: Option<usize> = None;
    for (interaction, btn) in &option_clicks { if *interaction == Interaction::Pressed { chosen = Some(btn.0); break; } }
    if chosen.is_none() {
        for (idx, code) in [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4].iter().enumerate() { if keys.just_pressed(*code) { chosen = Some(idx); break; } }
    }
    if let Some(idx) = chosen {
        if let Some(player) = session.buzzed {
            if idx == session.correct {
                session.scores[player] += 1;
                if session.scores[player] >= WIN_POINTS { session.winner = Some(player); }
                else { session.next_question(); }
            } else {
                session.buzzed = None;
            }
        }
    }
    for (field, mut text) in &mut texts {
        match field.0 {
            QuizCompField::Question => { *text = Text::new(session.question.clone()); }
            QuizCompField::Scores => { *text = Text::new(format!("Puntos: J1 {} - J2 {}  J3 {}  J4 {}  |  Primero en {} gana", session.scores[0], session.scores[1], session.scores[2], session.scores[3], WIN_POINTS)); }
            QuizCompField::Status => {
                *text = Text::new(if let Some(p)=session.buzzed { format!("¡Jugador {} ha reservado! Elige respuesta", p+1) } else { "¡Pulsad ¡Ya! para reservar turno (Q/P/Z/M)".to_string() });
            }
            _ => {}
        }
    }
    for (btn, children) in &mut option_texts {
        for child in children.iter() {
            if let Ok(mut text) = button_texts.get_mut(child) {
                *text = Text::new(format!("{}) {}", ['A','B','C','D'][btn.0], session.options[btn.0]));
            }
        }
    }
}
