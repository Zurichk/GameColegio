//! Juegos de memoria: emparejar parejas de letras, números o contenido mixto.
//!
//! El menú de Juegos de memoria inserta un recurso `MemoryConfig` con el tipo
//! de contenido y el número de parejas. Al entrar en `MemoryGame` se barajan
//! las tarjetas y se juega por parejas: se voltean dos cartas y, si coinciden,
//! se quedan descubiertas; si no, se vuelven a ocultar tras un pequeño retraso.

use bevy::prelude::*;
use rand::seq::SliceRandom;

use super::{screen_background, spawn_button};
use crate::audio::{play_click, play_success, Sfx};
use crate::game::GameState;
use crate::i18n::tr;

/// Tiempo (s) que se muestran dos cartas que NO coinciden antes de ocultarlas.
const LOCK_SECONDS: f32 = 1.2;

/// Tipo de contenido de las tarjetas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryKind {
    /// Parejas de letras.
    Letters,
    /// Parejas de números.
    Numbers,
    /// Contenido mixto (letras y números).
    Mixed,
    /// Parejas de formas geométricas.
    Shapes,
    /// Parejas de palabras para leer.
    Words,
    /// Parejas de colores (círculos coloreados).
    Colors,
    /// Parejas de banderas (emoji banderas).
    Flags,
}

impl MemoryKind {
    /// Nombre legible del tipo.
    pub fn title(self) -> &'static str {
        match self {
            MemoryKind::Letters => "Parejas de letras",
            MemoryKind::Numbers => "Parejas de números",
            MemoryKind::Mixed => "Parejas mixtas",
            MemoryKind::Shapes => "Parejas de formas",
            MemoryKind::Words => "Parejas de palabras",
            MemoryKind::Colors => "Parejas de colores",
            MemoryKind::Flags => "Parejas de banderas",
        }
    }
}

/// Configuración de la partida de memoria (la inserta el menú).
#[derive(Resource, Clone, Copy)]
pub struct MemoryConfig {
    pub kind: MemoryKind,
    pub pairs: usize,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        MemoryConfig {
            kind: MemoryKind::Mixed,
            pairs: 10,
        }
    }
}

/// Una tarjeta del tablero.
#[derive(Clone)]
struct MemoryCard {
    glyph: String,
    color: Color,
    matched: bool,
}

/// Partida de memoria activa.
#[derive(Resource, Clone)]
pub struct MemorySession {
    kind: MemoryKind,
    cards: Vec<MemoryCard>,
    /// Índice de la primera carta volteada esperando pareja.
    first: Option<usize>,
    /// Parejas encontradas.
    matched: usize,
    /// Movimientos realizados.
    moves: u32,
    /// Segundos restantes de bloqueo tras un fallo (0 = sin bloqueo).
    lock_timer: f32,
    /// Índices de las dos cartas mostradas durante el bloqueo.
    lock_pair: Option<(usize, usize)>,
    /// Segundos transcurridos.
    elapsed: f32,
    /// True cuando todas las parejas están encontradas.
    won: bool,
}

impl MemorySession {
    /// ¿Esta carta debe mostrarse boca arriba?
    fn is_face_up(&self, index: usize) -> bool {
        if self.cards[index].matched {
            return true;
        }
        if self.first == Some(index) {
            return true;
        }
        self.lock_pair.map_or(false, |(a, b)| a == index || b == index)
    }

    /// ¿El juego está bloqueado (mostrando un fallo)?
    fn locked(&self) -> bool {
        self.lock_timer > 0.0
    }
}

// ---- Componentes de la UI --------------------------------------------------

/// Raíz de la pantalla de memoria.
#[derive(Component)]
pub struct MemoryUiRoot;

/// Campo de texto etiquetado por su función.
#[derive(Component)]
pub struct MemoryText(MemoryField);

#[derive(Clone, Copy, PartialEq, Eq)]
enum MemoryField {
    Title,
    Stats,
    Feedback,
    ResultTitle,
    ResultDetail,
}

