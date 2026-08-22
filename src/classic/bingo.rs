//! Bingo 5×5 — cartón, bombo, línea y bingo.

use bevy::prelude::*;
use rand::seq::SliceRandom;

use crate::classic::dice_anim::{spawn_bingo_bombo_panel, BingoBombo};
use crate::game::GameState;
use crate::learning::screen_background;
use crate::i18n::tr;

const SIZE: usize = 5;

#[derive(Resource, Clone)]
struct BingoSession {
    cards: Vec<[[u8; SIZE]; SIZE]>,
    marked: Vec<[[bool; SIZE]; SIZE]>,
    drawn: Vec<u8>,
    line: Vec<bool>,
    bingo: Vec<bool>,
    pending_ball: Option<u8>,
    num_players: usize,
    setup_done: bool,
}

fn gen_card() -> [[u8; SIZE]; SIZE] {
    let mut rng = rand::thread_rng();
    let mut card = [[0u8; SIZE]; SIZE];
    for col in 0..SIZE {
        let range = match col { 0=>1..=15, 1=>16..=30, 2=>31..=45, 3=>46..=60, 4=>61..=75, _=>1..=15 };
        let mut nums: Vec<u8> = range.collect();
        nums.shuffle(&mut rng);
        for row in 0..SIZE { card[row][col] = nums[row]; }
    }
    card[2][2] = 0;
    card
}
impl BingoSession {
    fn new() -> Self { Self::new_with_players(1) }
    fn new_with_players(n: usize) -> Self {
        let num = n.clamp(1,4);
        let mut cards = Vec::new();
        let mut marked = Vec::new();
        let mut line = Vec::new();
        let mut bingo = Vec::new();
        for _ in 0..num {
            cards.push(gen_card());
            let mut m = [[false; SIZE]; SIZE];
            m[2][2] = true;
            marked.push(m);
            line.push(false);
            bingo.push(false);
        }
        Self { cards, marked, drawn: Vec::new(), line, bingo, pending_ball: None, num_players: num, setup_done: false }
    }
    fn is_any_bingo(&self) -> bool { self.bingo.iter().any(|&b| b) }
    #[allow(dead_code)]
    fn draw(&mut self) {
        if self.is_any_bingo() { return; }
        let mut rng = rand::thread_rng();
        let mut pool: Vec<u8> = (1..=75).filter(|n| !self.drawn.contains(n)).collect();
        pool.shuffle(&mut rng);
        if let Some(&num) = pool.first() {
            self.drawn.push(num);
            for p in 0..self.num_players {
                for r in 0..SIZE { for c in 0..SIZE { if self.cards[p][r][c]==num { self.marked[p][r][c]=true; } } }
            }
            self.check_win();
        }
    }
    fn check_win(&mut self) {
        for p in 0..self.num_players {
            let mut has_line = false;
            for r in 0..SIZE { if self.marked[p][r].iter().all(|&m| m) { has_line = true; } }
            for c in 0..SIZE { if (0..SIZE).all(|r| self.marked[p][r][c]) { has_line = true; } }
            if (0..SIZE).all(|i| self.marked[p][i][i]) || (0..SIZE).all(|i| self.marked[p][i][SIZE-1-i]) { has_line = true; }
            if has_line { self.line[p] = true; }
            if self.marked[p].iter().all(|row| row.iter().all(|&m| m)) { self.bingo[p] = true; }
        }
    }
}

#[derive(Component)]
struct BingoUiRoot;
#[derive(Component)]
struct BingoText(BingoField);
#[derive(Clone, Copy, PartialEq, Eq)]
enum BingoField { Title, Status, Last }
#[derive(Component)]
struct BingoCell(usize, usize, usize); // player, row, col
#[derive(Component)]
struct BingoCardGrid(usize);
#[derive(Component)]
struct BingoDrawButton;
#[derive(Component)]
struct BingoBackButton;
#[derive(Component)]
struct BingoRestartButton;
#[derive(Component)]
struct BingoSetupRoot;
#[derive(Component)]
struct BingoSetupButton(usize);

pub struct BingoPlugin;
impl Plugin for BingoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::BingoGame), spawn_bingo)
            .add_systems(OnExit(GameState::BingoGame), cleanup_bingo)
            .add_systems(Update, update_bingo.run_if(in_state(GameState::BingoGame)));
    }
}

