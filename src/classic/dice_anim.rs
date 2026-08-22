//! Animaciones compartidas: dado rodando y bombo de bingo.
//!
//! Se usa en Oca, Parchís, Tablero (dado) y Bingo (bola). El dado ocupa el
//! espacio libre lateral izq/der que menciona el usuario: no tapa el tablero
//! y es visible en responsive (se oculta en móviles muy estrechos via UiScale).

use bevy::prelude::*;
use rand::Rng;

// ---------------------------------------------------------------------------
// Dado animado
// ---------------------------------------------------------------------------

/// Estado de un dado que muestra animación de rodar antes de asentarse.
#[derive(Component, Clone, Debug)]
pub struct AnimatedDice {
    /// Valor visible actual (1..=6).
    pub display: u8,
    /// Valor final tras la animación.
    pub target: u8,
    /// Si está rodando.
    pub rolling: bool,
    /// Tiempo acumulado.
    pub elapsed: f32,
    /// Duración total de la animación.
    pub duration: f32,
}

impl Default for AnimatedDice {
    fn default() -> Self {
        Self { display: 1, target: 1, rolling: false, elapsed: 0.0, duration: 0.85 }
    }
}

impl AnimatedDice {
    pub fn roll_to(&mut self, target: u8) {
        self.target = target.clamp(1, 6);
        self.rolling = true;
        self.elapsed = 0.0;
        self.duration = 0.85;
    }
    /// Avanza la animación; devuelve el valor a mostrar este frame.
    pub fn tick(&mut self, dt: f32) -> u8 {
        if !self.rolling {
            return self.display;
        }
        self.elapsed += dt;
        if self.elapsed >= self.duration {
            self.rolling = false;
            self.display = self.target;
        } else {
            // Cambia de cara cada ~70ms + efecto rebote visual via escala manejada aparte.
            let step = (self.elapsed / 0.065) as u32;
            let prev_step = ((self.elapsed - dt) / 0.065) as u32;
            if step != prev_step {
                self.display = rand::thread_rng().gen_range(1..=6);
            }
        }
        self.display
    }
    pub fn is_rolling(&self) -> bool { self.rolling }
}

/// Marcador del texto/ícono que pinta el dado.
#[derive(Component)]
pub struct DiceFaceText;

/// Marcador de la caja 3D del dado (para escala rebote).
#[derive(Component)]
pub struct DiceCube;

/// Contenedor lateral del dado (izq o der).
#[derive(Component)]
pub struct SideDicePanel;

/// Spawnea un panel lateral con dado animado.
/// `side`: "IZQUIERDA" o "DERECHA" para el título. Devuelve la entidad del `AnimatedDice`.
pub fn spawn_side_dice(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    title: &str,
) -> Entity {
    let mut dice_entity = Entity::PLACEHOLDER;
    parent
        .spawn((
            SideDicePanel,
            Node {
                width: Val::Px(160.0),
                height: Val::Px(220.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(10.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.10, 0.14, 0.28, 0.88)),
            BorderColor(Color::srgba(0.60, 0.80, 1.0, 0.35)),
            BorderRadius::all(Val::Px(16.0)),
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new(title),
                TextFont { font: font.clone(), font_size: 13.0, ..default() },
                TextColor(Color::srgb(0.80, 0.85, 1.0)),
            ));
            // Caja del dado: fondo blanco con borde, gira/escala al rodar
            let e = panel
                .spawn((
                    AnimatedDice::default(),
                    DiceCube,
                    Node {
                        width: Val::Px(84.0),
                        height: Val::Px(84.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::WHITE),
                    BorderColor(Color::srgb(0.30, 0.30, 0.35)),
                    BorderRadius::all(Val::Px(14.0)),
                ))
                .with_children(|cube| {
                    // Texto del dado con puntos unicode + número grande
                    cube.spawn((
                        DiceFaceText,
                        Text::new("⚀"),
                        TextFont { font: font.clone(), font_size: 52.0, ..default() },
                        TextColor(Color::srgb(0.10, 0.10, 0.12)),
                    ));
                })
                .id();
            dice_entity = e;
            panel.spawn((
                Text::new("Tira el dado"),
                TextFont { font: font.clone(), font_size: 11.0, ..default() },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.70)),
            ));
        });
    dice_entity
}

/// Sistema genérico que anima todos los `AnimatedDice` (rebote + giro).
pub fn animate_dice_system(
    time: Res<Time>,
    mut dice_q: Query<(&mut AnimatedDice, &mut Transform)>,
) {
    let dt = time.delta_secs();
    for (mut dice, mut tf) in &mut dice_q {
        let was_rolling = dice.rolling;
        dice.tick(dt);
        if dice.rolling {
            let t = dice.elapsed / dice.duration;
            let bounce = 1.0 + (t * std::f32::consts::TAU * 3.0).sin() * 0.12 * (1.0 - t);
            tf.scale = Vec3::splat(bounce);
            tf.rotation = Quat::from_rotation_z(t * 8.0);
        } else if was_rolling {
            tf.scale = Vec3::ONE;
            tf.rotation = Quat::IDENTITY;
        }
    }
}

