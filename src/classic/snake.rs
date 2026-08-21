//! Snake 15×15 — WASD, comer, crecer.

use bevy::prelude::*;
use rand::Rng;

use crate::game::GameState;
use crate::learning::screen_background;
use crate::i18n::tr;

const SIZE: usize = 15;
const TICK: f32 = 0.18;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Dir { Up, Down, Left, Right }

#[derive(Resource, Clone)]
struct SnakeSession {
    snake: Vec<(usize, usize)>,
    dir: Dir,
    next_dir: Dir,
    food: (usize, usize),
    score: u32,
    timer: f32,
    game_over: bool,
}

impl SnakeSession {
    fn new() -> Self {
        let mut s = Self { snake: vec![(7,7), (7,6), (7,5)], dir: Dir::Right, next_dir: Dir::Right, food: (7,10), score: 0, timer: 0.0, game_over: false };
        s.place_food();
        s
    }
    fn place_food(&mut self) {
        let mut rng = rand::thread_rng();
        loop {
            let r = rng.gen_range(0..SIZE);
            let c = rng.gen_range(0..SIZE);
            if !self.snake.contains(&(r,c)) { self.food = (r,c); break; }
        }
    }
    fn step(&mut self) {
        if self.game_over { return; }
        self.dir = self.next_dir;
        let (hr, hc) = self.snake[0];
        let (nr, nc) = match self.dir {
            Dir::Up => (hr.checked_sub(1), Some(hc)),
            Dir::Down => (Some(hr+1), Some(hc)),
            Dir::Left => (Some(hr), hc.checked_sub(1)),
            Dir::Right => (Some(hr), Some(hc+1)),
        };
        let (nr, nc) = match (nr, nc) {
            (Some(r), Some(c)) if r < SIZE && c < SIZE => (r,c),
            _ => { self.game_over = true; return; }
        };
        if self.snake.contains(&(nr,nc)) { self.game_over = true; return; }
        self.snake.insert(0, (nr,nc));
        if (nr,nc) == self.food {
            self.score += 1;
            self.place_food();
        } else {
            self.snake.pop();
        }
    }
}

#[derive(Component)]
struct SnakeUiRoot;
#[derive(Component)]
struct SnakeText(SnakeField);
#[derive(Clone, Copy, PartialEq, Eq)]
enum SnakeField { Title, Score }
#[derive(Component)]
struct SnakeCell(usize, usize);
#[derive(Component)]
struct SnakeBackButton;
#[derive(Component)]
struct SnakeRestartButton;

pub struct SnakePlugin;
impl Plugin for SnakePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::SnakeGame), spawn_snake)
            .add_systems(OnExit(GameState::SnakeGame), cleanup_snake)
            .add_systems(Update, update_snake.run_if(in_state(GameState::SnakeGame)));
    }
}

