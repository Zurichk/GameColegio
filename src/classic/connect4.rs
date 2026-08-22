//! Conecta 4 — 6×7, gravedad, 4 en raya, vs CPU.

use bevy::prelude::*;
use rand::seq::SliceRandom;

use crate::game::GameState;
use crate::learning::screen_background;
use crate::i18n::tr;

const ROWS: usize = 6;
const COLS: usize = 7;

#[derive(Resource, Clone)]
struct Connect4Session {
    board: [[Option<char>; COLS]; ROWS],
    turn: char, // 'R' rojo, 'Y' amarillo
    winner: Option<char>,
    draw: bool,
    vs_cpu: bool,
    setup_done: bool,
}

impl Connect4Session {
    fn new() -> Self {
        Self { board: [[None; COLS]; ROWS], turn: 'R', winner: None, draw: false, vs_cpu: true, setup_done: false }
    }
    fn new_with_mode(vs_cpu: bool) -> Self {
        Self { board: [[None; COLS]; ROWS], turn: 'R', winner: None, draw: false, vs_cpu, setup_done: true }
    }
    fn drop_in_col(&mut self, col: usize) -> bool {
        if self.winner.is_some() || self.draw { return false; }
        if col >= COLS { return false; }
        // buscar fila más baja vacía
        for row in (0..ROWS).rev() {
            if self.board[row][col].is_none() {
                self.board[row][col] = Some(self.turn);
                self.check_winner();
                if self.winner.is_none() && !self.draw {
                    self.turn = if self.turn == 'R' { 'Y' } else { 'R' };
                }
                return true;
            }
        }
        false
    }
    fn check_winner(&mut self) {
        // 4 en raya
        for r in 0..ROWS {
            for c in 0..COLS {
                let Some(ch) = self.board[r][c] else { continue; };
                // horizontal
                if c + 3 < COLS && (1..4).all(|k| self.board[r][c+k] == Some(ch)) { self.winner = Some(ch); return; }
                // vertical
                if r + 3 < ROWS && (1..4).all(|k| self.board[r+k][c] == Some(ch)) { self.winner = Some(ch); return; }
                // diagonal down-right
                if r + 3 < ROWS && c + 3 < COLS && (1..4).all(|k| self.board[r+k][c+k] == Some(ch)) { self.winner = Some(ch); return; }
                // diagonal up-right
                if r >= 3 && c + 3 < COLS && (1..4).all(|k| self.board[r-k][c+k] == Some(ch)) { self.winner = Some(ch); return; }
            }
        }
        if self.board.iter().all(|row| row.iter().all(|c| c.is_some())) { self.draw = true; }
    }
    fn cpu_move(&mut self) {
        if self.turn != 'Y' || self.winner.is_some() || self.draw { return; }
        let mut cols: Vec<usize> = (0..COLS).filter(|&c| self.board[0][c].is_none()).collect();
        cols.shuffle(&mut rand::thread_rng());
        if let Some(&col) = cols.first() {
            self.drop_in_col(col);
        }
    }
}

#[derive(Component)]
struct Connect4UiRoot;
#[derive(Component)]
struct Connect4Text(Connect4Field);
#[derive(Clone, Copy, PartialEq, Eq)]
enum Connect4Field { Title, Status }
#[derive(Component)]
struct Connect4ColButton(usize);
#[derive(Component)]
struct Connect4CellText(usize, usize); // row, col
#[derive(Component)]
struct Connect4BackButton;
#[derive(Component)]
struct Connect4RestartButton;
#[derive(Component)]
struct Connect4SetupRoot;
#[derive(Component)]
struct Connect4VsCpuButton;
#[derive(Component)]
struct Connect4TwoPlayersButton;

pub struct Connect4Plugin;
impl Plugin for Connect4Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Connect4Game), spawn_connect4)
            .add_systems(OnExit(GameState::Connect4Game), cleanup_connect4)
            .add_systems(Update, update_connect4.run_if(in_state(GameState::Connect4Game)));
    }
}

