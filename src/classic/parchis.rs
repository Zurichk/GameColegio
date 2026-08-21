//! Parchís simplificado — 1 ficha por jugador, tablero 40 casillas, sacar con 6.

use bevy::prelude::*;
use rand::Rng;

use crate::game::GameState;
use crate::learning::screen_background;
use crate::i18n::tr;

const GOAL: usize = 40;
const SAFE: [usize; 4] = [0, 10, 20, 30];

#[derive(Resource, Clone)]
struct ParchisSession {
    pos: [Option<usize>; 4], // None = en casa
    turn: usize, // 0 = jugador, 1-3 CPU
    dice: u8,
    winner: Option<usize>,
    message: String,
}

impl ParchisSession {
    fn new() -> Self {
        Self { pos: [None; 4], turn: 0, dice: 1, winner: None, message: "¡Necesitas un 6 para salir de casa!".to_string() }
    }
    fn is_safe(pos: usize) -> bool { SAFE.contains(&pos) }
    fn move_turn(&mut self, is_player: bool) {
        if self.winner.is_some() { return; }
        let dice = rand::thread_rng().gen_range(1..=6);
        self.dice = dice;
        let idx = self.turn;
        let cur = self.pos[idx];
        let player_name = if is_player { "Tú".to_string() } else { format!("CPU {}", idx) };
        let new_pos = match cur {
            None => {
                if dice == 6 {
                    self.message = format!("{} saca 6 y sale a la casilla 0", player_name);
                    Some(0)
                } else {
                    self.message = format!("{} saca {} y necesita un 6", player_name, dice);
                    None
                }
            },
            Some(p) => {
                let mut np = p + dice as usize;
                if np > GOAL { np = GOAL - (np - GOAL); self.message = format!("¡Rebote! {} + {} → {}", p, dice, np); } else { self.message = format!("{} avanza {} → {}", player_name, dice, np); }
                Some(np)
            }
        };
        if let Some(np) = new_pos {
            // comer
            for (other_idx, other_pos) in self.pos.iter_mut().enumerate() {
                if other_idx != idx {
                    if let Some(op) = *other_pos {
                        if op == np && !Self::is_safe(np) {
                            *other_pos = None;
                            self.message = format!("{} ¡Comes a CPU {} y lo mandas a casa!", self.message, other_idx);
                        }
                    }
                }
            }
            self.pos[idx] = Some(np);
            if np == GOAL {
                self.winner = Some(idx);
                let winner_name = if is_player { "Tú".to_string() } else { format!("CPU {}", idx) };
                self.message = format!("¡{} gana!", winner_name);
                return;
            }
            if dice == 6 {
                self.message = format!("{} ¡6! Tiras otra vez", self.message);
                return;
            }
        }
        self.turn = (self.turn + 1) % 4;
    }
}

#[derive(Component)]
struct ParchisUiRoot;
#[derive(Component)]
struct ParchisText(ParchisField);
#[derive(Clone, Copy, PartialEq, Eq)]
enum ParchisField { Title, Status, Dice, Positions }
#[derive(Component)]
struct ParchisRollButton;
#[derive(Component)]
struct ParchisBackButton;
#[derive(Component)]
struct ParchisRestartButton;

pub struct ParchisPlugin;
impl Plugin for ParchisPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::ParchisGame), spawn_parchis)
            .add_systems(OnExit(GameState::ParchisGame), cleanup_parchis)
            .add_systems(Update, update_parchis.run_if(in_state(GameState::ParchisGame)));
    }
}

