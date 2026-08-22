//! Puzzle 15 — desliza fichas 1-15 para ordenar.

use bevy::prelude::*;
use rand::seq::SliceRandom;

use crate::game::GameState;
use crate::learning::screen_background;
use crate::i18n::tr;

const SIZE: usize = 4;

#[derive(Resource, Clone)]
struct PuzzleSession {
    board: [[u8; SIZE]; SIZE], // 0 = hueco
    empty: (usize, usize),
    moves: u32,
    won: bool,
}

impl PuzzleSession {
    fn new() -> Self {
        let mut board = [[0u8; SIZE]; SIZE];
        let mut v = 1u8;
        for r in 0..SIZE { for c in 0..SIZE { if r==SIZE-1 && c==SIZE-1 { board[r][c]=0; } else { board[r][c]=v; v+=1; } } }
        let mut s = Self { board, empty: (SIZE-1, SIZE-1), moves: 0, won: false };
        // Mezclar con 300 movimientos válidos para garantizar soluble
        let mut rng = rand::thread_rng();
        for _ in 0..300 {
            let mut opts = Vec::new();
            let (er, ec) = s.empty;
            for (dr,dc) in [(-1,0),(1,0),(0,-1),(0,1)] {
                let nr = er as i32 + dr; let nc = ec as i32 + dc;
                if nr>=0 && nr < SIZE as i32 && nc>=0 && nc < SIZE as i32 { opts.push((nr as usize, nc as usize)); }
            }
            if let Some(&(r,c)) = opts.choose(&mut rng) {
                s.board[s.empty.0][s.empty.1] = s.board[r][c];
                s.board[r][c]=0;
                s.empty=(r,c);
            }
        }
        s.moves=0;
        s
    }
    fn can_move(&self, r: usize, c: usize) -> bool {
        let (er, ec) = self.empty;
        (r as i32 - er as i32).abs() + (c as i32 - ec as i32).abs() == 1
    }
    fn do_move(&mut self, r: usize, c: usize) {
        if self.won || !self.can_move(r,c) { return; }
        let (er, ec) = self.empty;
        self.board[er][ec] = self.board[r][c];
        self.board[r][c]=0;
        self.empty=(r,c);
        self.moves+=1;
        // comprobar victoria
        let mut v=1;
        for rr in 0..SIZE { for cc in 0..SIZE {
            if rr==SIZE-1 && cc==SIZE-1 { if self.board[rr][cc]!=0 { return; } }
            else { if self.board[rr][cc]!=v { return; } v+=1; }
        }}
        self.won=true;
    }
}

#[derive(Component)]
struct PuzzleUiRoot;
#[derive(Component)]
struct PuzzleText(PuzzleField);
#[derive(Clone, Copy, PartialEq, Eq)]
enum PuzzleField { Title, Status }
#[derive(Component)]
struct PuzzleCell(usize, usize);
#[derive(Component)]
struct PuzzleBackButton;
#[derive(Component)]
struct PuzzleRestartButton;

pub struct Puzzle15Plugin;
impl Plugin for Puzzle15Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Puzzle15Game), spawn_puzzle)
            .add_systems(OnExit(GameState::Puzzle15Game), cleanup_puzzle)
            .add_systems(Update, update_puzzle.run_if(in_state(GameState::Puzzle15Game)));
    }
}

