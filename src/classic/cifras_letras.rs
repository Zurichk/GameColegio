//! Cifras y Letras — modo Cifras (alcanza objetivo) y Letras (palabra más larga).

use bevy::prelude::*;
use rand::seq::SliceRandom;
use rand::Rng;

use crate::game::GameState;
use crate::learning::screen_background;
use crate::i18n::tr;

#[derive(Resource, Clone, Copy, PartialEq, Eq)]
enum Mode { Cifras, Letras }

#[derive(Resource)]
struct CountdownSession {
    mode: Mode,
    // Cifras
    numbers: [u32; 6],
    target: u32,
    // Letras
    letters: [char; 9],
    options: Vec<String>,
    correct: usize,
    feedback: String,
    won: bool,
}

impl CountdownSession {
    fn new_cifras() -> Self {
        let mut rng = rand::thread_rng();
        let numbers = [rng.gen_range(1..=10), rng.gen_range(1..=10), rng.gen_range(1..=10), rng.gen_range(1..=25), rng.gen_range(1..=50), rng.gen_range(1..=100)];
        let target = rng.gen_range(100..=999);
        // generar opciones: una es alcanzable (suma de 2 números cerca), otras random
        let close = (numbers[0] + numbers[1] + numbers[2]).min(999);
        let mut opts = vec![close.to_string(), (close+10).to_string(), (close as i32 -10).max(0).to_string(), rng.gen_range(100..=999).to_string()];
        opts.shuffle(&mut rng);
        let correct = opts.iter().position(|o| o == &close.to_string()).unwrap_or(0);
        Self { mode: Mode::Cifras, numbers, target, letters: ['A';9], options: opts, correct, feedback: format!("Objetivo: {}", target), won: false }
    }
    fn new_letras() -> Self {
        let mut rng = rand::thread_rng();
        let alphabet: Vec<char> = "AEIOU AEIOU BCDFGHJKLMNPQRSTVWXYZ".chars().filter(|c| *c!=' ').collect();
        let mut letters = ['A';9];
        for i in 0..9 { letters[i] = *alphabet.choose(&mut rng).unwrap(); }
        // banco pequeño de palabras válidas que se pueden formar con esas letras (simplificado: usamos palabras del banco y filtramos)
        let bank = ["CASA", "MESA", "SILLA", "LIBRO", "ARBOL", "FLOR", "SOL", "LUNA", "CASA", "PERRO"];
        let mut candidates: Vec<String> = bank.iter().filter(|w| can_form(w, &letters)).map(|s| s.to_string()).collect();
        if candidates.is_empty() { candidates.push("SOL".to_string()); }
        candidates.shuffle(&mut rng);
        let correct_word = candidates[0].clone();
        let mut opts = vec![correct_word.clone()];
        let mut distractors = vec!["CASA", "MESA", "ROSA", "LUZ", "PAN", "MAR"];
        distractors.retain(|d| d != &correct_word.as_str());
        distractors.shuffle(&mut rng);
        while opts.len() < 4 { opts.push(distractors.pop().unwrap_or("SOL").to_string()); }
        opts.shuffle(&mut rng);
        let correct = opts.iter().position(|o| o == &correct_word).unwrap_or(0);
        Self { mode: Mode::Letras, numbers: [0;6], target: 0, letters, options: opts, correct, feedback: "Forma la palabra más larga".to_string(), won: false }
    }
}

fn can_form(word: &str, letters: &[char;9]) -> bool {
    let mut pool: Vec<char> = letters.iter().copied().collect();
    for ch in word.chars() {
        if let Some(pos) = pool.iter().position(|&c| c==ch) { pool.remove(pos); } else { return false; }
    }
    true
}

#[derive(Component)]
struct CountdownUiRoot;
#[derive(Component)]
struct CountdownText(CountdownField);
#[derive(Clone, Copy, PartialEq, Eq)]
enum CountdownField { Title, Prompt, Feedback }
#[derive(Component)]
struct CountdownOptionButton(usize);
#[derive(Component)]
struct CountdownModeButton(Mode);
#[derive(Component)]
struct CountdownBackButton;

pub struct CountdownPlugin;
impl Plugin for CountdownPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::CountdownGame), spawn_countdown)
            .add_systems(OnExit(GameState::CountdownGame), cleanup_countdown)
            .add_systems(Update, update_countdown.run_if(in_state(GameState::CountdownGame)));
    }
}