/// Actualiza el texto del dado según el valor visible. Usa número grande (1..6)
/// en lugar de unicode ⚀ (no soportado por Roboto) para garantizar visibilidad.
pub fn dice_text_update_system(
    dice_q: Query<&AnimatedDice>,
    mut text_q: Query<(&mut Text, &ChildOf), With<DiceFaceText>>,
    cube_q: Query<Entity, With<DiceCube>>,
) {
    for cube_entity in cube_q.iter() {
        let Ok(dice) = dice_q.get(cube_entity) else { continue; };
        for (mut text, parent) in text_q.iter_mut() {
            if parent.0 == cube_entity {
                // Mostrar número grande; durante el giro se ve cambiando 1..6
                *text = Text::new(format!("{}", dice.display));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Bingo — bombo y bola
// ---------------------------------------------------------------------------

#[derive(Component, Debug, Clone)]
pub struct BingoBombo {
    pub rolling: bool,
    pub elapsed: f32,
    pub duration: f32,
    pub display: u8,
    pub target: u8,
    pub last_balls: Vec<u8>,
}

impl Default for BingoBombo {
    fn default() -> Self {
        Self { rolling: false, elapsed: 0.0, duration: 1.1, display: 0, target: 0, last_balls: Vec::new() }
    }
}

impl BingoBombo {
    pub fn draw_to(&mut self, target: u8) {
        self.target = target;
        self.rolling = true;
        self.elapsed = 0.0;
        self.duration = 1.0;
    }
    pub fn tick(&mut self, dt: f32) -> u8 {
        if !self.rolling { return self.display; }
        self.elapsed += dt;
        if self.elapsed >= self.duration {
            self.rolling = false;
            self.display = self.target;
            if self.target != 0 && !self.last_balls.contains(&self.target) {
                self.last_balls.push(self.target);
                if self.last_balls.len() > 8 { self.last_balls.remove(0); }
            }
        } else {
            if (self.elapsed / 0.08) as u32 != ((self.elapsed - dt) / 0.08) as u32 {
                self.display = rand::thread_rng().gen_range(1..=75);
            }
        }
        self.display
    }
}

#[derive(Component)]
pub struct BingoBallText;
#[derive(Component)]
pub struct BingoBomboCube;
#[derive(Component)]
pub struct BingoSidePanel;

/// Color de bola según columna BINGO (usa rango 1..75).
pub fn bingo_ball_color(n: u8) -> Color {
    match n {
        1..=15 => Color::srgb(0.30, 0.55, 0.95),   // B - azul
        16..=30 => Color::srgb(0.90, 0.25, 0.25),  // I - rojo
        31..=45 => Color::srgb(0.92, 0.92, 0.92),  // N - blanco/gris claro
        46..=60 => Color::srgb(0.22, 0.65, 0.30),  // G - verde
        _ => Color::srgb(0.95, 0.60, 0.15),        // O - naranja
    }
}
pub fn bingo_ball_text_color(n: u8) -> Color {
    match n {
        31..=45 => Color::BLACK,
        _ => Color::WHITE,
    }
}

#[derive(Component)]
pub struct BingoHistorySlot(pub usize);

#[derive(Component)]
pub struct BingoHistoryText(pub usize);

/// Spawnea panel lateral del bombo (izquierda) con bola grande y lista de últimas.
pub fn spawn_bingo_bombo_panel(parent: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    parent
        .spawn((
            BingoSidePanel,
            Node {
                width: Val::Px(190.0),
                height: Val::Px(400.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexStart,
                row_gap: Val::Px(10.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.10, 0.14, 0.28, 0.92)),
            BorderColor(Color::srgba(0.60, 0.80, 1.0, 0.35)),
            BorderRadius::all(Val::Px(16.0)),
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new("BOMBO"),
                TextFont { font: font.clone(), font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.95, 0.85, 0.40)),
            ));
            // Esfera del bombo
            panel.spawn((
                Node {
                    width: Val::Px(110.0),
                    height: Val::Px(110.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(3.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.95, 0.85, 0.40)),
                BorderColor(Color::WHITE),
                BorderRadius::all(Val::Px(55.0)),
            ))
            .with_children(|bombo| {
                bombo.spawn((
                    BingoBombo::default(),
                    BingoBomboCube,
                    Node {
                        width: Val::Px(78.0),
                        height: Val::Px(78.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::WHITE),
                    BorderRadius::all(Val::Px(39.0)),
                ))
                .with_children(|ball| {
                    ball.spawn((
                        BingoBallText,
                        Text::new("—"),
                        TextFont { font: font.clone(), font_size: 32.0, ..default() },
                        TextColor(Color::BLACK),
                    ));
                });
            });
            panel.spawn((
                Text::new("Últimas 8 bolas"),
                TextFont { font: font.clone(), font_size: 11.0, ..default() },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.85)),
            ));
            // Grid 4x2 para historial de 8 bolas
            panel.spawn(Node {
                display: Display::Grid,
                grid_template_columns: vec![GridTrack::px(36.0); 4],
                grid_template_rows: vec![GridTrack::px(36.0); 2],
                column_gap: Val::Px(6.0),
                row_gap: Val::Px(6.0),
                ..default()
            }).with_children(|grid| {
                for i in 0..8 {
                    grid.spawn((
                        BingoHistorySlot(i),
                        Node {
                            width: Val::Px(36.0),
                            height: Val::Px(36.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.20, 0.22, 0.30, 0.85)),
                        BorderColor(Color::srgba(1.0, 1.0, 1.0, 0.18)),
                        BorderRadius::all(Val::Px(18.0)),
                    )).with_children(|slot| {
                        slot.spawn((
                            BingoHistoryText(i),
                            Text::new("·"),
                            TextFont { font: font.clone(), font_size: 14.0, ..default() },
                            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.45)),
                        ));
                    });
                }
            });
            panel.spawn((
                Text::new("B I N G O"),
                TextFont { font: font.clone(), font_size: 9.0, ..default() },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.55)),
            ));
        });
}