fn spawn_puzzle(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands.insert_resource(PuzzleSession::new());
    commands.spawn((PuzzleUiRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, padding: UiRect::all(Val::Px(8.0)), ..default() }, screen_background(), Visibility::Visible, ZIndex(30)))
        .with_children(|overlay| {
            overlay.spawn((Node { flex_direction: FlexDirection::Column, width: Val::Px(520.0), padding: UiRect::all(Val::Px(14.0)), row_gap: Val::Px(8.0), align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.07,0.09,0.18,0.96)), BorderRadius::all(Val::Px(16.0)))).with_children(|panel| {
                panel.spawn((PuzzleText(PuzzleField::Title), Text::new("PUZZLE 15"), TextFont { font: font.clone(), font_size: 28.0, ..default() }, TextColor(Color::srgb(0.95,0.85,0.40))));
                panel.spawn((PuzzleText(PuzzleField::Status), Text::new("Ordena 1-15"), TextFont { font: font.clone(), font_size: 16.0, ..default() }, TextColor(Color::WHITE)));
                panel.spawn((Node { display: Display::Grid, grid_template_columns: vec![GridTrack::px(72.0); SIZE], grid_template_rows: vec![GridTrack::px(72.0); SIZE], column_gap: Val::Px(4.0), row_gap: Val::Px(4.0), padding: UiRect::all(Val::Px(6.0)), ..default() }, BackgroundColor(Color::srgb(0.12,0.14,0.18)), BorderRadius::all(Val::Px(10.0)))).with_children(|grid| {
                    for r in 0..SIZE { for c in 0..SIZE {
                        grid.spawn((Button, PuzzleCell(r,c), Node { width: Val::Px(72.0), height: Val::Px(72.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(1.0)), ..default() }, BackgroundColor(Color::srgb(0.20,0.38,0.66)), BorderRadius::all(Val::Px(8.0)))).with_children(|cell| {
                            cell.spawn((Text::new("".to_string()), TextFont { font: font.clone(), font_size: 22.0, ..default() }, TextColor(Color::WHITE)));
                        });
                    }}
                });
                panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(10.0), ..default() }).with_children(|row| {
                    row.spawn((Button, PuzzleRestartButton, Node { width: Val::Px(140.0), height: Val::Px(36.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(1.0)), ..default() }, BackgroundColor(Color::srgb(0.20,0.38,0.66)), BorderRadius::all(Val::Px(8.0)))).with_children(|b|{ b.spawn((Text::new(tr("Reiniciar")), TextFont { font: font.clone(), font_size: 14.0, ..default() }, TextColor(Color::WHITE)));});
                    row.spawn((Button, PuzzleBackButton, Node { width: Val::Px(140.0), height: Val::Px(36.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(1.0)), ..default() }, BackgroundColor(Color::srgb(0.20,0.38,0.66)), BorderRadius::all(Val::Px(8.0)))).with_children(|b|{ b.spawn((Text::new(tr("Volver")), TextFont { font: font.clone(), font_size: 14.0, ..default() }, TextColor(Color::WHITE)));});
                });
            });
        });
}

fn cleanup_puzzle(mut commands: Commands, roots: Query<Entity, With<PuzzleUiRoot>>) {
    for r in &roots { commands.entity(r).despawn(); }
    commands.remove_resource::<PuzzleSession>();
}

fn update_puzzle(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut session: ResMut<PuzzleSession>,
    cell_clicks: Query<(&Interaction, &PuzzleCell), (Changed<Interaction>, Without<PuzzleBackButton>)>,
    back: Query<&Interaction, (Changed<Interaction>, With<PuzzleBackButton>)>,
    restart: Query<&Interaction, (Changed<Interaction>, With<PuzzleRestartButton>)>,
    mut texts: Query<(&PuzzleText, &mut Text)>,
    mut cells: Query<(&PuzzleCell, &mut BackgroundColor, &Children)>,
    mut cell_texts: Query<&mut Text, Without<PuzzleText>>,
) {
    if keys.just_pressed(KeyCode::Escape) || back.single().map_or(false, |i| *i==Interaction::Pressed) { commands.set_state(GameState::ClassicMenu); return; }
    if restart.single().map_or(false, |i| *i==Interaction::Pressed) { *session = PuzzleSession::new(); }
    let mut clicked: Option<(usize,usize)> = None;
    for (inter, cell) in &cell_clicks { if *inter==Interaction::Pressed { clicked=Some((cell.0, cell.1)); break; } }
    if let Some((r,c)) = clicked { session.do_move(r,c); }
    // teclas flechas mueven hueco inverso
    if !session.won {
        let (er, ec) = session.empty;
        if keys.just_pressed(KeyCode::ArrowUp) && er+1 < SIZE { let r=er+1; let c=ec; if session.can_move(r,c) { session.do_move(r,c); } }
        if keys.just_pressed(KeyCode::ArrowDown) && er>0 { let r=er-1; let c=ec; if session.can_move(r,c) { session.do_move(r,c); } }
        if keys.just_pressed(KeyCode::ArrowLeft) && ec+1 < SIZE { let r=er; let c=ec+1; if session.can_move(r,c) { session.do_move(r,c); } }
        if keys.just_pressed(KeyCode::ArrowRight) && ec>0 { let r=er; let c=ec-1; if session.can_move(r,c) { session.do_move(r,c); } }
    }
    for (field, mut text) in &mut texts {
        if field.0 == PuzzleField::Status {
            if session.won { *text = Text::new(format!("¡Completado! Movimientos: {}", session.moves)); }
            else { *text = Text::new(format!("Movimientos: {} — Toca ficha adyacente al hueco", session.moves)); }
        }
    }
    for (cell, mut bg, children) in &mut cells {
        let (r,c) = (cell.0, cell.1);
        let v = session.board[r][c];
        let is_empty = v==0;
        *bg = BackgroundColor(if is_empty { Color::srgba(0.12,0.14,0.18,0.0) } else if session.won { Color::srgb(0.30,0.70,0.30) } else { Color::srgb(0.20,0.38,0.66) });
        for child in children.iter() {
            if let Ok(mut text) = cell_texts.get_mut(child) {
                *text = Text::new(if is_empty { "".to_string() } else { v.to_string() });
            }
        }
    }
}