fn spawn_snake(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands.insert_resource(SnakeSession::new());
    commands
        .spawn((SnakeUiRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, screen_background(), Visibility::Visible, ZIndex(30)))
        .with_children(|overlay| {
            overlay.spawn((Node { flex_direction: FlexDirection::Column, width: Val::Px(640.0), padding: UiRect::axes(Val::Px(20.0), Val::Px(16.0)), row_gap: Val::Px(10.0), align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.07, 0.09, 0.18, 0.96)), BorderRadius::all(Val::Px(16.0)))).with_children(|panel| {
                panel.spawn((SnakeText(SnakeField::Title), Text::new("SNAKE"), TextFont { font: font.clone(), font_size: 28.0, ..default() }, TextColor(Color::srgb(0.95, 0.85, 0.40))));
                panel.spawn((SnakeText(SnakeField::Score), Text::new("Puntos: 0"), TextFont { font: font.clone(), font_size: 20.0, ..default() }, TextColor(Color::WHITE)));
                panel.spawn(Node { display: Display::Grid, grid_template_columns: vec![GridTrack::px(32.0); SIZE], grid_template_rows: vec![GridTrack::px(32.0); SIZE], column_gap: Val::Px(2.0), row_gap: Val::Px(2.0), ..default() }).with_children(|grid| {
                    for r in 0..SIZE { for c in 0..SIZE {
                        grid.spawn((SnakeCell(r,c), Node { width: Val::Px(32.0), height: Val::Px(32.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgb(0.12, 0.14, 0.24)), BorderRadius::all(Val::Px(4.0))));
                    }}
                });
                panel.spawn((Text::new("WASD / Flechas para mover"), TextFont { font: font.clone(), font_size: 16.0, ..default() }, TextColor(Color::srgba(1.0,1.0,1.0,0.6))));
                panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(12.0), ..default() }).with_children(|row| {
                    row.spawn((Button, SnakeRestartButton, Node { width: Val::Px(140.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Reiniciar")), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE))); });
                    row.spawn((Button, SnakeBackButton, Node { width: Val::Px(140.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Volver")), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE))); });
                });
            });
        });
}

fn cleanup_snake(mut commands: Commands, roots: Query<Entity, With<SnakeUiRoot>>) {
    for root in &roots { commands.entity(root).despawn(); }
    commands.remove_resource::<SnakeSession>();
}

fn update_snake(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut commands: Commands,
    mut session: ResMut<SnakeSession>,
    back_clicks: Query<&Interaction, (Changed<Interaction>, With<SnakeBackButton>)>,
    restart_clicks: Query<&Interaction, (Changed<Interaction>, With<SnakeRestartButton>)>,
    mut cell_query: Query<(&SnakeCell, &mut BackgroundColor)>,
    mut texts: Query<(&SnakeText, &mut Text)>,
) {
    if keys.just_pressed(KeyCode::Escape) || back_clicks.single().map_or(false, |i| *i == Interaction::Pressed) { commands.set_state(GameState::ClassicMenu); return; }
    if restart_clicks.single().map_or(false, |i| *i == Interaction::Pressed) { *session = SnakeSession::new(); }
    // dirección
    let dir = if keys.just_pressed(KeyCode::KeyW) || keys.just_pressed(KeyCode::ArrowUp) { Some(Dir::Up) }
    else if keys.just_pressed(KeyCode::KeyS) || keys.just_pressed(KeyCode::ArrowDown) { Some(Dir::Down) }
    else if keys.just_pressed(KeyCode::KeyA) || keys.just_pressed(KeyCode::ArrowLeft) { Some(Dir::Left) }
    else if keys.just_pressed(KeyCode::KeyD) || keys.just_pressed(KeyCode::ArrowRight) { Some(Dir::Right) } else { None };
    if let Some(d) = dir {
        // no reversa inmediata
        if !matches!((session.dir, d), (Dir::Up, Dir::Down) | (Dir::Down, Dir::Up) | (Dir::Left, Dir::Right) | (Dir::Right, Dir::Left)) {
            session.next_dir = d;
        }
    }
    if !session.game_over {
        session.timer += time.delta_secs();
        if session.timer >= TICK {
            session.timer = 0.0;
            session.step();
        }
    }
    for (field, mut text) in &mut texts {
        if field.0 == SnakeField::Score {
            *text = Text::new(if session.game_over { format!("¡Game Over! Puntos: {}", session.score) } else { format!("Puntos: {}", session.score) });
        }
    }
    for (cell, mut bg) in &mut cell_query {
        let pos = (cell.0, cell.1);
        *bg = BackgroundColor(if session.snake.contains(&pos) {
            if pos == session.snake[0] { Color::srgb(0.30, 0.85, 0.30) } else { Color::srgb(0.20, 0.60, 0.20) }
        } else if pos == session.food {
            Color::srgb(0.90, 0.30, 0.30)
        } else {
            Color::srgb(0.12, 0.14, 0.24)
        });
    }
}
