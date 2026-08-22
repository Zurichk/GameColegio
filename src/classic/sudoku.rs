//! Sudoku 9×9 — genera puzzle, valida y completa.

use bevy::prelude::*;
use rand::seq::SliceRandom;

use crate::game::GameState;
use crate::learning::screen_background;
use crate::i18n::tr;

#[derive(Resource, Clone)]
struct SudokuSession {
    puzzle: [[u8; 9]; 9], // 0 = vacío
    solution: [[u8; 9]; 9],
    selected: Option<(usize, usize)>,
    won: bool,
    mistakes: u32,
}

impl SudokuSession {
    fn new() -> Self {
        let solution = generate_solved();
        let mut puzzle = solution;
        // Quitar 45 celdas para puzzle medio
        let mut cells: Vec<(usize,usize)> = (0..9).flat_map(|r| (0..9).map(move |c| (r,c))).collect();
        cells.shuffle(&mut rand::thread_rng());
        for (r,c) in cells.into_iter().take(45) { puzzle[r][c]=0; }
        Self { puzzle, solution, selected: None, won: false, mistakes: 0 }
    }
    fn is_fixed(&self, r: usize, c: usize) -> bool { self.puzzle[r][c] != 0 }
    fn current_value(&self, r: usize, c: usize) -> u8 { self.puzzle[r][c] }
    fn set(&mut self, r: usize, c: usize, v: u8) {
        if self.is_fixed(r,c) || self.won { return; }
        if v==0 { self.puzzle[r][c]=0; return; }
        self.puzzle[r][c]=v;
        if v != self.solution[r][c] { self.mistakes+=1; }
        // comprobar victoria
        if (0..9).all(|r| (0..9).all(|c| self.puzzle[r][c]==self.solution[r][c])) {
            self.won=true;
        }
    }
    #[allow(dead_code)]
    fn is_valid(&self, r: usize, c: usize, v: u8) -> bool {
        for i in 0..9 { if self.puzzle[r][i]==v || self.puzzle[i][c]==v { return false; } }
        let br = r/3*3; let bc = c/3*3;
        for rr in br..br+3 { for cc in bc..bc+3 { if self.puzzle[rr][cc]==v { return false; } } }
        true
    }
}

fn generate_solved() -> [[u8;9];9] {
    let mut board = [[0u8;9];9];
    fill(&mut board, 0);
    board
}
fn fill(board: &mut [[u8;9];9], pos: usize) -> bool {
    if pos>=81 { return true; }
    let r = pos/9; let c = pos%9;
    if board[r][c]!=0 { return fill(board, pos+1); }
    let mut nums: Vec<u8> = (1..=9).collect();
    nums.shuffle(&mut rand::thread_rng());
    for n in nums {
        if valid(board, r,c,n) {
            board[r][c]=n;
            if fill(board, pos+1) { return true; }
            board[r][c]=0;
        }
    }
    false
}
fn valid(board: &[[u8;9];9], r: usize, c: usize, v: u8) -> bool {
    for i in 0..9 { if board[r][i]==v || board[i][c]==v { return false; } }
    let br=r/3*3; let bc=c/3*3;
    for rr in br..br+3 { for cc in bc..bc+3 { if board[rr][cc]==v { return false; } } }
    true
}

#[derive(Component)]
struct SudokuUiRoot;
#[derive(Component)]
struct SudokuText(SudField);
#[derive(Clone, Copy, PartialEq, Eq)]
enum SudField { Title, Status }
#[derive(Component)]
struct SudokuCell(usize, usize);
#[derive(Component)]
struct SudokuNumButton(u8);
#[derive(Component)]
struct SudokuBackButton;
#[derive(Component)]
struct SudokuRestartButton;
#[derive(Component)]
struct SudokuEraserButton;

pub struct SudokuPlugin;
impl Plugin for SudokuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::SudokuGame), spawn_sudoku)
            .add_systems(OnExit(GameState::SudokuGame), cleanup_sudoku)
            .add_systems(Update, update_sudoku.run_if(in_state(GameState::SudokuGame)));
    }
}

