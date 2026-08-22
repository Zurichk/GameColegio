//! Busca Diferencias — dos tableros 8×8 con 5 diferencias.

use bevy::prelude::*;
use rand::seq::SliceRandom;

use crate::game::GameState;
use crate::learning::screen_background;
use crate::i18n::tr;

const SIZE: usize = 8;
const DIFFS: usize = 5;

#[derive(Resource, Clone)]
struct DifferencesSession {
    left: [[char; SIZE]; SIZE],
    right: [[char; SIZE]; SIZE],
    diffs: Vec<(usize, usize)>,
    found: Vec<bool>,
    tries: u32,
}

impl DifferencesSession {
    fn new() -> Self {
        let mut rng = rand::thread_rng();
        let palette = ['●','▲','■','◆','★','♥','♦','♣','⬢','⬣'];
        let mut left = [[' '; SIZE]; SIZE];
        let mut right = [[' '; SIZE]; SIZE];
        for r in 0..SIZE { for c in 0..SIZE {
            let ch = *palette.choose(&mut rng).unwrap();
            left[r][c]=ch;
            right[r][c]=ch;
        }}
        let mut diffs = Vec::new();
        let mut cells: Vec<(usize,usize)> = (0..SIZE).flat_map(|r| (0..SIZE).map(move |c| (r,c))).collect();
        cells.shuffle(&mut rng);
        for (r,c) in cells.into_iter().take(DIFFS) {
            let mut other;
            loop { other = *palette.choose(&mut rng).unwrap(); if other != left[r][c] { break; } }
            right[r][c]=other;
            diffs.push((r,c));
        }
        Self { left, right, diffs, found: vec![false; DIFFS], tries: 0 }
    }
    fn click(&mut self, r: usize, c: usize, _is_right: bool) -> bool {
        // Solo cuenta clicks en el tablero derecho (o izquierdo, ambos valen)
        let idx = self.diffs.iter().position(|&(dr,dc)| dr==r && dc==c);
        if let Some(i) = idx {
            if !self.found[i] {
                self.found[i]=true;
                return true;
            }
        }
        self.tries+=1;
        false
    }
    fn all_found(&self) -> bool { self.found.iter().all(|&b| b) }
}

#[derive(Component)]
struct DiffUiRoot;
#[derive(Component)]
struct DiffText(DiffField);
#[derive(Clone, Copy, PartialEq, Eq)]
enum DiffField { Title, Status }
#[derive(Component)]
struct DiffCell(usize, usize, bool); // r,c, is_right
#[derive(Component)]
struct DiffBackButton;
#[derive(Component)]
struct DiffRestartButton;

pub struct DifferencesPlugin;
impl Plugin for DifferencesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::DifferencesGame), spawn_diff)
            .add_systems(OnExit(GameState::DifferencesGame), cleanup_diff)
            .add_systems(Update, update_diff.run_if(in_state(GameState::DifferencesGame)));
    }
}