fn spawn_bingo(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands.insert_resource(BingoSession::new());
    commands
        .spawn((BingoUiRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, padding: UiRect::all(Val::Px(8.0)), ..default() }, screen_background(), Visibility::Visible, ZIndex(30)))
        .with_children(|overlay| {
            overlay.spawn((BingoSetupRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.02, 0.04, 0.10, 0.85)), ZIndex(50))).with_children(|setup| {
                setup.spawn((Node { flex_direction: FlexDirection::Column, width: Val::Px(460.0), padding: UiRect::all(Val::Px(20.0)), row_gap: Val::Px(14.0), align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.10, 0.14, 0.28, 0.96)), BorderRadius::all(Val::Px(16.0)))).with_children(|panel| {
                    panel.spawn((Text::new("BINGO — Elige jugadores"), TextFont { font: font.clone(), font_size: 22.0, ..default() }, TextColor(Color::srgb(0.95, 0.85, 0.40))));
                    panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(10.0), ..default() }).with_children(|row| {
                        for n in 1..=4 {
                            row.spawn((Button, BingoSetupButton(n), Node { width: Val::Px(80.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(if n==1 { Color::srgb(0.15, 0.42, 0.25) } else { Color::srgb(0.20, 0.38, 0.66) }), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(format!("{n} J")), TextFont { font: font.clone(), font_size: 16.0, ..default() }, TextColor(Color::WHITE))); });
                        }
                    });
                });
            });
            overlay.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(12.0), align_items: AlignItems::Center, justify_content: JustifyContent::Center, ..default() }).with_children(|row| {
                spawn_bingo_bombo_panel(row, &font);
                row.spawn((Node { flex_direction: FlexDirection::Column, width: Val::Px(520.0), padding: UiRect::axes(Val::Px(20.0), Val::Px(16.0)), row_gap: Val::Px(10.0), align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.07, 0.09, 0.18, 0.96)), BorderRadius::all(Val::Px(16.0)))).with_children(|panel| {
                panel.spawn((BingoText(BingoField::Title), Text::new("BINGO 5×5"), TextFont { font: font.clone(), font_size: 28.0, ..default() }, TextColor(Color::srgb(0.95, 0.85, 0.40))));
                panel.spawn((BingoText(BingoField::Last), Text::new("Último: -"), TextFont { font: font.clone(), font_size: 22.0, ..default() }, TextColor(Color::srgb(0.80, 0.95, 1.0))));
                panel.spawn((BingoText(BingoField::Status), Text::new("¡Saca bolas!"), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE)));
                panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(10.0), row_gap: Val::Px(10.0), justify_content: JustifyContent::Center, flex_wrap: FlexWrap::Wrap, ..default() }).with_children(|cards_row| {
                    for p in 0..4 {
                        cards_row.spawn((BingoCardGrid(p), Node { display: Display::Grid, grid_template_columns: vec![GridTrack::px(48.0); SIZE], grid_template_rows: vec![GridTrack::px(36.0); SIZE], column_gap: Val::Px(3.0), row_gap: Val::Px(3.0), padding: UiRect::all(Val::Px(4.0)), ..default() }, BackgroundColor(Color::srgb(0.12, 0.34, 0.28)), BorderRadius::all(Val::Px(8.0)))).with_children(|grid| {
                            for r in 0..SIZE { for c in 0..SIZE {
                                grid.spawn((BingoCell(p,r,c), Node { width: Val::Px(48.0), height: Val::Px(36.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(1.0)), ..default() }, BackgroundColor(if (r+c)%2 == 0 { Color::srgb(0.96, 0.88, 0.62) } else { Color::srgb(0.90, 0.74, 0.40) }), BorderRadius::all(Val::Px(8.0)))).with_children(|cell| { cell.spawn((Text::new("".to_string()), TextFont { font: font.clone(), font_size: 14.0, ..default() }, TextColor(Color::BLACK))); });
                            }}
                        });
                    }
                });
                panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(12.0), ..default() }).with_children(|row| {
                    row.spawn((Button, BingoDrawButton, Node { width: Val::Px(160.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.15, 0.42, 0.25)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Sacar bola")), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE))); });
                    row.spawn((Button, BingoRestartButton, Node { width: Val::Px(140.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Reiniciar")), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE))); });
                    row.spawn((Button, BingoBackButton, Node { width: Val::Px(140.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Volver")), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE))); });
                });
            });
                // Panel derecho decorativo con último número grande
                {
                    let f = font.clone();
                    row.spawn((
                        Node { width: Val::Px(190.0), height: Val::Px(360.0), flex_direction: FlexDirection::Column, align_items: AlignItems::Center, justify_content: JustifyContent::Center, row_gap: Val::Px(10.0), padding: UiRect::all(Val::Px(10.0)), ..default() },
                        BackgroundColor(Color::srgba(0.10, 0.14, 0.28, 0.88)),
                        BorderRadius::all(Val::Px(16.0)),
                    )).with_children(|p| {
                        p.spawn((Text::new("ÚLTIMA"), TextFont { font: f.clone(), font_size: 13.0, ..default() }, TextColor(Color::srgb(0.80, 0.85, 1.0))));
                        p.spawn((Node { width: Val::Px(90.0), height: Val::Px(90.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::WHITE), BorderRadius::all(Val::Px(45.0)))).with_children(|b| {
                            b.spawn((Text::new("—"), TextFont { font: f.clone(), font_size: 36.0, ..default() }, TextColor(Color::BLACK)));
                        });
                    });
                }
            });
        });
}

fn cleanup_bingo(mut commands: Commands, roots: Query<Entity, With<BingoUiRoot>>) {
    for root in &roots { commands.entity(root).despawn(); }
    commands.remove_resource::<BingoSession>();
}

fn update_bingo(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut session: ResMut<BingoSession>,
    draw_clicks: Query<&Interaction, (Changed<Interaction>, With<BingoDrawButton>)>,
    back_clicks: Query<&Interaction, (Changed<Interaction>, With<BingoBackButton>)>,
    restart_clicks: Query<&Interaction, (Changed<Interaction>, With<BingoRestartButton>)>,
    mut texts: Query<(&BingoText, &mut Text, &mut TextColor)>,
    mut cell_query: Query<(&BingoCell, &mut BackgroundColor, &Children), Without<BingoDrawButton>>,
    mut cell_texts: Query<&mut Text, Without<BingoText>>,
    mut bombo: Query<&mut BingoBombo>,
    mut draw_btn_bg: Query<&mut BackgroundColor, (With<BingoDrawButton>, Without<BingoCell>)>,
    setup_buttons: Query<(&Interaction, &BingoSetupButton), Changed<Interaction>>,
    mut setup_root: Query<&mut Visibility, With<BingoSetupRoot>>,
    mut card_grids: Query<(&BingoCardGrid, &mut Visibility)>,
) {
    if keys.just_pressed(KeyCode::Escape) || back_clicks.single().map_or(false, |i| *i == Interaction::Pressed) { commands.set_state(GameState::ClassicMenu); return; }
    if !session.setup_done {
        for (interaction, btn) in &setup_buttons {
            if *interaction == Interaction::Pressed {
                let n = btn.0.clamp(1,4);
                *session = BingoSession::new_with_players(n);
                for mut b in &mut bombo { *b = BingoBombo::default(); }
                for mut v in &mut setup_root { *v = Visibility::Hidden; }
                return;
            }
        }
        return;
    }
    if restart_clicks.single().map_or(false, |i| *i == Interaction::Pressed) {
        let n = session.num_players;
        *session = BingoSession::new_with_players(n);
        // mantener setup oculto (ya elegido)
        for mut b in &mut bombo { *b = BingoBombo::default(); }
    }

    for (grid, mut vis) in &mut card_grids {
        *vis = if grid.0 < session.num_players { Visibility::Inherited } else { Visibility::Hidden };
    }

    // Resolver bola pendiente cuando termina la animación del bombo
    if let Some(pending) = session.pending_ball {
        let still_rolling = bombo.iter().any(|b| b.rolling);
        if !still_rolling {
            // Animación terminó: aplicar la bola
            session.pending_ball = None;
            session.drawn.push(pending);
            for p in 0..session.num_players { for r in 0..SIZE { for c in 0..SIZE { if session.cards[p][r][c]==pending { session.marked[p][r][c]=true; } } } }
            session.check_win();
        }
    }

    let is_rolling = session.pending_ball.is_some();

    let mut draw_pressed = keys.just_pressed(KeyCode::Space);
    for interaction in &draw_clicks { if *interaction == Interaction::Pressed { draw_pressed = true; break; } }
    if draw_pressed && !session.is_any_bingo() && !is_rolling {
        // Generar bola que no haya salido
        let mut pool: Vec<u8> = (1..=75).filter(|n| !session.drawn.contains(n) && Some(*n) != session.pending_ball).collect();
        if !pool.is_empty() {
            use rand::seq::SliceRandom;
            pool.shuffle(&mut rand::thread_rng());
            if let Some(&ball) = pool.first() {
                session.pending_ball = Some(ball);
                for mut b in &mut bombo { b.draw_to(ball); }
            }
        }
    }
    // Desactivar botón al conseguir bingo
    for mut bg in &mut draw_btn_bg {
        *bg = BackgroundColor(if session.is_any_bingo() { Color::srgb(0.30, 0.32, 0.36) } else { Color::srgb(0.15, 0.42, 0.25) });
    }
    let is_rolling_now = session.pending_ball.is_some();
    for (field, mut text, mut color) in &mut texts {
        match field.0 {
            BingoField::Last => {
                if is_rolling_now {
                    *text = Text::new("¡Girando bombo... 🎲".to_string());
                    *color = TextColor(Color::srgb(1.0, 0.90, 0.50));
                } else {
                    let last = session.drawn.last().copied().unwrap_or(0);
                    let label = if last == 0 { "Último: —".to_string() } else {
                        let col = match last { 1..=15 => "B", 16..=30 => "I", 31..=45 => "N", 46..=60 => "G", _ => "O" };
                        format!("Último: {} · {} ({})", last, col, last)
                    };
                    *text = Text::new(label);
                    *color = TextColor(if last == 0 { Color::srgb(0.80, 0.95, 1.0) } else if session.is_any_bingo() { Color::srgb(1.0, 0.90, 0.40) } else { Color::WHITE });
                }
            }
            BingoField::Status => {
                if is_rolling_now {
                    *text = Text::new("¡Girando bombo... la bola va a salir!".to_string());
                    *color = TextColor(Color::srgb(1.0, 0.90, 0.50));
                } else {
                    let len = session.drawn.len();
                    let (msg, col) = if session.is_any_bingo() {
                        (format!("¡BINGO! ¡Cartón completo! 🎉 — {} bolas", len), Color::srgb(1.0, 0.85, 0.30))
                    } else if session.line.iter().any(|&l| l) {
                        (format!("¡LÍNEA! Sigue para BINGO — {}/75", len), Color::srgb(0.95, 0.90, 0.50))
                    } else {
                        let pct = len as f32 / 75.0 * 100.0;
                        (format!("Bolas: {}/75 — {:.0}%  ·  Marca tu cartón", len, pct), Color::WHITE)
                    };
                    *text = Text::new(msg);
                    *color = TextColor(col);
                }
            }
            _ => {}
        }
    }
    for (cell, mut bg, children) in &mut cell_query {
        let (p,r,c) = (cell.0, cell.1, cell.2);
        if p >= session.num_players {
            continue;
        }
        let marked = session.marked[p][r][c];
        // Resaltar línea/bingo del jugador
        let is_winner = session.bingo[p];
        let is_line = session.line[p];
        let base = if is_winner { Color::srgb(0.95, 0.75, 0.20) } else if is_line { Color::srgb(0.85, 0.70, 0.25) } else if marked { Color::srgb(0.24, 0.66, 0.34) } else if (r+c)%2 == 0 { Color::srgb(0.96, 0.88, 0.62) } else { Color::srgb(0.90, 0.74, 0.40) };
        *bg = BackgroundColor(base);
        for child in children.iter() {
            if let Ok(mut text) = cell_texts.get_mut(child) {
                let val = session.cards[p][r][c];
                *text = Text::new(if r==2 && c==2 { "★".to_string() } else { val.to_string() });
            }
        }
    }
}
