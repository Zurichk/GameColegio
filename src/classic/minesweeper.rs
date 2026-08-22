//! Buscaminas — con selector de dificultad y 1-2 jugadores (turnos).

use bevy::prelude::*;
use rand::seq::SliceRandom;

use crate::game::GameState;
use crate::learning::screen_background;
use crate::i18n::tr;

const MAX_SIZE: usize = 16;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Difficulty { Easy, Medium, Hard }
impl Difficulty {
    fn size(&self) -> usize { match self { Self::Easy => 8, Self::Medium => 12, Self::Hard => 16 } }
    fn mines(&self) -> usize { match self { Self::Easy => 10, Self::Medium => 25, Self::Hard => 50 } }
    fn label(&self) -> &'static str { match self { Self::Easy => "Fácil 8×8", Self::Medium => "Medio 12×12", Self::Hard => "Difícil 16×16" } }
    fn all() -> [Self; 3] { [Self::Easy, Self::Medium, Self::Hard] }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Cell { Mine, Number(u8) }

#[derive(Clone, Copy, PartialEq, Eq)]
enum State { Hidden, Revealed, Flagged }

#[derive(Resource, Clone)]
struct MinesweeperSession {
    board: [[Cell; MAX_SIZE]; MAX_SIZE],
    state: [[State; MAX_SIZE]; MAX_SIZE],
    size: usize,
    mines: usize,
    revealed: usize,
    flagged: usize,
    first_click: bool,
    game_over: bool,
    won: bool,
    flag_mode: bool,
    difficulty: Difficulty,
    num_players: usize,
    current_player: usize,
    setup_done: bool,
}

impl MinesweeperSession {
    fn new_with_setup(difficulty: Difficulty, num_players: usize) -> Self {
        Self {
            board: [[Cell::Number(0); MAX_SIZE]; MAX_SIZE],
            state: [[State::Hidden; MAX_SIZE]; MAX_SIZE],
            size: difficulty.size(),
            mines: difficulty.mines(),
            revealed: 0,
            flagged: 0,
            first_click: true,
            game_over: false,
            won: false,
            flag_mode: false,
            difficulty,
            num_players: num_players.clamp(1, 2),
            current_player: 0,
            setup_done: true,
        }
    }
    fn new() -> Self {
        Self {
            board: [[Cell::Number(0); MAX_SIZE]; MAX_SIZE],
            state: [[State::Hidden; MAX_SIZE]; MAX_SIZE],
            size: 8,
            mines: 10,
            revealed: 0,
            flagged: 0,
            first_click: true,
            game_over: false,
            won: false,
            flag_mode: false,
            difficulty: Difficulty::Easy,
            num_players: 1,
            current_player: 0,
            setup_done: false,
        }
    }
    fn place_mines(&mut self, avoid_r: usize, avoid_c: usize) {
        let size = self.size;
        let mines = self.mines;
        let mut cells: Vec<(usize,usize)> = (0..size).flat_map(|r| (0..size).map(move |c| (r,c))).filter(|&(r,c)| !(r==avoid_r && c==avoid_c)).collect();
        cells.shuffle(&mut rand::thread_rng());
        for (r,c) in cells.into_iter().take(mines) {
            self.board[r][c] = Cell::Mine;
        }
        for r in 0..size { for c in 0..size {
            if self.board[r][c] == Cell::Mine { continue; }
            let mut count = 0;
            for dr in -1i32..=1 { for dc in -1i32..=1 {
                if dr==0 && dc==0 { continue; }
                let nr = r as i32 + dr;
                let nc = c as i32 + dc;
                if nr>=0 && nr < size as i32 && nc>=0 && nc < size as i32 {
                    if self.board[nr as usize][nc as usize] == Cell::Mine { count += 1; }
                }
            }}
            self.board[r][c] = Cell::Number(count);
        }}
    }
    fn reveal(&mut self, r: usize, c: usize) {
        if r >= self.size || c >= self.size { return; }
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
                    if nr>=0 && nr < self.size as i32 && nc>=0 && nc < self.size as i32 {
                        let (nr, nc) = (nr as usize, nc as usize);
                        if self.state[nr][nc]==State::Hidden { self.reveal(nr,nc); }
                    }
                }}
            },
            Cell::Number(_) => { self.state[r][c] = State::Revealed; self.revealed += 1; }
        }
        if self.revealed == self.size*self.size - self.mines { self.won = true; self.game_over = true; }
        else if !self.game_over && self.num_players == 2 {
            // En 2 jugadores, alterna turno tras cada revelado válido (no bandera)
            self.current_player = (self.current_player + 1) % 2;
        }
    }
    fn toggle_flag(&mut self, r: usize, c: usize) {
        if r >= self.size || c >= self.size { return; }
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
#[derive(Component)]
struct MinesweeperSetupRoot;
#[derive(Component)]
struct MinesweeperDiffButton(Difficulty);
#[derive(Component)]
struct MinesweeperPlayersButton(usize);
#[derive(Component)]
struct MinesweeperStartButton;

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
        .spawn((MinesweeperUiRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, padding: UiRect::all(Val::Px(8.0)), ..default() }, screen_background(), Visibility::Visible, ZIndex(30)))
        .with_children(|overlay| {
            // Setup overlay
            overlay.spawn((MinesweeperSetupRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.02, 0.04, 0.10, 0.85)), ZIndex(50))).with_children(|setup| {
                setup.spawn((Node { flex_direction: FlexDirection::Column, width: Val::Px(520.0), padding: UiRect::all(Val::Px(20.0)), row_gap: Val::Px(14.0), align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.10, 0.14, 0.28, 0.96)), BorderRadius::all(Val::Px(16.0)))).with_children(|panel| {
                    panel.spawn((Text::new("BUSCAMINAS — Elige dificultad"), TextFont { font: font.clone(), font_size: 22.0, ..default() }, TextColor(Color::srgb(0.95, 0.85, 0.40))));
                    panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(10.0), ..default() }).with_children(|row| {
                        for diff in Difficulty::all() {
                            row.spawn((Button, MinesweeperDiffButton(diff), Node { width: Val::Px(140.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(diff.label()), TextFont { font: font.clone(), font_size: 12.0, ..default() }, TextColor(Color::WHITE))); });
                        }
                    });
                    panel.spawn((Text::new("Jugadores"), TextFont { font: font.clone(), font_size: 16.0, ..default() }, TextColor(Color::WHITE)));
                    panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(10.0), ..default() }).with_children(|row| {
                        for n in 1..=2 {
                            row.spawn((Button, MinesweeperPlayersButton(n), Node { width: Val::Px(140.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(format!("{} Jugador{}", n, if n==1 {""} else {"es"})), TextFont { font: font.clone(), font_size: 14.0, ..default() }, TextColor(Color::WHITE))); });
                        }
                    });
                    panel.spawn((Button, MinesweeperStartButton, Node { width: Val::Px(200.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.15, 0.42, 0.25)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new("¡Jugar!"), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE))); });
                });
            });
            overlay.spawn((Node { flex_direction: FlexDirection::Column, width: Val::Px(720.0), padding: UiRect::axes(Val::Px(20.0), Val::Px(16.0)), row_gap: Val::Px(10.0), align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.07, 0.09, 0.18, 0.96)), BorderRadius::all(Val::Px(16.0)))).with_children(|panel| {
                panel.spawn((MinesweeperText(MinesweeperField::Title), Text::new("BUSCAMINAS"), TextFont { font: font.clone(), font_size: 28.0, ..default() }, TextColor(Color::srgb(0.95, 0.85, 0.40))));
                panel.spawn((MinesweeperText(MinesweeperField::Status), Text::new("Elige dificultad arriba"), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE)));
                // Grid se genera dinámicamente en update (placeholder)
                panel.spawn((Node { display: Display::Grid, grid_template_columns: vec![GridTrack::px(48.0); 8], grid_template_rows: vec![GridTrack::px(48.0); 8], column_gap: Val::Px(4.0), row_gap: Val::Px(4.0), padding: UiRect::all(Val::Px(8.0)), ..default() }, BackgroundColor(Color::srgb(0.16, 0.18, 0.20)), BorderRadius::all(Val::Px(12.0)), Visibility::Hidden));
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
    mut cell_query: Query<(&MinesweeperCellButton, &mut BackgroundColor, &Children, &mut Visibility), Without<MinesweeperSetupRoot>>,
    mut cell_texts: Query<&mut Text, Without<MinesweeperText>>,
    diff_buttons: Query<(&Interaction, &MinesweeperDiffButton), Changed<Interaction>>,
    player_buttons: Query<(&Interaction, &MinesweeperPlayersButton), Changed<Interaction>>,
    start_button: Query<&Interaction, (Changed<Interaction>, With<MinesweeperStartButton>)>,
    mut setup_root: Query<&mut Visibility, (With<MinesweeperSetupRoot>, Without<MinesweeperCellButton>)>,
) {
    if keys.just_pressed(KeyCode::Escape) { commands.set_state(GameState::ClassicMenu); return; }
    if back_clicks.single().map_or(false, |i| *i == Interaction::Pressed) { commands.set_state(GameState::ClassicMenu); return; }
    // Setup
    if !session.setup_done {
        let mut chosen_diff: Option<Difficulty> = None;
        for (interaction, btn) in &diff_buttons { if *interaction == Interaction::Pressed { chosen_diff = Some(btn.0); break; } }
        if let Some(d) = chosen_diff { session.difficulty = d; session.size = d.size(); session.mines = d.mines(); }
        for (interaction, btn) in &player_buttons { if *interaction == Interaction::Pressed { session.num_players = btn.0; break; } }
        for interaction in &start_button { if *interaction == Interaction::Pressed { session.setup_done = true; for mut v in &mut setup_root { *v = Visibility::Hidden; } break; } }
        // Mostrar selección actual en status
        for (field, mut text) in &mut texts {
            if field.0 == MinesweeperField::Status {
                *text = Text::new(format!("{} — {} — Pulsa ¡Jugar!", session.difficulty.label(), if session.num_players==1 {"1 Jugador"} else {"2 Jugadores"}));
            }
        }
        return;
    }
    if restart_clicks.single().map_or(false, |i| *i == Interaction::Pressed) {
        let diff = session.difficulty;
        let players = session.num_players;
        *session = MinesweeperSession::new_with_setup(diff, players);
    }
    for interaction in &flag_clicks { if *interaction == Interaction::Pressed { session.flag_mode = !session.flag_mode; } }
    if !session.game_over {
        for (interaction, btn) in &cell_clicks {
            if *interaction == Interaction::Pressed {
                let (r,c) = (btn.0, btn.1);
                if r >= session.size || c >= session.size { continue; }
                if session.flag_mode { session.toggle_flag(r,c); } else { session.reveal(r,c); }
            }
        }
    }
    for (field, mut text) in &mut texts {
        if field.0 == MinesweeperField::Status {
            if session.game_over {
                let who = if session.num_players==2 { format!(" (J{} pierde)", session.current_player+1) } else { "".to_string() };
                *text = Text::new(if session.won { format!("¡Ganaste!{} Todas las casillas seguras", who) } else { format!("¡Boom!{} Pisaste una mina", who) });
            } else {
                let turn = if session.num_players==2 { format!(" — Turno J{}", session.current_player+1) } else { "".to_string() };
                *text = Text::new(format!("{} — Minas: {}  Banderas: {}  Modo: {}{}", session.difficulty.label(), session.mines, session.flagged, if session.flag_mode {"BANDERA"} else {"REVELAR"}, turn));
            }
        }
        if field.0 == MinesweeperField::Title {
            *text = Text::new(format!("BUSCAMINAS {}×{} — {} minas", session.size, session.size, session.mines));
        }
    }
    for (btn, mut bg, children, mut vis) in &mut cell_query {
        let (r,c) = (btn.0, btn.1);
        if r >= session.size || c >= session.size {
            *vis = Visibility::Hidden;
            continue;
        }
        *vis = Visibility::Visible;
        let state = session.state[r][c];
        let cell = session.board[r][c];
        *bg = BackgroundColor(match state {
            State::Hidden => Color::srgb(0.30, 0.34, 0.36),
            State::Flagged => Color::srgb(0.72, 0.28, 0.20),
            State::Revealed => match cell { Cell::Mine => Color::srgb(0.85, 0.20, 0.20), Cell::Number(0) => Color::srgb(0.72, 0.70, 0.60), _ => Color::srgb(0.58, 0.60, 0.54) },
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
}
