//! Ruleta de la Fortuna — frase oculta + ruleta.

use bevy::prelude::*;
use rand::seq::SliceRandom;

use crate::game::GameState;
use crate::learning::screen_background;
use crate::i18n::tr;

const PHRASES_ES: [&str; 8] = ["CASA DE PAPEL", "DON QUIJOTE", "TORTILLA DE PATATAS", "VIAJE AL CENTRO DE LA TIERRA", "EL RATONCITO PEREZ", "BLANCA NIEVES", "CIEN AÑOS DE SOLEDAD", "ARROZ CON LECHE"];
const PHRASES_EN: [&str; 8] = ["HOUSE OF CARDS", "DON QUIXOTE", "SPANISH OMELETTE", "JOURNEY TO THE CENTER OF THE EARTH", "TOOTH FAIRY", "SNOW WHITE", "ONE HUNDRED YEARS OF SOLITUDE", "RICE PUDDING"];
const PHRASES_FR: [&str; 8] = ["MAISON DE PAPIER", "DON QUICHOTTE", "OMELETTE ESPAGNOLE", "VOYAGE AU CENTRE DE LA TERRE", "PETITE SOURIS", "BLANCHE NEIGE", "CENT ANS DE SOLITUDE", "RIZ AU LAIT"];

const WHEEL: [&str; 8] = ["100", "200", "300", "500", "Quiebra", "Pierde turno", "Comodín", "400"];

#[derive(Resource)]
struct RouletteSession {
    phrase: String,
    revealed: Vec<bool>,
    points: i32,
    wheel: String,
    guessed: Vec<char>,
    message: String,
    won: bool,
}

impl RouletteSession {
    fn new() -> Self {
        let mut rng = rand::thread_rng();
        let bank = match crate::i18n::language() {
            crate::i18n::Language::En => &PHRASES_EN,
            crate::i18n::Language::Fr => &PHRASES_FR,
            _ => &PHRASES_ES,
        };
        let phrase = bank.choose(&mut rng).unwrap().to_string();
        let revealed = phrase.chars().map(|c| !c.is_alphabetic()).collect();
        Self { phrase, revealed, points: 0, wheel: "—".to_string(), guessed: Vec::new(), message: "¡Gira la ruleta!".to_string(), won: false }
    }
    fn display(&self) -> String {
        self.phrase.chars().zip(self.revealed.iter()).map(|(c, r)| if *r { c } else if c==' ' { ' ' } else { '_' }).collect::<String>()
    }
}

#[derive(Component)]
struct RouletteUiRoot;
#[derive(Component)]
struct RouletteText(RouletteField);
#[derive(Clone, Copy, PartialEq, Eq)]
enum RouletteField { Title, Phrase, Wheel, Points, Message }
#[derive(Component)]
struct RouletteSpinButton;
#[derive(Component)]
struct RouletteLetterButton(char);
#[derive(Component)]
struct RouletteBackButton;

pub struct RoulettePlugin;
impl Plugin for RoulettePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::RouletteGame), spawn_roulette)
            .add_systems(OnExit(GameState::RouletteGame), cleanup_roulette)
            .add_systems(Update, update_roulette.run_if(in_state(GameState::RouletteGame)));
    }
}

