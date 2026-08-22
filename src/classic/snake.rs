//! Snake — con selector de dificultad (velocidad) y 1-2 jugadores.

use bevy::prelude::*;
use rand::Rng;

use crate::game::GameState;
use crate::learning::screen_background;
use crate::i18n::tr;

const SIZE: usize = 15;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Difficulty { Easy, Medium, Hard }
impl Difficulty {
    fn tick(&self) -> f32 { match self { Self::Easy => 0.22, Self::Medium => 0.14, Self::Hard => 0.08 } }
    fn label(&self) -> &'static str { match self { Self::Easy => "Fácil", Self::Medium => "Medio", Self::Hard => "Difícil" } }
    fn all() -> [Self; 3] { [Self::Easy, Self::Medium, Self::Hard] }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Dir { Up, Down, Left, Right }

#[derive(Resource, Clone)]
struct SnakeSession {
    snake: Vec<(usize, usize)>,
    snake2: Vec<(usize, usize)>,
    dir: Dir,
    next_dir: Dir,
    dir2: Dir,
    next_dir2: Dir,
    food: (usize, usize),
    score: u32,
    score2: u32,
    timer: f32,
    tick: f32,
    game_over: bool,
    winner: Option<usize>,
    difficulty: Difficulty,
    num_players: usize,
    setup_done: bool,
}

impl SnakeSession {
    fn new_with_setup(difficulty: Difficulty, num_players: usize) -> Self {
        let mut s = Self {
            snake: vec![(7,7), (7,6), (7,5)],
            snake2: vec![(7, 12), (7,11), (7,10)],
            dir: Dir::Right, next_dir: Dir::Right,
            dir2: Dir::Left, next_dir2: Dir::Left,
            food: (7,10),
            score: 0, score2: 0,
            timer: 0.0,
            tick: difficulty.tick(),
            game_over: false,
            winner: None,
            difficulty,
            num_players: num_players.clamp(1,2),
            setup_done: true,
        };
        s.place_food();
        s
    }
    fn new() -> Self {
        Self {
            snake: vec![(7,7), (7,6), (7,5)],
            snake2: vec![(7,12), (7,11), (7,10)],
            dir: Dir::Right, next_dir: Dir::Right,
            dir2: Dir::Left, next_dir2: Dir::Left,
            food: (7,10),
            score: 0, score2: 0,
            timer: 0.0,
            tick: Difficulty::Medium.tick(),
            game_over: false,
            winner: None,
            difficulty: Difficulty::Medium,
            num_players: 1,
            setup_done: false,
        }
    }
    fn place_food(&mut self) {
        let mut rng = rand::thread_rng();
        loop {
            let r = rng.gen_range(0..SIZE);
            let c = rng.gen_range(0..SIZE);
            let occupied = self.snake.contains(&(r,c)) || (self.num_players==2 && self.snake2.contains(&(r,c)));
            if !occupied { self.food = (r,c); break; }
        }
    }
    fn step(&mut self) {
        if self.game_over { return; }
        // P1
        self.dir = self.next_dir;
        let (hr, hc) = self.snake[0];
        let (nr, nc) = match self.dir {
            Dir::Up => (hr.checked_sub(1), Some(hc)),
            Dir::Down => (Some(hr+1), Some(hc)),
            Dir::Left => (Some(hr), hc.checked_sub(1)),
            Dir::Right => (Some(hr), Some(hc+1)),
        };
        let p1_alive = match (nr, nc) {
            (Some(r), Some(c)) if r < SIZE && c < SIZE => {
                if self.snake.contains(&(r,c)) || (self.num_players==2 && self.snake2.contains(&(r,c))) { false } else { true }
            },
            _ => false,
        };
        if !p1_alive {
            self.game_over = true;
            self.winner = if self.num_players==2 { Some(2) } else { None };
            return;
        }
        let (nr, nc) = (nr.unwrap(), nc.unwrap());
        self.snake.insert(0, (nr,nc));
        if (nr,nc) == self.food {
            self.score += 1;
            self.place_food();
        } else {
            self.snake.pop();
        }
        if self.num_players==2 {
            // P2
            self.dir2 = self.next_dir2;
            let (hr, hc) = self.snake2[0];
            let (nr, nc) = match self.dir2 {
                Dir::Up => (hr.checked_sub(1), Some(hc)),
                Dir::Down => (Some(hr+1), Some(hc)),
                Dir::Left => (Some(hr), hc.checked_sub(1)),
                Dir::Right => (Some(hr), Some(hc+1)),
            };
            let p2_alive = match (nr, nc) {
                (Some(r), Some(c)) if r < SIZE && c < SIZE => {
                    if self.snake2.contains(&(r,c)) || self.snake.contains(&(r,c)) { false } else { true }
                },
                _ => false,
            };
            if !p2_alive {
                self.game_over = true;
                self.winner = Some(1);
                return;
            }
            let (nr, nc) = (nr.unwrap(), nc.unwrap());
            self.snake2.insert(0, (nr,nc));
            if (nr,nc) == self.food {
                self.score2 += 1;
                self.place_food();
            } else {
                self.snake2.pop();
            }
            // colisión cabezas
            if self.snake[0] == self.snake2[0] {
                self.game_over = true;
                self.winner = None;
            }
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
#[derive(Component)]
struct SnakeSetupRoot;
#[derive(Component)]
struct SnakeDiffButton(Difficulty);
#[derive(Component)]
struct SnakePlayersButton(usize);
#[derive(Component)]
struct SnakeStartButton;

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
            overlay.spawn((SnakeSetupRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.02, 0.04, 0.10, 0.85)), ZIndex(50))).with_children(|setup| {
                setup.spawn((Node { flex_direction: FlexDirection::Column, width: Val::Px(520.0), padding: UiRect::all(Val::Px(20.0)), row_gap: Val::Px(14.0), align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.10, 0.14, 0.28, 0.96)), BorderRadius::all(Val::Px(16.0)))).with_children(|panel| {
                    panel.spawn((Text::new("SNAKE — Elige modo"), TextFont { font: font.clone(), font_size: 22.0, ..default() }, TextColor(Color::srgb(0.95, 0.85, 0.40))));
                    panel.spawn((Text::new("Dificultad (velocidad)"), TextFont { font: font.clone(), font_size: 16.0, ..default() }, TextColor(Color::WHITE)));
                    panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(10.0), ..default() }).with_children(|row| {
                        for d in Difficulty::all() {
                            row.spawn((Button, SnakeDiffButton(d), Node { width: Val::Px(120.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(d.label()), TextFont { font: font.clone(), font_size: 14.0, ..default() }, TextColor(Color::WHITE))); });
                        }
                    });
                    panel.spawn((Text::new("Jugadores"), TextFont { font: font.clone(), font_size: 16.0, ..default() }, TextColor(Color::WHITE)));
                    panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(10.0), ..default() }).with_children(|row| {
                        row.spawn((Button, SnakePlayersButton(1), Node { width: Val::Px(140.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.15, 0.42, 0.25)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new("1 Jugador"), TextFont { font: font.clone(), font_size: 14.0, ..default() }, TextColor(Color::WHITE))); });
                        row.spawn((Button, SnakePlayersButton(2), Node { width: Val::Px(140.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new("2 Jugadores"), TextFont { font: font.clone(), font_size: 14.0, ..default() }, TextColor(Color::WHITE))); });
                    });
                    panel.spawn((Button, SnakeStartButton, Node { width: Val::Px(200.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.15, 0.42, 0.25)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new("¡Jugar!"), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE))); });
                    panel.spawn((Text::new("1P: WASD — 2P: Flechas (J2)"), TextFont { font: font.clone(), font_size: 12.0, ..default() }, TextColor(Color::srgba(1.0,1.0,1.0,0.6))));
                });
            });
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
    diff_buttons: Query<(&Interaction, &SnakeDiffButton), Changed<Interaction>>,
    players_buttons: Query<(&Interaction, &SnakePlayersButton), Changed<Interaction>>,
    start_button: Query<&Interaction, (Changed<Interaction>, With<SnakeStartButton>)>,
    mut setup_root: Query<&mut Visibility, With<SnakeSetupRoot>>,
) {
    if keys.just_pressed(KeyCode::Escape) || back_clicks.single().map_or(false, |i| *i == Interaction::Pressed) { commands.set_state(GameState::ClassicMenu); return; }
    if !session.setup_done {
        for (interaction, btn) in &diff_buttons { if *interaction == Interaction::Pressed { session.difficulty = btn.0; session.tick = btn.0.tick(); } }
        for (interaction, btn) in &players_buttons { if *interaction == Interaction::Pressed { session.num_players = btn.0; } }
        for interaction in &start_button { if *interaction == Interaction::Pressed { session.setup_done = true; for mut v in &mut setup_root { *v = Visibility::Hidden; } } }
        return;
    }
    if restart_clicks.single().map_or(false, |i| *i == Interaction::Pressed) {
        let diff = session.difficulty;
        let players = session.num_players;
        *session = SnakeSession::new_with_setup(diff, players);
    }
    // dirección P1 WASD
    let dir = if keys.just_pressed(KeyCode::KeyW) { Some(Dir::Up) }
    else if keys.just_pressed(KeyCode::KeyS) { Some(Dir::Down) }
    else if keys.just_pressed(KeyCode::KeyA) { Some(Dir::Left) }
    else if keys.just_pressed(KeyCode::KeyD) { Some(Dir::Right) } else { None };
    if let Some(d) = dir {
        if !matches!((session.dir, d), (Dir::Up, Dir::Down) | (Dir::Down, Dir::Up) | (Dir::Left, Dir::Right) | (Dir::Right, Dir::Left)) {
            session.next_dir = d;
        }
    }
    // dirección P2 flechas
    if session.num_players==2 {
        let dir2 = if keys.just_pressed(KeyCode::ArrowUp) { Some(Dir::Up) }
        else if keys.just_pressed(KeyCode::ArrowDown) { Some(Dir::Down) }
        else if keys.just_pressed(KeyCode::ArrowLeft) { Some(Dir::Left) }
        else if keys.just_pressed(KeyCode::ArrowRight) { Some(Dir::Right) } else { None };
        if let Some(d) = dir2 {
            if !matches!((session.dir2, d), (Dir::Up, Dir::Down) | (Dir::Down, Dir::Up) | (Dir::Left, Dir::Right) | (Dir::Right, Dir::Left)) {
                session.next_dir2 = d;
            }
        }
    }
    if !session.game_over {
        session.timer += time.delta_secs();
        if session.timer >= session.tick {
            session.timer = 0.0;
            session.step();
        }
    }
    for (field, mut text) in &mut texts {
        if field.0 == SnakeField::Score {
            if session.game_over {
                let msg = if let Some(w) = session.winner { format!("¡Gana J{}! P1:{} P2:{}", w, session.score, session.score2) } else { format!("¡Game Over! Puntos: {} {}", session.score, if session.num_players==2 { format!("— P2: {}", session.score2) } else { "".to_string() }) };
                *text = Text::new(msg);
            } else {
                *text = Text::new(if session.num_players==2 { format!("P1: {}  P2: {}  |  {} {:?}", session.score, session.score2, session.difficulty.label(), session.tick) } else { format!("Puntos: {}  |  {}", session.score, session.difficulty.label()) });
            }
        }
    }
    for (cell, mut bg) in &mut cell_query {
        let pos = (cell.0, cell.1);
        let is_p1_head = session.snake.first() == Some(&pos);
        let is_p1_body = session.snake.contains(&pos);
        let is_p2_head = session.num_players==2 && session.snake2.first() == Some(&pos);
        let is_p2_body = session.num_players==2 && session.snake2.contains(&pos);
        *bg = BackgroundColor(if is_p1_head { Color::srgb(0.30, 0.85, 0.30) } else if is_p1_body { Color::srgb(0.20, 0.60, 0.20) } else if is_p2_head { Color::srgb(0.30, 0.45, 0.90) } else if is_p2_body { Color::srgb(0.20, 0.35, 0.70) } else if pos == session.food { Color::srgb(0.90, 0.30, 0.30) } else { Color::srgb(0.12, 0.14, 0.24) });
    }
}
