//! Adivina el Número — 1..100, 10 intentos, pista mayor/menor.

use bevy::prelude::*;
use rand::Rng;

use super::screen_background;
use crate::audio::{play_click, play_success, Sfx};
use crate::game::GameState;
use crate::i18n::tr;

const MAX_ATTEMPTS: usize = 10;

#[derive(Resource)]
pub struct GuessSession {
    secret: u32,
    attempts: Vec<u32>,
    feedback: String,
    won: bool,
    done: bool,
}

#[derive(Component)]
struct GuessUiRoot;
#[derive(Component)]
struct GuessText(GuessField);
#[derive(Clone, Copy, PartialEq, Eq)]
enum GuessField { Title, Prompt, Feedback, History, ResultTitle, ResultDetail }
#[derive(Component)]
struct GuessInputText;
#[derive(Component)]
struct GuessNumberButton(u32);
#[derive(Component)]
struct GuessSubmitButton;
#[derive(Component)]
struct GuessBackButton;
#[derive(Component)]
struct GuessRestartButton;

pub struct GuessNumberPlugin;
impl Plugin for GuessNumberPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::GuessNumberGame), spawn_guess_ui)
            .add_systems(OnExit(GameState::GuessNumberGame), cleanup_guess)
            .add_systems(Update, update_guess.run_if(in_state(GameState::GuessNumberGame)));
    }
}

fn guess_text(parent: &mut ChildSpawnerCommands, field: GuessField, text: &str, size: f32, font: &Handle<Font>) {
    parent.spawn((GuessText(field), Text::new(text.to_string()), TextFont { font: font.clone(), font_size: size, ..default() }, TextColor(Color::WHITE), TextLayout { linebreak: LineBreak::WordBoundary, ..default() }, Node { max_width: Val::Px(700.0), ..default() }));
}