/// Botón de una tarjeta.
#[derive(Component)]
pub struct MemoryCardButton(pub usize);

/// Texto de una tarjeta (hijo del botón).
#[derive(Component)]
pub struct MemoryCardText(pub usize);

/// Botón de volver al menú de Juegos de memoria.
#[derive(Component)]
pub struct MemoryBackButton;

/// Botón de jugar otra vez.
#[derive(Component)]
pub struct MemoryPlayAgainButton;

/// Contenedor de resultados (oculto hasta ganar).
#[derive(Component)]
pub struct MemoryResultBox;

/// Plugin del juego de memoria.
pub struct MemoryPlugin;

impl Plugin for MemoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MemoryGame), spawn_memory_ui)
            .add_systems(OnExit(GameState::MemoryGame), cleanup_memory)
            .add_systems(
                Update,
                update_memory.run_if(in_state(GameState::MemoryGame)),
            );
    }
}

/// Paleta de colores de las cartas (se asigna a cada pareja).
const CARD_COLORS: [Color; 10] = [
    Color::srgb(0.95, 0.55, 0.35),
    Color::srgb(0.40, 0.75, 0.95),
    Color::srgb(0.55, 0.90, 0.45),
    Color::srgb(0.95, 0.80, 0.30),
    Color::srgb(0.75, 0.55, 0.95),
    Color::srgb(0.95, 0.45, 0.70),
    Color::srgb(0.40, 0.85, 0.80),
    Color::srgb(0.85, 0.70, 0.45),
    Color::srgb(0.60, 0.65, 0.95),
    Color::srgb(0.90, 0.60, 0.55),
];

/// Genera el conjunto de glifos según el tipo de contenido.
fn glyphs_for(kind: MemoryKind) -> Vec<String> {
    match kind {
        MemoryKind::Letters => ('A'..='L').map(|c| c.to_string()).collect(),
        MemoryKind::Numbers => (1..=8).map(|n| n.to_string()).collect(),
        // Mixto: 5 letras + 5 números.
        MemoryKind::Mixed => ('A'..='E')
            .map(|c| c.to_string())
            .chain((1..=5).map(|n| n.to_string()))
            .collect(),
        // Formas geométricas (Roboto incluye estos glifos de WGL4).
        MemoryKind::Shapes => ["●", "▲", "■", "◆", "★", "♥", "♦", "♣"]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        // Palabras sencillas para leer (la cantidad la recorta el menú).
        MemoryKind::Words => words_bank()
            .iter()
            .map(|s| s.to_string())
            .collect(),
        // Colores (círculo coloreado + nombre, el color se asigna por pareja).
        MemoryKind::Colors => ["●", "●", "●", "●", "●", "●", "●", "●", "●", "●"]
            .iter()
            .enumerate()
            .map(|(i, _)| format!("C{}", i + 1))
            .collect(),
        // Banderas (emoji, fallback a texto si la fuente no los tiene).
        MemoryKind::Flags => ["🇪🇸", "🇫🇷", "🇬🇧", "🇩🇪", "🇮🇹", "🇵🇹", "🇺🇸", "🇯🇵", "🇲🇽", "🇦🇷"]
            .iter()
            .map(|s| s.to_string())
            .collect(),
    }
}

/// Palabras de las tarjetas de memoria según el idioma activo.
fn words_bank() -> &'static [&'static str] {
    match crate::i18n::language() {
        crate::i18n::Language::En => &WORDS_EN,
        crate::i18n::Language::Fr => &WORDS_FR,
        crate::i18n::Language::Es => &WORDS_ES,
    }
}

/// Palabras sencillas en español.
const WORDS_ES: [&str; 10] = ["gato", "sol", "mar", "luna", "flor", "pez", "pan", "rosa", "luz", "mano"];

/// Palabras sencillas en inglés.
const WORDS_EN: [&str; 10] = ["cat", "sun", "sea", "moon", "flower", "fish", "bread", "rose", "light", "hand"];

