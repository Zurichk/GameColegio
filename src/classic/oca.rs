//! La Oca — tablero 63 con ocas, puentes y trampas.

use bevy::prelude::*;
use rand::Rng;

use crate::classic::dice_anim::{spawn_side_dice, AnimatedDice};
use crate::game::GameState;
use crate::learning::screen_background;
use crate::i18n::tr;

const GOAL: usize = 63;
// Ocas oficiales: 5,9,14,18,23,27,32,36,41,45,50,54,59 + meta 63 "Jardín de la Oca"
// La casilla 1 NO es oca en el estándar español.
const OCAS: [usize; 13] = [5, 9, 14, 18, 23, 27, 32, 36, 41, 45, 50, 54, 59];

#[derive(Resource, Clone)]
struct OcaSession {
    pos_player: usize,
    pos_cpu: usize,
    turn: char, // 'P' player 1, 'C' player2/CPU
    dice: u8,
    skip_player: u8,
    skip_cpu: u8,
    winner: Option<char>,
    message: String,
    /// Dado pendiente que se aplicará cuando termine la animación del cubo.
    pending_dice: Option<u8>,
    pending_is_player: bool,
    vs_cpu: bool,
    setup_done: bool,
}

impl OcaSession {
    fn new() -> Self {
        Self { pos_player: 0, pos_cpu: 0, turn: 'P', dice: 1, skip_player: 0, skip_cpu: 0, winner: None, message: "¡Tira el dado!".to_string(), pending_dice: None, pending_is_player: true, vs_cpu: true, setup_done: false }
    }
    fn new_with_mode(vs_cpu: bool) -> Self {
        Self { pos_player: 0, pos_cpu: 0, turn: 'P', dice: 1, skip_player: 0, skip_cpu: 0, winner: None, message: "¡Tira el dado!".to_string(), pending_dice: None, pending_is_player: true, vs_cpu, setup_done: true }
    }
    fn next_oca(from: usize) -> Option<usize> {
        OCAS.iter().find(|&&o| o > from).copied()
    }
    fn apply_special(pos: usize) -> (usize, String, u8) {
        if OCAS.contains(&pos) {
            if let Some(next) = Self::next_oca(pos) {
                return (next, format!("¡Oca en {}! De oca a oca y tiro porque me toca → {}", pos, next), 0);
            } else if pos == 59 {
                // Última oca salta a meta 63
                return (63, "¡Oca en 59! De oca a oca al Jardín → 63".to_string(), 0);
            }
        }
        match pos {
            6 => (12, "¡Puente 6→12! De puente a puente y tiro porque me lleva la corriente".to_string(), 0),
            12 => (6, "¡Puente 12→6! De puente a puente y tiro porque me lleva la corriente".to_string(), 0),
            19 => (19, "¡Posada! Pierdes 1 turno".to_string(), 1),
            26 => (53, "¡Dados 26→53! De dado a dado y tiro porque me ha tocado".to_string(), 0),
            53 => (26, "¡Dados 53→26! De dado a dado y tiro porque me ha tocado".to_string(), 0),
            31 => (31, "¡Pozo! Quedas atrapado hasta que otro caiga aquí".to_string(), 2),
            42 => (30, "¡Laberinto 42→30!".to_string(), 0),
            52 => (52, "¡Cárcel! Pierdes 3 turnos (o hasta que otro caiga)".to_string(), 3),
            58 => (0, "¡Calavera 58→0! Vuelves al inicio".to_string(), 0),
            _ => (pos, "".to_string(), 0),
        }
    }
    #[allow(dead_code)]
    fn move_player(&mut self, is_player: bool) {
        let dice = rand::thread_rng().gen_range(1..=6);
        self.move_player_with_dice(is_player, dice);
    }
    fn move_player_with_dice(&mut self, is_player: bool, dice: u8) {
        if self.winner.is_some() { return; }
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
        // Rescate de pozo/cárcel: si caes donde está atrapado el rival, lo liberas
        if new_pos == 31 {
            if is_player && self.pos_cpu == 31 && self.skip_cpu > 0 {
                self.skip_cpu = 0;
                self.message = format!("{} — ¡Rescatas a CPU del pozo!", self.message);
            } else if !is_player && self.pos_player == 31 && self.skip_player > 0 {
                self.skip_player = 0;
                self.message = format!("{} — ¡CPU te rescata del pozo!", self.message);
            }
        }
        if new_pos == 52 {
            if is_player && self.pos_cpu == 52 && self.skip_cpu > 0 {
                self.skip_cpu = 0;
                self.message = format!("{} — ¡Rescatas a CPU de la cárcel!", self.message);
            } else if !is_player && self.pos_player == 52 && self.skip_player > 0 {
                self.skip_player = 0;
                self.message = format!("{} — ¡CPU te rescata de la cárcel!", self.message);
            }
        }
        if is_player { self.pos_player = new_pos; } else { self.pos_cpu = new_pos; }
        if new_pos == GOAL {
            self.winner = Some(if is_player { 'P' } else { 'C' });
            self.message = format!("¡{} gana! Llegó a 63 — Jardín de la Oca", if is_player { "Tú" } else { "CPU" });
        } else if extra_msg.contains("tiro porque me toca") || extra_msg.contains("lleva la corriente") || extra_msg.contains("me ha tocado") {
            // oca/puente/dados dan turno extra
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
#[derive(Component)]
struct OcaBoardCell(usize);
#[derive(Component)]
struct OcaSetupRoot;
#[derive(Component)]
struct OcaVsCpuButton;
#[derive(Component)]
struct OcaTwoPlayersButton;

fn oca_board_position(index: usize) -> (usize, usize) {
    let mut path = Vec::with_capacity(63);
    let (mut left, mut top, mut right, mut bottom) = (0usize, 0usize, 8usize, 6usize);
    while left <= right && top <= bottom {
        for column in left..=right { path.push((column, top)); }
        top += 1;
        for row in top..=bottom { path.push((right, row)); }
        if top <= bottom {
            right = right.saturating_sub(1);
            for column in (left..=right).rev() { path.push((column, bottom)); }
        }
        if left <= right && top <= bottom {
            bottom = bottom.saturating_sub(1);
            for row in (top..=bottom).rev() { path.push((left, row)); }
        }
        left += 1;
    }
    path[index]
}

#[cfg(test)]
mod tests {
    use super::oca_board_position;
    use std::collections::HashSet;

    #[test]
    fn oca_path_has_one_position_per_cell() {
        let positions: Vec<_> = (0..63).map(oca_board_position).collect();
        assert_eq!(positions.len(), 63);
        assert_eq!(positions.iter().collect::<HashSet<_>>().len(), 63);
    }
}

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
        .spawn((OcaUiRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, padding: UiRect::all(Val::Px(8.0)), ..default() }, screen_background(), Visibility::Visible, ZIndex(30)))
        .with_children(|overlay| {
            overlay.spawn((OcaSetupRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.02, 0.04, 0.10, 0.85)), ZIndex(50))).with_children(|setup| {
                setup.spawn((Node { flex_direction: FlexDirection::Column, width: Val::Px(460.0), padding: UiRect::all(Val::Px(20.0)), row_gap: Val::Px(14.0), align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.10, 0.14, 0.28, 0.96)), BorderRadius::all(Val::Px(16.0)))).with_children(|panel| {
                    panel.spawn((Text::new("LA OCA — Elige modo"), TextFont { font: font.clone(), font_size: 22.0, ..default() }, TextColor(Color::srgb(0.95, 0.85, 0.40))));
                    panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(12.0), ..default() }).with_children(|row| {
                        row.spawn((Button, OcaVsCpuButton, Node { width: Val::Px(180.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.15, 0.42, 0.25)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new("1 Jugador vs CPU"), TextFont { font: font.clone(), font_size: 14.0, ..default() }, TextColor(Color::WHITE))); });
                        row.spawn((Button, OcaTwoPlayersButton, Node { width: Val::Px(180.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new("2 Jugadores"), TextFont { font: font.clone(), font_size: 14.0, ..default() }, TextColor(Color::WHITE))); });
                    });
                });
            });
            // Fila con dados laterales + panel central
            overlay.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(12.0), align_items: AlignItems::Center, justify_content: JustifyContent::Center, ..default() }).with_children(|row| {
                spawn_side_dice(row, &font, "DADO IZQ");
                row.spawn((Node { flex_direction: FlexDirection::Column, width: Val::Px(780.0), max_width: Val::Px(780.0), padding: UiRect::axes(Val::Px(24.0), Val::Px(20.0)), row_gap: Val::Px(10.0), align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.07, 0.09, 0.18, 0.96)), BorderRadius::all(Val::Px(16.0)))).with_children(|panel| {
                panel.spawn((OcaText(OcaField::Title), Text::new("LA OCA — 63 casillas"), TextFont { font: font.clone(), font_size: 28.0, ..default() }, TextColor(Color::srgb(0.95, 0.85, 0.40))));
                panel.spawn((OcaText(OcaField::Status), Text::new("¡Tira el dado!"), TextFont { font: font.clone(), font_size: 20.0, ..default() }, TextColor(Color::WHITE), TextLayout { linebreak: LineBreak::WordBoundary, ..default() }, Node { max_width: Val::Px(720.0), ..default() }));
                panel.spawn((OcaText(OcaField::Dice), Text::new("Dado: -"), TextFont { font: font.clone(), font_size: 22.0, ..default() }, TextColor(Color::srgb(0.80, 0.95, 1.0))));
                panel.spawn((OcaText(OcaField::Positions), Text::new("Tú: 0  CPU: 0"), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE)));
                // Tablero clásico en espiral: 9 x 7 casillas y jardín central.
                panel.spawn((Node { position_type: PositionType::Relative, width: Val::Px(468.0), height: Val::Px(364.0), ..default() }, BackgroundColor(Color::srgb(0.34, 0.17, 0.08)), BorderRadius::all(Val::Px(24.0)))).with_children(|board| {
                    for n in 1..=63 {
                    let (column, row) = oca_board_position(n - 1);
                        let is_oca = OCAS.contains(&n);
                        let is_special = [6,12,19,26,31,42,52,53,58].contains(&n);
                        let bg = if n==63 { Color::srgb(0.86, 0.62, 0.16) } else if is_oca { Color::srgb(0.18, 0.52, 0.30) } else if is_special { Color::srgb(0.62, 0.28, 0.20) } else if n % 2 == 0 { Color::srgb(0.16, 0.32, 0.42) } else { Color::srgb(0.12, 0.25, 0.36) };
                        let label = if n == 63 { "63 META".to_string() } else if n==6 || n==12 { format!("{} ≋", n) } else if n==26 || n==53 { format!("{} ⚄", n) } else if is_oca { format!("{} O", n) } else if is_special { format!("{} !", n) } else { n.to_string() };
                        board.spawn((OcaBoardCell(n), Node { position_type: PositionType::Absolute, left: Val::Px(4.0 + column as f32 * 52.0), top: Val::Px(4.0 + row as f32 * 52.0), width: Val::Px(48.0), height: Val::Px(48.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(1.0)), ..default() }, BackgroundColor(bg), BorderRadius::all(Val::Px(24.0)))).with_children(|c| { c.spawn((Text::new(label), TextFont { font: font.clone(), font_size: 10.0, ..default() }, TextColor(Color::WHITE))); });
                    }
                });
                panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(12.0), ..default() }).with_children(|row| {
                    row.spawn((Button, OcaRollButton, Node { width: Val::Px(160.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.15, 0.42, 0.25)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Tirar dado")), TextFont { font: font.clone(), font_size: 20.0, ..default() }, TextColor(Color::WHITE))); });
                    row.spawn((Button, OcaRestartButton, Node { width: Val::Px(160.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Reiniciar")), TextFont { font: font.clone(), font_size: 20.0, ..default() }, TextColor(Color::WHITE))); });
                    row.spawn((Button, OcaBackButton, Node { width: Val::Px(160.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Volver")), TextFont { font: font.clone(), font_size: 20.0, ..default() }, TextColor(Color::WHITE))); });
                });
            });
                spawn_side_dice(row, &font, "DADO DER");
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
    mut board_cells: Query<(&OcaBoardCell, &Children, &mut BackgroundColor)>,
    mut cell_texts: Query<&mut Text, Without<OcaText>>,
    time: Res<Time>,
    mut dice_anim: Query<&mut AnimatedDice>,
    setup_vs_cpu: Query<&Interaction, (Changed<Interaction>, With<OcaVsCpuButton>)>,
    setup_two: Query<&Interaction, (Changed<Interaction>, With<OcaTwoPlayersButton>)>,
    mut setup_root: Query<&mut Visibility, With<OcaSetupRoot>>,
) {
    if keys.just_pressed(KeyCode::Escape) { commands.set_state(GameState::ClassicMenu); return; }
    if back_clicks.single().map_or(false, |i| *i == Interaction::Pressed) { commands.set_state(GameState::ClassicMenu); return; }
    if restart_clicks.single().map_or(false, |i| *i == Interaction::Pressed) {
        let vs = session.vs_cpu;
        let done = session.setup_done;
        if done { *session = OcaSession::new_with_mode(vs); } else { *session = OcaSession::new(); }
    }
    // Setup: elegir 1 vs CPU o 2 jugadores
    if !session.setup_done {
        for interaction in &setup_vs_cpu { if *interaction == Interaction::Pressed { *session = OcaSession::new_with_mode(true); for mut v in &mut setup_root { *v = Visibility::Hidden; } return; } }
        for interaction in &setup_two { if *interaction == Interaction::Pressed { *session = OcaSession::new_with_mode(false); for mut v in &mut setup_root { *v = Visibility::Hidden; } return; } }
        return;
    }

    // Resolver tirada pendiente cuando termina la animación del dado
    if let Some(pending) = session.pending_dice {
        let still_rolling = dice_anim.iter().any(|d| d.rolling);
        if still_rolling {
            session.message = "🎲 ¡Rodando dado...!".to_string();
        } else {
            let is_p = session.pending_is_player;
            session.pending_dice = None;
            session.move_player_with_dice(is_p, pending);
        }
    }

    let is_rolling = session.pending_dice.is_some();

    if !is_rolling && session.winner.is_none() {
        if session.turn == 'P' {
            let mut clicked = false;
            for interaction in &roll_clicks { if *interaction == Interaction::Pressed { clicked = true; break; } }
            if keys.just_pressed(KeyCode::Space) { clicked = true; }
            if clicked {
                let dice = rand::thread_rng().gen_range(1..=6);
                session.pending_dice = Some(dice);
                session.pending_is_player = true;
                session.message = format!("🎲 ¡Lanzando dado... ({})", dice);
                for mut d in &mut dice_anim { d.roll_to(dice); }
            }
        } else if session.vs_cpu {
            let dt = time.delta_secs();
            if rand::thread_rng().gen_bool(0.03) {
                let dice = rand::thread_rng().gen_range(1..=6);
                session.pending_dice = Some(dice);
                session.pending_is_player = false;
                session.message = format!("🎲 CPU lanza dado... ({})", dice);
                for mut d in &mut dice_anim { d.roll_to(dice); }
            }
            let _ = dt;
        } else {
            // 2 jugadores: turno de J2 humano
            let mut clicked = false;
            for interaction in &roll_clicks { if *interaction == Interaction::Pressed { clicked = true; break; } }
            if keys.just_pressed(KeyCode::Space) { clicked = true; }
            if clicked {
                let dice = rand::thread_rng().gen_range(1..=6);
                session.pending_dice = Some(dice);
                session.pending_is_player = false;
                session.message = format!("🎲 J2 lanza dado... ({})", dice);
                for mut d in &mut dice_anim { d.roll_to(dice); }
            }
        }
    }
    for (field, mut text) in &mut texts {
        match field.0 {
            OcaField::Status => { *text = Text::new(session.message.clone()); }
            OcaField::Dice => {
                if session.pending_dice.is_some() {
                    *text = Text::new("Dado: 🎲 rodando...".to_string());
                } else {
                    *text = Text::new(format!("Dado: {}", session.dice));
                }
            }
            OcaField::Positions => {
                if session.vs_cpu {
                    *text = Text::new(format!("Tú: {}  CPU: {}  Turno: {}", session.pos_player, session.pos_cpu, if session.turn=='P' {"Tú"} else {"CPU"}));
                } else {
                    *text = Text::new(format!("J1: {}  J2: {}  Turno: {}", session.pos_player, session.pos_cpu, if session.turn=='P' {"J1"} else {"J2"}));
                }
            }
            _ => {}
        }
    }
    for (cell, children, mut background) in &mut board_cells {
        let player_here = session.pos_player == cell.0;
        let cpu_here = session.pos_cpu == cell.0;
        *background = BackgroundColor(if player_here && cpu_here { Color::srgb(0.72, 0.38, 0.16) } else if player_here { Color::srgb(0.12, 0.62, 0.82) } else if cpu_here { Color::srgb(0.82, 0.22, 0.30) } else if cell.0 == 63 { Color::srgb(0.86, 0.62, 0.16) } else if OCAS.contains(&cell.0) { Color::srgb(0.18, 0.52, 0.30) } else if [6,12,19,26,31,42,52,53,58].contains(&cell.0) { Color::srgb(0.62, 0.28, 0.20) } else if cell.0 % 2 == 0 { Color::srgb(0.16, 0.32, 0.42) } else { Color::srgb(0.12, 0.25, 0.36) });
        for child in children.iter() {
            if let Ok(mut text) = cell_texts.get_mut(child) {
                let marker = match (player_here, cpu_here) { (true, true) => "T+C", (true, false) => "T", (false, true) => "C", _ => "" };
                let base = if cell.0 == 63 { "63 META".to_string() } else if cell.0==6 || cell.0==12 { format!("{} ≋", cell.0) } else if cell.0==26 || cell.0==53 { format!("{} ⚄", cell.0) } else if OCAS.contains(&cell.0) { format!("{} O", cell.0) } else if [19,31,42,52,58].contains(&cell.0) { format!("{} !", cell.0) } else { cell.0.to_string() };
                *text = Text::new(if marker.is_empty() { base } else { format!("{}\n{}", base, marker) });
            }
        }
    }
}
