//! La Oca — tablero 63 con ocas, puentes y trampas.

use bevy::prelude::*;
use rand::Rng;

use crate::game::GameState;
use crate::learning::screen_background;
use crate::i18n::tr;

const GOAL: usize = 63;
const OCAS: [usize; 13] = [5, 9, 14, 18, 23, 27, 32, 36, 41, 45, 50, 54, 59];

#[derive(Resource, Clone)]
struct OcaSession {
    pos_player: usize,
    pos_cpu: usize,
    turn: char, // 'P' player, 'C' cpu
    dice: u8,
    skip_player: u8,
    skip_cpu: u8,
    winner: Option<char>,
    message: String,
}

impl OcaSession {
    fn new() -> Self {
        Self { pos_player: 0, pos_cpu: 0, turn: 'P', dice: 1, skip_player: 0, skip_cpu: 0, winner: None, message: "¡Tira el dado!".to_string() }
    }
    fn next_oca(from: usize) -> Option<usize> {
        OCAS.iter().find(|&&o| o > from).copied()
    }
    fn apply_special(pos: usize) -> (usize, String, u8) {
        if OCAS.contains(&pos) {
            if let Some(next) = Self::next_oca(pos) {
                return (next, format!("¡Oca en {}! De oca a oca y tiro porque me toca → {}", pos, next), 0);
            }
        }
        match pos {
            6 => (12, "¡Puente 6→12!".to_string(), 0),
            12 => (6, "¡Puente 12→6!".to_string(), 0),
            19 => (19, "¡Posada! Pierdes 1 turno".to_string(), 1),
            31 => (31, "¡Pozo! Pierdes 2 turnos".to_string(), 2),
            42 => (30, "¡Laberinto 42→30!".to_string(), 0),
            56 => (56, "¡Cárcel! Pierdes 2 turnos".to_string(), 2),
            58 => (0, "¡Muerte 58→0! Vuelves al inicio".to_string(), 0),
            _ => (pos, "".to_string(), 0),
        }
    }
    fn move_player(&mut self, is_player: bool) {
        if self.winner.is_some() { return; }
        let dice = rand::thread_rng().gen_range(1..=6);
        self.dice = dice;
        let (pos, skip) = if is_player { (self.pos_player, &mut self.skip_player) } else { (self.pos_cpu, &mut self.skip_cpu) };
        // check skip
        if *skip > 0 {
            *skip -= 1;
            self.message = format!("{} pierde turno (quedan {})", if is_player { "Tú" } else { "CPU" }, *skip);
            return;
        }
        let mut new_pos = pos + dice as usize;
        if new_pos > GOAL {
            new_pos = GOAL - (new_pos - GOAL);
            self.message = format!("¡Rebote! {} + {} se pasa, vuelve a {}", pos, dice, new_pos);
        } else {
            self.message = format!("{} saca {} y va a {}", if is_player { "Tú" } else { "CPU" }, dice, new_pos);
        }
        let (final_pos, extra_msg, extra_skip) = Self::apply_special(new_pos);
        if extra_msg.contains("Pierdes") {
            if is_player { self.skip_player = extra_skip; } else { self.skip_cpu = extra_skip; }
        }
        if extra_msg.is_empty() {
            // no special
        } else if extra_msg.contains("De oca") || extra_msg.contains("Puente") || extra_msg.contains("Laberinto") || extra_msg.contains("Muerte") {
            new_pos = final_pos;
            self.message = format!("{} — {}", self.message, extra_msg);
        } else {
            self.message = format!("{} — {}", self.message, extra_msg);
            new_pos = final_pos;
        }
        // handle oca extra jump already done
        if is_player { self.pos_player = new_pos; } else { self.pos_cpu = new_pos; }
        if new_pos == GOAL {
            self.winner = Some(if is_player { 'P' } else { 'C' });
            self.message = format!("¡{} gana! Llegó a 63", if is_player { "Tú" } else { "CPU" });
        } else if extra_msg.contains("tiro porque me toca") {
            // oca gives extra turn, don't switch turn
            self.message = format!("{} — ¡tiras otra vez!", self.message);
        } else {
            self.turn = if is_player { 'C' } else { 'P' };
        }
    }
}

#[derive(Component)]
struct OcaUiRoot;
#[derive(Component)]
struct OcaText(OcaField);
#[derive(Clone, Copy, PartialEq, Eq)]
enum OcaField { Title, Status, Dice, Positions }
#[derive(Component)]
struct OcaRollButton;
#[derive(Component)]
struct OcaBackButton;
#[derive(Component)]
struct OcaRestartButton;

pub struct OcaPlugin;
impl Plugin for OcaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::OcaGame), spawn_oca)
            .add_systems(OnExit(GameState::OcaGame), cleanup_oca)
            .add_systems(Update, update_oca.run_if(in_state(GameState::OcaGame)));
    }
}