/// Palabras sencillas en francés.
const WORDS_FR: [&str; 10] = ["chat", "soleil", "mer", "lune", "fleur", "poisson", "pain", "rose", "lumière", "main"];

/// Tamaño de letra del glifo de las tarjetas según el tipo de contenido.
fn card_font_size(kind: MemoryKind) -> f32 {
    match kind {
        // Las palabras se leen mejor con letra más pequeña.
        MemoryKind::Words => 30.0,
        _ => 44.0,
    }
}

/// Crea un texto del campo indicado.
fn memory_text(
    parent: &mut ChildSpawnerCommands,
    field: MemoryField,
    text: &str,
    size: f32,
    font: &Handle<Font>,
) {
    parent.spawn((
        MemoryText(field),
        Text::new(text.to_string()),
        TextFont {
            font: font.clone(),
            font_size: size,
            ..default()
        },
        TextColor(Color::WHITE),
        TextLayout {
            linebreak: LineBreak::WordBoundary,
            ..default()
        },
        Node {
            max_width: Val::Px(620.0),
            ..default()
        },
    ));
}

/// Construye el tablero de tarjetas.
fn spawn_card_grid(
    commands: &mut ChildSpawnerCommands,
    session: &MemorySession,
    font: &Handle<Font>,
    glyph_size: f32,
) {
    for index in 0..session.cards.len() {
        let card = &session.cards[index];
        commands
            .spawn((
                Button,
                MemoryCardButton(index),
                Node {
                    width: Val::Px(108.0),
                    height: Val::Px(126.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.12, 0.14, 0.24)),
                BorderColor(Color::srgb(0.45, 0.50, 0.65)),
                BorderRadius::all(Val::Px(10.0)),
            ))
            .with_children(|card_node| {
                card_node.spawn((
                    MemoryCardText(index),
                    Text::new(if session.is_face_up(index) {
                        card.glyph.clone()
                    } else {
                        "?".to_string()
                    }),
                    TextFont {
                        font: font.clone(),
                        font_size: glyph_size,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
    }
}

/// Construye la pantalla del juego de memoria.
fn spawn_memory_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    config: Option<Res<MemoryConfig>>,
) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    let config = config.map(|c| *c).unwrap_or_default();
    let mut session = MemorySession {
        kind: config.kind,
        cards: Vec::new(),
        first: None,
        matched: 0,
        moves: 0,
        lock_timer: 0.0,
        lock_pair: None,
        elapsed: 0.0,
        won: false,
    };
    let mut glyphs = glyphs_for(config.kind);
    glyphs.truncate(config.pairs);
    for (glyph, color) in glyphs.into_iter().zip(CARD_COLORS.iter().copied()) {
        session.cards.push(MemoryCard {
            glyph: glyph.clone(),
            color,
            matched: false,
        });
        session.cards.push(MemoryCard {
            glyph,
            color,
            matched: false,
        });
    }
    session.cards.shuffle(&mut rand::thread_rng());
    commands.insert_resource(session.clone());

    commands
        .spawn((
            MemoryUiRoot,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            screen_background(),
            Visibility::Visible,
            ZIndex(30),
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        width: Val::Px(680.0),
                        padding: UiRect::axes(Val::Px(24.0), Val::Px(20.0)),
                        row_gap: Val::Px(12.0),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.07, 0.09, 0.18, 0.96)),
                    BorderRadius::all(Val::Px(16.0)),
                ))
                .with_children(|panel| {
                    memory_text(panel, MemoryField::Title, config.kind.title(), 26.0, &font);
                    memory_text(panel, MemoryField::Stats, "", 18.0, &font);
                    memory_text(panel, MemoryField::Feedback, "", 20.0, &font);

                    // Tablero de tarjetas.
                    panel
                        .spawn((
                            Node {
                                width: Val::Px(620.0),
                                flex_wrap: FlexWrap::Wrap,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(10.0),
                                row_gap: Val::Px(10.0),
                                ..default()
                            },
                        ))
                        .with_children(|grid| {
                            spawn_card_grid(grid, &session, &font, card_font_size(session.kind));
                        });

                    // Resultados (ocultos hasta ganar).
                    panel
                        .spawn((
                            MemoryResultBox,
                            Node {
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                row_gap: Val::Px(10.0),
                                ..default()
                            },
                            Visibility::Hidden,
                        ))
                        .with_children(|results| {
                            memory_text(results, MemoryField::ResultTitle, "", 26.0, &font);
                            memory_text(results, MemoryField::ResultDetail, "", 20.0, &font);
                            spawn_button(
                                results,
                                "Jugar otra vez",
                                MemoryPlayAgainButton,
                                &font,
                            );
                            spawn_button(results, "Volver a Juegos de memoria", MemoryBackButton, &font);
                        });
                });
        });
}

