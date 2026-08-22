//! Tangram — 7 piezas, forma la silueta.

use bevy::prelude::*;

use crate::game::GameState;
use crate::learning::screen_background;
use crate::i18n::tr;

const SILHOUETTES: [(&str, &str, [&str; 7]); 5] = [
    ("Casa", "🏠", ["▲","■","■","▲","▲","◆","▭"]),
    ("Gato", "🐱", ["▲","▲","■","◆","■","▭","●"]),
    ("Barco", "⛵", ["▲","▲","■","■","◆","▭","▲"]),
    ("Pez", "🐟", ["▲","■","◆","▲","■","▭","●"]),
    ("Avión", "✈️", ["▲","▲","■","■","◆","▲","▭"]),
];

#[derive(Resource, Clone)]
struct TangramSession {
    round: usize,
    silhouette: usize,
    placed: [bool; 7],
    won: bool,
    steps: u32,
}

impl TangramSession {
    fn new() -> Self {
        Self { round: 0, silhouette: rand::random::<usize>() % SILHOUETTES.len(), placed: [false;7], won: false, steps: 0 }
    }
    fn current(&self) -> (&str, &str, [&str;7]) {
        let (name, emoji, pieces) = SILHOUETTES[self.silhouette];
        (name, emoji, pieces)
    }
    fn toggle(&mut self, idx: usize) {
        if self.won { return; }
        self.placed[idx] = !self.placed[idx];
        self.steps += 1;
        if self.placed.iter().all(|&b| b) {
            self.won = true;
        }
    }
    fn next(&mut self) {
        self.silhouette = (self.silhouette + 1) % SILHOUETTES.len();
        self.placed = [false;7];
        self.won = false;
        self.round += 1;
    }
}

#[derive(Component)]
struct TangramUiRoot;
#[derive(Component)]
struct TangramText(TangField);
#[derive(Clone, Copy, PartialEq, Eq)]
enum TangField { Title, Status, Silhouette }
#[derive(Component)]
struct TangramPieceButton(usize);
#[derive(Component)]
struct TangramNextButton;
#[derive(Component)]
struct TangramBackButton;
#[derive(Component)]
struct TangramRestartButton;

pub struct TangramPlugin;
impl Plugin for TangramPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::TangramGame), spawn_tangram)
            .add_systems(OnExit(GameState::TangramGame), cleanup_tangram)
            .add_systems(Update, update_tangram.run_if(in_state(GameState::TangramGame)));
    }
}

fn spawn_tangram(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands.insert_resource(TangramSession::new());
    commands.spawn((TangramUiRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, padding: UiRect::all(Val::Px(8.0)), ..default() }, screen_background(), Visibility::Visible, ZIndex(30)))
        .with_children(|overlay| {
            overlay.spawn((Node { flex_direction: FlexDirection::Column, width: Val::Px(640.0), padding: UiRect::all(Val::Px(16.0)), row_gap: Val::Px(10.0), align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.07,0.09,0.18,0.96)), BorderRadius::all(Val::Px(16.0)))).with_children(|panel| {
                panel.spawn((TangramText(TangField::Title), Text::new("TANGRAM — 7 piezas"), TextFont { font: font.clone(), font_size: 28.0, ..default() }, TextColor(Color::srgb(0.95,0.85,0.40))));
                panel.spawn((TangramText(TangField::Silhouette), Text::new("🏠"), TextFont { font: font.clone(), font_size: 64.0, ..default() }, TextColor(Color::WHITE)));
                panel.spawn((TangramText(TangField::Status), Text::new("Toca las 7 piezas para formar la silueta"), TextFont { font: font.clone(), font_size: 16.0, ..default() }, TextColor(Color::WHITE)));
                panel.spawn(Node { display: Display::Grid, grid_template_columns: vec![GridTrack::px(80.0); 4], grid_template_rows: vec![GridTrack::px(80.0); 2], column_gap: Val::Px(10.0), row_gap: Val::Px(10.0), justify_content: JustifyContent::Center, ..default() }).with_children(|grid| {
                    for i in 0..7 {
                        grid.spawn((Button, TangramPieceButton(i), Node { width: Val::Px(80.0), height: Val::Px(80.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20,0.38,0.66)), BorderColor(Color::srgb(0.60,0.80,1.0)), BorderRadius::all(Val::Px(12.0)))).with_children(|b| {
                            b.spawn((Text::new("".to_string()), TextFont { font: font.clone(), font_size: 28.0, ..default() }, TextColor(Color::WHITE)));
                        });
                    }
                });
                panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(10.0), ..default() }).with_children(|row| {
                    row.spawn((Button, TangramNextButton, Node { width: Val::Px(140.0), height: Val::Px(36.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(1.0)), ..default() }, BackgroundColor(Color::srgb(0.15,0.42,0.25)), BorderRadius::all(Val::Px(8.0)))).with_children(|b|{ b.spawn((Text::new("Siguiente"), TextFont { font: font.clone(), font_size: 14.0, ..default() }, TextColor(Color::WHITE)));});
                    row.spawn((Button, TangramRestartButton, Node { width: Val::Px(140.0), height: Val::Px(36.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(1.0)), ..default() }, BackgroundColor(Color::srgb(0.20,0.38,0.66)), BorderRadius::all(Val::Px(8.0)))).with_children(|b|{ b.spawn((Text::new(tr("Reiniciar")), TextFont { font: font.clone(), font_size: 14.0, ..default() }, TextColor(Color::WHITE)));});
                    row.spawn((Button, TangramBackButton, Node { width: Val::Px(140.0), height: Val::Px(36.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(1.0)), ..default() }, BackgroundColor(Color::srgb(0.20,0.38,0.66)), BorderRadius::all(Val::Px(8.0)))).with_children(|b|{ b.spawn((Text::new(tr("Volver")), TextFont { font: font.clone(), font_size: 14.0, ..default() }, TextColor(Color::WHITE)));});
                });
            });
        });
}

