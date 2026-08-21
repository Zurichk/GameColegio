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
#[derive(Component)]
struct ParchisBoardCell(usize);

fn parchis_board_position(index: usize) -> (usize, usize) {
    let mut path = Vec::with_capacity(40);
    for row in 0..3 { for column in 3..6 { path.push((column, row)); } }
    for column in 6..9 { for row in 3..6 { path.push((column, row)); } }
    for row in (6..9).rev() { for column in (3..6).rev() { path.push((column, row)); } }
    for column in (0..3).rev() { for row in (3..6).rev() { path.push((column, row)); } }
    path.extend([(3, 3), (5, 3), (5, 5), (3, 5)]);
    path.push((4, 4));
    path[index]
}

#[cfg(test)]
mod tests {
    use super::parchis_board_position;
    use std::collections::HashSet;

    #[test]
    fn parchis_path_has_one_position_per_cell() {
        let positions: Vec<_> = (0..=40).map(parchis_board_position).collect();
        assert_eq!(positions.len(), 41);
        assert_eq!(positions.iter().collect::<HashSet<_>>().len(), 41);
    }
}

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
            panel.spawn((Node { position_type: PositionType::Relative, width: Val::Px(480.0), height: Val::Px(480.0), ..default() }, BackgroundColor(Color::srgb(0.28, 0.15, 0.08)), BorderRadius::all(Val::Px(24.0)))).with_children(|board| {
                for (left, top, color) in [
                    (Val::Px(12.0), Val::Px(12.0), Color::srgb(0.78, 0.20, 0.22)),
                    (Val::Px(312.0), Val::Px(12.0), Color::srgb(0.96, 0.72, 0.16)),
                    (Val::Px(12.0), Val::Px(312.0), Color::srgb(0.18, 0.42, 0.78)),
                    (Val::Px(312.0), Val::Px(312.0), Color::srgb(0.22, 0.66, 0.34)),
                ] {
                    board.spawn((Node { position_type: PositionType::Absolute, left, top, width: Val::Px(144.0), height: Val::Px(144.0), ..default() }, BackgroundColor(color), BorderRadius::all(Val::Px(18.0))));
                }
                board.spawn((Node { position_type: PositionType::Absolute, left: Val::Px(186.0), top: Val::Px(186.0), width: Val::Px(96.0), height: Val::Px(96.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgb(0.86, 0.62, 0.16)), BorderRadius::all(Val::Px(48.0)))).with_children(|goal| {
                    goal.spawn((Text::new("META"), TextFont { font: font.clone(), font_size: 13.0, ..default() }, TextColor(Color::WHITE)));
                });
                for n in 0..=40 {
                    let (column, row) = parchis_board_position(n);
                    let is_safe = SAFE.contains(&n);
                    let bg = if n==40 { Color::srgb(0.86, 0.62, 0.16) } else if is_safe { Color::srgb(0.18, 0.52, 0.30) } else if n % 2 == 0 { Color::srgb(0.16, 0.32, 0.42) } else { Color::srgb(0.12, 0.25, 0.36) };
                    let label = if n == 40 { "META".to_string() } else if is_safe { format!("{} *", n) } else { n.to_string() };
                    board.spawn((ParchisBoardCell(n), Node { position_type: PositionType::Absolute, left: Val::Px(10.0 + column as f32 * 52.0), top: Val::Px(10.0 + row as f32 * 52.0), width: Val::Px(44.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(1.0)), ..default() }, BackgroundColor(bg), BorderRadius::all(Val::Px(22.0)))).with_children(|c| { c.spawn((Text::new(label), TextFont { font: font.clone(), font_size: 10.0, ..default() }, TextColor(Color::WHITE))); });
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
    mut board_cells: Query<(&ParchisBoardCell, &Children, &mut BackgroundColor)>,
    mut cell_texts: Query<&mut Text, Without<ParchisText>>,
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
    for (cell, children, mut background) in &mut board_cells {
        let occupants: Vec<usize> = session.pos.iter().enumerate().filter_map(|(idx, pos)| (*pos == Some(cell.0)).then_some(idx)).collect();
        *background = if cell.0 == 40 { BackgroundColor(Color::srgb(0.86, 0.62, 0.16)) } else if occupants.is_empty() && ParchisSession::is_safe(cell.0) { BackgroundColor(Color::srgb(0.18, 0.52, 0.30)) } else if occupants.is_empty() && cell.0 % 2 == 0 { BackgroundColor(Color::srgb(0.16, 0.32, 0.42)) } else if occupants.is_empty() { BackgroundColor(Color::srgb(0.12, 0.25, 0.36)) } else { BackgroundColor(Color::srgb(0.74, 0.32, 0.18)) };
        for child in children.iter() {
            if let Ok(mut text) = cell_texts.get_mut(child) {
                let base = if cell.0 == 40 { "META".to_string() } else if ParchisSession::is_safe(cell.0) { format!("{} *", cell.0) } else { cell.0.to_string() };
                let tokens = occupants.iter().map(|idx| match idx { 0 => "T", 1 => "1", 2 => "2", _ => "3" }).collect::<Vec<_>>().join(" ");
                *text = Text::new(if tokens.is_empty() { base } else { format!("{}\n{}", base, tokens) });
            }
        }
    }
}