/// Destruye la pantalla y la sesión al salir.
fn cleanup_memory(mut commands: Commands, roots: Query<Entity, With<MemoryUiRoot>>) {
    for root in &roots {
        commands.entity(root).despawn();
    }
    commands.remove_resource::<MemorySession>();
}

/// Colores de las tarjetas según su estado.
fn card_background(session: &MemorySession, index: usize) -> Color {
    if session.cards[index].matched {
        Color::srgb(0.14, 0.42, 0.22)
    } else if session.is_face_up(index) {
        session.cards[index].color
    } else {
        Color::srgb(0.12, 0.14, 0.24)
    }
}

/// Gestiona la partida: voltear, emparejar, temporizador y victoria.
fn update_memory(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut commands: Commands,
    sfx: Res<Sfx>,
    mut session: ResMut<MemorySession>,
    mut texts: Query<(&MemoryText, &mut Text, &mut TextColor, &mut Visibility)>,
    mut card_texts: Query<
        (&MemoryCardText, &mut Text),
        (
            Without<MemoryText>,
            Without<MemoryCardButton>,
            Without<MemoryResultBox>,
        ),
    >,
    mut card_colors: Query<
        (&MemoryCardButton, &mut BackgroundColor),
        (Without<MemoryText>, Without<MemoryResultBox>),
    >,
    card_clicks: Query<
        (&Interaction, &MemoryCardButton),
        (Changed<Interaction>, Without<MemoryText>, Without<MemoryResultBox>),
    >,
    mut result_box: Query<
        &mut Visibility,
        (With<MemoryResultBox>, Without<MemoryText>, Without<MemoryCardButton>),
    >,
    back_clicks: Query<&Interaction, (Changed<Interaction>, With<MemoryBackButton>)>,
    again_clicks: Query<&Interaction, (Changed<Interaction>, With<MemoryPlayAgainButton>)>,
    config: Option<Res<MemoryConfig>>,
) {
    // Escape: volver al menú de Juegos de memoria.
    if keys.just_pressed(KeyCode::Escape) {
        commands.set_state(GameState::MemoryMenu);
        return;
    }

    let dt = time.delta_secs();
    if !session.won {
        session.elapsed += dt;
    }

    // Reiniciar partida: reconstruir las cartas y resetear la sesión.
    if session.won
        && again_clicks
            .single()
            .map_or(false, |i| *i == Interaction::Pressed)
    {
        let config = config.map(|c| *c).unwrap_or_default();
        let pairs = config.pairs.clamp(2, 10);
        let mut glyphs = glyphs_for(config.kind);
        glyphs.truncate(pairs);
        session.cards = glyphs
            .into_iter()
            .zip(CARD_COLORS.iter().copied())
            .flat_map(|(glyph, color)| {
                [
                    MemoryCard {
                        glyph: glyph.clone(),
                        color,
                        matched: false,
                    },
                    MemoryCard {
                        glyph,
                        color,
                        matched: false,
                    },
                ]
            })
            .collect();
        session.cards.shuffle(&mut rand::thread_rng());
        session.first = None;
        session.matched = 0;
        session.moves = 0;
        session.lock_timer = 0.0;
        session.lock_pair = None;
        session.elapsed = 0.0;
        session.won = false;
        if let Ok(mut vis) = result_box.single_mut() {
            *vis = Visibility::Hidden;
        }
        return;
    }

    // 1) Resultados.
    if session.won {
        let back = back_clicks.single().map_or(false, |i| *i == Interaction::Pressed)
            || keys.just_pressed(KeyCode::Enter)
            || keys.just_pressed(KeyCode::KeyQ);
        if back {
            commands.set_state(GameState::MemoryMenu);
            return;
        }
        for (field, mut text, mut color, mut vis) in &mut texts {
            match field.0 {
                MemoryField::ResultTitle => {
                    *text = Text::new(tr("¡Has ganado!"));
                    *color = TextColor(Color::srgb(0.40, 0.90, 0.50));
                    *vis = Visibility::Visible;
                }
                MemoryField::ResultDetail => {
                    *text = Text::new(tr("Parejas: {} · Movimientos: {} · Tiempo: {} s").replace("{}", &session.matched.to_string()).replace("{}", &session.moves.to_string()).replace("{}", &format!("{:.0}", session.elapsed)));
                    *vis = Visibility::Visible;
                }
                _ => {}
            }
        }
        if let Ok(mut vis) = result_box.single_mut() {
            *vis = Visibility::Visible;
        }
        return;
    }

    // 2) Bloqueo tras un fallo.
    if session.locked() {
        session.lock_timer -= dt;
        for (field, mut text, mut color, mut vis) in &mut texts {
            if field.0 == MemoryField::Feedback {
                *text = Text::new(tr("No coinciden"));
                *color = TextColor(Color::srgb(0.95, 0.40, 0.40));
                *vis = Visibility::Visible;
            }
        }
        if session.lock_timer <= 0.0 {
            session.lock_timer = 0.0;
            session.lock_pair = None;
        }
    } else {
        // 3) Responder: voltear una carta.
        let mut chosen: Option<usize> = None;
        for (interaction, card) in &card_clicks {
            if *interaction == Interaction::Pressed {
                chosen = Some(card.0);
                break;
            }
        }
        if let Some(index) = chosen {
            let card = &session.cards[index];
            let already_flipped = session.first == Some(index)
                || session.lock_pair.map_or(false, |(a, b)| a == index || b == index);
            if !card.matched && !already_flipped {
                play_click(&mut commands, &sfx);
                match session.first {
                    None => {
                        session.first = Some(index);
                    }
                    Some(first) => {
                        session.moves += 1;
                        if session.cards[first].glyph == session.cards[index].glyph {
                            // ¡Pareja!
                            session.cards[first].matched = true;
                            session.cards[index].matched = true;
                            session.matched += 1;
                            session.first = None;
                            if session.matched == session.cards.len() / 2 {
                                session.won = true;
                                play_success(&mut commands, &sfx);
                            }
                        } else {
                            // Fallo: mostrar ambas durante un momento.
                            session.lock_timer = LOCK_SECONDS;
                            session.lock_pair = Some((first, index));
                            session.first = None;
                        }
                    }
                }
            }
        }
        for (field, _text, _color, mut vis) in &mut texts {
            if field.0 == MemoryField::Feedback {
                *vis = Visibility::Hidden;
            }
        }
    }

    // 4) Actualizar textos.
    for (field, mut text, _color, _vis) in &mut texts {
        match field.0 {
            MemoryField::Title => {
                *text = Text::new(tr(session.kind.title()));
            }
            MemoryField::Stats => {
                *text = Text::new(tr("Parejas: {}/{} · Movimientos: {} · Tiempo: {} s").replace("{}", &session.matched.to_string()).replace("{}", &(session.cards.len() / 2).to_string()).replace("{}", &session.moves.to_string()).replace("{}", &format!("{:.0}", session.elapsed)));
            }
            _ => {}
        }
    }
    for (card, mut text) in &mut card_texts {
        *text = Text::new(if session.is_face_up(card.0) {
            session.cards[card.0].glyph.clone()
        } else {
            "?".to_string()
        });
    }
    for (card, mut bg) in &mut card_colors {
        *bg = BackgroundColor(card_background(&session, card.0));
    }
}