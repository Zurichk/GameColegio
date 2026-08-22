//! Hundir la Flota — 10×10, flota Hasbro oficial, turnos, vs CPU.
//!
//! Tablero real: 10×10 (A-J, 1-10) y 5 barcos por jugador:
//! Portaaviones 5, Acorazado 4, Crucero 3, Submarino 3, Destructor 2 (17 casillas).
//! Se dispara por coordenadas y se marca con clavijas rojas (tocado) / blancas (agua).

use bevy::prelude::*;
use rand::Rng;
use rand::seq::SliceRandom;

use crate::game::GameState;
use crate::learning::screen_background;
use crate::i18n::tr;

const SIZE: usize = 10;
/// Flota Hasbro clásica: longitudes reales.
const FLEET: [usize; 5] = [5, 4, 3, 3, 2];
const TOTAL_SHIP_CELLS: usize = 17; // 5+4+3+3+2

#[derive(Clone, Copy, PartialEq, Eq)]
enum Cell { Empty, Ship, Hit, Miss }

#[derive(Resource, Clone)]
struct BattleshipSession {
    player: [[Cell; SIZE]; SIZE],
    cpu: [[Cell; SIZE]; SIZE],
    player_ships: usize,
    cpu_ships: usize,
    turn: char,
    winner: Option<char>,
    message: String,
    vs_cpu: bool,
    setup_done: bool,
}

impl BattleshipSession {
    fn new() -> Self {
        let mut s = Self {
            player: [[Cell::Empty; SIZE]; SIZE],
            cpu: [[Cell::Empty; SIZE]; SIZE],
            player_ships: TOTAL_SHIP_CELLS,
            cpu_ships: TOTAL_SHIP_CELLS,
            turn: 'P',
            winner: None,
            message: "Elige modo arriba".to_string(),
            vs_cpu: true,
            setup_done: false,
        };
        s.place_ships(true);
        s.place_ships(false);
        s
    }
    fn new_with_mode(vs_cpu: bool) -> Self {
        let mut s = Self {
            player: [[Cell::Empty; SIZE]; SIZE],
            cpu: [[Cell::Empty; SIZE]; SIZE],
            player_ships: TOTAL_SHIP_CELLS,
            cpu_ships: TOTAL_SHIP_CELLS,
            turn: 'P',
            winner: None,
            message: "¡Flota 10×10 lista! Dispara al mar enemigo (A1-J10)".to_string(),
            vs_cpu,
            setup_done: true,
        };
        s.place_ships(true);
        s.place_ships(false);
        s
    }
    fn place_ships(&mut self, is_player: bool) {
        let mut rng = rand::thread_rng();
        for &len in &FLEET {
            let mut attempts = 0;
            loop {
                attempts += 1;
                if attempts > 500 { break; }
                let horizontal = rng.gen_bool(0.5);
                let r = rng.gen_range(0..SIZE);
                let c = rng.gen_range(0..SIZE);
                if horizontal {
                    if c + len > SIZE { continue; }
                    let free = (0..len).all(|k| self.get_board(is_player)[r][c + k] == Cell::Empty);
                    if !free { continue; }
                    // Opcional: evitar adyacencia (regla oficial: no tocar ni en diagonal).
                    // Lo dejamos permisivo para no complicar, pero se puede activar.
                    for k in 0..len { self.get_board_mut(is_player)[r][c + k] = Cell::Ship; }
                    break;
                } else {
                    if r + len > SIZE { continue; }
                    let free = (0..len).all(|k| self.get_board(is_player)[r + k][c] == Cell::Empty);
                    if !free { continue; }
                    for k in 0..len { self.get_board_mut(is_player)[r + k][c] = Cell::Ship; }
                    break;
                }
            }
        }
    }
    fn get_board(&self, is_player: bool) -> &[[Cell; SIZE]; SIZE] {
        if is_player { &self.player } else { &self.cpu }
    }
    fn get_board_mut(&mut self, is_player: bool) -> &mut [[Cell; SIZE]; SIZE] {
        if is_player { &mut self.player } else { &mut self.cpu }
    }
    fn shoot(&mut self, is_player_attacking: bool, r: usize, c: usize) -> bool {
        let board = self.get_board_mut(!is_player_attacking);
        match board[r][c] {
            Cell::Ship => {
                board[r][c] = Cell::Hit;
                if is_player_attacking { self.cpu_ships -= 1; } else { self.player_ships -= 1; }
                true
            },
            Cell::Empty => { board[r][c] = Cell::Miss; false },
            _ => false,
        }
    }
    fn check_win(&mut self) {
        if self.cpu_ships == 0 { self.winner = Some('P'); self.message = "¡Hundiste toda la flota enemiga! (17/17)".to_string(); }
        else if self.player_ships == 0 { self.winner = Some('C'); self.message = if self.vs_cpu { "¡La CPU hundió tu flota!".to_string() } else { "¡Jugador 2 hundió tu flota!".to_string() }; }
    }
    fn cpu_turn(&mut self) {
        if self.turn != 'C' || self.winner.is_some() { return; }
        let mut rng = rand::thread_rng();
        let mut empties: Vec<(usize,usize)> = Vec::new();
        for r in 0..SIZE { for c in 0..SIZE { if matches!(self.player[r][c], Cell::Empty | Cell::Ship) { empties.push((r,c)); } } }
        empties.shuffle(&mut rng);
        if let Some((r,c)) = empties.first() {
            let hit = self.shoot(false, *r, *c);
            let coord = format!("{}{}", (b'A' + *c as u8) as char, r + 1);
            self.message = if hit { format!("CPU disparó {} ¡Tocado!", coord) } else { format!("CPU disparó {} Agua", coord) };
            self.check_win();
            if self.winner.is_none() { self.turn = 'P'; self.message = format!("{} — Tu turno", self.message); }
        }
    }
}

