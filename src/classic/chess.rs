//! Ajedrez 2 jugadores — 8×8, movimientos básicos, jaque.

use bevy::prelude::*;

use crate::game::GameState;
use crate::learning::screen_background;
use crate::i18n::tr;

const SIZE: usize = 8;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Piece { Empty, WPawn, WRook, WKnight, WBishop, WQueen, WKing, BPawn, BRook, BKnight, BBishop, BQueen, BKing }

impl Piece {
    fn is_white(self) -> bool { matches!(self, Piece::WPawn|Piece::WRook|Piece::WKnight|Piece::WBishop|Piece::WQueen|Piece::WKing) }
    fn is_black(self) -> bool { matches!(self, Piece::BPawn|Piece::BRook|Piece::BKnight|Piece::BBishop|Piece::BQueen|Piece::BKing) }
    fn symbol(self) -> &'static str {
        match self {
            Piece::Empty => "",
            Piece::WPawn => "P", Piece::WRook => "R", Piece::WKnight => "N", Piece::WBishop => "B", Piece::WQueen => "Q", Piece::WKing => "K",
            Piece::BPawn => "p", Piece::BRook => "r", Piece::BKnight => "n", Piece::BBishop => "b", Piece::BQueen => "q", Piece::BKing => "k",
        }
    }
}

#[derive(Resource, Clone)]
struct ChessSession {
    board: [[Piece; SIZE]; SIZE],
    selected: Option<(usize,usize)>,
    turn_white: bool,
    message: String,
    vs_cpu: bool,
    setup_done: bool,
}

impl ChessSession {
    fn new() -> Self { Self::new_setup() }
    fn new_setup() -> Self {
        let mut board = [[Piece::Empty; SIZE]; SIZE];
        let back_black = [Piece::BRook, Piece::BKnight, Piece::BBishop, Piece::BQueen, Piece::BKing, Piece::BBishop, Piece::BKnight, Piece::BRook];
        let back_white = [Piece::WRook, Piece::WKnight, Piece::WBishop, Piece::WQueen, Piece::WKing, Piece::WBishop, Piece::WKnight, Piece::WRook];
        for c in 0..SIZE { board[0][c] = back_black[c]; board[1][c] = Piece::BPawn; board[6][c] = Piece::WPawn; board[7][c] = back_white[c]; }
        Self { board, selected: None, turn_white: true, message: "Elige modo arriba".to_string(), vs_cpu: false, setup_done: false }
    }
    fn new_with_mode(vs_cpu: bool) -> Self {
        let mut board = [[Piece::Empty; SIZE]; SIZE];
        let back_black = [Piece::BRook, Piece::BKnight, Piece::BBishop, Piece::BQueen, Piece::BKing, Piece::BBishop, Piece::BKnight, Piece::BRook];
        let back_white = [Piece::WRook, Piece::WKnight, Piece::WBishop, Piece::WQueen, Piece::WKing, Piece::WBishop, Piece::WKnight, Piece::WRook];
        for c in 0..SIZE { board[0][c] = back_black[c]; board[1][c] = Piece::BPawn; board[6][c] = Piece::WPawn; board[7][c] = back_white[c]; }
        Self { board, selected: None, turn_white: true, message: "Turno: Blancas".to_string(), vs_cpu, setup_done: true }
    }
    fn can_move(&self, from: (usize,usize), to: (usize,usize)) -> bool {
        let piece = self.board[from.0][from.1];
        let target = self.board[to.0][to.1];
        if piece==Piece::Empty { return false; }
        if piece.is_white() == target.is_white() && target!=Piece::Empty { return false; }
        if piece.is_white() != self.turn_white { return false; }
        let (fr, fc) = (from.0 as i32, from.1 as i32);
        let (tr, tc) = (to.0 as i32, to.1 as i32);
        let dr = tr - fr;
        let dc = tc - fc;
        match piece {
            Piece::WPawn => {
                if fc==tc && dr==-1 && target==Piece::Empty { true }
                else if fc==tc && fr==6 && dr==-2 && target==Piece::Empty && self.board[5][fc]==Piece::Empty { true }
                else if dc.abs()==1 && dr==-1 && target.is_black() { true }
                else { false }
            },
            Piece::BPawn => {
                if fc==tc && dr==1 && target==Piece::Empty { true }
                else if fc==tc && fr==1 && dr==2 && target==Piece::Empty && self.board[2][fc]==Piece::Empty { true }
                else if dc.abs()==1 && dr==1 && target.is_white() { true }
                else { false }
            },
            Piece::WRook | Piece::BRook => {
                if fr!=tr && fc!=tc { return false; }
                let step_r = if dr==0 {0} else {dr.signum()};
                let step_c = if dc==0 {0} else {dc.signum()};
                let mut r = fr + step_r; let mut c = fc + step_c;
                while r != tr || c != tc {
                    if self.board[r as usize][c as usize]!=Piece::Empty { return false; }
                    r += step_r; c += step_c;
                }
                true
            },
            Piece::WBishop | Piece::BBishop => {
                if dr.abs()!=dc.abs() { return false; }
                let step_r = dr.signum(); let step_c = dc.signum();
                let mut r = fr + step_r; let mut c = fc + step_c;
                while r != tr || c != tc {
                    if self.board[r as usize][c as usize]!=Piece::Empty { return false; }
                    r += step_r; c += step_c;
                }
                true
            },
            Piece::WQueen | Piece::BQueen => {
                // rook or bishop
                if fr==tr || fc==tc {
                    let step_r = if dr==0 {0} else {dr.signum()};
                    let step_c = if dc==0 {0} else {dc.signum()};
                    let mut r = fr + step_r; let mut c = fc + step_c;
                    while r != tr || c != tc {
                        if self.board[r as usize][c as usize]!=Piece::Empty { return false; }
                        r += step_r; c += step_c;
                    }
                    true
                } else if dr.abs()==dc.abs() {
                    let step_r = dr.signum(); let step_c = dc.signum();
                    let mut r = fr + step_r; let mut c = fc + step_c;
                    while r != tr || c != tc {
                        if self.board[r as usize][c as usize]!=Piece::Empty { return false; }
                        r += step_r; c += step_c;
                    }
                    true
                } else { false }
            },
            Piece::WKnight | Piece::BKnight => { (dr.abs()==2 && dc.abs()==1) || (dr.abs()==1 && dc.abs()==2) },
            Piece::WKing | Piece::BKing => { dr.abs()<=1 && dc.abs()<=1 },
            _ => false,
        }
    }
}