fn spawn_oca(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands.insert_resource(OcaSession::new());
    commands
        .spawn((OcaUiRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, screen_background(), Visibility::Visible, ZIndex(30)))
        .with_children(|overlay| {
            overlay.spawn((Node { flex_direction: FlexDirection::Column, width: Val::Px(780.0), padding: UiRect::axes(Val::Px(24.0), Val::Px(20.0)), row_gap: Val::Px(10.0), align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.07, 0.09, 0.18, 0.96)), BorderRadius::all(Val::Px(16.0)))).with_children(|panel| {
                panel.spawn((OcaText(OcaField::Title), Text::new("LA OCA — 63 casillas"), TextFont { font: font.clone(), font_size: 28.0, ..default() }, TextColor(Color::srgb(0.95, 0.85, 0.40))));
                panel.spawn((OcaText(OcaField::Status), Text::new("¡Tira el dado!"), TextFont { font: font.clone(), font_size: 20.0, ..default() }, TextColor(Color::WHITE), TextLayout { linebreak: LineBreak::WordBoundary, ..default() }, Node { max_width: Val::Px(720.0), ..default() }));
                panel.spawn((OcaText(OcaField::Dice), Text::new("Dado: -"), TextFont { font: font.clone(), font_size: 22.0, ..default() }, TextColor(Color::srgb(0.80, 0.95, 1.0))));
                panel.spawn((OcaText(OcaField::Positions), Text::new("Tú: 0  CPU: 0"), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE)));
                // tablero 7x9 =63
                panel.spawn(Node { display: Display::Grid, grid_template_columns: vec![GridTrack::px(40.0); 9], grid_template_rows: vec![GridTrack::px(40.0); 7], column_gap: Val::Px(4.0), row_gap: Val::Px(4.0), ..default() }).with_children(|grid| {
                    for n in 1..=63 {
                        let is_oca = OCAS.contains(&n);
                        let bg = if n==63 { Color::srgb(0.85, 0.75, 0.20) } else if is_oca { Color::srgb(0.20, 0.60, 0.20) } else if [6,12,19,31,42,56,58].contains(&n) { Color::srgb(0.60, 0.30, 0.20) } else { Color::srgb(0.15, 0.18, 0.28) };
                        grid.spawn((Node { width: Val::Px(40.0), height: Val::Px(40.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(1.0)), ..default() }, BackgroundColor(bg), BorderRadius::all(Val::Px(6.0)))).with_children(|c| { c.spawn((Text::new(n.to_string()), TextFont { font: font.clone(), font_size: 12.0, ..default() }, TextColor(Color::WHITE))); });
                    }
                });
                panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(12.0), ..default() }).with_children(|row| {
                    row.spawn((Button, OcaRollButton, Node { width: Val::Px(160.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.15, 0.42, 0.25)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Tirar dado")), TextFont { font: font.clone(), font_size: 20.0, ..default() }, TextColor(Color::WHITE))); });
                    row.spawn((Button, OcaRestartButton, Node { width: Val::Px(160.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Reiniciar")), TextFont { font: font.clone(), font_size: 20.0, ..default() }, TextColor(Color::WHITE))); });
                    row.spawn((Button, OcaBackButton, Node { width: Val::Px(160.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Volver")), TextFont { font: font.clone(), font_size: 20.0, ..default() }, TextColor(Color::WHITE))); });
                });
            });
        });
}

fn cleanup_oca(mut commands: Commands, roots: Query<Entity, With<OcaUiRoot>>) {
    for root in &roots { commands.entity(root).despawn(); }
    commands.remove_resource::<OcaSession>();
}

fn update_oca(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut session: ResMut<OcaSession>,
    roll_clicks: Query<&Interaction, (Changed<Interaction>, With<OcaRollButton>)>,
    back_clicks: Query<&Interaction, (Changed<Interaction>, With<OcaBackButton>)>,
    restart_clicks: Query<&Interaction, (Changed<Interaction>, With<OcaRestartButton>)>,
    mut texts: Query<(&OcaText, &mut Text)>,
    time: Res<Time>,
) {
    if keys.just_pressed(KeyCode::Escape) { commands.set_state(GameState::ClassicMenu); return; }
    if back_clicks.single().map_or(false, |i| *i == Interaction::Pressed) { commands.set_state(GameState::ClassicMenu); return; }
    if restart_clicks.single().map_or(false, |i| *i == Interaction::Pressed) { *session = OcaSession::new(); }
    if session.winner.is_some() { /* no moves */ } else if session.turn == 'P' {
        let mut clicked = false;
        for interaction in &roll_clicks { if *interaction == Interaction::Pressed { clicked = true; break; } }
        if clicked {
            session.move_player(true);
            // if still player's turn due to oca, stay; else CPU will move next frame
        }
    } else {
        // CPU turn with delay 0.8s
        let dt = time.delta_secs();
        // simple timer: use dice as timer hack? Instead just move immediately with 50% chance each frame
        // For now, move CPU instantly after player's move if turn is C
        // Add small delay via random
        if rand::thread_rng().gen_bool(0.03) {
            session.move_player(false);
        }
        let _ = dt;
    }
    for (field, mut text) in &mut texts {
        match field.0 {
            OcaField::Status => { *text = Text::new(session.message.clone()); }
            OcaField::Dice => { *text = Text::new(format!("Dado: {}", session.dice)); }
            OcaField::Positions => { *text = Text::new(format!("Tú: {}  CPU: {}  Turno: {}", session.pos_player, session.pos_cpu, if session.turn=='P' {"Tú"} else {"CPU"})); }
            _ => {}
        }
    }
}