pub fn animate_bingo_system(
    time: Res<Time>,
    mut bombo_q: Query<(&mut BingoBombo, &mut Transform)>,
    mut ball_text_q: Query<(&mut Text, &ChildOf), With<BingoBallText>>,
) {
    let dt = time.delta_secs();
    for (mut bombo, mut tf) in &mut bombo_q {
        let display = bombo.tick(dt);
        if bombo.rolling {
            let t = bombo.elapsed / bombo.duration;
            let scale = 1.0 + (t * std::f32::consts::TAU * 4.0).sin() * 0.10 * (1.0 - t);
            tf.scale = Vec3::splat(scale);
            tf.rotation = Quat::from_rotation_z(t * 10.0);
            tf.translation.x = (t * 30.0).sin() * 2.0;
        } else {
            tf.scale = Vec3::ONE;
            tf.rotation = Quat::IDENTITY;
            tf.translation = Vec3::ZERO;
        }
        for (mut text, _parent) in ball_text_q.iter_mut() {
            let label = if display == 0 { "—".to_string() } else { display.to_string() };
            *text = Text::new(label);
        }
    }
}

/// Pinta el historial de 8 últimas bolas con color BINGO.
pub fn update_bingo_history_system(
    bombo_q: Query<&BingoBombo>,
    mut slots: Query<(&BingoHistorySlot, &mut BackgroundColor, &Children)>,
    mut texts: Query<(&BingoHistoryText, &mut Text, &mut TextColor)>,
) {
    let Ok(bombo) = bombo_q.single() else { return; };
    // Mapa slot i -> texto hijo
    for (slot, mut bg, children) in slots.iter_mut() {
        let idx = slot.0;
        // last_balls está en orden cronológico (0 más antigua, último es la más reciente)
        // Queremos mostrarlas en orden reciente arriba: invertimos
        let val_opt = if idx < bombo.last_balls.len() {
            // Mostrar las últimas con el más reciente al final del grid
            Some(bombo.last_balls[idx])
        } else {
            None
        };
        let (bg_color, border_alpha) = match val_opt {
            Some(n) => (bingo_ball_color(n), 1.0),
            None => (Color::srgba(0.20, 0.22, 0.30, 0.85), 0.18),
        };
        // Resalta la más reciente (última del vec)
        let is_latest = val_opt.is_some() && idx + 1 == bombo.last_balls.len() && !bombo.rolling;
        let mut final_bg = bg_color;
        if is_latest {
            // Pulso suave: aclarar
            final_bg = bg_color.mix(&Color::WHITE, 0.18);
        }
        *bg = BackgroundColor(final_bg);
        for child in children.iter() {
            if let Ok((hist, mut text, mut color)) = texts.get_mut(child) {
                if hist.0 != idx { continue; }
                if let Some(n) = val_opt {
                    *text = Text::new(n.to_string());
                    *color = TextColor(bingo_ball_text_color(n));
                } else {
                    *text = Text::new("·".to_string());
                    *color = TextColor(Color::srgba(1.0, 1.0, 1.0, 0.45));
                }
                let _ = border_alpha;
            }
        }
    }
}
