//! Laberinto 15×15 — encuentra la salida.

use bevy::prelude::*;
use rand::seq::SliceRandom;

use crate::game::GameState;
use crate::learning::screen_background;
use crate::i18n::tr;

const W: usize = 15;
const H: usize = 15;

#[derive(Resource, Clone)]
struct LabyrinthSession {
    walls: [[bool; W]; H], // true = muro
    player: (usize, usize),
    goal: (usize, usize),
    steps: u32,
    won: bool,
}

impl LabyrinthSession {
    fn new() -> Self {
        let mut walls = [[true; W]; H];
        // DFS maze generation (odd cells son pasillos)
        let mut stack = vec![(1usize,1usize)];
        walls[1][1] = false;
        let mut rng = rand::thread_rng();
        let dirs = [(0i32,2i32),(2,0),(0,-2),(-2,0)];
        while let Some(&(r,c)) = stack.last() {
            let mut neigh = Vec::new();
            for (dr,dc) in dirs {
                let nr = r as i32 + dr;
                let nc = c as i32 + dc;
                if nr>0 && nr < (H as i32-1) && nc>0 && nc < (W as i32-1) && walls[nr as usize][nc as usize] {
                    neigh.push((nr as usize, nc as usize, (r as i32+dr/2) as usize, (c as i32+dc/2) as usize));
                }
            }
            if neigh.is_empty() { stack.pop(); }
            else {
                neigh.shuffle(&mut rng);
                let (nr,nc,wr,wc) = neigh[0];
                walls[nr][nc]=false;
                walls[wr][wc]=false;
                stack.push((nr,nc));
            }
        }
        walls[1][1]=false;
        walls[H-2][W-2]=false;
        Self { walls, player: (1,1), goal: (H-2,W-2), steps:0, won:false }
    }
    fn can_move(&self, r: usize, c: usize) -> bool {
        r < H && c < W && !self.walls[r][c]
    }
    fn try_move(&mut self, dr: i32, dc: i32) {
        if self.won { return; }
        let (r,c) = self.player;
        let nr = r as i32 + dr;
        let nc = c as i32 + dc;
        if nr>=0 && nc>=0 {
            let (ur, uc) = (nr as usize, nc as usize);
            if self.can_move(ur, uc) {
                self.player = (ur, uc);
                self.steps += 1;
                if self.player == self.goal { self.won = true; }
            }
        }
    }
}

#[derive(Component)]
struct LabyrinthUiRoot;
#[derive(Component)]
struct LabyrinthText(LabField);
#[derive(Clone, Copy, PartialEq, Eq)]
enum LabField { Title, Status }
#[derive(Component)]
struct LabyrinthCell(usize, usize);
#[derive(Component)]
struct LabyrinthBackButton;
#[derive(Component)]
struct LabyrinthRestartButton;
#[derive(Component)]
struct LabyrinthUpButton;
#[derive(Component)]
struct LabyrinthDownButton;
#[derive(Component)]
struct LabyrinthLeftButton;
#[derive(Component)]
struct LabyrinthRightButton;

pub struct LabyrinthPlugin;
impl Plugin for LabyrinthPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::LabyrinthGame), spawn_labyrinth)
            .add_systems(OnExit(GameState::LabyrinthGame), cleanup_labyrinth)
            .add_systems(Update, update_labyrinth.run_if(in_state(GameState::LabyrinthGame)));
    }
}