fn spawn_sudoku(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands.insert_resource(SudokuSession::new());
    commands.spawn((SudokuUiRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, padding: UiRect::all(Val::Px(8.0)), ..default() }, screen_background(), Visibility::Visible, ZIndex(30)))
        .with_children(|overlay| {
            overlay.spawn((Node { flex_direction: FlexDirection::Column, width: Val::Px(520.0), padding: UiRect::all(Val::Px(14.0)), row_gap: Val::Px(8.0), align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.07,0.09,0.18,0.96)), BorderRadius::all(Val::Px(16.0)))).with_children(|panel| {
                panel.spawn((SudokuText(SudField::Title), Text::new("SUDOKU 9×9"), TextFont { font: font.clone(), font_size: 28.0, ..default() }, TextColor(Color::srgb(0.95,0.85,0.40))));
                panel.spawn((SudokuText(SudField::Status), Text::new("Toca una celda y elige número"), TextFont { font: font.clone(), font_size: 16.0, ..default() }, TextColor(Color::WHITE)));
                panel.spawn((Node { display: Display::Grid, grid_template_columns: vec![GridTrack::px(52.0); 9], grid_template_rows: vec![GridTrack::px(52.0); 9], column_gap: Val::Px(2.0), row_gap: Val::Px(2.0), padding: UiRect::all(Val::Px(6.0)), ..default() }, BackgroundColor(Color::srgb(0.10,0.12,0.14)), BorderRadius::all(Val::Px(10.0)))).with_children(|grid| {
                    for r in 0..9 { for c in 0..9 {
                        let is_box_border = c%3==0 || r%3==0;
                        grid.spawn((Button, SudokuCell(r,c), Node { width: Val::Px(52.0), height: Val::Px(52.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(if is_box_border {2.0} else {1.0})), ..default() }, BackgroundColor(Color::srgb(0.96,0.96,0.94)), BorderColor(Color::srgb(0.60,0.65,0.75)), BorderRadius::all(Val::Px(6.0)))).with_children(|cell| {
                            cell.spawn((Text::new("".to_string()), TextFont { font: font.clone(), font_size: 22.0, ..default() }, TextColor(Color::BLACK)));
                        });
                    }}
                });
                panel.spawn(Node { display: Display::Grid, grid_template_columns: vec![GridTrack::px(52.0); 5], column_gap: Val::Px(6.0), row_gap: Val::Px(6.0), ..default() }).with_children(|row| {
                    for n in 1..=9 {
                        row.spawn((Button, SudokuNumButton(n), Node { width: Val::Px(52.0), height: Val::Px(36.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(1.0)), ..default() }, BackgroundColor(Color::srgb(0.20,0.38,0.66)), BorderColor(Color::srgb(0.60,0.80,1.0)), BorderRadius::all(Val::Px(8.0)))).with_children(|b|{ b.spawn((Text::new(n.to_string()), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE)));});
                    }
                    row.spawn((Button, SudokuEraserButton, Node { width: Val::Px(52.0), height: Val::Px(36.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(1.0)), ..default() }, BackgroundColor(Color::srgb(0.50,0.20,0.20)), BorderRadius::all(Val::Px(8.0)))).with_children(|b|{ b.spawn((Text::new("⌫"), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE)));});
                });
                panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(10.0), ..default() }).with_children(|row| {
                    row.spawn((Button, SudokuRestartButton, Node { width: Val::Px(140.0), height: Val::Px(36.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(1.0)), ..default() }, BackgroundColor(Color::srgb(0.20,0.38,0.66)), BorderRadius::all(Val::Px(8.0)))).with_children(|b|{ b.spawn((Text::new(tr("Reiniciar")), TextFont { font: font.clone(), font_size: 16.0, ..default() }, TextColor(Color::WHITE)));});
                    row.spawn((Button, SudokuBackButton, Node { width: Val::Px(140.0), height: Val::Px(36.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(1.0)), ..default() }, BackgroundColor(Color::srgb(0.20,0.38,0.66)), BorderRadius::all(Val::Px(8.0)))).with_children(|b|{ b.spawn((Text::new(tr("Volver")), TextFont { font: font.clone(), font_size: 16.0, ..default() }, TextColor(Color::WHITE)));});
                });
            });
        });
}

