//! Hundir la Flota — 5×5, 3 barcos ×2 casillas, turnos, vs CPU.
use bevy::prelude::*;
use rand::Rng;
use rand::seq::SliceRandom;

use crate::game::GameState;
use crate::learning::screen_background;
use crate::i18n::tr;

const SIZE: usize = 5;
const SHIPS: usize = 3;
const SHIP_LEN: usize = 2;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Cell { Empty, Ship, Hit, Miss }

#[derive(Resource, Clone)]
struct BattleshipSession {
    player: [[Cell; SIZE]; SIZE],
    cpu: [[Cell; SIZE]; SIZE],
    player_ships: usize, // celdas restantes
    cpu_ships: usize,
    turn: char, // 'P' player, 'C' cpu
    winner: Option<char>,
    message: String,
}

impl BattleshipSession {
    fn new() -> Self {
        let mut s = Self { player: [[Cell::Empty; SIZE]; SIZE], cpu: [[Cell::Empty; SIZE]; SIZE], player_ships: 0, cpu_ships: 0, turn: 'P', winner: None, message: "¡Dispara al mar enemigo!".to_string() };
        s.place_ships(true);
        s.place_ships(false);
        s.player_ships = SHIPS * SHIP_LEN;
        s.cpu_ships = SHIPS * SHIP_LEN;
        s
    }
    fn place_ships(&mut self, is_player: bool) {
        let mut rng = rand::thread_rng();
        let mut placed = 0;
        while placed < SHIPS {
            let horizontal = rng.gen_bool(0.5);
            let r = rng.gen_range(0..SIZE);
            let c = rng.gen_range(0..SIZE);
            if horizontal {
                if c + SHIP_LEN > SIZE { continue; }
                let free = (0..SHIP_LEN).all(|k| self.get_board(is_player)[r][c+k] == Cell::Empty);
                if !free { continue; }
                for k in 0..SHIP_LEN { self.get_board_mut(is_player)[r][c+k] = Cell::Ship; }
            } else {
                if r + SHIP_LEN > SIZE { continue; }
                let free = (0..SHIP_LEN).all(|k| self.get_board(is_player)[r+k][c] == Cell::Empty);
                if !free { continue; }
                for k in 0..SHIP_LEN { self.get_board_mut(is_player)[r+k][c] = Cell::Ship; }
            }
            placed += 1;
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
            Cell::Ship => { board[r][c] = Cell::Hit; if is_player_attacking { self.cpu_ships -= 1; } else { self.player_ships -= 1; } true },
            Cell::Empty => { board[r][c] = Cell::Miss; false },
            _ => false,
        }
    }
    fn check_win(&mut self) {
        if self.cpu_ships == 0 { self.winner = Some('P'); self.message = "¡Hundiste toda la flota enemiga!".to_string(); }
        else if self.player_ships == 0 { self.winner = Some('C'); self.message = "¡La CPU hundió tu flota!".to_string(); }
    }
    fn cpu_turn(&mut self) {
        if self.turn != 'C' || self.winner.is_some() { return; }
        let mut rng = rand::thread_rng();
        let mut empties: Vec<(usize,usize)> = Vec::new();
        for r in 0..SIZE { for c in 0..SIZE { if matches!(self.player[r][c], Cell::Empty | Cell::Ship) { empties.push((r,c)); } } }
        empties.shuffle(&mut rng);
        if let Some((r,c)) = empties.first() {
            let hit = self.shoot(false, *r, *c);
            self.message = if hit { format!("CPU disparó ({}, {}) ¡Tocado!", r+1, c+1) } else { format!("CPU disparó ({}, {}) Agua", r+1, c+1) };
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
        .spawn((BattleshipUiRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, screen_background(), Visibility::Visible, ZIndex(30)))
        .with_children(|overlay| {
            overlay.spawn((Node { flex_direction: FlexDirection::Column, width: Val::Px(720.0), padding: UiRect::axes(Val::Px(20.0), Val::Px(16.0)), row_gap: Val::Px(10.0), align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.07, 0.09, 0.18, 0.96)), BorderRadius::all(Val::Px(16.0)))).with_children(|panel| {
                panel.spawn((BattleshipText(BattleshipField::Title), Text::new("HUNDIR LA FLOTA"), TextFont { font: font.clone(), font_size: 28.0, ..default() }, TextColor(Color::srgb(0.95, 0.85, 0.40))));
                panel.spawn((BattleshipText(BattleshipField::Status), Text::new("¡Dispara al mar enemigo!"), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE)));
                panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(24.0), ..default() }).with_children(|row| {
                    // Enemigo
                    row.spawn(Node { flex_direction: FlexDirection::Column, align_items: AlignItems::Center, row_gap: Val::Px(6.0), ..default() }).with_children(|col| {
                        col.spawn((BattleshipText(BattleshipField::EnemyLabel), Text::new("Mar enemigo (toca para disparar)"), TextFont { font: font.clone(), font_size: 16.0, ..default() }, TextColor(Color::srgb(0.80, 0.85, 1.0))));
                        col.spawn(Node { display: Display::Grid, grid_template_columns: vec![GridTrack::px(56.0); SIZE], grid_template_rows: vec![GridTrack::px(56.0); SIZE], column_gap: Val::Px(4.0), row_gap: Val::Px(4.0), ..default() }).with_children(|grid| {
                            for r in 0..SIZE { for c in 0..SIZE { grid.spawn((Button, BattleshipEnemyCell(r,c), Node { width: Val::Px(56.0), height: Val::Px(56.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.15, 0.22, 0.34)), BorderColor(Color::srgb(0.50, 0.55, 0.70)), BorderRadius::all(Val::Px(6.0)))).with_children(|cell| { cell.spawn((Text::new("·".to_string()), TextFont { font: font.clone(), font_size: 28.0, ..default() }, TextColor(Color::WHITE))); }); } }
                        });
                    });
                    // Jugador
                    row.spawn(Node { flex_direction: FlexDirection::Column, align_items: AlignItems::Center, row_gap: Val::Px(6.0), ..default() }).with_children(|col| {
                        col.spawn((BattleshipText(BattleshipField::PlayerLabel), Text::new("Tu mar"), TextFont { font: font.clone(), font_size: 16.0, ..default() }, TextColor(Color::srgb(0.80, 0.85, 1.0))));
                        col.spawn(Node { display: Display::Grid, grid_template_columns: vec![GridTrack::px(40.0); SIZE], grid_template_rows: vec![GridTrack::px(40.0); SIZE], column_gap: Val::Px(3.0), row_gap: Val::Px(3.0), ..default() }).with_children(|grid| {
                            for r in 0..SIZE { for c in 0..SIZE { grid.spawn((BattleshipPlayerCell(r,c), Node { width: Val::Px(40.0), height: Val::Px(40.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(1.0)), ..default() }, BackgroundColor(Color::srgb(0.12, 0.14, 0.24)), BorderRadius::all(Val::Px(4.0)))).with_children(|cell| { cell.spawn((Text::new("·".to_string()), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::srgba(1.0,1.0,1.0,0.6)))); }); } }
                        });
                    });
                });
                panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(12.0), ..default() }).with_children(|row| {
                    row.spawn((Button, BattleshipRestartButton, Node { width: Val::Px(160.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Reiniciar")), TextFont { font: font.clone(), font_size: 20.0, ..default() }, TextColor(Color::WHITE))); });
                    row.spawn((Button, BattleshipBackButton, Node { width: Val::Px(160.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Volver")), TextFont { font: font.clone(), font_size: 20.0, ..default() }, TextColor(Color::WHITE))); });
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
    back_clicks: Query<&Interaction, (Changed<Interaction>, With<BattleshipBackButton>)>,
    restart_clicks: Query<&Interaction, (Changed<Interaction>, With<BattleshipRestartButton>)>,
    mut texts: Query<(&BattleshipText, &mut Text), Without<BattleshipEnemyCell>>,
    mut enemy_cells: Query<(&BattleshipEnemyCell, &mut BackgroundColor, &Children), Without<BattleshipPlayerCell>>,
    mut player_cells: Query<(&BattleshipPlayerCell, &mut BackgroundColor), Without<BattleshipEnemyCell>>,
    mut cell_texts: Query<&mut Text, Without<BattleshipText>>,
) {
    if keys.just_pressed(KeyCode::Escape) { commands.set_state(GameState::ClassicMenu); return; }
    if back_clicks.single().map_or(false, |i| *i == Interaction::Pressed) { commands.set_state(GameState::ClassicMenu); return; }
    if restart_clicks.single().map_or(false, |i| *i == Interaction::Pressed) { *session = BattleshipSession::new(); }
    if session.winner.is_some() {
        for (field, mut text) in &mut texts { if field.0 == BattleshipField::Status { *text = Text::new(session.message.clone()); } }
    } else if session.turn == 'P' {
        let mut clicked: Option<(usize,usize)> = None;
        for (interaction, cell) in &enemy_clicks { if *interaction == Interaction::Pressed { clicked = Some((cell.0, cell.1)); break; } }
        if let Some((r,c)) = clicked {
            if matches!(session.cpu[r][c], Cell::Empty | Cell::Ship) {
                let hit = session.shoot(true, r, c);
                session.message = if hit { format!("¡Tocado en {},{}!", r+1, c+1) } else { format!("Agua en {},{}", r+1, c+1) };
                session.check_win();
                if session.winner.is_none() {
                    session.turn = 'C';
                    // CPU responde
                    session.cpu_turn();
                }
            }
        }
        for (field, mut text) in &mut texts { if field.0 == BattleshipField::Status { *text = Text::new(session.message.clone()); } }
    }
    // Actualizar colores de celdas enemigas (solo revela Hit/Miss, no Ship)
    for (cell, mut bg, children) in &mut enemy_cells {
        let state = session.cpu[cell.0][cell.1];
        *bg = BackgroundColor(match state { Cell::Hit => Color::srgb(0.90, 0.25, 0.25), Cell::Miss => Color::srgb(0.25, 0.35, 0.55), _ => Color::srgb(0.15, 0.22, 0.34) });
        for child in children.iter() {
            if let Ok(mut text) = cell_texts.get_mut(child) {
                *text = Text::new(match state { Cell::Hit => "✕".to_string(), Cell::Miss => "○".to_string(), _ => "·".to_string() });
            }
        }
    }
    for (cell, mut bg) in &mut player_cells {
        let state = session.player[cell.0][cell.1];
        *bg = BackgroundColor(match state { Cell::Ship => Color::srgb(0.30, 0.55, 0.30), Cell::Hit => Color::srgb(0.90, 0.25, 0.25), Cell::Miss => Color::srgb(0.25, 0.35, 0.55), _ => Color::srgb(0.12, 0.14, 0.24) });
    }
}