fn spawn_guess_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    let secret = rand::thread_rng().gen_range(1..=100);
    commands.insert_resource(GuessSession { secret, attempts: Vec::new(), feedback: "¡Adivina el número entre 1 y 100!".to_string(), won: false, done: false });
    commands
        .spawn((GuessUiRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, screen_background(), Visibility::Visible, ZIndex(30)))
        .with_children(|overlay| {
            overlay.spawn((Node { flex_direction: FlexDirection::Column, width: Val::Px(680.0), padding: UiRect::axes(Val::Px(28.0), Val::Px(24.0)), row_gap: Val::Px(12.0), align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.07, 0.09, 0.18, 0.96)), BorderRadius::all(Val::Px(16.0)))).with_children(|panel| {
                guess_text(panel, GuessField::Title, "ADIVINA EL NÚMERO", 28.0, &font);
                guess_text(panel, GuessField::Prompt, "Escribe un número (1-100) y pulsa Enviar", 20.0, &font);
                panel.spawn((GuessInputText, Text::new("".to_string()), TextFont { font: font.clone(), font_size: 32.0, ..default() }, TextColor(Color::srgb(0.80, 0.95, 1.0))));
                guess_text(panel, GuessField::Feedback, "¡Adivina el número entre 1 y 100!", 22.0, &font);
                guess_text(panel, GuessField::History, "Intentos: -", 18.0, &font);
                // Botones 0-9
                panel.spawn(Node { flex_direction: FlexDirection::Row, flex_wrap: FlexWrap::Wrap, width: Val::Px(400.0), justify_content: JustifyContent::Center, column_gap: Val::Px(8.0), row_gap: Val::Px(8.0), ..default() }).with_children(|row| {
                    for n in 0..=9 {
                        row.spawn((Button, GuessNumberButton(n), Node { width: Val::Px(60.0), height: Val::Px(50.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.15, 0.18, 0.28)), BorderColor(Color::srgb(0.50, 0.55, 0.70)), BorderRadius::all(Val::Px(8.0)))).with_children(|b| { b.spawn((Text::new(n.to_string()), TextFont { font: font.clone(), font_size: 22.0, ..default() }, TextColor(Color::WHITE))); });
                    }
                    row.spawn((Button, GuessNumberButton(100), Node { width: Val::Px(60.0), height: Val::Px(50.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.25, 0.20, 0.20)), BorderColor(Color::srgb(0.80, 0.50, 0.50)), BorderRadius::all(Val::Px(8.0)))).with_children(|b| { b.spawn((Text::new("⌫".to_string()), TextFont { font: font.clone(), font_size: 22.0, ..default() }, TextColor(Color::WHITE))); });
                });
                panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(12.0), ..default() }).with_children(|row| {
                    row.spawn((Button, GuessSubmitButton, Node { width: Val::Px(160.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.15, 0.42, 0.25)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Enviar")), TextFont { font: font.clone(), font_size: 20.0, ..default() }, TextColor(Color::WHITE))); });
                    row.spawn((Button, GuessRestartButton, Node { width: Val::Px(160.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Reiniciar")), TextFont { font: font.clone(), font_size: 20.0, ..default() }, TextColor(Color::WHITE))); });
                });
                panel.spawn((Button, GuessBackButton, Node { width: Val::Px(220.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Volver")), TextFont { font: font.clone(), font_size: 20.0, ..default() }, TextColor(Color::WHITE))); });
                guess_text(panel, GuessField::ResultTitle, "", 26.0, &font);
                guess_text(panel, GuessField::ResultDetail, "", 20.0, &font);
            });
        });
}

fn cleanup_guess(mut commands: Commands, roots: Query<Entity, With<GuessUiRoot>>) {
    for root in &roots { commands.entity(root).despawn(); }
    commands.remove_resource::<GuessSession>();
}

fn update_guess(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    sfx: Res<Sfx>,
    mut session: ResMut<GuessSession>,
    mut input: Query<&mut Text, (With<GuessInputText>, Without<GuessText>)>,
    mut texts: Query<(&GuessText, &mut Text, &mut TextColor), Without<GuessInputText>>,
    number_clicks: Query<(&Interaction, &GuessNumberButton), (Changed<Interaction>, Without<GuessBackButton>)>,
    submit_clicks: Query<&Interaction, (Changed<Interaction>, With<GuessSubmitButton>)>,
    back_clicks: Query<&Interaction, (Changed<Interaction>, With<GuessBackButton>)>,
    restart_clicks: Query<&Interaction, (Changed<Interaction>, With<GuessRestartButton>)>,
) {
    // estado typed local
    let mut typed = String::new();
    for text in &input { typed = text.0.clone(); }

    if keys.just_pressed(KeyCode::Escape) { commands.set_state(GameState::MathMenu); return; }
    if back_clicks.single().map_or(false, |i| *i == Interaction::Pressed) { commands.set_state(GameState::MathMenu); return; }
    if restart_clicks.single().map_or(false, |i| *i == Interaction::Pressed) {
        session.secret = rand::thread_rng().gen_range(1..=100);
        session.attempts.clear();
        session.feedback = "¡Nuevo número! Adivina entre 1 y 100".to_string();
        session.won = false;
        session.done = false;
        for mut text in &mut input { *text = Text::new("".to_string()); }
        return;
    }
    if session.done {
        for (field, mut text, mut color) in &mut texts {
            if field.0 == GuessField::ResultTitle {
                *text = Text::new(if session.won { tr("¡Has ganado!") } else { tr("¡Se acabaron los intentos!") });
                *color = TextColor(if session.won { Color::srgb(0.40, 0.90, 0.50) } else { Color::srgb(0.95, 0.55, 0.30) });
            }
            if field.0 == GuessField::ResultDetail {
                *text = Text::new(if session.won { format!("Lo lograste en {} intentos. Número: {}", session.attempts.len(), session.secret) } else { format!("El número era {}. ¡Intenta de nuevo!", session.secret) });
            }
        }
        return;
    }

    // entrada numérica
    for (interaction, btn) in &number_clicks {
        if *interaction == Interaction::Pressed {
            play_click(&mut commands, &sfx);
            if btn.0 == 100 {
                typed.pop();
            } else {
                if typed.len() < 3 {
                    typed.push_str(&btn.0.to_string());
                }
            }
            for mut text in &mut input { *text = Text::new(typed.clone()); }
        }
    }
    for key in keys.get_just_pressed() {
        if let Some(ch) = key_char(*key) {
            if typed.len() < 3 { typed.push(ch); for mut text in &mut input { *text = Text::new(typed.clone()); } }
        }
        if *key == KeyCode::Backspace {
            typed.pop();
            for mut text in &mut input { *text = Text::new(typed.clone()); }
        }
    }

    let submit = submit_clicks.single().map_or(false, |i| *i == Interaction::Pressed) || keys.just_pressed(KeyCode::Enter);
    if submit {
        if let Ok(num) = typed.trim().parse::<u32>() {
            if num < 1 || num > 100 {
                session.feedback = "El número debe estar entre 1 y 100".to_string();
            } else if session.attempts.contains(&num) {
                session.feedback = format!("Ya probaste el {}", num);
            } else {
                session.attempts.push(num);
                for mut text in &mut input { *text = Text::new("".to_string()); }
                if num == session.secret {
                    session.feedback = format!("¡Correcto! Era el {}", session.secret);
                    session.won = true;
                    session.done = true;
                    play_success(&mut commands, &sfx);
                } else if session.attempts.len() >= MAX_ATTEMPTS {
                    session.feedback = format!("¡Agotaste los intentos! Era el {}", session.secret);
                    session.done = true;
                } else if num < session.secret {
                    session.feedback = format!("{} es pequeño. Intenta más alto. ({}/{})", num, session.attempts.len(), MAX_ATTEMPTS);
                } else {
                    session.feedback = format!("{} es grande. Intenta más bajo. ({}/{})", num, session.attempts.len(), MAX_ATTEMPTS);
                }
            }
        } else {
            session.feedback = "Escribe un número".to_string();
        }
    }

    for (field, mut text, mut color) in &mut texts {
        match field.0 {
            GuessField::Feedback => { *text = Text::new(session.feedback.clone()); *color = TextColor(if session.won { Color::srgb(0.40, 0.90, 0.50) } else { Color::WHITE }); }
            GuessField::History => { *text = Text::new(if session.attempts.is_empty() { "Intentos: -".to_string() } else { format!("Intentos: {}", session.attempts.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(", ")) }); }
            _ => {}
        }
    }
}

fn key_char(key: KeyCode) -> Option<char> {
    match key {
        KeyCode::Digit0 => Some('0'),
        KeyCode::Digit1 => Some('1'),
        KeyCode::Digit2 => Some('2'),
        KeyCode::Digit3 => Some('3'),
        KeyCode::Digit4 => Some('4'),
        KeyCode::Digit5 => Some('5'),
        KeyCode::Digit6 => Some('6'),
        KeyCode::Digit7 => Some('7'),
        KeyCode::Digit8 => Some('8'),
        KeyCode::Digit9 => Some('9'),
        _ => None,
    }
}
