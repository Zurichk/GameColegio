//! Tres en Raya (Tic Tac Toe) — 3×3, 2 jugadores o vs CPU (X vs O).

use bevy::prelude::*;
use rand::seq::SliceRandom;

use crate::game::GameState;
use crate::i18n::tr;
use crate::learning::screen_background;

#[derive(Resource, Clone)]
struct TicTacToeSession {
    board: [Option<char>; 9],
    turn: char, // 'X' or 'O'
    winner: Option<char>,
    draw: bool,
}

impl TicTacToeSession {
    fn new() -> Self {
        Self { board: [None; 9], turn: 'X', winner: None, draw: false }
    }
    fn check_winner(&mut self) {
        let lines = [[0,1,2],[3,4,5],[6,7,8],[0,3,6],[1,4,7],[2,5,8],[0,4,8],[2,4,6]];
        for [a,b,c] in lines {
            if let (Some(x), Some(y), Some(z)) = (self.board[a], self.board[b], self.board[c]) {
                if x==y && y==z { self.winner = Some(x); return; }
            }
        }
        if self.board.iter().all(|c| c.is_some()) { self.draw = true; }
    }
}

#[derive(Component)]
struct TicTacToeUiRoot;
#[derive(Component)]
struct TicTacToeText(TicTacToeField);
#[derive(Clone, Copy, PartialEq, Eq)]
enum TicTacToeField { Title, Status }
#[derive(Component)]
struct TicTacToeCellButton(usize);
#[derive(Component)]
struct TicTacToeCellText(usize);
#[derive(Component)]
struct TicTacToeBackButton;
#[derive(Component)]
struct TicTacToeRestartButton;

pub struct TicTacToePlugin;
impl Plugin for TicTacToePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::TicTacToeGame), spawn_tictactoe)
            .add_systems(OnExit(GameState::TicTacToeGame), cleanup_tictactoe)
            .add_systems(Update, update_tictactoe.run_if(in_state(GameState::TicTacToeGame)));
    }
}