fn spawn_parchis(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands.insert_resource(ParchisSession::new());
    commands.spawn((ParchisUiRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, screen_background(), Visibility::Visible, ZIndex(30))).with_children(|overlay| {
        overlay.spawn((Node { flex_direction: FlexDirection::Column, width: Val::Px(720.0), padding: UiRect::axes(Val::Px(24.0), Val::Px(20.0)), row_gap: Val::Px(10.0), align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.07, 0.09, 0.18, 0.96)), BorderRadius::all(Val::Px(16.0)))).with_children(|panel| {
            panel.spawn((ParchisText(ParchisField::Title), Text::new("PARCHÍS — 1 ficha"), TextFont { font: font.clone(), font_size: 28.0, ..default() }, TextColor(Color::srgb(0.95, 0.85, 0.40))));
            panel.spawn((ParchisText(ParchisField::Status), Text::new("¡Necesitas un 6 para salir!"), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE), TextLayout { linebreak: LineBreak::WordBoundary, ..default() }, Node { max_width: Val::Px(680.0), ..default() }));
            panel.spawn((ParchisText(ParchisField::Dice), Text::new("Dado: -"), TextFont { font: font.clone(), font_size: 20.0, ..default() }, TextColor(Color::srgb(0.80, 0.95, 1.0))));
            panel.spawn((ParchisText(ParchisField::Positions), Text::new("Tú: Casa  CPU1: Casa  CPU2: Casa  CPU3: Casa"), TextFont { font: font.clone(), font_size: 16.0, ..default() }, TextColor(Color::WHITE)));
            panel.spawn(Node { display: Display::Grid, grid_template_columns: vec![GridTrack::px(36.0); 10], grid_template_rows: vec![GridTrack::px(36.0); 4], column_gap: Val::Px(4.0), row_gap: Val::Px(4.0), ..default() }).with_children(|grid| {
                for n in 0..=40 {
                    let is_safe = SAFE.contains(&n);
                    let bg = if n==40 { Color::srgb(0.85, 0.75, 0.20) } else if is_safe { Color::srgb(0.20, 0.60, 0.20) } else { Color::srgb(0.15, 0.18, 0.28) };
                    grid.spawn((Node { width: Val::Px(36.0), height: Val::Px(36.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(1.0)), ..default() }, BackgroundColor(bg), BorderRadius::all(Val::Px(6.0)))).with_children(|c| { c.spawn((Text::new(n.to_string()), TextFont { font: font.clone(), font_size: 10.0, ..default() }, TextColor(Color::WHITE))); });
                }
            });
            panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(12.0), ..default() }).with_children(|row| {
                row.spawn((Button, ParchisRollButton, Node { width: Val::Px(160.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.15, 0.42, 0.25)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Tirar dado")), TextFont { font: font.clone(), font_size: 20.0, ..default() }, TextColor(Color::WHITE))); });
                row.spawn((Button, ParchisRestartButton, Node { width: Val::Px(160.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Reiniciar")), TextFont { font: font.clone(), font_size: 20.0, ..default() }, TextColor(Color::WHITE))); });
                row.spawn((Button, ParchisBackButton, Node { width: Val::Px(160.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Volver")), TextFont { font: font.clone(), font_size: 20.0, ..default() }, TextColor(Color::WHITE))); });
            });
        });
    });
}

fn cleanup_parchis(mut commands: Commands, roots: Query<Entity, With<ParchisUiRoot>>) {
    for root in &roots { commands.entity(root).despawn(); }
    commands.remove_resource::<ParchisSession>();
}

fn update_parchis(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut session: ResMut<ParchisSession>,
    roll_clicks: Query<&Interaction, (Changed<Interaction>, With<ParchisRollButton>)>,
    back_clicks: Query<&Interaction, (Changed<Interaction>, With<ParchisBackButton>)>,
    restart_clicks: Query<&Interaction, (Changed<Interaction>, With<ParchisRestartButton>)>,
    mut texts: Query<(&ParchisText, &mut Text)>,
) {
    if keys.just_pressed(KeyCode::Escape) { commands.set_state(GameState::ClassicMenu); return; }
    if back_clicks.single().map_or(false, |i| *i == Interaction::Pressed) { commands.set_state(GameState::ClassicMenu); return; }
    if restart_clicks.single().map_or(false, |i| *i == Interaction::Pressed) { *session = ParchisSession::new(); }
    if session.winner.is_some() { /* no move */ } else if session.turn == 0 {
        let mut clicked = false;
        for interaction in &roll_clicks { if *interaction == Interaction::Pressed { clicked = true; break; } }
        if clicked { session.move_turn(true); }
    } else {
        // CPU auto
        if rand::thread_rng().gen_bool(0.02) {
            let is_cpu = session.turn != 0;
            if is_cpu { session.move_turn(false); }
        }
    }
    for (field, mut text) in &mut texts {
        match field.0 {
            ParchisField::Status => { *text = Text::new(session.message.clone()); }
            ParchisField::Dice => {
                let turn_name = if session.turn==0 { "Tú".to_string() } else { format!("CPU {}", session.turn) };
                *text = Text::new(format!("Dado: {}  Turno: {}", session.dice, turn_name));
            }
            ParchisField::Positions => {
                let fmt = |opt: Option<usize>| opt.map(|p| p.to_string()).unwrap_or("Casa".to_string());
                *text = Text::new(format!("Tú: {}  CPU1: {}  CPU2: {}  CPU3: {}", fmt(session.pos[0]), fmt(session.pos[1]), fmt(session.pos[2]), fmt(session.pos[3])));
            }
            _ => {}
        }
    }
}