fn spawn_diff(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands.insert_resource(DifferencesSession::new());
    commands.spawn((DiffUiRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, padding: UiRect::all(Val::Px(8.0)), ..default() }, screen_background(), Visibility::Visible, ZIndex(30)))
        .with_children(|overlay| {
            overlay.spawn((Node { flex_direction: FlexDirection::Column, width: Val::Px(720.0), padding: UiRect::all(Val::Px(14.0)), row_gap: Val::Px(8.0), align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.07,0.09,0.18,0.96)), BorderRadius::all(Val::Px(16.0)))).with_children(|panel| {
                panel.spawn((DiffText(DiffField::Title), Text::new("BUSCA DIFERENCIAS — 5 diferencias"), TextFont { font: font.clone(), font_size: 22.0, ..default() }, TextColor(Color::srgb(0.95,0.85,0.40))));
                panel.spawn((DiffText(DiffField::Status), Text::new("Toca la diferencia en cualquier tablero"), TextFont { font: font.clone(), font_size: 14.0, ..default() }, TextColor(Color::WHITE)));
                panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(12.0), justify_content: JustifyContent::Center, ..default() }).with_children(|row| {
                    for is_right in [false, true] {
                        row.spawn(Node { flex_direction: FlexDirection::Column, align_items: AlignItems::Center, row_gap: Val::Px(4.0), ..default() }).with_children(|col| {
                            col.spawn((Text::new(if is_right {"DERECHA"} else {"IZQUIERDA"}), TextFont { font: font.clone(), font_size: 12.0, ..default() }, TextColor(Color::srgb(0.80,0.85,1.0))));
                            col.spawn((Node { display: Display::Grid, grid_template_columns: vec![GridTrack::px(36.0); SIZE], grid_template_rows: vec![GridTrack::px(36.0); SIZE], column_gap: Val::Px(2.0), row_gap: Val::Px(2.0), padding: UiRect::all(Val::Px(4.0)), ..default() }, BackgroundColor(Color::srgb(0.10,0.12,0.14)), BorderRadius::all(Val::Px(8.0)))).with_children(|grid| {
                                for r in 0..SIZE { for c in 0..SIZE {
                                    grid.spawn((Button, DiffCell(r,c,is_right), Node { width: Val::Px(36.0), height: Val::Px(36.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgb(0.96,0.96,0.94)), BorderRadius::all(Val::Px(4.0)))).with_children(|cell| {
                                        cell.spawn((Text::new("".to_string()), TextFont { font: font.clone(), font_size: 16.0, ..default() }, TextColor(Color::BLACK)));
                                    });
                                }}
                            });
                        });
                    }
                });
                panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(10.0), ..default() }).with_children(|row| {
                    row.spawn((Button, DiffRestartButton, Node { width: Val::Px(140.0), height: Val::Px(36.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(1.0)), ..default() }, BackgroundColor(Color::srgb(0.20,0.38,0.66)), BorderRadius::all(Val::Px(8.0)))).with_children(|b|{ b.spawn((Text::new(tr("Reiniciar")), TextFont { font: font.clone(), font_size: 14.0, ..default() }, TextColor(Color::WHITE)));});
                    row.spawn((Button, DiffBackButton, Node { width: Val::Px(140.0), height: Val::Px(36.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(1.0)), ..default() }, BackgroundColor(Color::srgb(0.20,0.38,0.66)), BorderRadius::all(Val::Px(8.0)))).with_children(|b|{ b.spawn((Text::new(tr("Volver")), TextFont { font: font.clone(), font_size: 14.0, ..default() }, TextColor(Color::WHITE)));});
                });
            });
        });
}

fn cleanup_diff(mut commands: Commands, roots: Query<Entity, With<DiffUiRoot>>) {
    for r in &roots { commands.entity(r).despawn(); }
    commands.remove_resource::<DifferencesSession>();
}

fn update_diff(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut session: ResMut<DifferencesSession>,
    cell_clicks: Query<(&Interaction, &DiffCell), (Changed<Interaction>, Without<DiffBackButton>)>,
    back: Query<&Interaction, (Changed<Interaction>, With<DiffBackButton>)>,
    restart: Query<&Interaction, (Changed<Interaction>, With<DiffRestartButton>)>,
    mut texts: Query<(&DiffText, &mut Text)>,
    mut cells: Query<(&DiffCell, &mut BackgroundColor, &Children)>,
    mut cell_texts: Query<&mut Text, Without<DiffText>>,
) {
    if keys.just_pressed(KeyCode::Escape) || back.single().map_or(false, |i| *i==Interaction::Pressed) { commands.set_state(GameState::ClassicMenu); return; }
    if restart.single().map_or(false, |i| *i==Interaction::Pressed) { *session = DifferencesSession::new(); }
    let mut clicked: Option<(usize,usize,bool)> = None;
    for (inter, cell) in &cell_clicks { if *inter==Interaction::Pressed { clicked = Some((cell.0, cell.1, cell.2)); break; } }
    if let Some((r,c,is_right)) = clicked { session.click(r,c,is_right); }
    for (field, mut text) in &mut texts {
        if field.0 == DiffField::Status {
            if session.all_found() { *text = Text::new(format!("¡Todas encontradas! 🎉 Intentos: {}", session.tries)); }
            else { let found = session.found.iter().filter(|&&b| b).count(); *text = Text::new(format!("Encontradas {}/{} — Intentos: {}", found, DIFFS, session.tries)); }
        }
    }
    for (cell, mut bg, children) in &mut cells {
        let (r,c,is_right) = (cell.0, cell.1, cell.2);
        let ch = if is_right { session.right[r][c] } else { session.left[r][c] };
        let is_diff = session.diffs.contains(&(r,c));
        let is_found = is_diff && session.found[session.diffs.iter().position(|&p| p==(r,c)).unwrap_or(0)];
        *bg = BackgroundColor(if is_found { Color::srgb(0.30,0.70,0.30) } else { Color::srgb(0.96,0.96,0.94) });
        for child in children.iter() {
            if let Ok(mut text) = cell_texts.get_mut(child) {
                *text = Text::new(ch.to_string());
            }
        }
    }
}
