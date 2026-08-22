//! Pintar punteado — une los puntos numerados para revelar el dibujo.

use bevy::prelude::*;

use crate::game::GameState;
use crate::learning::screen_background;
use crate::i18n::tr;

/// Puntos del dibujo "casita" (10 puntos) en coordenadas relativas 0..400 dentro del lienzo 520x400
const POINTS: &[(f32, f32)] = &[
    (80.0, 280.0),  // 1 base izq
    (80.0, 180.0),  // 2 pared izq
    (140.0, 100.0), // 3 tejado izq
    (260.0, 40.0),  // 4 pico
    (380.0, 100.0), // 5 tejado der
    (440.0, 180.0), // 6 pared der
    (440.0, 280.0), // 7 base der
    (260.0, 280.0), // 8 base centro (puerta arriba)
    (260.0, 200.0), // 9 puerta abajo
    (80.0, 280.0),  // 10 cierre (vuelve a 1)
];

#[derive(Resource, Clone)]
struct PaintingSession {
    next: usize, // próximo punto esperado (0..POINTS.len())
    completed: bool,
    wrong_flash: f32,
}

impl PaintingSession {
    fn new() -> Self { Self { next: 0, completed: false, wrong_flash: 0.0 } }
}

#[derive(Component)]
struct PaintingUiRoot;
#[derive(Component)]
struct PaintingText(PaintField);
#[derive(Clone, Copy, PartialEq, Eq)]
enum PaintField { Title, Status }
#[derive(Component)]
struct PaintingDot(usize);
#[derive(Component)]
struct PaintingBackButton;
#[derive(Component)]
struct PaintingRestartButton;
#[derive(Component)]
struct PaintingClearButton;

pub struct PaintingPlugin;
impl Plugin for PaintingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::PaintingGame), spawn_painting)
            .add_systems(OnExit(GameState::PaintingGame), cleanup_painting)
            .add_systems(Update, update_painting.run_if(in_state(GameState::PaintingGame)));
    }
}