fn spawn_countdown(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands.insert_resource(CountdownSession::new_cifras());
    commands
        .spawn((CountdownUiRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, screen_background(), Visibility::Visible, ZIndex(30)))
        .with_children(|overlay| {
            overlay.spawn((Node { flex_direction: FlexDirection::Column, width: Val::Px(680.0), padding: UiRect::axes(Val::Px(28.0), Val::Px(24.0)), row_gap: Val::Px(12.0), align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.07, 0.09, 0.18, 0.96)), BorderRadius::all(Val::Px(16.0)))).with_children(|panel| {
                panel.spawn((CountdownText(CountdownField::Title), Text::new("CIFRAS Y LETRAS"), TextFont { font: font.clone(), font_size: 28.0, ..default() }, TextColor(Color::srgb(0.95, 0.85, 0.40))));
                panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(12.0), ..default() }).with_children(|row| {
                    row.spawn((Button, CountdownModeButton(Mode::Cifras), Node { width: Val::Px(160.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new("Cifras"), TextFont { font: font.clone(), font_size: 20.0, ..default() }, TextColor(Color::WHITE))); });
                    row.spawn((Button, CountdownModeButton(Mode::Letras), Node { width: Val::Px(160.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new("Letras"), TextFont { font: font.clone(), font_size: 20.0, ..default() }, TextColor(Color::WHITE))); });
                });
                panel.spawn((CountdownText(CountdownField::Prompt), Text::new(""), TextFont { font: font.clone(), font_size: 22.0, ..default() }, TextColor(Color::WHITE), TextLayout { linebreak: LineBreak::WordBoundary, ..default() }, Node { max_width: Val::Px(640.0), ..default() }));
                for i in 0..4 {
                    panel.spawn((Button, CountdownOptionButton(i), Node { width: Val::Px(600.0), height: Val::Px(46.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.15, 0.18, 0.28)), BorderColor(Color::srgb(0.50, 0.55, 0.70)), BorderRadius::all(Val::Px(8.0)))).with_children(|b| { b.spawn((Text::new("".to_string()), TextFont { font: font.clone(), font_size: 20.0, ..default() }, TextColor(Color::WHITE))); });
                }
                panel.spawn((CountdownText(CountdownField::Feedback), Text::new(""), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE)));
                panel.spawn((Button, CountdownBackButton, Node { width: Val::Px(200.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Volver")), TextFont { font: font.clone(), font_size: 20.0, ..default() }, TextColor(Color::WHITE))); });
            });
        });
}

fn cleanup_countdown(mut commands: Commands, roots: Query<Entity, With<CountdownUiRoot>>) {
    for root in &roots { commands.entity(root).despawn(); }
    commands.remove_resource::<CountdownSession>();
}

fn update_countdown(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut session: ResMut<CountdownSession>,
    back_clicks: Query<&Interaction, (Changed<Interaction>, With<CountdownBackButton>)>,
    mode_clicks: Query<(&Interaction, &CountdownModeButton), (Changed<Interaction>, Without<CountdownBackButton>)>,
    option_clicks: Query<(&Interaction, &CountdownOptionButton), (Changed<Interaction>, Without<CountdownBackButton>)>,
    mut texts: Query<(&CountdownText, &mut Text)>,
    mut option_texts: Query<(&CountdownOptionButton, &Children)>,
    mut button_texts: Query<&mut Text, Without<CountdownText>>,
) {
    if keys.just_pressed(KeyCode::Escape) || back_clicks.single().map_or(false, |i| *i == Interaction::Pressed) { commands.set_state(GameState::ClassicMenu); return; }
    for (interaction, btn) in &mode_clicks {
        if *interaction == Interaction::Pressed {
            *session = match btn.0 { Mode::Cifras => CountdownSession::new_cifras(), Mode::Letras => CountdownSession::new_letras() };
        }
    }
    let mut chosen: Option<usize> = None;
    for (interaction, btn) in &option_clicks { if *interaction == Interaction::Pressed { chosen = Some(btn.0); break; } }
    if chosen.is_none() { for (idx, code) in [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4].iter().enumerate() { if keys.just_pressed(*code) { chosen = Some(idx); break; } } }
    if let Some(idx) = chosen {
        if idx == session.correct { session.feedback = "¡Correcto!".to_string(); session.won = true; } else { session.feedback = format!("Fallaste. Era {}", session.options[session.correct]); }
    }
    for (field, mut text) in &mut texts {
        match field.0 {
            CountdownField::Prompt => {
                *text = Text::new(match session.mode { Mode::Cifras => format!("Números: {:?} → Objetivo: {}", session.numbers, session.target), Mode::Letras => format!("Letras: {} → Elige la palabra más larga", session.letters.iter().collect::<String>()) });
            }
            CountdownField::Feedback => { *text = Text::new(session.feedback.clone()); }
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