fn spawn_tictactoe(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands.insert_resource(TicTacToeSession::new());
    commands
        .spawn((TicTacToeUiRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, screen_background(), Visibility::Visible, ZIndex(30)))
        .with_children(|overlay| {
            overlay.spawn((Node { flex_direction: FlexDirection::Column, width: Val::Px(520.0), padding: UiRect::axes(Val::Px(24.0), Val::Px(20.0)), row_gap: Val::Px(12.0), align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.07, 0.09, 0.18, 0.96)), BorderRadius::all(Val::Px(16.0)))).with_children(|panel| {
                panel.spawn((TicTacToeText(TicTacToeField::Title), Text::new("TRES EN RAYA"), TextFont { font: font.clone(), font_size: 28.0, ..default() }, TextColor(Color::srgb(0.95, 0.85, 0.40))));
                panel.spawn((TicTacToeText(TicTacToeField::Status), Text::new("Turno: X"), TextFont { font: font.clone(), font_size: 22.0, ..default() }, TextColor(Color::WHITE)));
                panel.spawn((Node { display: Display::Grid, grid_template_columns: vec![GridTrack::px(100.0), GridTrack::px(100.0), GridTrack::px(100.0)], grid_template_rows: vec![GridTrack::px(100.0), GridTrack::px(100.0), GridTrack::px(100.0)], column_gap: Val::Px(8.0), row_gap: Val::Px(8.0), padding: UiRect::all(Val::Px(8.0)), ..default() }, BackgroundColor(Color::srgb(0.45, 0.28, 0.16)), BorderRadius::all(Val::Px(14.0)))).with_children(|grid| {
                    for i in 0..9 {
                        grid.spawn((Button, TicTacToeCellButton(i), Node { width: Val::Px(100.0), height: Val::Px(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(3.0)), ..default() }, BackgroundColor(if i % 2 == 0 { Color::srgb(0.86, 0.70, 0.42) } else { Color::srgb(0.78, 0.56, 0.30) }), BorderColor(Color::srgb(0.98, 0.86, 0.52)), BorderRadius::all(Val::Px(12.0)))).with_children(|c| { c.spawn((TicTacToeCellText(i), Text::new("".to_string()), TextFont { font: font.clone(), font_size: 48.0, ..default() }, TextColor(Color::WHITE))); });
                    }
                });
                panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(12.0), ..default() }).with_children(|row| {
                    row.spawn((Button, TicTacToeRestartButton, Node { width: Val::Px(160.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Reiniciar")), TextFont { font: font.clone(), font_size: 20.0, ..default() }, TextColor(Color::WHITE))); });
                    row.spawn((Button, TicTacToeBackButton, Node { width: Val::Px(160.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Volver")), TextFont { font: font.clone(), font_size: 20.0, ..default() }, TextColor(Color::WHITE))); });
                });
            });
        });
}

fn cleanup_tictactoe(mut commands: Commands, roots: Query<Entity, With<TicTacToeUiRoot>>) {
    for root in &roots { commands.entity(root).despawn(); }
    commands.remove_resource::<TicTacToeSession>();
}

fn update_tictactoe(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut session: ResMut<TicTacToeSession>,
    cell_clicks: Query<(&Interaction, &TicTacToeCellButton), (Changed<Interaction>, Without<TicTacToeBackButton>)>,
    back_clicks: Query<&Interaction, (Changed<Interaction>, With<TicTacToeBackButton>)>,
    restart_clicks: Query<&Interaction, (Changed<Interaction>, With<TicTacToeRestartButton>)>,
    mut cell_texts: Query<(&TicTacToeCellText, &mut Text), Without<TicTacToeText>>,
    mut status_text: Query<(&TicTacToeText, &mut Text), Without<TicTacToeCellText>>,
) {
    if keys.just_pressed(KeyCode::Escape) { commands.set_state(GameState::ClassicMenu); return; }
    if back_clicks.single().map_or(false, |i| *i == Interaction::Pressed) { commands.set_state(GameState::ClassicMenu); return; }
    if restart_clicks.single().map_or(false, |i| *i == Interaction::Pressed) { *session = TicTacToeSession::new(); }
    if session.winner.is_some() || session.draw {
        for (field, mut text) in &mut status_text { if field.0 == TicTacToeField::Status { if let Some(w) = session.winner { *text = Text::new(format!("¡Gana {}!", w)); } else if session.draw { *text = Text::new("¡Empate!"); } } }
        for (cell, mut text) in &mut cell_texts {
            if let Some(ch) = session.board[cell.0] { *text = Text::new(ch.to_string()); } else { *text = Text::new("".to_string()); }
        }
        return;
    }
    let mut clicked: Option<usize> = None;
    for (interaction, btn) in &cell_clicks { if *interaction == Interaction::Pressed { clicked = Some(btn.0); break; } }
    if let Some(idx) = clicked {
        if session.board[idx].is_none() {
            session.board[idx] = Some(session.turn);
            session.check_winner();
            if session.winner.is_none() && !session.draw {
                session.turn = if session.turn == 'X' { 'O' } else { 'X' };
                if session.turn == 'O' {
                    let empties: Vec<usize> = session.board.iter().enumerate().filter_map(|(i, c)| if c.is_none() { Some(i) } else { None }).collect();
                    if let Some(&choice) = empties.choose(&mut rand::thread_rng()) {
                        session.board[choice] = Some('O');
                        session.check_winner();
                        if session.winner.is_none() && !session.draw { session.turn = 'X'; }
                    }
                }
            }
        }
    }
    for (field, mut text) in &mut status_text {
        if field.0 == TicTacToeField::Status {
            if let Some(w) = session.winner { *text = Text::new(format!("¡Gana {}!", w)); } else if session.draw { *text = Text::new("¡Empate!"); } else { *text = Text::new(format!("Turno: {}", session.turn)); }
        }
    }
    for (cell, mut text) in &mut cell_texts {
        if let Some(ch) = session.board[cell.0] { *text = Text::new(ch.to_string()); } else { *text = Text::new("".to_string()); }
    }
}
