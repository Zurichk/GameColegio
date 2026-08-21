//! Dominó 2-4 jugadores — 28 fichas (0-0 a 6-6), emparejar números.

use bevy::prelude::*;
use rand::seq::SliceRandom;

use crate::game::GameState;
use crate::learning::screen_background;
use crate::i18n::tr;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Tile(u8, u8);

#[derive(Resource, Clone)]
struct DominoSession {
    hands: Vec<Vec<Tile>>,
    board: Vec<Tile>, // fichas en mesa en orden
    left: u8,  // número abierto a la izquierda
    right: u8, // número abierto a la derecha
    turn: usize,
    players: usize,
    winner: Option<usize>,
}

impl DominoSession {
    fn new(players: usize) -> Self {
        let mut tiles = Vec::new();
        for a in 0..=6 { for b in a..=6 { tiles.push(Tile(a,b)); } }
        tiles.shuffle(&mut rand::thread_rng());
        let mut hands = vec![Vec::new(); players];
        for i in 0..7 { for p in 0..players { if let Some(t)=tiles.pop() { hands[p].push(t); } let _ = i; } }
        // ficha inicial doble más alta de algún jugador
        let mut board = Vec::new();
        let mut left = 0; let mut right = 0;
        for p in 0..players {
            if let Some(pos) = hands[p].iter().position(|t| t.0==t.1) {
                let t = hands[p].remove(pos);
                left = t.0; right = t.1; board.push(t); break;
            }
        }
        if board.is_empty() {
            let t = hands[0].remove(0);
            left = t.0; right = t.1; board.push(t);
        }
        Self { hands, board, left, right, turn: 0, players, winner: None }
    }
    fn can_play(&self, tile: Tile) -> bool {
        tile.0==self.left || tile.1==self.left || tile.0==self.right || tile.1==self.right
    }
}

#[derive(Component)]
struct DominoUiRoot;
#[derive(Component)]
struct DominoText(DominoField);
#[derive(Clone, Copy, PartialEq, Eq)]
enum DominoField { Title, Board, Hand, Status }
#[derive(Component)]
struct DominoTileButton(usize);
#[derive(Component)]
struct DominoPassButton;
#[derive(Component)]
struct DominoBackButton;

pub struct DominoPlugin;
impl Plugin for DominoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::DominoGame), spawn_domino)
            .add_systems(OnExit(GameState::DominoGame), cleanup_domino)
            .add_systems(Update, update_domino.run_if(in_state(GameState::DominoGame)));
    }
}

