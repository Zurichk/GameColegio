//! Buscaminas 8×8 — 10 minas, banderas, flood fill.

use bevy::prelude::*;
use rand::seq::SliceRandom;

use crate::game::GameState;
use crate::learning::screen_background;
use crate::i18n::tr;

const SIZE: usize = 8;
const MINES: usize = 10;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Cell { Mine, Number(u8) }

#[derive(Clone, Copy, PartialEq, Eq)]
enum State { Hidden, Revealed, Flagged }

#[derive(Resource, Clone)]
struct MinesweeperSession {
    board: [[Cell; SIZE]; SIZE],
    state: [[State; SIZE]; SIZE],
    revealed: usize,
    flagged: usize,
    first_click: bool,
    game_over: bool,
    won: bool,
    flag_mode: bool,
}

impl MinesweeperSession {
    fn new() -> Self {
        Self { board: [[Cell::Number(0); SIZE]; SIZE], state: [[State::Hidden; SIZE]; SIZE], revealed: 0, flagged: 0, first_click: true, game_over: false, won: false, flag_mode: false }
    }
    fn place_mines(&mut self, avoid_r: usize, avoid_c: usize) {
        let mut cells: Vec<(usize,usize)> = (0..SIZE).flat_map(|r| (0..SIZE).map(move |c| (r,c))).filter(|&(r,c)| !(r==avoid_r && c==avoid_c)).collect();
        cells.shuffle(&mut rand::thread_rng());
        for (r,c) in cells.into_iter().take(MINES) {
            self.board[r][c] = Cell::Mine;
        }
        // números
        for r in 0..SIZE { for c in 0..SIZE {
            if self.board[r][c] == Cell::Mine { continue; }
            let mut count = 0;
            for dr in -1i32..=1 { for dc in -1i32..=1 {
                if dr==0 && dc==0 { continue; }
                let nr = r as i32 + dr;
                let nc = c as i32 + dc;
                if nr>=0 && nr < SIZE as i32 && nc>=0 && nc < SIZE as i32 {
                    if self.board[nr as usize][nc as usize] == Cell::Mine { count += 1; }
                }
            }}
            self.board[r][c] = Cell::Number(count);
        }}
    }
    fn reveal(&mut self, r: usize, c: usize) {
        if self.state[r][c] != State::Hidden { return; }
        if self.first_click {
            self.place_mines(r,c);
            self.first_click = false;
        }
        match self.board[r][c] {
            Cell::Mine => { self.state[r][c] = State::Revealed; self.game_over = true; }
            Cell::Number(0) => {
                self.state[r][c] = State::Revealed; self.revealed += 1;
                for dr in -1i32..=1 { for dc in -1i32..=1 {
                    if dr==0 && dc==0 { continue; }
                    let nr = r as i32 + dr;
                    let nc = c as i32 + dc;
                    if nr>=0 && nr < SIZE as i32 && nc>=0 && nc < SIZE as i32 {
                        let (nr, nc) = (nr as usize, nc as usize);
                        if self.state[nr][nc]==State::Hidden { self.reveal(nr,nc); }
                    }
                }}
            },
            Cell::Number(_) => { self.state[r][c] = State::Revealed; self.revealed += 1; }
        }
        if self.revealed == SIZE*SIZE - MINES { self.won = true; self.game_over = true; }
    }
    fn toggle_flag(&mut self, r: usize, c: usize) {
        match self.state[r][c] {
            State::Hidden => { self.state[r][c] = State::Flagged; self.flagged += 1; },
            State::Flagged => { self.state[r][c] = State::Hidden; self.flagged -= 1; },
            _ => {}
        }
    }
}

#[derive(Component)]
struct MinesweeperUiRoot;
#[derive(Component)]
struct MinesweeperText(MinesweeperField);
#[derive(Clone, Copy, PartialEq, Eq)]
enum MinesweeperField { Title, Status }
#[derive(Component)]
struct MinesweeperCellButton(usize, usize);
#[derive(Component)]
struct MinesweeperFlagToggle;
#[derive(Component)]
struct MinesweeperBackButton;
#[derive(Component)]
struct MinesweeperRestartButton;

pub struct MinesweeperPlugin;
impl Plugin for MinesweeperPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MinesweeperGame), spawn_minesweeper)
            .add_systems(OnExit(GameState::MinesweeperGame), cleanup_minesweeper)
            .add_systems(Update, update_minesweeper.run_if(in_state(GameState::MinesweeperGame)));
    }
}