fn spawn_painting(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands.insert_resource(PaintingSession::new());
    commands
        .spawn((PaintingUiRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, padding: UiRect::all(Val::Px(8.0)), ..default() }, screen_background(), Visibility::Visible, ZIndex(30)))
        .with_children(|overlay| {
            overlay.spawn((Node { flex_direction: FlexDirection::Column, width: Val::Px(720.0), padding: UiRect::axes(Val::Px(20.0), Val::Px(16.0)), row_gap: Val::Px(10.0), align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.07, 0.09, 0.18, 0.96)), BorderRadius::all(Val::Px(16.0)))).with_children(|panel| {
                panel.spawn((PaintingText(PaintField::Title), Text::new("PINTAR PUNTEADO"), TextFont { font: font.clone(), font_size: 28.0, ..default() }, TextColor(Color::srgb(0.95, 0.85, 0.40))));
                panel.spawn((PaintingText(PaintField::Status), Text::new("Une los puntos en orden 1 → 10"), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE)));
                // Lienzo 520x400 con puntos absolutos
                panel.spawn((Node { position_type: PositionType::Relative, width: Val::Px(520.0), height: Val::Px(400.0), ..default() }, BackgroundColor(Color::srgb(0.96, 0.96, 0.94)), BorderRadius::all(Val::Px(16.0)))).with_children(|canvas| {
                    // Líneas punteadas guía (fondo)
                    for i in 0..POINTS.len()-1 {
                        let (x0,y0) = POINTS[i];
                        let (x1,y1) = POINTS[i+1];
                        let dx = x1 - x0; let dy = y1 - y0;
                        let len = (dx*dx + dy*dy).sqrt();
                        let angle = dy.atan2(dx);
                        // línea fina punteada como guía
                        canvas.spawn((
                            Node { position_type: PositionType::Absolute, left: Val::Px(x0), top: Val::Px(y0), width: Val::Px(len), height: Val::Px(2.0), ..default() },
                            BackgroundColor(Color::srgba(0.70, 0.70, 0.70, 0.35)),
                            Transform::from_rotation(Quat::from_rotation_z(angle)),
                        ));
                    }
                    for (i, (x,y)) in POINTS.iter().enumerate().take(POINTS.len()-1) {
                        // No duplicar el último que es cierre
                        if i == POINTS.len()-1 { continue; }
                        canvas.spawn((
                            Button,
                            PaintingDot(i),
                            Node { position_type: PositionType::Absolute, left: Val::Px(x - 18.0), top: Val::Px(y - 18.0), width: Val::Px(36.0), height: Val::Px(36.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() },
                            BackgroundColor(Color::WHITE),
                            BorderColor(Color::srgb(0.20, 0.20, 0.22)),
                            BorderRadius::all(Val::Px(18.0)),
                        )).with_children(|dot| {
                            dot.spawn((Text::new((i+1).to_string()), TextFont { font: font.clone(), font_size: 16.0, ..default() }, TextColor(Color::BLACK)));
                        });
                    }
                });
                panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(12.0), ..default() }).with_children(|row| {
                    row.spawn((Button, PaintingClearButton, Node { width: Val::Px(140.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.30, 0.35, 0.55)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new("Borrar"), TextFont { font: font.clone(), font_size: 16.0, ..default() }, TextColor(Color::WHITE))); });
                    row.spawn((Button, PaintingRestartButton, Node { width: Val::Px(140.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Reiniciar")), TextFont { font: font.clone(), font_size: 16.0, ..default() }, TextColor(Color::WHITE))); });
                    row.spawn((Button, PaintingBackButton, Node { width: Val::Px(140.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Volver")), TextFont { font: font.clone(), font_size: 16.0, ..default() }, TextColor(Color::WHITE))); });
                });
            });
        });
}

fn cleanup_painting(mut commands: Commands, roots: Query<Entity, With<PaintingUiRoot>>) {
    for root in &roots { commands.entity(root).despawn(); }
    commands.remove_resource::<PaintingSession>();
}

fn update_painting(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut commands: Commands,
    mut session: ResMut<PaintingSession>,
    dot_clicks: Query<(&Interaction, &PaintingDot), (Changed<Interaction>, Without<PaintingBackButton>)>,
    back_clicks: Query<&Interaction, (Changed<Interaction>, With<PaintingBackButton>)>,
    restart_clicks: Query<&Interaction, (Changed<Interaction>, With<PaintingRestartButton>)>,
    clear_clicks: Query<&Interaction, (Changed<Interaction>, With<PaintingClearButton>)>,
    mut dots: Query<(&PaintingDot, &mut BackgroundColor, &mut BorderColor)>,
    mut texts: Query<(&PaintingText, &mut Text)>,
) {
    if keys.just_pressed(KeyCode::Escape) || back_clicks.single().map_or(false, |i| *i == Interaction::Pressed) { commands.set_state(GameState::ClassicMenu); return; }
    if restart_clicks.single().map_or(false, |i| *i == Interaction::Pressed) { *session = PaintingSession::new(); }
    if clear_clicks.single().map_or(false, |i| *i == Interaction::Pressed) { session.next = 0; session.completed = false; session.wrong_flash = 0.0; }
    if session.wrong_flash > 0.0 { session.wrong_flash -= time.delta_secs(); }

    let mut clicked: Option<usize> = None;
    for (interaction, dot) in &dot_clicks { if *interaction == Interaction::Pressed { clicked = Some(dot.0); break; } }
    if let Some(idx) = clicked {
        if session.completed { return; }
        if idx == session.next {
            session.next += 1;
            if session.next >= POINTS.len()-1 {
                session.completed = true;
            }
        } else {
            session.wrong_flash = 0.5;
        }
    }

    for (field, mut text) in &mut texts {
        if field.0 == PaintField::Status {
            if session.completed {
                *text = Text::new("¡Dibujo completado! 🎉 ¡Muy bien!");
            } else if session.wrong_flash > 0.0 {
                *text = Text::new(format!("¡Ese no! Toca el {} → Esperado {}", clicked.map(|c| (c+1).to_string()).unwrap_or("?".to_string()), session.next+1));
            } else {
                *text = Text::new(format!("Une en orden: {} / {}  — Siguiente: {}", session.next, POINTS.len()-1, session.next+1));
            }
        }
    }
    for (dot, mut bg, mut border) in &mut dots {
        let idx = dot.0;
        if idx < session.next {
            *bg = BackgroundColor(Color::srgb(0.30, 0.85, 0.30));
            *border = BorderColor(Color::srgb(0.15, 0.55, 0.15));
        } else if idx == session.next && session.wrong_flash > 0.0 {
            *bg = BackgroundColor(Color::srgb(0.90, 0.30, 0.30));
            *border = BorderColor(Color::srgb(0.70, 0.15, 0.15));
        } else if idx == session.next {
            *bg = BackgroundColor(Color::srgb(0.95, 0.85, 0.40));
            *border = BorderColor(Color::srgb(0.80, 0.60, 0.10));
        } else {
            *bg = BackgroundColor(Color::WHITE);
            *border = BorderColor(Color::srgb(0.20, 0.20, 0.22));
        }
    }
}