fn cleanup_tangram(mut commands: Commands, roots: Query<Entity, With<TangramUiRoot>>) {
    for r in &roots { commands.entity(r).despawn(); }
    commands.remove_resource::<TangramSession>();
}

fn update_tangram(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut session: ResMut<TangramSession>,
    piece_clicks: Query<(&Interaction, &TangramPieceButton), (Changed<Interaction>, Without<TangramBackButton>)>,
    back: Query<&Interaction, (Changed<Interaction>, With<TangramBackButton>)>,
    restart: Query<&Interaction, (Changed<Interaction>, With<TangramRestartButton>)>,
    next: Query<&Interaction, (Changed<Interaction>, With<TangramNextButton>)>,
    mut texts: Query<(&TangramText, &mut Text)>,
    mut pieces: Query<(&TangramPieceButton, &mut BackgroundColor, &Children)>,
    mut piece_texts: Query<&mut Text, Without<TangramText>>,
) {
    if keys.just_pressed(KeyCode::Escape) || back.single().map_or(false, |i| *i==Interaction::Pressed) { commands.set_state(GameState::ClassicMenu); return; }
    if restart.single().map_or(false, |i| *i==Interaction::Pressed) { *session = TangramSession::new(); }
    if next.single().map_or(false, |i| *i==Interaction::Pressed) { session.next(); }
    for (inter, btn) in &piece_clicks { if *inter==Interaction::Pressed { session.toggle(btn.0); } }
    for (field, mut text) in &mut texts {
        match field.0 {
            TangField::Silhouette => {
                let (name, emoji, _) = session.current();
                *text = Text::new(format!("{} {}", emoji, name));
            },
            TangField::Status => {
                if session.won { *text = Text::new(format!("¡Silueta completada! 🎉 Pasos: {} — Siguiente", session.steps)); }
                else { *text = Text::new(format!("Colocadas {}/7 — toca piezas", session.placed.iter().filter(|&&b| b).count())); }
            },
            _=>{}
        }
    }
    for (btn, mut bg, children) in &mut pieces {
        let is_placed = session.placed[btn.0];
        *bg = BackgroundColor(if is_placed { Color::srgb(0.30,0.70,0.30) } else { Color::srgb(0.20,0.38,0.66) });
        for child in children.iter() {
            if let Ok(mut text) = piece_texts.get_mut(child) {
                let (_, _, pieces) = session.current();
                *text = Text::new(pieces[btn.0].to_string());
            }
        }
    }
}