#[derive(Component)]
struct ChessUiRoot;
#[derive(Component)]
struct ChessText(ChessField);
#[derive(Clone, Copy, PartialEq, Eq)]
enum ChessField { Title, Status }
#[derive(Component)]
struct ChessCellButton(usize, usize);
#[derive(Component)]
struct ChessBackButton;
#[derive(Component)]
struct ChessRestartButton;
#[derive(Component)]
struct ChessSetupRoot;
#[derive(Component)]
struct ChessVsCpuButton;
#[derive(Component)]
struct ChessTwoPlayersButton;
#[derive(Component)]
struct ChessPieceNode(usize, usize);

pub struct ChessPlugin;
impl Plugin for ChessPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::ChessGame), spawn_chess)
            .add_systems(OnExit(GameState::ChessGame), cleanup_chess)
            .add_systems(Update, update_chess.run_if(in_state(GameState::ChessGame)));
    }
}

fn spawn_chess(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands.insert_resource(ChessSession::new());
    commands
        .spawn((ChessUiRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, screen_background(), Visibility::Visible, ZIndex(30)))
        .with_children(|overlay| {
            overlay.spawn((ChessSetupRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.02, 0.04, 0.10, 0.85)), ZIndex(50))).with_children(|setup| {
                setup.spawn((Node { flex_direction: FlexDirection::Column, width: Val::Px(460.0), padding: UiRect::all(Val::Px(20.0)), row_gap: Val::Px(14.0), align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.10, 0.14, 0.28, 0.96)), BorderRadius::all(Val::Px(16.0)))).with_children(|panel| {
                    panel.spawn((Text::new("AJEDREZ — Elige modo"), TextFont { font: font.clone(), font_size: 22.0, ..default() }, TextColor(Color::srgb(0.95, 0.85, 0.40))));
                    panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(12.0), ..default() }).with_children(|row| {
                        row.spawn((Button, ChessVsCpuButton, Node { width: Val::Px(180.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.15, 0.42, 0.25)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new("1 Jugador vs CPU"), TextFont { font: font.clone(), font_size: 14.0, ..default() }, TextColor(Color::WHITE))); });
                        row.spawn((Button, ChessTwoPlayersButton, Node { width: Val::Px(180.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new("2 Jugadores"), TextFont { font: font.clone(), font_size: 14.0, ..default() }, TextColor(Color::WHITE))); });
                    });
                });
            });
            overlay.spawn((Node { flex_direction: FlexDirection::Column, width: Val::Px(680.0), padding: UiRect::axes(Val::Px(20.0), Val::Px(16.0)), row_gap: Val::Px(10.0), align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.07, 0.09, 0.18, 0.96)), BorderRadius::all(Val::Px(16.0)))).with_children(|panel| {
                panel.spawn((ChessText(ChessField::Title), Text::new("AJEDREZ 2 JUGADORES"), TextFont { font: font.clone(), font_size: 28.0, ..default() }, TextColor(Color::srgb(0.95, 0.85, 0.40))));
                panel.spawn((ChessText(ChessField::Status), Text::new("Turno: Blancas"), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE)));
                panel.spawn((Node { display: Display::Grid, grid_template_columns: vec![GridTrack::px(56.0); SIZE], grid_template_rows: vec![GridTrack::px(56.0); SIZE], column_gap: Val::Px(2.0), row_gap: Val::Px(2.0), padding: UiRect::all(Val::Px(8.0)), ..default() }, BackgroundColor(Color::srgb(0.22, 0.12, 0.06)), BorderRadius::all(Val::Px(14.0)))).with_children(|grid| {
                    for r in 0..SIZE { for c in 0..SIZE {
                        let is_black = (r+c)%2==1;
                        let bg = if is_black { Color::srgb(0.36, 0.20, 0.12) } else { Color::srgb(0.88, 0.70, 0.46) };
                        grid.spawn((Button, ChessCellButton(r,c), Node { width: Val::Px(56.0), height: Val::Px(56.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(bg), BorderRadius::all(Val::Px(4.0)))).with_children(|cell| {
                            cell.spawn((ChessPieceNode(r,c), Node { width: Val::Px(44.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::NONE), BorderRadius::all(Val::Px(22.0)))).with_children(|piece| {
                                piece.spawn((Text::new("".to_string()), TextFont { font: font.clone(), font_size: 20.0, ..default() }, TextColor(Color::WHITE)));
                            });
                        });
                    }}
                });
                panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(12.0), ..default() }).with_children(|row| {
                    row.spawn((Button, ChessRestartButton, Node { width: Val::Px(140.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Reiniciar")), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE))); });
                    row.spawn((Button, ChessBackButton, Node { width: Val::Px(140.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Volver")), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE))); });
                });
            });
        });
}

fn cleanup_chess(mut commands: Commands, roots: Query<Entity, With<ChessUiRoot>>) {
    for root in &roots { commands.entity(root).despawn(); }
    commands.remove_resource::<ChessSession>();
}

fn update_chess(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut session: ResMut<ChessSession>,
    cell_clicks: Query<(&Interaction, &ChessCellButton), (Changed<Interaction>, Without<ChessBackButton>)>,
    back_clicks: Query<&Interaction, (Changed<Interaction>, With<ChessBackButton>)>,
    restart_clicks: Query<&Interaction, (Changed<Interaction>, With<ChessRestartButton>)>,
    mut cell_query: Query<(&ChessCellButton, &mut BackgroundColor, &Children)>,
    mut piece_nodes: Query<(&ChessPieceNode, &mut BackgroundColor, &Children)>,
    mut piece_texts: Query<(&mut Text, &mut TextColor), Without<ChessText>>,
    mut texts: Query<(&ChessText, &mut Text), Without<ChessCellButton>>,
    setup_vs_cpu: Query<&Interaction, (Changed<Interaction>, With<ChessVsCpuButton>)>,
    setup_two: Query<&Interaction, (Changed<Interaction>, With<ChessTwoPlayersButton>)>,
    mut setup_root: Query<&mut Visibility, With<ChessSetupRoot>>,
) {
    if keys.just_pressed(KeyCode::Escape) || back_clicks.single().map_or(false, |i| *i == Interaction::Pressed) { commands.set_state(GameState::ClassicMenu); return; }
    if restart_clicks.single().map_or(false, |i| *i == Interaction::Pressed) {
        let vs = session.vs_cpu;
        let done = session.setup_done;
        if done { *session = ChessSession::new_with_mode(vs); } else { *session = ChessSession::new(); }
    }
    if !session.setup_done {
        for interaction in &setup_vs_cpu { if *interaction == Interaction::Pressed { *session = ChessSession::new_with_mode(true); for mut v in &mut setup_root { *v = Visibility::Hidden; } return; } }
        for interaction in &setup_two { if *interaction == Interaction::Pressed { *session = ChessSession::new_with_mode(false); for mut v in &mut setup_root { *v = Visibility::Hidden; } return; } }
        return;
    }
    let mut clicked: Option<(usize,usize)> = None;
    for (interaction, btn) in &cell_clicks { if *interaction == Interaction::Pressed { clicked = Some((btn.0, btn.1)); break; } }
    if let Some((r,c)) = clicked {
        if let Some((sr, sc)) = session.selected {
            if (r,c) == (sr,sc) { session.selected = None; }
            else if session.can_move((sr,sc), (r,c)) {
                let piece = session.board[sr][sc];
                session.board[r][c] = piece;
                session.board[sr][sc] = Piece::Empty;
                // promoción peón
                if r==0 && piece==Piece::WPawn { session.board[r][c] = Piece::WQueen; }
                if r==7 && piece==Piece::BPawn { session.board[r][c] = Piece::BQueen; }
                session.selected = None;
                session.turn_white = !session.turn_white;
                session.message = if session.turn_white { "Turno: Blancas".to_string() } else { "Turno: Negras".to_string() };
            } else {
                session.selected = None;
            }
        } else {
            let piece = session.board[r][c];
            if (session.turn_white && piece.is_white()) || (!session.turn_white && piece.is_black()) {
                session.selected = Some((r,c));
            }
        }
    }
    for (field, mut text) in &mut texts {
        if field.0 == ChessField::Status { *text = Text::new(session.message.clone()); }
    }
    for (btn, mut bg, _children) in &mut cell_query {
        let (r,c) = (btn.0, btn.1);
        let is_selected = session.selected == Some((r,c));
        let is_black = (r+c)%2==1;
        let base = if is_black { Color::srgb(0.36, 0.20, 0.12) } else { Color::srgb(0.88, 0.70, 0.46) };
        *bg = BackgroundColor(if is_selected { Color::srgb(0.30, 0.60, 0.30) } else { base });
    }
    for (piece, mut bg, children) in &mut piece_nodes {
        let (r,c) = (piece.0, piece.1);
        let p = session.board[r][c];
        let (bg_col, txt, txt_col) = match p {
            Piece::Empty => (Color::NONE, "".to_string(), Color::NONE),
            Piece::WPawn | Piece::WRook | Piece::WKnight | Piece::WBishop | Piece::WQueen | Piece::WKing => (Color::srgb(0.96, 0.96, 0.98), p.symbol().to_string(), Color::srgb(0.12, 0.12, 0.14)),
            _ => (Color::srgb(0.18, 0.20, 0.26), p.symbol().to_string(), Color::srgb(0.96, 0.96, 0.98)),
        };
        *bg = BackgroundColor(bg_col);
        for child in children.iter() {
            if let Ok((mut text, mut color)) = piece_texts.get_mut(child) {
                *text = Text::new(txt.clone());
                *color = TextColor(txt_col);
            }
        }
    }
}