fn spawn_roulette(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands.insert_resource(RouletteSession::new());
    commands
        .spawn((RouletteUiRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, screen_background(), Visibility::Visible, ZIndex(30)))
        .with_children(|overlay| {
            overlay.spawn((Node { flex_direction: FlexDirection::Column, width: Val::Px(720.0), padding: UiRect::axes(Val::Px(24.0), Val::Px(20.0)), row_gap: Val::Px(10.0), align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.07, 0.09, 0.18, 0.96)), BorderRadius::all(Val::Px(16.0)))).with_children(|panel| {
                panel.spawn((RouletteText(RouletteField::Title), Text::new("RULETA DE LA FORTUNA"), TextFont { font: font.clone(), font_size: 28.0, ..default() }, TextColor(Color::srgb(0.95, 0.85, 0.40))));
                panel.spawn((RouletteText(RouletteField::Phrase), Text::new("".to_string()), TextFont { font: font.clone(), font_size: 28.0, ..default() }, TextColor(Color::WHITE), TextLayout { linebreak: LineBreak::WordBoundary, ..default() }, Node { max_width: Val::Px(680.0), ..default() }));
                panel.spawn((RouletteText(RouletteField::Wheel), Text::new("Ruleta: —"), TextFont { font: font.clone(), font_size: 22.0, ..default() }, TextColor(Color::srgb(0.80, 0.95, 1.0))));
                panel.spawn((RouletteText(RouletteField::Points), Text::new("Puntos: 0"), TextFont { font: font.clone(), font_size: 20.0, ..default() }, TextColor(Color::WHITE)));
                panel.spawn((RouletteText(RouletteField::Message), Text::new("¡Gira la ruleta!"), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE)));
                panel.spawn((Button, RouletteSpinButton, Node { width: Val::Px(200.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.15, 0.42, 0.25)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Girar ruleta")), TextFont { font: font.clone(), font_size: 20.0, ..default() }, TextColor(Color::WHITE))); });
                panel.spawn(Node { flex_direction: FlexDirection::Row, flex_wrap: FlexWrap::Wrap, width: Val::Px(680.0), justify_content: JustifyContent::Center, column_gap: Val::Px(6.0), row_gap: Val::Px(6.0), ..default() }).with_children(|row| {
                    for ch in 'A'..='Z' {
                        row.spawn((Button, RouletteLetterButton(ch), Node { width: Val::Px(42.0), height: Val::Px(42.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.18, 0.22, 0.34)), BorderColor(Color::srgb(0.50, 0.55, 0.70)), BorderRadius::all(Val::Px(6.0)))).with_children(|b| { b.spawn((Text::new(ch.to_string()), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE))); });
                    }
                    row.spawn((Button, RouletteLetterButton('Ñ'), Node { width: Val::Px(42.0), height: Val::Px(42.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.18, 0.22, 0.34)), BorderColor(Color::srgb(0.50, 0.55, 0.70)), BorderRadius::all(Val::Px(6.0)))).with_children(|b| { b.spawn((Text::new("Ñ".to_string()), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE))); });
                });
                panel.spawn((Button, RouletteBackButton, Node { width: Val::Px(200.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Volver")), TextFont { font: font.clone(), font_size: 20.0, ..default() }, TextColor(Color::WHITE))); });
            });
        });
}

fn cleanup_roulette(mut commands: Commands, roots: Query<Entity, With<RouletteUiRoot>>) {
    for root in &roots { commands.entity(root).despawn(); }
    commands.remove_resource::<RouletteSession>();
}

fn update_roulette(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut session: ResMut<RouletteSession>,
    spin_clicks: Query<&Interaction, (Changed<Interaction>, With<RouletteSpinButton>)>,
    letter_clicks: Query<(&Interaction, &RouletteLetterButton), (Changed<Interaction>, Without<RouletteSpinButton>)>,
    back_clicks: Query<&Interaction, (Changed<Interaction>, With<RouletteBackButton>)>,
    mut texts: Query<(&RouletteText, &mut Text)>,
) {
    if keys.just_pressed(KeyCode::Escape) || back_clicks.single().map_or(false, |i| *i == Interaction::Pressed) { commands.set_state(GameState::ClassicMenu); return; }
    if session.won { return; }
    for interaction in &spin_clicks { if *interaction == Interaction::Pressed { let mut rng = rand::thread_rng(); let w = *WHEEL.choose(&mut rng).unwrap(); session.wheel = w.to_string(); session.message = match w { "Quiebra" => { session.points = 0; "¡Quiebra! Pierdes todo".to_string() }, "Pierde turno" => "¡Pierdes turno!".to_string(), _ => format!("¡{} puntos! Elige letra", w) }; } }
    let mut chosen: Option<char> = None;
    for (interaction, btn) in &letter_clicks { if *interaction == Interaction::Pressed { chosen = Some(btn.0); break; } }
    if chosen.is_none() {
        for code in [KeyCode::KeyA, KeyCode::KeyB, KeyCode::KeyC, KeyCode::KeyD, KeyCode::KeyE, KeyCode::KeyF, KeyCode::KeyG, KeyCode::KeyH, KeyCode::KeyI, KeyCode::KeyJ, KeyCode::KeyK, KeyCode::KeyL, KeyCode::KeyM, KeyCode::KeyN, KeyCode::KeyO, KeyCode::KeyP, KeyCode::KeyQ, KeyCode::KeyR, KeyCode::KeyS, KeyCode::KeyT, KeyCode::KeyU, KeyCode::KeyV, KeyCode::KeyW, KeyCode::KeyX, KeyCode::KeyY, KeyCode::KeyZ].iter() {
            if keys.just_pressed(*code) { if let Some(ch) = key_to_char(*code) { chosen = Some(ch); break; } }
        }
    }
    if let Some(ch) = chosen {
        if session.guessed.contains(&ch) { session.message = format!("Ya probaste la {}", ch); }
        else {
            session.guessed.push(ch);
            let mut found = 0;
            let phrase = session.phrase.clone();
            for (i, c) in phrase.chars().enumerate() { if c.to_ascii_uppercase() == ch { session.revealed[i] = true; found += 1; } }
            if found > 0 {
                let pts: i32 = session.wheel.parse().unwrap_or(0);
                session.points += pts * found as i32;
                session.message = format!("¡{} aparece {} veces! +{} pts", ch, found, pts * found as i32);
            } else {
                session.message = format!("La {} no está", ch);
            }
            if session.revealed.iter().all(|r| *r) { session.won = true; let pts = session.points; session.message = format!("¡Frase completada! Total: {} pts", pts); }
        }
    }
    for (field, mut text) in &mut texts {
        match field.0 {
            RouletteField::Phrase => { *text = Text::new(session.display()); }
            RouletteField::Wheel => { *text = Text::new(format!("Ruleta: {}", session.wheel)); }
            RouletteField::Points => { *text = Text::new(format!("Puntos: {}", session.points)); }
            RouletteField::Message => { *text = Text::new(session.message.clone()); }
            _ => {}
        }
    }
}
fn key_to_char(code: KeyCode) -> Option<char> {
    match code { KeyCode::KeyA=>'A', KeyCode::KeyB=>'B', KeyCode::KeyC=>'C', KeyCode::KeyD=>'D', KeyCode::KeyE=>'E', KeyCode::KeyF=>'F', KeyCode::KeyG=>'G', KeyCode::KeyH=>'H', KeyCode::KeyI=>'I', KeyCode::KeyJ=>'J', KeyCode::KeyK=>'K', KeyCode::KeyL=>'L', KeyCode::KeyM=>'M', KeyCode::KeyN=>'N', KeyCode::KeyO=>'O', KeyCode::KeyP=>'P', KeyCode::KeyQ=>'Q', KeyCode::KeyR=>'R', KeyCode::KeyS=>'S', KeyCode::KeyT=>'T', KeyCode::KeyU=>'U', KeyCode::KeyV=>'V', KeyCode::KeyW=>'W', KeyCode::KeyX=>'X', KeyCode::KeyY=>'Y', KeyCode::KeyZ=>'Z', _=>return None, }.into()
}