#[derive(Component)]
struct BattleshipUiRoot;
#[derive(Component)]
struct BattleshipText(BattleshipField);
#[derive(Clone, Copy, PartialEq, Eq)]
enum BattleshipField { Title, Status, EnemyLabel, PlayerLabel }
#[derive(Component)]
struct BattleshipEnemyCell(usize, usize);
#[derive(Component)]
struct BattleshipPlayerCell(usize, usize);
#[derive(Component)]
struct BattleshipBackButton;
#[derive(Component)]
struct BattleshipRestartButton;
#[derive(Component)]
struct BattleshipSetupRoot;
#[derive(Component)]
struct BattleshipVsCpuButton;
#[derive(Component)]
struct BattleshipTwoPlayersButton;

pub struct BattleshipPlugin;
impl Plugin for BattleshipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::BattleshipGame), spawn_battleship)
            .add_systems(OnExit(GameState::BattleshipGame), cleanup_battleship)
            .add_systems(Update, update_battleship.run_if(in_state(GameState::BattleshipGame)));
    }
}

fn spawn_battleship(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands.insert_resource(BattleshipSession::new());
    commands
        .spawn((BattleshipUiRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, padding: UiRect::all(Val::Px(8.0)), ..default() }, screen_background(), Visibility::Visible, ZIndex(30)))
        .with_children(|overlay| {
            overlay.spawn((BattleshipSetupRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.02, 0.04, 0.10, 0.85)), ZIndex(50))).with_children(|setup| {
                setup.spawn((Node { flex_direction: FlexDirection::Column, width: Val::Px(460.0), padding: UiRect::all(Val::Px(20.0)), row_gap: Val::Px(14.0), align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.10, 0.14, 0.28, 0.96)), BorderRadius::all(Val::Px(16.0)))).with_children(|panel| {
                    panel.spawn((Text::new("HUNDIR LA FLOTA — Elige modo"), TextFont { font: font.clone(), font_size: 22.0, ..default() }, TextColor(Color::srgb(0.95, 0.85, 0.40))));
                    panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(12.0), ..default() }).with_children(|row| {
                        row.spawn((Button, BattleshipVsCpuButton, Node { width: Val::Px(180.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.15, 0.42, 0.25)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new("1 Jugador vs CPU"), TextFont { font: font.clone(), font_size: 14.0, ..default() }, TextColor(Color::WHITE))); });
                        row.spawn((Button, BattleshipTwoPlayersButton, Node { width: Val::Px(180.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new("2 Jugadores"), TextFont { font: font.clone(), font_size: 14.0, ..default() }, TextColor(Color::WHITE))); });
                    });
                });
            });
            overlay.spawn((Node { flex_direction: FlexDirection::Column, width: Val::Px(920.0), max_width: Val::Percent(96.0), padding: UiRect::axes(Val::Px(16.0), Val::Px(12.0)), row_gap: Val::Px(8.0), align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.07, 0.09, 0.18, 0.96)), BorderRadius::all(Val::Px(16.0)))).with_children(|panel| {
                panel.spawn((BattleshipText(BattleshipField::Title), Text::new("HUNDIR LA FLOTA — 10×10"), TextFont { font: font.clone(), font_size: 26.0, ..default() }, TextColor(Color::srgb(0.95, 0.85, 0.40))));
                panel.spawn((BattleshipText(BattleshipField::Status), Text::new("¡Flota 10×10! Portaaviones 5 · Acorazado 4 · Crucero 3 · Submarino 3 · Destructor 2"), TextFont { font: font.clone(), font_size: 14.0, ..default() }, TextColor(Color::WHITE), TextLayout { linebreak: LineBreak::WordBoundary, ..default() }, Node { max_width: Val::Px(860.0), ..default() }));
                panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(16.0), flex_wrap: FlexWrap::Wrap, justify_content: JustifyContent::Center, ..default() }).with_children(|row| {
                    // Enemigo 10x10 - más grande
                    row.spawn(Node { flex_direction: FlexDirection::Column, align_items: AlignItems::Center, row_gap: Val::Px(4.0), ..default() }).with_children(|col| {
                        col.spawn((BattleshipText(BattleshipField::EnemyLabel), Text::new("MAR ENEMIGO (A-J, 1-10) — toca para disparar"), TextFont { font: font.clone(), font_size: 13.0, ..default() }, TextColor(Color::srgb(0.80, 0.85, 1.0))));
                        col.spawn((Node { display: Display::Grid, grid_template_columns: vec![GridTrack::px(36.0); SIZE], grid_template_rows: vec![GridTrack::px(36.0); SIZE], column_gap: Val::Px(2.0), row_gap: Val::Px(2.0), padding: UiRect::all(Val::Px(6.0)), ..default() }, BackgroundColor(Color::srgb(0.04, 0.26, 0.42)), BorderRadius::all(Val::Px(12.0)))).with_children(|grid| {
                            for r in 0..SIZE { for c in 0..SIZE { grid.spawn((Button, BattleshipEnemyCell(r,c), Node { width: Val::Px(36.0), height: Val::Px(36.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(1.0)), ..default() }, BackgroundColor(Color::srgb(0.15, 0.22, 0.34)), BorderColor(Color::srgb(0.50, 0.55, 0.70)), BorderRadius::all(Val::Px(4.0)))).with_children(|cell| { cell.spawn((Text::new("·".to_string()), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE))); }); } }
                        });
                    });
                    // Jugador 10x10 - más pequeño pero también 10x10
                    row.spawn(Node { flex_direction: FlexDirection::Column, align_items: AlignItems::Center, row_gap: Val::Px(4.0), ..default() }).with_children(|col| {
                        col.spawn((BattleshipText(BattleshipField::PlayerLabel), Text::new("TU MAR (tus 5 barcos)"), TextFont { font: font.clone(), font_size: 13.0, ..default() }, TextColor(Color::srgb(0.80, 0.85, 1.0))));
                        col.spawn((Node { display: Display::Grid, grid_template_columns: vec![GridTrack::px(28.0); SIZE], grid_template_rows: vec![GridTrack::px(28.0); SIZE], column_gap: Val::Px(2.0), row_gap: Val::Px(2.0), padding: UiRect::all(Val::Px(4.0)), ..default() }, BackgroundColor(Color::srgb(0.04, 0.26, 0.42)), BorderRadius::all(Val::Px(10.0)))).with_children(|grid| {
                            for r in 0..SIZE { for c in 0..SIZE { grid.spawn((Button, BattleshipPlayerCell(r,c), Node { width: Val::Px(28.0), height: Val::Px(28.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(1.0)), ..default() }, BackgroundColor(Color::srgb(0.12, 0.14, 0.24)), BorderRadius::all(Val::Px(3.0)))).with_children(|cell| { cell.spawn((Text::new("·".to_string()), TextFont { font: font.clone(), font_size: 14.0, ..default() }, TextColor(Color::srgba(1.0,1.0,1.0,0.6)))); }); } }
                        });
                    });
                });
                panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(12.0), ..default() }).with_children(|row| {
                    row.spawn((Button, BattleshipRestartButton, Node { width: Val::Px(140.0), height: Val::Px(40.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Reiniciar")), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE))); });
                    row.spawn((Button, BattleshipBackButton, Node { width: Val::Px(140.0), height: Val::Px(40.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Volver")), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE))); });
                });
            });
        });
}

fn cleanup_battleship(mut commands: Commands, roots: Query<Entity, With<BattleshipUiRoot>>) {
    for root in &roots { commands.entity(root).despawn(); }
    commands.remove_resource::<BattleshipSession>();
}

fn update_battleship(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut session: ResMut<BattleshipSession>,
    enemy_clicks: Query<(&Interaction, &BattleshipEnemyCell), (Changed<Interaction>, Without<BattleshipBackButton>)>,
    player_clicks: Query<(&Interaction, &BattleshipPlayerCell), (Changed<Interaction>, Without<BattleshipBackButton>)>,
    back_clicks: Query<&Interaction, (Changed<Interaction>, With<BattleshipBackButton>)>,
    restart_clicks: Query<&Interaction, (Changed<Interaction>, With<BattleshipRestartButton>)>,
    mut texts: Query<(&BattleshipText, &mut Text), Without<BattleshipEnemyCell>>,
    mut enemy_cells: Query<(&BattleshipEnemyCell, &mut BackgroundColor, &Children), Without<BattleshipPlayerCell>>,
    mut player_cells: Query<(&BattleshipPlayerCell, &mut BackgroundColor), Without<BattleshipEnemyCell>>,
    mut cell_texts: Query<&mut Text, Without<BattleshipText>>,
    setup_vs_cpu: Query<&Interaction, (Changed<Interaction>, With<BattleshipVsCpuButton>)>,
    setup_two: Query<&Interaction, (Changed<Interaction>, With<BattleshipTwoPlayersButton>)>,
    mut setup_root: Query<&mut Visibility, With<BattleshipSetupRoot>>,
) {
    if keys.just_pressed(KeyCode::Escape) { commands.set_state(GameState::ClassicMenu); return; }
    if back_clicks.single().map_or(false, |i| *i == Interaction::Pressed) { commands.set_state(GameState::ClassicMenu); return; }
    if restart_clicks.single().map_or(false, |i| *i == Interaction::Pressed) {
        let vs = session.vs_cpu;
        let done = session.setup_done;
        if done { *session = BattleshipSession::new_with_mode(vs); } else { *session = BattleshipSession::new(); }
    }
    if !session.setup_done {
        for interaction in &setup_vs_cpu { if *interaction == Interaction::Pressed { *session = BattleshipSession::new_with_mode(true); for mut v in &mut setup_root { *v = Visibility::Hidden; } return; } }
        for interaction in &setup_two { if *interaction == Interaction::Pressed { *session = BattleshipSession::new_with_mode(false); for mut v in &mut setup_root { *v = Visibility::Hidden; } return; } }
        return;
    }
    if session.winner.is_some() {
        for (field, mut text) in &mut texts { if field.0 == BattleshipField::Status { *text = Text::new(session.message.clone()); } }
    } else if session.turn == 'P' {
        let mut clicked: Option<(usize,usize)> = None;
        for (interaction, cell) in &enemy_clicks { if *interaction == Interaction::Pressed { clicked = Some((cell.0, cell.1)); break; } }
        if let Some((r,c)) = clicked {
            if matches!(session.cpu[r][c], Cell::Empty | Cell::Ship) {
                let hit = session.shoot(true, r, c);
                let coord = format!("{}{}", (b'A' + c as u8) as char, r + 1);
                session.message = if hit { format!("¡Tocado en {}!", coord) } else { format!("Agua en {}", coord) };
                session.check_win();
                if session.winner.is_none() {
                    session.turn = 'C';
                    if session.vs_cpu {
                        session.cpu_turn();
                    } else {
                        session.message = format!("{} — Turno J2", session.message);
                    }
                }
            }
        }
        for (field, mut text) in &mut texts { if field.0 == BattleshipField::Status { *text = Text::new(session.message.clone()); } }
    } else if session.turn == 'C' && !session.vs_cpu {
        let mut clicked: Option<(usize,usize)> = None;
        for (interaction, cell) in &player_clicks { if *interaction == Interaction::Pressed { clicked = Some((cell.0, cell.1)); break; } }
        if let Some((r,c)) = clicked {
            if matches!(session.player[r][c], Cell::Empty | Cell::Ship) {
                let hit = session.shoot(false, r, c);
                let coord = format!("{}{}", (b'A' + c as u8) as char, r + 1);
                session.message = if hit { format!("J2 ¡Tocado en {}!", coord) } else { format!("J2 Agua en {}", coord) };
                session.check_win();
                if session.winner.is_none() { session.turn = 'P'; session.message = format!("{} — Turno J1", session.message); }
            }
        }
        for (field, mut text) in &mut texts { if field.0 == BattleshipField::Status { *text = Text::new(session.message.clone()); } }
    }
    for (cell, mut bg, children) in &mut enemy_cells {
        let state = session.cpu[cell.0][cell.1];
        *bg = BackgroundColor(match state { Cell::Hit => Color::srgb(0.90, 0.25, 0.25), Cell::Miss => Color::srgb(0.38, 0.72, 0.82), _ => Color::srgb(0.08, 0.42, 0.58) });
        for child in children.iter() {
            if let Ok(mut text) = cell_texts.get_mut(child) {
                *text = Text::new(match state { Cell::Hit => "✕".to_string(), Cell::Miss => "○".to_string(), _ => "·".to_string() });
            }
        }
    }
    for (cell, mut bg) in &mut player_cells {
        let state = session.player[cell.0][cell.1];
        *bg = BackgroundColor(match state { Cell::Ship => Color::srgb(0.28, 0.42, 0.48), Cell::Hit => Color::srgb(0.90, 0.25, 0.25), Cell::Miss => Color::srgb(0.38, 0.72, 0.82), _ => Color::srgb(0.08, 0.34, 0.48) });
    }
}