fn spawn_labyrinth(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands.insert_resource(LabyrinthSession::new());
    commands.spawn((LabyrinthUiRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, padding: UiRect::all(Val::Px(8.0)), ..default() }, screen_background(), Visibility::Visible, ZIndex(30)))
        .with_children(|overlay| {
            overlay.spawn((Node { flex_direction: FlexDirection::Column, width: Val::Px(520.0), padding: UiRect::all(Val::Px(14.0)), row_gap: Val::Px(8.0), align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.07,0.09,0.18,0.96)), BorderRadius::all(Val::Px(16.0)))).with_children(|panel| {
                panel.spawn((LabyrinthText(LabField::Title), Text::new("LABERINTO"), TextFont { font: font.clone(), font_size: 28.0, ..default() }, TextColor(Color::srgb(0.95,0.85,0.40))));
                panel.spawn((LabyrinthText(LabField::Status), Text::new("Usa WASD / Flechas"), TextFont { font: font.clone(), font_size: 16.0, ..default() }, TextColor(Color::WHITE)));
                panel.spawn((Node { display: Display::Grid, grid_template_columns: vec![GridTrack::px(28.0); W], grid_template_rows: vec![GridTrack::px(28.0); H], column_gap: Val::Px(1.0), row_gap: Val::Px(1.0), ..default() }, BackgroundColor(Color::srgb(0.10,0.12,0.14)), BorderRadius::all(Val::Px(8.0)))).with_children(|grid| {
                    for r in 0..H { for c in 0..W {
                        grid.spawn((LabyrinthCell(r,c), Node { width: Val::Px(28.0), height: Val::Px(28.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgb(0.85,0.85,0.85)), BorderRadius::all(Val::Px(3.0))));
                    }}
                });
                // Controles táctiles
                panel.spawn(Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(4.0), align_items: AlignItems::Center, ..default() }).with_children(|pad| {
                    pad.spawn((Button, LabyrinthUpButton, Node { width: Val::Px(56.0), height: Val::Px(36.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(1.0)), ..default() }, BackgroundColor(Color::srgb(0.20,0.38,0.66)), BorderRadius::all(Val::Px(8.0)))).with_children(|b|{ b.spawn((Text::new("▲"), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE)));});
                    pad.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(6.0), ..default() }).with_children(|row| {
                        row.spawn((Button, LabyrinthLeftButton, Node { width: Val::Px(56.0), height: Val::Px(36.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(1.0)), ..default() }, BackgroundColor(Color::srgb(0.20,0.38,0.66)), BorderRadius::all(Val::Px(8.0)))).with_children(|b|{ b.spawn((Text::new("◀"), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE)));});
                        row.spawn((Button, LabyrinthDownButton, Node { width: Val::Px(56.0), height: Val::Px(36.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(1.0)), ..default() }, BackgroundColor(Color::srgb(0.20,0.38,0.66)), BorderRadius::all(Val::Px(8.0)))).with_children(|b|{ b.spawn((Text::new("▼"), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE)));});
                        row.spawn((Button, LabyrinthRightButton, Node { width: Val::Px(56.0), height: Val::Px(36.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(1.0)), ..default() }, BackgroundColor(Color::srgb(0.20,0.38,0.66)), BorderRadius::all(Val::Px(8.0)))).with_children(|b|{ b.spawn((Text::new("▶"), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE)));});
                    });
                });
                panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(10.0), ..default() }).with_children(|row| {
                    row.spawn((Button, LabyrinthRestartButton, Node { width: Val::Px(140.0), height: Val::Px(36.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(1.0)), ..default() }, BackgroundColor(Color::srgb(0.20,0.38,0.66)), BorderRadius::all(Val::Px(8.0)))).with_children(|b|{ b.spawn((Text::new(tr("Reiniciar")), TextFont { font: font.clone(), font_size: 16.0, ..default() }, TextColor(Color::WHITE)));});
                    row.spawn((Button, LabyrinthBackButton, Node { width: Val::Px(140.0), height: Val::Px(36.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(1.0)), ..default() }, BackgroundColor(Color::srgb(0.20,0.38,0.66)), BorderRadius::all(Val::Px(8.0)))).with_children(|b|{ b.spawn((Text::new(tr("Volver")), TextFont { font: font.clone(), font_size: 16.0, ..default() }, TextColor(Color::WHITE)));});
                });
            });
        });
}

fn cleanup_labyrinth(mut commands: Commands, roots: Query<Entity, With<LabyrinthUiRoot>>) {
    for r in &roots { commands.entity(r).despawn(); }
    commands.remove_resource::<LabyrinthSession>();
}

fn update_labyrinth(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut session: ResMut<LabyrinthSession>,
    up: Query<&Interaction, (Changed<Interaction>, With<LabyrinthUpButton>)>,
    down: Query<&Interaction, (Changed<Interaction>, With<LabyrinthDownButton>)>,
    left: Query<&Interaction, (Changed<Interaction>, With<LabyrinthLeftButton>)>,
    right: Query<&Interaction, (Changed<Interaction>, With<LabyrinthRightButton>)>,
    back: Query<&Interaction, (Changed<Interaction>, With<LabyrinthBackButton>)>,
    restart: Query<&Interaction, (Changed<Interaction>, With<LabyrinthRestartButton>)>,
    mut texts: Query<(&LabyrinthText, &mut Text)>,
    mut cells: Query<(&LabyrinthCell, &mut BackgroundColor)>,
) {
    if keys.just_pressed(KeyCode::Escape) || back.single().map_or(false, |i| *i==Interaction::Pressed) { commands.set_state(GameState::ClassicMenu); return; }
    if restart.single().map_or(false, |i| *i==Interaction::Pressed) { *session = LabyrinthSession::new(); }
    let mut moved = false;
    if keys.just_pressed(KeyCode::KeyW) || keys.just_pressed(KeyCode::ArrowUp) || up.single().map_or(false, |i| *i==Interaction::Pressed) { session.try_move(-1,0); moved=true; }
    if keys.just_pressed(KeyCode::KeyS) || keys.just_pressed(KeyCode::ArrowDown) || down.single().map_or(false, |i| *i==Interaction::Pressed) { session.try_move(1,0); moved=true; }
    if keys.just_pressed(KeyCode::KeyA) || keys.just_pressed(KeyCode::ArrowLeft) || left.single().map_or(false, |i| *i==Interaction::Pressed) { session.try_move(0,-1); moved=true; }
    if keys.just_pressed(KeyCode::KeyD) || keys.just_pressed(KeyCode::ArrowRight) || right.single().map_or(false, |i| *i==Interaction::Pressed) { session.try_move(0,1); moved=true; }
    let _ = moved;
    for (field, mut text) in &mut texts {
        if field.0 == LabField::Status {
            if session.won { *text = Text::new(format!("¡Salida encontrada! Pasos: {} — Pulsa Reiniciar", session.steps)); }
            else { *text = Text::new(format!("Pasos: {} — Llega al dorado", session.steps)); }
        }
    }
    for (cell, mut bg) in &mut cells {
        let (r,c) = (cell.0, cell.1);
        let is_wall = session.walls[r][c];
        let is_player = session.player == (r,c);
        let is_goal = session.goal == (r,c);
        *bg = BackgroundColor(if is_player { Color::srgb(0.30,0.85,0.30) } else if is_goal { Color::srgb(0.95,0.85,0.30) } else if is_wall { Color::srgb(0.12,0.14,0.18) } else { Color::srgb(0.96,0.96,0.94) });
    }
}
