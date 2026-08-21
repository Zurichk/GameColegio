//! Bingo 5×5 — cartón, bombo, línea y bingo.

use bevy::prelude::*;
use rand::seq::SliceRandom;

use crate::game::GameState;
use crate::learning::screen_background;
use crate::i18n::tr;

const SIZE: usize = 5;

#[derive(Resource, Clone)]
struct BingoSession {
    card: [[u8; SIZE]; SIZE],
    marked: [[bool; SIZE]; SIZE],
    drawn: Vec<u8>,
    line: bool,
    bingo: bool,
}

impl BingoSession {
    fn new() -> Self {
        let mut rng = rand::thread_rng();
        let mut card = [[0u8; SIZE]; SIZE];
        for col in 0..SIZE {
            let range = match col { 0=>1..=15, 1=>16..=30, 2=>31..=45, 3=>46..=60, 4=>61..=75, _=>1..=15 };
            let mut nums: Vec<u8> = range.collect();
            nums.shuffle(&mut rng);
            for row in 0..SIZE { card[row][col] = nums[row]; }
        }
        card[2][2] = 0; // libre
        let mut marked = [[false; SIZE]; SIZE];
        marked[2][2] = true;
        Self { card, marked, drawn: Vec::new(), line: false, bingo: false }
    }
    fn draw(&mut self) {
        if self.bingo { return; }
        let mut rng = rand::thread_rng();
        let mut pool: Vec<u8> = (1..=75).filter(|n| !self.drawn.contains(n)).collect();
        pool.shuffle(&mut rng);
        if let Some(&num) = pool.first() {
            self.drawn.push(num);
            for r in 0..SIZE { for c in 0..SIZE { if self.card[r][c]==num { self.marked[r][c]=true; } } }
            self.check_win();
        }
    }
    fn check_win(&mut self) {
        // línea (fila completa)
        for r in 0..SIZE { if self.marked[r].iter().all(|&m| m) { self.line = true; } }
        for c in 0..SIZE { if (0..SIZE).all(|r| self.marked[r][c]) { self.line = true; } }
        if (0..SIZE).all(|i| self.marked[i][i]) || (0..SIZE).all(|i| self.marked[i][SIZE-1-i]) { self.line = true; }
        // bingo
        if self.marked.iter().all(|row| row.iter().all(|&m| m)) { self.bingo = true; }
    }
}

#[derive(Component)]
struct BingoUiRoot;
#[derive(Component)]
struct BingoText(BingoField);
#[derive(Clone, Copy, PartialEq, Eq)]
enum BingoField { Title, Status, Last }
#[derive(Component)]
struct BingoCell(usize, usize);
#[derive(Component)]
struct BingoDrawButton;
#[derive(Component)]
struct BingoBackButton;
#[derive(Component)]
struct BingoRestartButton;

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
        .spawn((BingoUiRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, screen_background(), Visibility::Visible, ZIndex(30)))
        .with_children(|overlay| {
            overlay.spawn((Node { flex_direction: FlexDirection::Column, width: Val::Px(520.0), padding: UiRect::axes(Val::Px(20.0), Val::Px(16.0)), row_gap: Val::Px(10.0), align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.07, 0.09, 0.18, 0.96)), BorderRadius::all(Val::Px(16.0)))).with_children(|panel| {
                panel.spawn((BingoText(BingoField::Title), Text::new("BINGO 5×5"), TextFont { font: font.clone(), font_size: 28.0, ..default() }, TextColor(Color::srgb(0.95, 0.85, 0.40))));
                panel.spawn((BingoText(BingoField::Last), Text::new("Último: -"), TextFont { font: font.clone(), font_size: 22.0, ..default() }, TextColor(Color::srgb(0.80, 0.95, 1.0))));
                panel.spawn((BingoText(BingoField::Status), Text::new("¡Saca bolas!"), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE)));
                panel.spawn((Node { display: Display::Grid, grid_template_columns: vec![GridTrack::px(80.0); SIZE], grid_template_rows: vec![GridTrack::px(60.0); SIZE], column_gap: Val::Px(4.0), row_gap: Val::Px(4.0), padding: UiRect::all(Val::Px(8.0)), ..default() }, BackgroundColor(Color::srgb(0.12, 0.34, 0.28)), BorderRadius::all(Val::Px(14.0)))).with_children(|grid| {
                    for r in 0..SIZE { for c in 0..SIZE {
                        grid.spawn((BingoCell(r,c), Node { width: Val::Px(80.0), height: Val::Px(60.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(1.0)), ..default() }, BackgroundColor(if (r+c)%2 == 0 { Color::srgb(0.96, 0.88, 0.62) } else { Color::srgb(0.90, 0.74, 0.40) }), BorderRadius::all(Val::Px(18.0)))).with_children(|cell| { cell.spawn((Text::new("".to_string()), TextFont { font: font.clone(), font_size: 20.0, ..default() }, TextColor(Color::BLACK))); });
                    }}
                });
                panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(12.0), ..default() }).with_children(|row| {
                    row.spawn((Button, BingoDrawButton, Node { width: Val::Px(160.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.15, 0.42, 0.25)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Sacar bola")), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE))); });
                    row.spawn((Button, BingoRestartButton, Node { width: Val::Px(140.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Reiniciar")), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE))); });
                    row.spawn((Button, BingoBackButton, Node { width: Val::Px(140.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Volver")), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE))); });
                });
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
    mut texts: Query<(&BingoText, &mut Text)>,
    mut cell_query: Query<(&BingoCell, &mut BackgroundColor, &Children)>,
    mut cell_texts: Query<&mut Text, Without<BingoText>>,
) {
    if keys.just_pressed(KeyCode::Escape) || back_clicks.single().map_or(false, |i| *i == Interaction::Pressed) { commands.set_state(GameState::ClassicMenu); return; }
    if restart_clicks.single().map_or(false, |i| *i == Interaction::Pressed) { *session = BingoSession::new(); }
    for interaction in &draw_clicks { if *interaction == Interaction::Pressed { session.draw(); } }
    for (field, mut text) in &mut texts {
        match field.0 {
            BingoField::Last => { *text = Text::new(format!("Último: {}", session.drawn.last().map(|n| n.to_string()).unwrap_or("-".to_string()))); }
            BingoField::Status => {
                *text = Text::new(if session.bingo { "¡BINGO! ¡Cartón completo!".to_string() } else if session.line { "¡Línea! Sigue para bingo".to_string() } else { format!("Bolas sacadas: {}", session.drawn.len()) });
            }
            _ => {}
        }
    }
    for (cell, mut bg, children) in &mut cell_query {
        let (r,c) = (cell.0, cell.1);
        let marked = session.marked[r][c];
        *bg = BackgroundColor(if marked { Color::srgb(0.24, 0.66, 0.34) } else if (r+c)%2 == 0 { Color::srgb(0.96, 0.88, 0.62) } else { Color::srgb(0.90, 0.74, 0.40) });
        for child in children.iter() {
            if let Ok(mut text) = cell_texts.get_mut(child) {
                let val = session.card[r][c];
                *text = Text::new(if r==2 && c==2 { "★".to_string() } else { val.to_string() });
            }
        }
    }
}
