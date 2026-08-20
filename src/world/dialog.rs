//! Sistema de diálogos (Fase 4): caja de diálogo con texto, avance con
//! espacio o clic y textos personalizados por profesor/asignatura.
//!
//! Al pulsar **E** cerca de un profesor se abre una caja de diálogo con el
//! nombre del profesor y sus líneas (cada asignatura tiene las suyas). Se
//! avanza con **Espacio** o **clic izquierdo**; al terminar (o alejarse),
//! la caja se cierra.

use bevy::prelude::*;

use crate::game::GameState;
use crate::i18n::tr;
use crate::player::Player;
use crate::world::quiz::QuizSession;
use crate::world::teacher::Teacher;

/// Distancia máxima (metros) para hablar con un profesor.
const TALK_DISTANCE: f32 = 2.6;
/// Distancia a partir de la que se cierra el diálogo si el jugador se aleja.
const LEAVE_DISTANCE: f32 = TALK_DISTANCE * 1.6;

/// Sesión de diálogo activa: mientras exista, la caja está visible.
#[derive(Resource)]
pub struct DialogSession {
    /// Profesor con el que se está hablando.
    pub teacher: Entity,
    /// Nombre de la asignatura (título de la caja).
    pub subject: &'static str,
    /// Color de la asignatura (título de la caja).
    pub accent: Color,
    /// Líneas del diálogo.
    pub lines: Vec<&'static str>,
    /// Línea actual.
    pub index: usize,
}

/// Marca la raíz de la caja de diálogo (se oculta/muestra entera).
#[derive(Component)]
pub struct DialogBox;

/// Marca el texto con el nombre del profesor.
#[derive(Component)]
pub struct DialogName;

/// Marca el texto con la línea de diálogo.
#[derive(Component)]
pub struct DialogLine;

/// Marca el aviso "E — Hablar" sobre el jugador.
#[derive(Component)]
pub struct TeacherPrompt;

/// Plugin de diálogos: crea la UI y gestiona la sesión durante el juego.
pub struct DialogPlugin;

impl Plugin for DialogPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_teacher_prompt, spawn_dialog_box)).add_systems(
            Update,
            (update_dialog, update_teacher_prompt).run_if(in_state(GameState::Playing)),
        );
    }
}

/// Crea el aviso "E — Hablar" (oculto hasta acercarse a un profesor).
fn spawn_teacher_prompt(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands
        .spawn((
            TeacherPrompt,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(48.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            Visibility::Hidden,
            ZIndex(10),
        ))
        .with_children(|prompt| {
            prompt.spawn((
                Text::new(tr("E — Hablar · Q — Cuestionario")),
                TextFont {
                    font: font.clone(),
                    font_size: 26.0,
                    ..default()
                },
                TextColor(Color::srgb(0.85, 0.95, 0.80)),
                Node {
                    padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.05, 0.07, 0.16, 0.85)),
                BorderRadius::all(Val::Px(10.0)),
            ));
        });
}