fn spawn_minesweeper(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands.insert_resource(MinesweeperSession::new());
    commands
        .spawn((MinesweeperUiRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, screen_background(), Visibility::Visible, ZIndex(30)))
        .with_children(|overlay| {
            overlay.spawn((Node { flex_direction: FlexDirection::Column, width: Val::Px(720.0), padding: UiRect::axes(Val::Px(20.0), Val::Px(16.0)), row_gap: Val::Px(10.0), align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.07, 0.09, 0.18, 0.96)), BorderRadius::all(Val::Px(16.0)))).with_children(|panel| {
                panel.spawn((MinesweeperText(MinesweeperField::Title), Text::new("BUSCAMINAS 8×8"), TextFont { font: font.clone(), font_size: 28.0, ..default() }, TextColor(Color::srgb(0.95, 0.85, 0.40))));
                panel.spawn((MinesweeperText(MinesweeperField::Status), Text::new("Minas: 10  Banderas: 0"), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE)));
                panel.spawn(Node { display: Display::Grid, grid_template_columns: vec![GridTrack::px(48.0); SIZE], grid_template_rows: vec![GridTrack::px(48.0); SIZE], column_gap: Val::Px(4.0), row_gap: Val::Px(4.0), ..default() }).with_children(|grid| {
                    for r in 0..SIZE { for c in 0..SIZE {
                        grid.spawn((Button, MinesweeperCellButton(r,c), Node { width: Val::Px(48.0), height: Val::Px(48.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(1.0)), ..default() }, BackgroundColor(Color::srgb(0.18, 0.22, 0.34)), BorderColor(Color::srgb(0.45, 0.50, 0.65)), BorderRadius::all(Val::Px(6.0)))).with_children(|cell| { cell.spawn((Text::new("".to_string()), TextFont { font: font.clone(), font_size: 20.0, ..default() }, TextColor(Color::WHITE))); });
                    }}
                });
                panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(12.0), ..default() }).with_children(|row| {
                    row.spawn((Button, MinesweeperFlagToggle, Node { width: Val::Px(160.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.30, 0.40, 0.20)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Modo Bandera: OFF")), TextFont { font: font.clone(), font_size: 16.0, ..default() }, TextColor(Color::WHITE))); });
                    row.spawn((Button, MinesweeperRestartButton, Node { width: Val::Px(120.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Reiniciar")), TextFont { font: font.clone(), font_size: 16.0, ..default() }, TextColor(Color::WHITE))); });
                    row.spawn((Button, MinesweeperBackButton, Node { width: Val::Px(120.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Volver")), TextFont { font: font.clone(), font_size: 16.0, ..default() }, TextColor(Color::WHITE))); });
                });
            });
        });
}

fn cleanup_minesweeper(mut commands: Commands, roots: Query<Entity, With<MinesweeperUiRoot>>) {
    for root in &roots { commands.entity(root).despawn(); }
    commands.remove_resource::<MinesweeperSession>();
}

fn update_minesweeper(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut session: ResMut<MinesweeperSession>,
    cell_clicks: Query<(&Interaction, &MinesweeperCellButton), (Changed<Interaction>, Without<MinesweeperBackButton>)>,
    flag_clicks: Query<&Interaction, (Changed<Interaction>, With<MinesweeperFlagToggle>)>,
    back_clicks: Query<&Interaction, (Changed<Interaction>, With<MinesweeperBackButton>)>,
    restart_clicks: Query<&Interaction, (Changed<Interaction>, With<MinesweeperRestartButton>)>,
    mut texts: Query<(&MinesweeperText, &mut Text)>,
    mut cell_query: Query<(&MinesweeperCellButton, &mut BackgroundColor, &Children)>,
    mut cell_texts: Query<&mut Text, Without<MinesweeperText>>,
) {
    if keys.just_pressed(KeyCode::Escape) { commands.set_state(GameState::ClassicMenu); return; }
    if back_clicks.single().map_or(false, |i| *i == Interaction::Pressed) { commands.set_state(GameState::ClassicMenu); return; }
    if restart_clicks.single().map_or(false, |i| *i == Interaction::Pressed) { *session = MinesweeperSession::new(); }
    for interaction in &flag_clicks { if *interaction == Interaction::Pressed { session.flag_mode = !session.flag_mode; } }
    if !session.game_over {
        for (interaction, btn) in &cell_clicks {
            if *interaction == Interaction::Pressed {
                let (r,c) = (btn.0, btn.1);
                if session.flag_mode { session.toggle_flag(r,c); } else { session.reveal(r,c); }
            }
        }
    }
    for (field, mut text) in &mut texts {
        if field.0 == MinesweeperField::Status {
            if session.game_over {
                *text = Text::new(if session.won { "¡Ganaste! Todas las casillas seguras reveladas" } else { "¡Boom! Pisaste una mina" }.to_string());
            } else {
                *text = Text::new(format!("Minas: {}  Banderas: {}  Modo: {}", MINES, session.flagged, if session.flag_mode {"BANDERA"} else {"REVELAR"}));
            }
        }
    }
    for (btn, mut bg, children) in &mut cell_query {
        let (r,c) = (btn.0, btn.1);
        let state = session.state[r][c];
        let cell = session.board[r][c];
        *bg = BackgroundColor(match state {
            State::Hidden => Color::srgb(0.18, 0.22, 0.34),
            State::Flagged => Color::srgb(0.60, 0.30, 0.20),
            State::Revealed => match cell { Cell::Mine => Color::srgb(0.85, 0.20, 0.20), Cell::Number(0) => Color::srgb(0.12, 0.14, 0.24), _ => Color::srgb(0.22, 0.24, 0.34) },
        });
        for child in children.iter() {
            if let Ok(mut text) = cell_texts.get_mut(child) {
                *text = Text::new(match state {
                    State::Hidden => "".to_string(),
                    State::Flagged => "🚩".to_string(),
                    State::Revealed => match cell { Cell::Mine => "💣".to_string(), Cell::Number(0) => "".to_string(), Cell::Number(n) => n.to_string() },
                });
            }
        }
    }
    // actualizar texto del botón bandera
    // (no crítico, se actualiza vía status)
}
