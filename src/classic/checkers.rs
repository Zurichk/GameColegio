//! Damas 8×8 — peones, captura obligatoria, coronación.

use bevy::prelude::*;

use crate::game::GameState;
use crate::learning::screen_background;
use crate::i18n::tr;

const SIZE: usize = 8;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Piece { Empty, Red, Black, RedKing, BlackKing }

#[derive(Resource, Clone)]
struct CheckersSession {
    board: [[Piece; SIZE]; SIZE],
    selected: Option<(usize, usize)>,
    turn: Piece,
    vs_cpu: bool,
    setup_done: bool,
}

impl CheckersSession {
    fn new() -> Self { Self::new_setup() }
    fn new_with_mode(vs_cpu: bool) -> Self {
        let mut board = [[Piece::Empty; SIZE]; SIZE];
        for r in 0..3 {
            for c in 0..SIZE {
                if (r + c) % 2 == 1 { board[r][c] = Piece::Black; }
            }
        }
        for r in 5..8 {
            for c in 0..SIZE {
                if (r + c) % 2 == 1 { board[r][c] = Piece::Red; }
            }
        }
        let mut s = Self { board, selected: None, turn: Piece::Red, vs_cpu, setup_done: true };
        if !vs_cpu { s.setup_done = true; }
        s
    }
    fn new_setup() -> Self {
        let mut board = [[Piece::Empty; SIZE]; SIZE];
        for r in 0..3 {
            for c in 0..SIZE {
                if (r + c) % 2 == 1 { board[r][c] = Piece::Black; }
            }
        }
        for r in 5..8 {
            for c in 0..SIZE {
                if (r + c) % 2 == 1 { board[r][c] = Piece::Red; }
            }
        }
        Self { board, selected: None, turn: Piece::Red, vs_cpu: true, setup_done: false }
    }
    fn can_move(&self, from: (usize,usize), to: (usize,usize)) -> bool {
        let (fr, fc) = from;
        let (tr, tc) = to;
        if self.board[tr][tc] != Piece::Empty { return false; }
        if (tr + tc) % 2 == 0 { return false; }
        let piece = self.board[fr][fc];
        let dr = tr as i32 - fr as i32;
        let dc = tc as i32 - fc as i32;
        let is_king = matches!(piece, Piece::RedKing | Piece::BlackKing);
        let dir_ok = match piece {
            Piece::Red => dr == -1 || (is_king && dr.abs()==1),
            Piece::Black => dr == 1 || (is_king && dr.abs()==1),
            _ => false,
        };
        if !dir_ok || dc.abs()!=1 { 
            // captura a distancia 2
            if dc.abs()==2 && dr.abs()==2 {
                let mr = (fr as i32 + dr/2) as usize;
                let mc = (fc as i32 + dc/2) as usize;
                let mid = self.board[mr][mc];
                return (piece==Piece::Red || piece==Piece::RedKing) && (mid==Piece::Black || mid==Piece::BlackKing) ||
                       (piece==Piece::Black || piece==Piece::BlackKing) && (mid==Piece::Red || mid==Piece::RedKing);
            }
            return false;
        }
        true
    }
}

#[derive(Component)]
struct CheckersUiRoot;
#[derive(Component)]
struct CheckersText(CheckersField);
#[derive(Clone, Copy, PartialEq, Eq)]
enum CheckersField { Title, Status }
#[derive(Component)]
struct CheckersCellButton(usize, usize);
#[derive(Component)]
struct CheckersBackButton;
#[derive(Component)]
struct CheckersRestartButton;
#[derive(Component)]
struct CheckersSetupRoot;
#[derive(Component)]
struct CheckersVsCpuButton;
#[derive(Component)]
struct CheckersTwoPlayersButton;

pub struct CheckersPlugin;
impl Plugin for CheckersPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::CheckersGame), spawn_checkers)
            .add_systems(OnExit(GameState::CheckersGame), cleanup_checkers)
            .add_systems(Update, update_checkers.run_if(in_state(GameState::CheckersGame)));
    }
}