/// Crea la caja de diálogo (oculta hasta que haya una sesión activa).
fn spawn_dialog_box(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands
        .spawn((
            DialogBox,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(96.0),
                width: Val::Px(700.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::axes(Val::Px(22.0), Val::Px(14.0)),
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.07, 0.16, 0.92)),
            BorderRadius::all(Val::Px(14.0)),
            Visibility::Hidden,
            ZIndex(20),
        ))
        .with_children(|dialog| {
            // Nombre del profesor (con el color de su asignatura).
            dialog.spawn((
                DialogName,
                Text::new(""),
                TextFont {
                    font: font.clone(),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            // Línea de diálogo (se ajusta a la anchura de la caja).
            dialog.spawn((
                DialogLine,
                Text::new(""),
                TextFont {
                    font: font.clone(),
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(0.92, 0.93, 0.98)),
                TextLayout {
                    linebreak: LineBreak::WordBoundary,
                    ..default()
                },
                Node {
                    max_width: Val::Px(656.0),
                    ..default()
                },
            ));
            // Ayuda de navegación.
            dialog.spawn((
                Text::new(tr("Espacio / Clic — Continuar")),
                TextFont {
                    font,
                    font_size: 15.0,
                    ..default()
                },
                TextColor(Color::srgb(0.60, 0.68, 0.88)),
            ));
        });
}

/// Gestiona la sesión de diálogo: la abre con E cerca de un profesor y la
/// avanza con Espacio o clic. Si el jugador se aleja, la cierra.
fn update_dialog(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    player_q: Query<&Transform, (With<Player>, Without<Teacher>)>,
    teachers: Query<(Entity, &Teacher, &Transform), Without<Player>>,
    quiz: Option<Res<QuizSession>>,
    session: Option<ResMut<DialogSession>>,
    mut box_root: Query<&mut Visibility, (With<DialogBox>, Without<DialogName>, Without<DialogLine>)>,
    mut texts: ParamSet<(
        Query<(&mut Text, &mut TextColor, &mut Visibility), With<DialogName>>,
        Query<(&mut Text, &mut Visibility), With<DialogLine>>,
    )>,
) {
    let Ok(player_tf) = player_q.single() else {
        return;
    };

    // 1) ¿Sesión activa? Avanzar, cerrar o actualizar el texto.
    if let Some(mut session) = session {
        let close = {
            // ¿El jugador se alejó del profesor?
            let far = match teachers.get(session.teacher) {
                Ok((_, _, tf)) => {
                    let dx = tf.translation.x - player_tf.translation.x;
                    let dz = tf.translation.z - player_tf.translation.z;
                    Vec2::new(dx, dz).length() > LEAVE_DISTANCE
                }
                Err(_) => true,
            };
            if far {
                true
            } else if keys.just_pressed(KeyCode::Space) || mouse.just_pressed(MouseButton::Left) {
                session.index += 1;
                session.index >= session.lines.len()
            } else {
                false
            }
        };

        if close {
            commands.remove_resource::<DialogSession>();
        } else {
            // Nombre del profesor (con el color de su asignatura).
            {
                let mut name_query = texts.p0();
                let Ok(mut name) = name_query.single_mut() else {
                    return;
                };
                *name.0 = Text::new(tr(session.subject));
                *name.1 = TextColor(session.accent);
                *name.2 = Visibility::Visible;
            }
            // Línea de diálogo actual.
            {
                let mut line_query = texts.p1();
                let Ok((mut line, mut line_vis)) = line_query.single_mut() else {
                    return;
                };
                *line = Text::new(tr(session.lines[session.index]));
                *line_vis = Visibility::Visible;
            }
        }
        return;
    }

    // 2) Sin sesión: la caja queda oculta.
    if let Ok(mut vis) = box_root.single_mut() {
        *vis = Visibility::Hidden;
    }

    // 3) ¿E cerca de un profesor (y sin cuestionario abierto)? Diálogo.
    if quiz.is_some() {
        return;
    }
    if keys.just_pressed(KeyCode::KeyE) {
        let mut nearest: Option<(f32, Entity)> = None;
        for (entity, _teacher, tf) in &teachers {
            let dx = tf.translation.x - player_tf.translation.x;
            let dz = tf.translation.z - player_tf.translation.z;
            let dist = Vec2::new(dx, dz).length();
            if dist < TALK_DISTANCE && nearest.map_or(true, |(d, _)| dist < d) {
                nearest = Some((dist, entity));
            }
        }
        if let Some((_, entity)) = nearest {
            if let Ok((_, teacher, _)) = teachers.get(entity) {
                commands.insert_resource(DialogSession {
                    teacher: entity,
                    subject: teacher.subject,
                    accent: teacher.accent,
                    lines: teacher.lines.to_vec(),
                    index: 0,
                });
            }
        }
    }
}

/// Muestra u oculta el aviso "E — Hablar · Q — Cuestionario" según el
/// profesor más cercano (oculto durante un diálogo o un cuestionario).
fn update_teacher_prompt(
    player_q: Query<&Transform, (With<Player>, Without<Teacher>)>,
    teachers: Query<&Transform, (With<Teacher>, Without<Player>)>,
    mut prompt: Query<(&mut Visibility, &mut Text), With<TeacherPrompt>>,
    dialog: Option<Res<DialogSession>>,
    quiz: Option<Res<QuizSession>>,
) {
    if dialog.is_some() || quiz.is_some() {
        if let Ok((mut vis, _)) = prompt.single_mut() {
            *vis = Visibility::Hidden;
        }
        return;
    }
    let Ok(player_tf) = player_q.single() else {
        return;
    };
    let mut near = false;
    for tf in &teachers {
        let dx = tf.translation.x - player_tf.translation.x;
        let dz = tf.translation.z - player_tf.translation.z;
        if Vec2::new(dx, dz).length() < TALK_DISTANCE {
            near = true;
            break;
        }
    }
    if let Ok((mut vis, _)) = prompt.single_mut() {
        *vis = if near { Visibility::Visible } else { Visibility::Hidden };
    }
}