fn cleanup_sudoku(mut commands: Commands, roots: Query<Entity, With<SudokuUiRoot>>) {
    for r in &roots { commands.entity(r).despawn(); }
    commands.remove_resource::<SudokuSession>();
}

fn update_sudoku(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut session: ResMut<SudokuSession>,
    cell_clicks: Query<(&Interaction, &SudokuCell), (Changed<Interaction>, Without<SudokuBackButton>)>,
    num_clicks: Query<(&Interaction, &SudokuNumButton), Changed<Interaction>>,
    eraser_clicks: Query<&Interaction, (Changed<Interaction>, With<SudokuEraserButton>)>,
    back_clicks: Query<&Interaction, (Changed<Interaction>, With<SudokuBackButton>)>,
    restart_clicks: Query<&Interaction, (Changed<Interaction>, With<SudokuRestartButton>)>,
    mut texts: Query<(&SudokuText, &mut Text)>,
    mut cells: Query<(&SudokuCell, &mut BackgroundColor, &Children)>,
    mut cell_texts: Query<&mut Text, Without<SudokuText>>,
) {
    if keys.just_pressed(KeyCode::Escape) || back_clicks.single().map_or(false, |i| *i==Interaction::Pressed) { commands.set_state(GameState::ClassicMenu); return; }
    if restart_clicks.single().map_or(false, |i| *i==Interaction::Pressed) { *session = SudokuSession::new(); }
    let mut clicked_cell: Option<(usize,usize)> = None;
    for (inter, cell) in &cell_clicks { if *inter==Interaction::Pressed { clicked_cell = Some((cell.0, cell.1)); break; } }
    if let Some((r,c)) = clicked_cell { session.selected = Some((r,c)); }
    if let Some((r,c)) = session.selected {
        let mut num: Option<u8> = None;
        for (inter, btn) in &num_clicks { if *inter==Interaction::Pressed { num = Some(btn.0); break; } }
        if eraser_clicks.single().map_or(false, |i| *i==Interaction::Pressed) { num = Some(0); }
        // teclado 1-9 y 0/Borrar
        for (k, n) in [(KeyCode::Digit1,1),(KeyCode::Digit2,2),(KeyCode::Digit3,3),(KeyCode::Digit4,4),(KeyCode::Digit5,5),(KeyCode::Digit6,6),(KeyCode::Digit7,7),(KeyCode::Digit8,8),(KeyCode::Digit9,9),(KeyCode::Digit0,0),(KeyCode::Backspace,0)] {
            if keys.just_pressed(k) { num = Some(n); break; }
        }
        if let Some(n) = num { session.set(r,c,n); }
    }
    for (field, mut text) in &mut texts {
        if field.0 == SudField::Status {
            if session.won { *text = Text::new(format!("¡Sudoku completado! 🎉 Errores: {}", session.mistakes)); }
            else if let Some((r,c)) = session.selected {
                let fixed = session.is_fixed(r,c);
                *text = Text::new(if fixed { format!("Celda fija ({},{})", r+1, c+1) } else { format!("Seleccionada ({},{}) — elige número", r+1, c+1) });
            } else { *text = Text::new("Toca una celda y elige número"); }
        }
    }
    for (cell, mut bg, children) in &mut cells {
        let (r,c) = (cell.0, cell.1);
        let is_selected = session.selected == Some((r,c));
        let is_fixed = session.is_fixed(r,c);
        let val = session.current_value(r,c);
        let is_error = val !=0 && val != session.solution[r][c];
        *bg = BackgroundColor(if is_selected { Color::srgb(0.30,0.60,0.85) } else if is_error { Color::srgb(0.85,0.45,0.45) } else if is_fixed { Color::srgb(0.88,0.88,0.86) } else { Color::srgb(0.96,0.96,0.94) });
        for child in children.iter() {
            if let Ok(mut text) = cell_texts.get_mut(child) {
                *text = Text::new(if val==0 { "".to_string() } else { val.to_string() });
            }
        }
    }
}