fn spawn_checkers(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands.insert_resource(CheckersSession::new());
    commands
        .spawn((CheckersUiRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, screen_background(), Visibility::Visible, ZIndex(30)))
        .with_children(|overlay| {
            overlay.spawn((CheckersSetupRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.02, 0.04, 0.10, 0.85)), ZIndex(50))).with_children(|setup| {
                setup.spawn((Node { flex_direction: FlexDirection::Column, width: Val::Px(460.0), padding: UiRect::all(Val::Px(20.0)), row_gap: Val::Px(14.0), align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.10, 0.14, 0.28, 0.96)), BorderRadius::all(Val::Px(16.0)))).with_children(|panel| {
                    panel.spawn((Text::new("DAMAS — Elige modo"), TextFont { font: font.clone(), font_size: 22.0, ..default() }, TextColor(Color::srgb(0.95, 0.85, 0.40))));
                    panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(12.0), ..default() }).with_children(|row| {
                        row.spawn((Button, CheckersVsCpuButton, Node { width: Val::Px(180.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.15, 0.42, 0.25)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new("1 Jugador vs CPU"), TextFont { font: font.clone(), font_size: 14.0, ..default() }, TextColor(Color::WHITE))); });
                        row.spawn((Button, CheckersTwoPlayersButton, Node { width: Val::Px(180.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new("2 Jugadores"), TextFont { font: font.clone(), font_size: 14.0, ..default() }, TextColor(Color::WHITE))); });
                    });
                });
            });
            overlay.spawn((Node { flex_direction: FlexDirection::Column, width: Val::Px(680.0), padding: UiRect::axes(Val::Px(20.0), Val::Px(16.0)), row_gap: Val::Px(10.0), align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.07, 0.09, 0.18, 0.96)), BorderRadius::all(Val::Px(16.0)))).with_children(|panel| {
                panel.spawn((CheckersText(CheckersField::Title), Text::new("DAMAS 8×8"), TextFont { font: font.clone(), font_size: 28.0, ..default() }, TextColor(Color::srgb(0.95, 0.85, 0.40))));
                panel.spawn((CheckersText(CheckersField::Status), Text::new("Turno: Rojas (tú)"), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE)));
                panel.spawn((Node { display: Display::Grid, grid_template_columns: vec![GridTrack::px(56.0); SIZE], grid_template_rows: vec![GridTrack::px(56.0); SIZE], column_gap: Val::Px(2.0), row_gap: Val::Px(2.0), padding: UiRect::all(Val::Px(8.0)), ..default() }, BackgroundColor(Color::srgb(0.30, 0.16, 0.09)), BorderRadius::all(Val::Px(14.0)))).with_children(|grid| {
                    for r in 0..SIZE { for c in 0..SIZE {
                        let is_black = (r + c) % 2 == 1;
                        let bg = if is_black { Color::srgb(0.36, 0.20, 0.12) } else { Color::srgb(0.88, 0.70, 0.46) };
                        grid.spawn((Button, CheckersCellButton(r,c), Node { width: Val::Px(56.0), height: Val::Px(56.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(1.0)), ..default() }, BackgroundColor(bg), BorderRadius::all(Val::Px(4.0)))).with_children(|cell| { cell.spawn((Text::new("".to_string()), TextFont { font: font.clone(), font_size: 28.0, ..default() }, TextColor(Color::WHITE))); });
                    }}
                });
                panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(12.0), ..default() }).with_children(|row| {
                    row.spawn((Button, CheckersRestartButton, Node { width: Val::Px(140.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Reiniciar")), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE))); });
                    row.spawn((Button, CheckersBackButton, Node { width: Val::Px(140.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Volver")), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE))); });
                });
            });
        });
}

fn cleanup_checkers(mut commands: Commands, roots: Query<Entity, With<CheckersUiRoot>>) {
    for root in &roots { commands.entity(root).despawn(); }
    commands.remove_resource::<CheckersSession>();
}

fn update_checkers(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut session: ResMut<CheckersSession>,
    cell_clicks: Query<(&Interaction, &CheckersCellButton), (Changed<Interaction>, Without<CheckersBackButton>)>,
    back_clicks: Query<&Interaction, (Changed<Interaction>, With<CheckersBackButton>)>,
    restart_clicks: Query<&Interaction, (Changed<Interaction>, With<CheckersRestartButton>)>,
    mut cell_query: Query<(&CheckersCellButton, &mut BackgroundColor, &Children)>,
    mut cell_texts: Query<(&mut Text, &mut TextColor), Without<CheckersText>>,
    mut texts: Query<(&CheckersText, &mut Text), Without<CheckersCellButton>>,
    setup_vs_cpu: Query<&Interaction, (Changed<Interaction>, With<CheckersVsCpuButton>)>,
    setup_two: Query<&Interaction, (Changed<Interaction>, With<CheckersTwoPlayersButton>)>,
    mut setup_root: Query<&mut Visibility, With<CheckersSetupRoot>>,
) {
    if keys.just_pressed(KeyCode::Escape) || back_clicks.single().map_or(false, |i| *i == Interaction::Pressed) { commands.set_state(GameState::ClassicMenu); return; }
    if restart_clicks.single().map_or(false, |i| *i == Interaction::Pressed) {
        let vs = session.vs_cpu;
        let done = session.setup_done;
        if done { *session = CheckersSession::new_with_mode(vs); } else { *session = CheckersSession::new(); }
    }
    if !session.setup_done {
        for interaction in &setup_vs_cpu { if *interaction == Interaction::Pressed { *session = CheckersSession::new_with_mode(true); for mut v in &mut setup_root { *v = Visibility::Hidden; } return; } }
        for interaction in &setup_two { if *interaction == Interaction::Pressed { *session = CheckersSession::new_with_mode(false); for mut v in &mut setup_root { *v = Visibility::Hidden; } return; } }
        return;
    }
    let mut clicked: Option<(usize,usize)> = None;
    for (interaction, btn) in &cell_clicks { if *interaction == Interaction::Pressed { clicked = Some((btn.0, btn.1)); break; } }
    if let Some((r,c)) = clicked {
        if let Some((sr, sc)) = session.selected {
            // intentar mover
            if session.can_move((sr,sc), (r,c)) {
                let piece = session.board[sr][sc];
                let dr = r as i32 - sr as i32;
                // captura
                if dr.abs()==2 {
                    let mr = (sr as i32 + dr/2) as usize;
                    let mc = (sc as i32 + (c as i32 - sc as i32)/2) as usize;
                    session.board[mr][mc] = Piece::Empty;
                }
                session.board[r][c] = piece;
                session.board[sr][sc] = Piece::Empty;
                // coronación
                if r==0 && piece==Piece::Red { session.board[r][c] = Piece::RedKing; }
                if r==7 && piece==Piece::Black { session.board[r][c] = Piece::BlackKing; }
                session.selected = None;
                session.turn = if session.turn==Piece::Red { Piece::Black } else { Piece::Red };
            } else {
                session.selected = None;
            }
        } else {
            let piece = session.board[r][c];
            if (session.turn==Piece::Red && (piece==Piece::Red || piece==Piece::RedKing)) || (session.turn==Piece::Black && (piece==Piece::Black || piece==Piece::BlackKing)) {
                session.selected = Some((r,c));
            }
        }
    }
    for (field, mut text) in &mut texts {
        if field.0 == CheckersField::Status {
            let turn_label = if session.turn==Piece::Red { if session.vs_cpu {"Rojas (tú)"} else {"Rojas (J1)"} } else { if session.vs_cpu {"Negras (CPU)"} else {"Negras (J2)"} };
            *text = Text::new(format!("Turno: {} {}", turn_label, session.selected.map(|(r,c)| format!("  Seleccionada: {},{}", r+1, c+1)).unwrap_or("".to_string())));
        }
    }
    for (btn, mut bg, children) in &mut cell_query {
        let (r,c) = (btn.0, btn.1);
        let is_selected = session.selected == Some((r,c));
        let base = if (r+c)%2==1 { Color::srgb(0.36, 0.20, 0.12) } else { Color::srgb(0.88, 0.70, 0.46) };
        *bg = BackgroundColor(if is_selected { Color::srgb(0.30, 0.60, 0.30) } else { base });
        for child in children.iter() {
            if let Ok((mut text, mut text_color)) = cell_texts.get_mut(child) {
                // Usar colores visibles sobre casilla oscura (0.36,0.20,0.12): blanco para negras, rojo vivo para rojas
                let (symbol, color) = match session.board[r][c] {
                    Piece::Empty => ("", Color::NONE),
                    Piece::Red => ("●", Color::srgb(0.95, 0.25, 0.25)),
                    Piece::Black => ("●", Color::srgb(0.95, 0.95, 0.96)),
                    Piece::RedKing => ("★", Color::srgb(1.0, 0.85, 0.25)),
                    Piece::BlackKing => ("★", Color::srgb(1.0, 0.90, 0.50)),
                };
                *text = Text::new(symbol);
                *text_color = TextColor(color);
            }
        }
    }
}