fn spawn_connect4(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands.insert_resource(Connect4Session::new());
    commands
        .spawn((Connect4UiRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, screen_background(), Visibility::Visible, ZIndex(30)))
        .with_children(|overlay| {
            overlay.spawn((Connect4SetupRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.02, 0.04, 0.10, 0.85)), ZIndex(40))).with_children(|setup| {
                setup.spawn((Node { flex_direction: FlexDirection::Column, width: Val::Px(460.0), padding: UiRect::all(Val::Px(20.0)), row_gap: Val::Px(14.0), align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.10, 0.14, 0.28, 0.96)), BorderRadius::all(Val::Px(16.0)))).with_children(|panel| {
                    panel.spawn((Text::new("CONECTA 4 — Elige modo"), TextFont { font: font.clone(), font_size: 22.0, ..default() }, TextColor(Color::srgb(0.95, 0.85, 0.40))));
                    panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(12.0), ..default() }).with_children(|row| {
                        row.spawn((Button, Connect4VsCpuButton, Node { width: Val::Px(180.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.15, 0.42, 0.25)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new("1 Jugador vs CPU"), TextFont { font: font.clone(), font_size: 14.0, ..default() }, TextColor(Color::WHITE))); });
                        row.spawn((Button, Connect4TwoPlayersButton, Node { width: Val::Px(180.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new("2 Jugadores"), TextFont { font: font.clone(), font_size: 14.0, ..default() }, TextColor(Color::WHITE))); });
                    });
                });
            });
            overlay.spawn((Node { flex_direction: FlexDirection::Column, width: Val::Px(720.0), padding: UiRect::axes(Val::Px(24.0), Val::Px(20.0)), row_gap: Val::Px(10.0), align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.07, 0.09, 0.18, 0.96)), BorderRadius::all(Val::Px(16.0)))).with_children(|panel| {
                panel.spawn((Connect4Text(Connect4Field::Title), Text::new("CONECTA 4"), TextFont { font: font.clone(), font_size: 28.0, ..default() }, TextColor(Color::srgb(0.95, 0.85, 0.40))));
                panel.spawn((Connect4Text(Connect4Field::Status), Text::new("Turno: Rojo"), TextFont { font: font.clone(), font_size: 20.0, ..default() }, TextColor(Color::WHITE)));
                // botones de columnas
                panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(6.0), ..default() }).with_children(|row| {
                    for col in 0..COLS {
                        row.spawn((Button, Connect4ColButton(col), Node { width: Val::Px(80.0), height: Val::Px(36.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(8.0)))).with_children(|b| { b.spawn((Text::new(format!("↓ {}", col+1)), TextFont { font: font.clone(), font_size: 16.0, ..default() }, TextColor(Color::WHITE))); });
                    }
                });
                // grid 6x7
                panel.spawn((Node { display: Display::Grid, grid_template_columns: vec![GridTrack::px(60.0); COLS], grid_template_rows: vec![GridTrack::px(60.0); ROWS], column_gap: Val::Px(6.0), row_gap: Val::Px(6.0), padding: UiRect::all(Val::Px(10.0)), ..default() }, BackgroundColor(Color::srgb(0.08, 0.25, 0.58)), BorderRadius::all(Val::Px(18.0)))).with_children(|grid| {
                    for r in 0..ROWS {
                        for c in 0..COLS {
                            grid.spawn((Node { width: Val::Px(60.0), height: Val::Px(60.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.92, 0.86, 0.62)), BorderColor(Color::srgb(0.04, 0.16, 0.40)), BorderRadius::all(Val::Px(30.0)))).with_children(|cell| {
                                cell.spawn((Connect4CellText(r,c), Text::new("".to_string()), TextFont { font: font.clone(), font_size: 32.0, ..default() }, TextColor(Color::WHITE)));
                            });
                        }
                    }
                });
                panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(12.0), ..default() }).with_children(|row| {
                    row.spawn((Button, Connect4RestartButton, Node { width: Val::Px(160.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Reiniciar")), TextFont { font: font.clone(), font_size: 20.0, ..default() }, TextColor(Color::WHITE))); });
                    row.spawn((Button, Connect4BackButton, Node { width: Val::Px(160.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Volver")), TextFont { font: font.clone(), font_size: 20.0, ..default() }, TextColor(Color::WHITE))); });
                });
            });
        });
}

fn cleanup_connect4(mut commands: Commands, roots: Query<Entity, With<Connect4UiRoot>>) {
    for root in &roots { commands.entity(root).despawn(); }
    commands.remove_resource::<Connect4Session>();
}

fn update_connect4(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut session: ResMut<Connect4Session>,
    col_clicks: Query<(&Interaction, &Connect4ColButton), (Changed<Interaction>, Without<Connect4BackButton>)>,
    back_clicks: Query<&Interaction, (Changed<Interaction>, With<Connect4BackButton>)>,
    restart_clicks: Query<&Interaction, (Changed<Interaction>, With<Connect4RestartButton>)>,
    mut cell_texts: Query<(&Connect4CellText, &mut Text, &mut TextColor), Without<Connect4Text>>,
    mut status_text: Query<(&Connect4Text, &mut Text), Without<Connect4CellText>>,
    setup_vs_cpu: Query<&Interaction, (Changed<Interaction>, With<Connect4VsCpuButton>)>,
    setup_two: Query<&Interaction, (Changed<Interaction>, With<Connect4TwoPlayersButton>)>,
    mut setup_root: Query<&mut Visibility, With<Connect4SetupRoot>>,
) {
    if keys.just_pressed(KeyCode::Escape) { commands.set_state(GameState::ClassicMenu); return; }
    if back_clicks.single().map_or(false, |i| *i == Interaction::Pressed) { commands.set_state(GameState::ClassicMenu); return; }
    if !session.setup_done {
        for interaction in &setup_vs_cpu { if *interaction == Interaction::Pressed { *session = Connect4Session::new_with_mode(true); for mut v in &mut setup_root { *v = Visibility::Hidden; } return; } }
        for interaction in &setup_two { if *interaction == Interaction::Pressed { *session = Connect4Session::new_with_mode(false); for mut v in &mut setup_root { *v = Visibility::Hidden; } return; } }
        return;
    }
    if restart_clicks.single().map_or(false, |i| *i == Interaction::Pressed) { let vs_cpu = session.vs_cpu; *session = Connect4Session::new_with_mode(vs_cpu); }
    if session.winner.is_none() && !session.draw {
        let mut clicked: Option<usize> = None;
        for (interaction, btn) in &col_clicks { if *interaction == Interaction::Pressed { clicked = Some(btn.0); break; } }
        if let Some(col) = clicked {
            if session.board[0][col].is_none() {
                session.drop_in_col(col);
                if session.vs_cpu && session.winner.is_none() && !session.draw && session.turn == 'Y' {
                    session.cpu_move();
                }
            }
        }
    }
    for (field, mut text) in &mut status_text {
        if field.0 == Connect4Field::Status {
            if !session.setup_done { *text = Text::new("Elige modo arriba".to_string()); }
            else if let Some(w) = session.winner {
                *text = Text::new(if w == 'R' { "¡Gana Rojo!" } else { "¡Gana Amarillo!" }.to_string());
            } else if session.draw {
                *text = Text::new("¡Empate!".to_string());
            } else {
                let suffix = if session.vs_cpu { if session.turn == 'R' { "Rojo (J1)" } else { "Amarillo (CPU)" } } else { if session.turn == 'R' { "Rojo (J1)" } else { "Amarillo (J2)" } };
                *text = Text::new(format!("Turno: {}", suffix));
            }
        }
    }
    for (cell, mut text, mut color) in &mut cell_texts {
        let ch = session.board[cell.0][cell.1];
        *text = Text::new(ch.map(|c| if c=='R' { "●".to_string() } else { "●".to_string() }).unwrap_or("".to_string()));
        *color = TextColor(match ch { Some('R') => Color::srgb(0.95, 0.30, 0.30), Some('Y') => Color::srgb(0.95, 0.85, 0.30), _ => Color::WHITE });
    }
    // colorear celdas según ficha
    // (el color del texto ya lo hace)
}