fn spawn_domino(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands.insert_resource(DominoSession::new(2));
    commands
        .spawn((DominoUiRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, screen_background(), Visibility::Visible, ZIndex(30)))
        .with_children(|overlay| {
            overlay.spawn((Node { flex_direction: FlexDirection::Column, width: Val::Px(720.0), padding: UiRect::axes(Val::Px(20.0), Val::Px(16.0)), row_gap: Val::Px(10.0), align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.07, 0.09, 0.18, 0.96)), BorderRadius::all(Val::Px(16.0)))).with_children(|panel| {
                panel.spawn((DominoText(DominoField::Title), Text::new("DOMINÓ 2 JUGADORES"), TextFont { font: font.clone(), font_size: 28.0, ..default() }, TextColor(Color::srgb(0.95, 0.85, 0.40))));
                panel.spawn((DominoText(DominoField::Board), Text::new("Mesa: "), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE), TextLayout { linebreak: LineBreak::WordBoundary, ..default() }, Node { max_width: Val::Px(680.0), ..default() }));
                panel.spawn((DominoText(DominoField::Hand), Text::new("Tu mano:"), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE)));
                panel.spawn(Node { flex_direction: FlexDirection::Row, flex_wrap: FlexWrap::Wrap, width: Val::Px(680.0), justify_content: JustifyContent::Center, column_gap: Val::Px(8.0), row_gap: Val::Px(8.0), ..default() }).with_children(|row| {
                    for i in 0..7 {
                        row.spawn((Button, DominoTileButton(i), Node { width: Val::Px(80.0), height: Val::Px(50.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(1.0)), ..default() }, BackgroundColor(Color::srgb(0.85, 0.80, 0.70)), BorderRadius::all(Val::Px(6.0)))).with_children(|b| { b.spawn((Text::new("".to_string()), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::BLACK))); });
                    }
                });
                panel.spawn((DominoText(DominoField::Status), Text::new("Tu turno"), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::srgb(0.80, 0.95, 1.0))));
                panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(12.0), ..default() }).with_children(|row| {
                    row.spawn((Button, DominoPassButton, Node { width: Val::Px(140.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.60, 0.30, 0.20)), BorderColor(Color::srgb(0.80, 0.50, 0.50)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Pasar")), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE))); });
                    row.spawn((Button, DominoBackButton, Node { width: Val::Px(140.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Volver")), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE))); });
                });
            });
        });
}

fn cleanup_domino(mut commands: Commands, roots: Query<Entity, With<DominoUiRoot>>) {
    for root in &roots { commands.entity(root).despawn(); }
    commands.remove_resource::<DominoSession>();
}

fn update_domino(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut session: ResMut<DominoSession>,
    tile_clicks: Query<(&Interaction, &DominoTileButton), (Changed<Interaction>, Without<DominoBackButton>)>,
    pass_clicks: Query<&Interaction, (Changed<Interaction>, With<DominoPassButton>)>,
    back_clicks: Query<&Interaction, (Changed<Interaction>, With<DominoBackButton>)>,
    mut texts: Query<(&DominoText, &mut Text)>,
    mut tile_buttons: Query<(&DominoTileButton, &mut BackgroundColor, &Children)>,
    mut tile_texts: Query<&mut Text, Without<DominoText>>,
) {
    if keys.just_pressed(KeyCode::Escape) || back_clicks.single().map_or(false, |i| *i == Interaction::Pressed) { commands.set_state(GameState::ClassicMenu); return; }
    if session.winner.is_some() { return; }
    let mut played = false;
    for (interaction, btn) in &tile_clicks {
        if *interaction == Interaction::Pressed {
            let idx = btn.0;
            if idx < session.hands[0].len() {
                let tile = session.hands[0][idx];
                if session.can_play(tile) {
                    // colocar
                    if tile.0 == session.left { session.left = tile.1; session.board.insert(0, tile); }
                    else if tile.1 == session.left { session.left = tile.0; session.board.insert(0, tile); }
                    else if tile.0 == session.right { session.right = tile.1; session.board.push(tile); }
                    else if tile.1 == session.right { session.right = tile.0; session.board.push(tile); }
                    else { continue; }
                    session.hands[0].remove(idx);
                    if session.hands[0].is_empty() { session.winner = Some(0); }
                    played = true;
                    break;
                }
            }
        }
    }
    if pass_clicks.single().map_or(false, |i| *i == Interaction::Pressed) && !played {
        // pasar
        played = true;
    }
    if played && session.winner.is_none() {
        // turno CPU simple
        session.turn = 1;
        // CPU juega si puede
        if let Some(pos) = session.hands[1].iter().position(|t| session.can_play(*t)) {
            let tile = session.hands[1].remove(pos);
            if tile.0 == session.left { session.left = tile.1; session.board.insert(0, tile); }
            else if tile.1 == session.left { session.left = tile.0; session.board.insert(0, tile); }
            else if tile.0 == session.right { session.right = tile.1; session.board.push(tile); }
            else { session.right = tile.0; session.board.push(tile); }
            if session.hands[1].is_empty() { session.winner = Some(1); }
        }
        session.turn = 0;
    }
    for (field, mut text) in &mut texts {
        match field.0 {
            DominoField::Board => { *text = Text::new(format!("Mesa ({}-{}): {}", session.left, session.right, session.board.iter().map(|t| format!("{}|{}", t.0, t.1)).collect::<Vec<_>>().join(" "))); }
            DominoField::Hand => { *text = Text::new(format!("Tu mano ({} fichas): {}", session.hands[0].len(), session.hands[0].iter().map(|t| format!("{}|{}", t.0, t.1)).collect::<Vec<_>>().join(" "))); }
            DominoField::Status => { *text = Text::new(if let Some(w)=session.winner { if w==0 {"¡Ganas tú!".to_string()} else {"¡Gana CPU!".to_string()} } else { "Tu turno — toca ficha que encaje".to_string() }); }
            _ => {}
        }
    }
    for (btn, mut bg, children) in &mut tile_buttons {
        let can = btn.0 < session.hands[0].len() && session.can_play(session.hands[0][btn.0]);
        *bg = BackgroundColor(if can { Color::srgb(0.25, 0.55, 0.25) } else { Color::srgb(0.50, 0.50, 0.50) });
        for child in children.iter() {
            if let Ok(mut text) = tile_texts.get_mut(child) {
                if btn.0 < session.hands[0].len() {
                    let t = session.hands[0][btn.0];
                    *text = Text::new(format!("{}|{}", t.0, t.1));
                } else {
                    *text = Text::new("".to_string());
                }
            }
        }
    }
}
