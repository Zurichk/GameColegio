//! Interfaces del modo tablero: configuración de partida y tablero de juego.

use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use bevy::input::ButtonState;

use crate::board::questions::{Category, Difficulty, Question};
use crate::board::{
    board_cell_position, cell_kind, pawn_display_position, radio_cell_px, BoardConfig,
    BoardState, CellKind, QuestionTimer, TurnPhase, BOARD_STEP, BOARD_SIZE, PLAYER_COLORS,
    RADIO_LEN,
};
use crate::game::GameState;
use crate::i18n::tr;

/// Tamaño del contenedor del tablero en píxeles (hexágono de radio 9).
const BOARD_W: f32 = 1260.0;
const BOARD_H: f32 = 765.0;
/// Centro del hexágono dentro del contenedor (desplazado a la izquierda
/// para dejar sitio al panel de Estrellitas en el lateral derecho).
const CENTER_X: f32 = 445.0;
const CENTER_Y: f32 = BOARD_H / 2.0;
/// Tamaño de cada casilla de la pista (px). Casi pegan entre sí para que se
/// vea claramente en qué casilla está la ficha.
const CELL: f32 = BOARD_STEP - 4.0;
/// Tamaño de cada casilla de los radios (px).
const RADIO_CELL: f32 = 34.0;

// ---------- Marcadores de la pantalla de configuración ----------

#[derive(Component)]
pub struct SetupUiRoot;

#[derive(Component)]
pub struct PlayerCountText;

#[derive(Component)]
pub struct MinusButton;

#[derive(Component)]
pub struct PlusButton;

#[derive(Component)]
pub struct DifficultyButton(pub Difficulty);

#[derive(Component)]
pub struct StartButton;

#[derive(Component)]
pub struct SetupBackButton;

// ---------- Marcadores de la interfaz del tablero ----------

#[derive(Component)]
pub struct BoardUiRoot;

#[derive(Component)]
pub struct TurnText;

#[derive(Component)]
pub struct DiceText;

#[derive(Component)]
pub struct RollButton;

#[derive(Component)]
pub struct BoardMenuButton;

/// Marcador de las casillas estáticas de la pista.
#[derive(Component)]
pub struct BoardCell;

/// Ficha de un jugador (posicionada de forma absoluta sobre el tablero).
#[derive(Component)]
pub struct Pawn(pub usize);

/// Punto (punta) de la estrella de la ficha: índice 0-5 = Estrellita de la
/// categoría correspondiente, 6 = la 7ª Estrellita (reto final Tabú).
#[derive(Component)]
pub struct StarPoint(pub usize);

/// Panel central (hub) con las Estrellitas de cada jugador.
#[derive(Component)]
pub struct WedgesPanel;

/// Capa modal que cubre la pantalla durante preguntas y resultados.
#[derive(Component)]
pub struct QuestionOverlay;

#[derive(Component)]
pub struct QuestionPanel;

#[derive(Component)]
pub struct OptionButton(pub usize);

#[derive(Component)]
pub struct ContinueButton;

/// Botón de elegir la dirección izquierda en un vértice.
#[derive(Component)]
pub struct LeftButton;

/// Botón de elegir la dirección derecha en un vértice.
#[derive(Component)]
pub struct RightButton;

/// Botón de elegir el radio (Estrellita) por el que salir del centro.
#[derive(Component)]
pub struct SpokeButton(pub usize);

/// Campo de respuesta escrita para preguntas abiertas (sin opciones).
#[derive(Component)]
pub struct AnswerInput {
    /// Texto tecleado.
    pub text: String,
    /// `true` si el campo está enfocado (recibe el teclado).
    pub focused: bool,
}

/// Botón para enviar la respuesta escrita de una pregunta abierta.
#[derive(Component)]
pub struct SubmitAnswerButton;

/// Botón "✓ ¡Acertada!" de una tarjeta Tabú: el equipo adivinó la palabra.
#[derive(Component)]
pub struct TabooGuessedButton;

/// Botón "✗ No acertada" de una tarjeta Tabú: el equipo no la adivinó.
#[derive(Component)]
pub struct TabooMissButton;

/// Texto del temporizador de 1 minuto de cada pregunta.
#[derive(Component)]
pub struct TimerText;

/// Estado interno de la interfaz del tablero.
#[derive(Resource)]
pub struct BoardUiState {
    /// Última revisión de la partida que ya se ha pintado.
    pub last_revision: u64,
}

// ---------- Configuración de partida ----------

/// Inserta la configuración por defecto la primera vez.
pub fn ensure_config(mut commands: Commands, config: Option<Res<BoardConfig>>) {
    if config.is_none() {
        commands.insert_resource(BoardConfig::default());
    }
}

/// Construye la pantalla de configuración de la partida.
pub fn spawn_setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands
        .spawn((
            SetupUiRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(14.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.04, 0.10, 0.85)),
            Visibility::Visible,
        ))
        .with_children(|root| {
            root.spawn(ui_text("MODO TABLERO", 46.0, Color::srgb(1.0, 0.90, 0.55), &font));
            root.spawn(ui_text(
                "Rueda con 6 radios: sal del centro, reúne las 6 Estrellitas\ny vuelve al centro para el reto final Tabú (7ª Estrellita).",
                20.0,
                Color::srgb(0.85, 0.90, 1.0),
                &font,
            ));
            root.spawn(Node {
                height: Val::Px(12.0),
                ..default()
            });

            root.spawn((
                ui_text("Jugadores: 2", 28.0, Color::WHITE, &font),
                PlayerCountText,
            ));
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(14.0),
                    ..default()
                },
            ))
            .with_children(|row| {
                spawn_child_button(row, "-", MinusButton, &font);
                spawn_child_button(row, "+", PlusButton, &font);
            });

            root.spawn(ui_text("Dificultad", 26.0, Color::WHITE, &font));
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(12.0),
                    ..default()
                },
            ))
            .with_children(|row| {
                for difficulty in Difficulty::all() {
                    spawn_child_button(row, difficulty.name(), DifficultyButton(difficulty), &font);
                }
            });

            root.spawn(Node {
                height: Val::Px(14.0),
                ..default()
            });
            spawn_child_button(root, "Comenzar partida", StartButton, &font);
            spawn_child_button(root, "Volver", SetupBackButton, &font);
        });
}

/// Destruye la pantalla de configuración.
pub fn despawn_setup_ui(mut commands: Commands, roots: Query<Entity, With<SetupUiRoot>>) {
    for root in &roots {
        commands.entity(root).despawn();
    }
}

/// Procesa los clics de la pantalla de configuración.
pub fn setup_input(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut config: ResMut<BoardConfig>,
    count_text: Query<Entity, With<PlayerCountText>>,
    minus: Query<&Interaction, (Changed<Interaction>, With<MinusButton>)>,
    plus: Query<&Interaction, (Changed<Interaction>, With<PlusButton>)>,
    difficulties: Query<(&Interaction, &DifficultyButton), Changed<Interaction>>,
    start: Query<&Interaction, (Changed<Interaction>, With<StartButton>)>,
    back: Query<&Interaction, (Changed<Interaction>, With<SetupBackButton>)>,
) {
    for interaction in &minus {
        if *interaction == Interaction::Pressed && config.num_players > 2 {
            config.num_players -= 1;
            update_player_count(&mut commands, &count_text, config.num_players);
        }
    }
    for interaction in &plus {
        if *interaction == Interaction::Pressed && config.num_players < 4 {
            config.num_players += 1;
            update_player_count(&mut commands, &count_text, config.num_players);
        }
    }
    for (interaction, difficulty) in &difficulties {
        if *interaction == Interaction::Pressed {
            config.difficulty = difficulty.0;
        }
    }
    for interaction in &start {
        if *interaction == Interaction::Pressed {
            commands.insert_resource(BoardState::new(*config));
            next_state.set(GameState::BoardGame);
        }
    }
    for interaction in &back {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::MainMenu);
        }
    }
}

/// Mantiene resaltada la dificultad seleccionada.
pub fn refresh_setup_ui(
    config: Res<BoardConfig>,
    mut buttons: Query<(&DifficultyButton, &mut BackgroundColor)>,
) {
    for (difficulty, mut background) in &mut buttons {
        background.0 = if difficulty.0 == config.difficulty {
            Color::srgb(0.20, 0.68, 0.32)
        } else {
            Color::srgb(0.22, 0.27, 0.38)
        };
    }
}

fn update_player_count(
    commands: &mut Commands,
    texts: &Query<Entity, With<PlayerCountText>>,
    count: usize,
) {
    for entity in texts {
        commands
            .entity(entity)
            .insert(Text::new(tr("Jugadores: {count}").replace("{count}", &count.to_string())));
    }
}

// ---------- Tablero de juego ----------

/// Construye la interfaz del tablero (rueda con 6 radios y hub central).
pub fn spawn_board_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    state: Option<Res<BoardState>>,
) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    let num_players = state.as_deref().map(|s| s.config.num_players).unwrap_or(2);
    commands
        .spawn((
            BoardUiRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(10.0)),
                row_gap: Val::Px(6.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.010, 0.015, 0.040)),
            Visibility::Visible,
        ))
        .with_children(|root| {
            // Cabecera: turno actual y controles.
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                },
            ))
            .with_children(|head| {
                head.spawn((
                    ui_text(&tr("Turno: {}").replace("{}", &state.as_deref().unwrap().player_name(0)), 28.0, Color::WHITE, &font),
                    TurnText,
                ));
                head.spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(14.0),
                        ..default()
                    },
                ))
                .with_children(|controls| {
                    spawn_child_button(controls, "Lanzar dado", RollButton, &font);
                    controls.spawn((
                        ui_text("Dado: -", 22.0, Color::srgb(1.0, 0.90, 0.55), &font),
                        DiceText,
                    ));
                    spawn_child_button(controls, "Menú", BoardMenuButton, &font);
                });
            });

            // Tablero: radios (debajo) + pista exterior + hub central. El
            // panel oscuro cubre toda la zona para que no se vea la escena
            // del colegio detrás (ni el personaje).
            root.spawn((
                Node {
                    width: Val::Px(BOARD_W),
                    height: Val::Px(BOARD_H),
                    align_self: AlignSelf::Center,
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.035, 0.050, 0.095)),
                BorderColor(Color::srgb(0.18, 0.22, 0.36)),
                BorderRadius::all(Val::Px(24.0)),
            ))
            .with_children(|board| {
                // ---- 6 radios: 3 casillas de paso entre el vértice y el
                // centro (como los spokes del Trivial clásico). ----
                for vertex in 0..6 {
                    let color = Category::colored()[vertex].color();
                    for step in 0..RADIO_LEN {
                        let (x, y) = radio_cell_px(vertex, step);
                        board
                            .spawn((
                                BoardCell,
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(CENTER_X + x - RADIO_CELL / 2.0),
                                    top: Val::Px(CENTER_Y + y - RADIO_CELL / 2.0),
                                    width: Val::Px(RADIO_CELL),
                                    height: Val::Px(RADIO_CELL),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(
                                    color.to_srgba().red,
                                    color.to_srgba().green,
                                    color.to_srgba().blue,
                                    0.55,
                                )),
                                BorderColor(Color::srgb(0.10, 0.12, 0.20)),
                                BorderRadius::all(Val::Px(8.0)),
                            ))
                            .with_children(|cell_node| {
                                cell_node.spawn(ui_text("•", 18.0, Color::WHITE, &font));
                            });
                    }
                }

                // ---- Pista exterior: 54 casillas (6 Estrellitas en los
                // vértices, categorías, dados y Tabú). ----
                for cell in 0..BOARD_SIZE {
                    let (x, y) = board_cell_position(cell);
                    let (color, content) = match cell_kind(cell) {
                        // Vértice: Estrellita del color de su categoría.
                        CellKind::Wedge(category) => (category.color(), CellContent::Wedge),
                        // Casilla normal: solo el color de su tema.
                        CellKind::Question(category) => (category.color(), CellContent::Plain),
                        // Casilla Tabú: adivinanza sin decir la palabra.
                        CellKind::Taboo => (Category::Taboo.color(), CellContent::Taboo),
                        // Casilla de dado: se vuelve a tirar.
                        CellKind::Dice => (Color::srgb(0.90, 0.92, 0.96), CellContent::Dice),
                    };
                    board
                        .spawn((
                            BoardCell,
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(CENTER_X + x - CELL / 2.0),
                                top: Val::Px(CENTER_Y + y - CELL / 2.0),
                                width: Val::Px(CELL),
                                height: Val::Px(CELL),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(color),
                            BorderColor(Color::srgb(0.06, 0.06, 0.10)),
                            BorderRadius::all(Val::Px(8.0)),
                        ))
                        .with_children(|cell_node| match content {
                            CellContent::Plain => {}
                            CellContent::Wedge => {
                                spawn_star_badge(cell_node);
                            }
                            CellContent::Taboo => {
                                spawn_taboo_icon(cell_node, &font);
                            }
                            CellContent::Dice => {
                                spawn_dice_icon(cell_node, 5);
                            }
                        });
                }

                // ---- Hub central: casilla de salida grande en el centro
                // del hexágono (el panel de Estrellitas está en el lateral). ----
                board
                    .spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(CENTER_X - 38.0),
                            top: Val::Px(CENTER_Y - 38.0),
                            width: Val::Px(76.0),
                            height: Val::Px(76.0),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.10, 0.12, 0.22, 0.95)),
                        BorderColor(Color::srgb(0.95, 0.80, 0.25)),
                        BorderRadius::all(Val::Px(38.0)),
                        ZIndex(4),
                    ))
                    .with_children(|hub| {
                        hub.spawn(ui_text(
                            "SALIDA",
                            15.0,
                            Color::srgb(1.0, 0.90, 0.55),
                            &font,
                        ));
                    });

                // ---- Panel lateral de Estrellitas (a la derecha, sin tapar el
                // tablero): "CENTRO · SALIDA", reto final y Estrellitas. ----
                board
                    .spawn((
                        WedgesPanel,
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(905.0),
                            top: Val::Px(45.0),
                            width: Val::Px(320.0),
                            height: Val::Px(360.0),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::FlexStart,
                            align_items: AlignItems::Center,
                            row_gap: Val::Px(10.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.05, 0.06, 0.14, 0.94)),
                        BorderColor(Color::srgb(0.95, 0.80, 0.25)),
                        BorderRadius::all(Val::Px(14.0)),
                        ZIndex(5),
                    ))
                    .with_children(|center| {
                        center.spawn(ui_text(
                            "CENTRO · SALIDA",
                            20.0,
                            Color::srgb(1.0, 0.90, 0.55),
                            &font,
                        ));
                        center.spawn(ui_text(
                            "7ª Estrellita (TABÚ)",
                            15.0,
                            Category::Taboo.color(),
                            &font,
                        ));
                    });

                // Fichas de los jugadores: estrellas de 7 puntas posicionadas
                // de forma absoluta sobre el tablero y movidas por
                // `update_pawn_positions`. Cada punta se ilumina con el color
                // dla Estrellita conseguida (las 6 de color + la 7ª dorada del
                // reto final).
                for player in 0..num_players {
                    board
                        .spawn((
                            Pawn(player),
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(CENTER_X),
                                top: Val::Px(CENTER_Y),
                                width: Val::Px(42.0),
                                height: Val::Px(42.0),
                                ..default()
                            },
                            ZIndex(30),
                            Visibility::Visible,
                        ))
                        .with_children(|star| {
                            // Círculo central con el número de jugador.
                            star.spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(12.0),
                                    top: Val::Px(12.0),
                                    width: Val::Px(18.0),
                                    height: Val::Px(18.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(PLAYER_COLORS[player]),
                                BorderColor(Color::srgb(0.05, 0.05, 0.08)),
                                BorderRadius::all(Val::Px(9.0)),
                                ZIndex(1),
                            ))
                            .with_children(|center| {
                                center.spawn(ui_text(
                                    &(player + 1).to_string(),
                                    13.0,
                                    Color::WHITE,
                                    &font,
                                ));
                            });
                            // 7 puntas alrededor, una por Estrellita.
                            for point in 0..7 {
                                let theta = point as f32 * (std::f32::consts::TAU / 7.0);
                                let (dx, dy) = (14.0 * theta.cos(), 14.0 * theta.sin());
                                star.spawn((
                                    StarPoint(point),
                                    Node {
                                        position_type: PositionType::Absolute,
                                        left: Val::Px(21.0 + dx - 5.5),
                                        top: Val::Px(21.0 + dy - 5.5),
                                        width: Val::Px(11.0),
                                        height: Val::Px(11.0),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.25, 0.25, 0.28)),
                                    BorderColor(Color::srgb(0.05, 0.05, 0.08)),
                                    BorderRadius::all(Val::Px(3.0)),
                                    Transform::from_rotation(Quat::from_rotation_z(theta)),
                                ));
                            }
                        });
                }
            });

            // Capa modal: dirección, Estrellita de salida, pregunta, dado,
            // reto final, feedback o resultado.
            root.spawn((
                QuestionOverlay,
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.01, 0.02, 0.06, 0.55)),
                Visibility::Hidden,
                ZIndex(100),
            ))
            .with_children(|overlay| {
                overlay.spawn((
                    QuestionPanel,
                    Node {
                        width: Val::Px(680.0),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(10.0),
                        padding: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.04, 0.06, 0.14, 0.97)),
                    BorderColor(Color::srgb(0.55, 0.60, 0.75)),
                    BorderRadius::all(Val::Px(16.0)),
                ));
            });

            // Leyenda: tipos de casilla y colores de las categorías.
            root.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(2.0),
                ..default()
            })
            .with_children(|legend| {
                legend
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        flex_wrap: FlexWrap::Wrap,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(12.0),
                        row_gap: Val::Px(2.0),
                        ..default()
                    })
                    .with_children(|row| {
                        for category in Category::colored() {
                            legend_chip(row, category.name(), category.color(), &font);
                        }
                        legend_chip(row, "Tabú", Category::Taboo.color(), &font);
                        legend_chip(row, "Dado", Color::srgb(0.90, 0.92, 0.96), &font);
                    });
                legend.spawn(ui_text(
                    "",
                    13.0,
                    Color::srgb(0.60, 0.68, 0.88),
                    &font,
                ));
            });
        });

    // Fuerza el primer repintado en el primer frame.
    commands.insert_resource(BoardUiState {
        last_revision: u64::MAX,
    });
}

/// Contenido visual de una casilla de la pista.
enum CellContent {
    /// Casilla de categoría: solo el color (sin números ni símbolos).
    Plain,
    Wedge,
    Taboo,
    Dice,
}

/// Dibuja el icono de un dado blanco con puntos negros (cara 1-6).
fn spawn_dice_icon(parent: &mut ChildSpawnerCommands, face: u8) {
    const SIZE: f32 = 30.0;
    const P: f32 = 7.0; // tamaño del punto
    const M: f32 = 5.0; // margen del punto a la esquina

    let positions: &[(f32, f32)] = match face {
        1 => &[(SIZE / 2.0 - P / 2.0, SIZE / 2.0 - P / 2.0)],
        2 => &[(M, M), (SIZE - M - P, SIZE - M - P)],
        3 => &[
            (M, M),
            (SIZE / 2.0 - P / 2.0, SIZE / 2.0 - P / 2.0),
            (SIZE - M - P, SIZE - M - P),
        ],
        4 => &[
            (M, M),
            (SIZE - M - P, M),
            (M, SIZE - M - P),
            (SIZE - M - P, SIZE - M - P),
        ],
        5 => &[
            (M, M),
            (SIZE - M - P, M),
            (SIZE / 2.0 - P / 2.0, SIZE / 2.0 - P / 2.0),
            (M, SIZE - M - P),
            (SIZE - M - P, SIZE - M - P),
        ],
        _ => &[
            (M, M),
            (SIZE - M - P, M),
            (M, SIZE / 2.0 - P / 2.0),
            (SIZE - M - P, SIZE / 2.0 - P / 2.0),
            (M, SIZE - M - P),
            (SIZE - M - P, SIZE - M - P),
        ],
    };

    parent
        .spawn((
            Node {
                width: Val::Px(SIZE),
                height: Val::Px(SIZE),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::WHITE),
            BorderColor(Color::srgb(0.30, 0.30, 0.35)),
            BorderRadius::all(Val::Px(6.0)),
        ))
        .with_children(|die| {
            for &(px, py) in positions {
                die.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(px),
                        top: Val::Px(py),
                        width: Val::Px(P),
                        height: Val::Px(P),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.10, 0.10, 0.12)),
                    BorderRadius::all(Val::Px(P / 2.0)),
                ));
            }
        });
}

/// Dibuja una estrella blanca de 5 puntas para las casillas Estrellita, en
/// lugar de usar el glifo "★" del texto: así se ve nítida a cualquier
/// tamaño y no depende de la fuente.
fn spawn_star_badge(parent: &mut ChildSpawnerCommands) {
    const SIZE: f32 = 26.0;
    const C: f32 = SIZE / 2.0; // centro del contenedor
    const R: f32 = 7.0; // radio donde se colocan las puntas
    const PT: f32 = 9.0; // tamaño de cada punta

    // Ángulos de las 5 puntas (grados; 0 = derecha, crece hacia abajo).
    const ANGLES: [f32; 5] = [90.0, 162.0, 234.0, 306.0, 18.0];

    parent
        .spawn((
            Node {
                width: Val::Px(SIZE),
                height: Val::Px(SIZE),
                ..default()
            },
            ZIndex(3),
        ))
        .with_children(|star| {
            for &deg in &ANGLES {
                let rad = deg.to_radians();
                let px = C + R * rad.cos();
                let py = C + R * rad.sin();
                star.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(px - PT / 2.0),
                        top: Val::Px(py - PT / 2.0),
                        width: Val::Px(PT),
                        height: Val::Px(PT),
                        ..default()
                    },
                    // Rota la punta para que una esquina mire hacia fuera
                    // (una esquina del cuadrado sin rotar está a 45°).
                    Transform::from_rotation(Quat::from_rotation_z(
                        (deg - 45.0).to_radians(),
                    )),
                    BackgroundColor(Color::WHITE),
                    BorderRadius::all(Val::Px(1.5)),
                ));
            }
            // Centro de la estrella.
            star.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(C - 5.0),
                    top: Val::Px(C - 5.0),
                    width: Val::Px(10.0),
                    height: Val::Px(10.0),
                    ..default()
                },
                BackgroundColor(Color::WHITE),
                BorderRadius::all(Val::Px(5.0)),
            ));
        });
}

/// Dibuja el icono de la casilla Tabú: círculo rojo con signo de
/// prohibido (barra diagonal sobre un "!").
fn spawn_taboo_icon(parent: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    const SIZE: f32 = 30.0;
    parent
        .spawn((
            Node {
                width: Val::Px(SIZE),
                height: Val::Px(SIZE),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.85, 0.20, 0.20)),
            BorderColor(Color::WHITE),
            BorderRadius::all(Val::Px(SIZE / 2.0)),
        ))
        .with_children(|icon| {
            icon.spawn(ui_text("!", 18.0, Color::WHITE, font));
        });
}

/// Destruye la interfaz del tablero.
pub fn despawn_board_ui(mut commands: Commands, roots: Query<Entity, With<BoardUiRoot>>) {
    for root in &roots {
        commands.entity(root).despawn();
    }
}

/// Procesa los clics del tablero y avanza la lógica de la partida.
pub fn board_input(
    mut next_state: ResMut<NextState<GameState>>,
    state: Option<ResMut<BoardState>>,
    roll: Query<&Interaction, (Changed<Interaction>, With<RollButton>)>,
    menu: Query<&Interaction, (Changed<Interaction>, With<BoardMenuButton>)>,
    left: Query<&Interaction, (Changed<Interaction>, With<LeftButton>)>,
    right: Query<&Interaction, (Changed<Interaction>, With<RightButton>)>,
    spokes: Query<(&Interaction, &SpokeButton), Changed<Interaction>>,
    options: Query<(&Interaction, &OptionButton), Changed<Interaction>>,
    continue_button: Query<&Interaction, (Changed<Interaction>, With<ContinueButton>)>,
    taboo_guessed: Query<&Interaction, (Changed<Interaction>, With<TabooGuessedButton>)>,
    taboo_miss: Query<&Interaction, (Changed<Interaction>, With<TabooMissButton>)>,
) {
    let Some(mut state) = state else {
        return;
    };
    for interaction in &roll {
        if *interaction == Interaction::Pressed && state.phase == TurnPhase::Roll {
            state.roll_dice();
        }
    }
    for interaction in &menu {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::MainMenu);
        }
    }
    for interaction in &left {
        if *interaction == Interaction::Pressed
            && matches!(state.phase, TurnPhase::ChooseDirection { .. })
        {
            state.choose_direction(-1);
        }
    }
    for interaction in &right {
        if *interaction == Interaction::Pressed
            && matches!(state.phase, TurnPhase::ChooseDirection { .. })
        {
            state.choose_direction(1);
        }
    }
    for (interaction, spoke) in &spokes {
        if *interaction == Interaction::Pressed && state.phase == TurnPhase::ChooseSpoke {
            state.choose_spoke(spoke.0);
        }
    }
    for (interaction, option) in &options {
        if *interaction == Interaction::Pressed
            && (state.phase == TurnPhase::Question || state.phase == TurnPhase::Final)
        {
            state.answer(option.0);
        }
    }
    for interaction in &taboo_guessed {
        if *interaction == Interaction::Pressed
            && (state.phase == TurnPhase::Taboo || state.phase == TurnPhase::Final)
        {
            state.taboo_result(true);
        }
    }
    for interaction in &taboo_miss {
        if *interaction == Interaction::Pressed
            && (state.phase == TurnPhase::Taboo || state.phase == TurnPhase::Final)
        {
            state.taboo_result(false);
        }
    }
    for interaction in &continue_button {
        if *interaction == Interaction::Pressed
            && (state.phase == TurnPhase::Feedback || state.phase == TurnPhase::ExtraTurn)
        {
            state.continue_turn();
        }
    }
}

/// Entrada de teclado para las preguntas abiertas (sin opciones): se teclea
/// la respuesta en `AnswerInput` y se envía con Enter o el botón "Responder".
///
/// Bevy 0.16 no trae campo de texto listo, así que se construye a mano con
/// los eventos `KeyboardInput` (que traen el texto producido) y se dibuja el
/// cursor "▌" en un `Text` normal.
pub fn typing_input(
    mut keyboard_events: EventReader<KeyboardInput>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: Option<ResMut<BoardState>>,
    mut inputs: Query<(Entity, &mut AnswerInput, &mut Text, &Interaction)>,
    submit_buttons: Query<&Interaction, (With<SubmitAnswerButton>, Changed<Interaction>)>,
) {
    let Some(state) = state.as_deref_mut() else {
        return;
    };
    if !matches!(state.phase, TurnPhase::Question | TurnPhase::Final) {
        return;
    }

    // Recoge el texto producido por las teclas pulsadas este frame (el lector
    // de eventos solo se puede recorrer una vez por sistema y por frame).
    let mut typed = String::new();
    for event in keyboard_events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }
        if let Some(text) = &event.text {
            for ch in text.chars() {
                if !ch.is_control() {
                    typed.push(ch);
                }
            }
        }
    }

    let mut submit = keyboard.just_pressed(KeyCode::Enter);
    for interaction in &submit_buttons {
        if *interaction == Interaction::Pressed {
            submit = true;
        }
    }
    let backspace = keyboard.just_pressed(KeyCode::Backspace);

    for (_, mut input, mut text, interaction) in &mut inputs {
        if *interaction == Interaction::Pressed {
            input.focused = true;
        }
        if !input.focused {
            continue;
        }
        if backspace {
            input.text.pop();
        }
        if !typed.is_empty() {
            input.text.push_str(&typed);
        }
        // El cursor "▌" solo parpadea visiblemente cuando el campo tiene el
        // foco; sin foco se muestra la respuesta tal cual.
        let shown = if input.focused {
            format!("{}▌", input.text)
        } else {
            input.text.clone()
        };
        *text = Text::new(shown);
        if submit && !input.text.trim().is_empty() {
            let answer = std::mem::take(&mut input.text);
            input.focused = false;
            state.answer_text(&answer);
        }
    }
}

/// Temporizador de 1 minuto para responder cada pregunta (pregunta normal o
/// reto final). Cuando llega a 0 cuenta como fallo y pasa el turno. Actualiza
/// el `TimerText` del modal cada frame mientras la pregunta está en curso.
pub fn tick_question_timer(
    time: Res<Time>,
    mut timer: ResMut<QuestionTimer>,
    mut state: Option<ResMut<BoardState>>,
    mut timer_text: Query<&mut Text, With<TimerText>>,
) {
    const QUESTION_SECONDS: f32 = 60.0;

    let Some(state) = state.as_deref_mut() else {
        return;
    };
    let questioning = matches!(
        state.phase,
        TurnPhase::Question | TurnPhase::Taboo | TurnPhase::Final
    );
    if !questioning {
        timer.armed = false;
        timer.remaining = QUESTION_SECONDS;
        return;
    }
    if !timer.armed {
        timer.armed = true;
        timer.remaining = QUESTION_SECONDS;
    }
    timer.remaining -= time.delta_secs();
    let secs = timer.remaining.max(0.0).ceil() as u32;
    for mut text in &mut timer_text {
        *text = Text::new(format!("⏱ {}:{:02}", secs / 60, secs % 60));
    }
    if timer.remaining <= 0.0 {
        timer.armed = false;
        timer.remaining = QUESTION_SECONDS;
        state.timeout_question();
    }
}

/// Repinta las secciones dinámicas cuando el estado de la partida cambia.
pub fn refresh_board_ui(
    mut commands: Commands,
    state: Option<Res<BoardState>>,
    mut ui: ResMut<BoardUiState>,
    asset_server: Res<AssetServer>,
    turn_text: Query<Entity, With<TurnText>>,
    dice_text: Query<Entity, With<DiceText>>,
    wedges: Query<Entity, With<WedgesPanel>>,
    question: Query<Entity, With<QuestionPanel>>,
    mut overlay: Query<&mut Visibility, With<QuestionOverlay>>,
) {
    let Some(state) = state else {
        return;
    };
    if state.revision == ui.last_revision {
        return;
    }
    ui.last_revision = state.revision;

    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    for entity in &turn_text {
        commands.entity(entity).insert(Text::new(
            tr("Turno: {}").replace("{}", &state.player_name(state.current)),
        ));
    }
    let dice_label = match state.dice {
        Some(dice) => tr("Dado: {dice}").replace("{dice}", &dice.to_string()),
        None => tr("Dado: -"),
    };
    for entity in &dice_text {
        commands.entity(entity).insert(Text::new(dice_label.clone()));
    }
    for entity in &wedges {
        refresh_wedges(&mut commands, entity, &state, &font);
    }
    for entity in &question {
        refresh_modal_panel(&mut commands, entity, &state, &font);
    }
    // La capa modal solo se muestra con elección de dirección/Estrellita,
    // pregunta, dado, feedback o resultado (nunca durante las animaciones).
    let modal_visible = matches!(
        state.phase,
        TurnPhase::ChooseSpoke
            | TurnPhase::ChooseDirection { .. }
            | TurnPhase::Question
            | TurnPhase::Taboo
            | TurnPhase::ExtraTurn
            | TurnPhase::Final
            | TurnPhase::Feedback
            | TurnPhase::Won
    );
    for mut visibility in &mut overlay {
        *visibility = if modal_visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

/// Avanza la animación del jugador actual: movimiento por la pista o por un
/// radio (salida o vuelta al centro), una casilla cada ~0,15 s.
pub fn animate_pawns(
    time: Res<Time>,
    mut accumulator: Local<f32>,
    state: Option<ResMut<BoardState>>,
) {
    const STEP_SECONDS: f32 = 0.15;
    let Some(mut state) = state else {
        return;
    };
    let animating = matches!(
        state.phase,
        TurnPhase::Move { .. }
            | TurnPhase::LeaveCenter { .. }
            | TurnPhase::MoveSpoke { .. }
            | TurnPhase::ReturnToCenter { .. }
    );
    if !animating {
        *accumulator = 0.0;
        return;
    }
    *accumulator += time.delta_secs();
    while *accumulator >= STEP_SECONDS {
        *accumulator -= STEP_SECONDS;
        state.advance_animation();
        let still = matches!(
            state.phase,
            TurnPhase::Move { .. }
                | TurnPhase::LeaveCenter { .. }
                | TurnPhase::MoveSpoke { .. }
                | TurnPhase::ReturnToCenter { .. }
        );
        if !still {
            *accumulator = 0.0;
            break;
        }
    }
}

/// Pequeños desplazamientos para que las fichas no se solapen del todo
/// cuando varios jugadores comparten casilla.
const PAWN_OFFSETS: [(f32, f32); 4] = [(0.0, 0.0), (11.0, 0.0), (0.0, 11.0), (11.0, 11.0)];

/// Coloca cada ficha en la posición que le corresponde (pista, radio o
/// centro). Se ejecuta cada frame, por lo que también refleja la animación
/// casilla a casilla que produce `animate_pawns`.
pub fn update_pawn_positions(state: Option<Res<BoardState>>, mut pawns: Query<(&Pawn, &mut Node)>) {
    let Some(state) = state else {
        return;
    };
    for (pawn, mut node) in &mut pawns {
        let (x, y) = pawn_display_position(&state, pawn.0);
        let (offset_x, offset_y) = PAWN_OFFSETS[pawn.0 % 4];
        node.left = Val::Px(CENTER_X + x + offset_x);
        node.top = Val::Px(CENTER_Y + y + offset_y);
    }
}

/// Rellena las puntas de la estrella de cada jugador con el color de los
/// Estrellitas conseguidos: las puntas 0-5 son las 6 categorías (mismo orden
/// que `Category::colored()`) y la punta 6 es la 7ª Estrellita dorada del reto
/// final (se enciende al reunir los 6 de color).
pub fn update_pawn_star(
    state: Option<Res<BoardState>>,
    stars: Query<(Entity, &Pawn, &Children)>,
    mut points: Query<(&StarPoint, &mut BackgroundColor, &ChildOf)>,
) {
    let Some(state) = state else {
        return;
    };
    let empty = Color::srgb(0.25, 0.25, 0.28);
    for (star, pawn, children) in &stars {
        let player = pawn.0;
        let all = state.wedges[player].iter().all(|&w| w);
        for child in children.iter() {
            let Ok((point, mut background, parent)) = points.get_mut(child) else {
                continue;
            };
            if parent.0 != star {
                continue;
            }
            background.0 = if point.0 == 6 {
                if all {
                    Category::Taboo.color()
                } else {
                    empty
                }
            } else if state.wedges[player][point.0] {
                Category::colored()[point.0].color()
            } else {
                empty
            };
        }
    }
}

/// Repinta las Estrellitas de cada jugador en el hub central (7 huecos: las 6
/// de color + el 7º del reto final, que se ilumina en dorado).
fn refresh_wedges(
    commands: &mut Commands,
    root: Entity,
    state: &BoardState,
    font: &Handle<Font>,
) {
    commands
        .entity(root)
        .despawn_related::<Children>()
        .with_children(|wedges| {
            wedges.spawn(ui_text(
                "CENTRO · SALIDA",
                22.0,
                Color::srgb(1.0, 0.90, 0.55),
                font,
            ));
            wedges.spawn(ui_text("7ª Estrellita (TABÚ)", 16.0, Category::Taboo.color(), font));
            for player in 0..state.config.num_players {
                wedges
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(5.0),
                            ..default()
                        },
                    ))
                    .with_children(|row| {
                        row.spawn(ui_text(&state.player_name(player), 17.0, Color::WHITE, font));
                        // Los 6 Estrellitas de color.
                        for (category_index, &owned) in state.wedges[player].iter().enumerate() {
                            let color = if owned {
                                Category::colored()[category_index].color()
                            } else {
                                Color::srgb(0.25, 0.25, 0.28)
                            };
                            row.spawn((
                                Node {
                                    width: Val::Px(18.0),
                                    height: Val::Px(18.0),
                                    ..default()
                                },
                                BackgroundColor(color),
                                BorderColor(Color::srgb(0.05, 0.05, 0.08)),
                                BorderRadius::all(Val::Px(9.0)),
                            ));
                        }
                        // La 7ª Estrellita (reto final Tabú en el centro).
                        let final_ready = state.wedges[player].iter().all(|&owned| owned);
                        row.spawn((
                            Node {
                                width: Val::Px(18.0),
                                height: Val::Px(18.0),
                                ..default()
                            },
                            BackgroundColor(if final_ready {
                                Color::srgb(0.95, 0.80, 0.25)
                            } else {
                                Color::srgb(0.25, 0.25, 0.28)
                            }),
                            BorderColor(Color::srgb(0.05, 0.05, 0.08)),
                            BorderRadius::all(Val::Px(9.0)),
                        ));
                    });
            }
        });
}

/// Repinta el panel modal: encrucijada, pregunta, dado, reto final,
/// feedback o resultado.
fn refresh_modal_panel(
    commands: &mut Commands,
    root: Entity,
    state: &BoardState,
    font: &Handle<Font>,
) {
    commands
        .entity(root)
        .despawn_related::<Children>()
        .with_children(|mut question_panel| {
            match state.phase {
                TurnPhase::ChooseSpoke => {
                    question_panel.spawn(ui_text(
                        "¡Salida del centro!",
                        30.0,
                        Color::srgb(1.0, 0.85, 0.40),
                        font,
                    ));
                    question_panel.spawn(ui_text(
                        &tr("{} está en el centro y elige la Estrellita a la que quiere ir.\nEl dado ({}) cuenta las casillas: la Estrellita está a 4 del centro.")
                            .replace("({})", &state.dice.unwrap_or(0).to_string())
                            .replace("{}", &state.player_name(state.current)),
                        18.0,
                        Color::WHITE,
                        font,
                    ));
                    question_panel
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Row,
                                flex_wrap: FlexWrap::Wrap,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(10.0),
                                row_gap: Val::Px(8.0),
                                ..default()
                            },
                        ))
                        .with_children(|row| {
                            for vertex in 0..6 {
                                let category = Category::colored()[vertex];
                                row.spawn((
                                    Button,
                                    Node {
                                        width: Val::Px(200.0),
                                        height: Val::Px(44.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        border: UiRect::all(Val::Px(2.0)),
                                        ..default()
                                    },
                                    BackgroundColor(category.color()),
                                    BorderColor(Color::srgb(0.85, 0.88, 1.0)),
                                    BorderRadius::all(Val::Px(8.0)),
                                    SpokeButton(vertex),
                                ))
                                .with_children(|button| {
                                    button.spawn(ui_text(category.name(), 16.0, Color::WHITE, font));
                                });
                            }
                        });
                }
                TurnPhase::ChooseDirection { .. } => {
                    question_panel.spawn(ui_text(
                        "¡Elige la dirección!",
                        30.0,
                        Color::srgb(1.0, 0.85, 0.40),
                        font,
                    ));
                    question_panel.spawn(ui_text(
                        &tr("{} lanzó un {}.\nElige hacia dónde mover su ficha (cuenta {} casillas) para reunir las Estrellitas en el orden que quieras:")
                            .replace("{}", &state.player_name(state.current))
                            .replace("{}", &state.dice.unwrap_or(0).to_string()),
                        18.0,
                        Color::WHITE,
                        font,
                    ));
                    question_panel
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(16.0),
                                ..default()
                            },
                        ))
                        .with_children(|row| {
                            spawn_child_button(row, "← Izquierda", LeftButton, font);
                            spawn_child_button(row, "Derecha →", RightButton, font);
                        });
                }
                TurnPhase::Question => {
                    if let Some(question) = state.question {
                        question_panel.spawn(ui_text(
                            &question.category.name(),
                            20.0,
                            question.category.color(),
                            font,
                        ));
                        question_panel.spawn(ui_text(&question.text, 26.0, Color::WHITE, font));
                        if question.open {
                            // Pregunta abierta: como el Trivial real, no
                            // todas las preguntas ofrecen opciones.
                            question_panel.spawn(ui_text(
                                "Escribe tu respuesta:",
                                20.0,
                                Color::srgb(0.70, 0.75, 0.90),
                                font,
                            ));
                            spawn_answer_input(&mut question_panel, font);
                            spawn_child_button(
                                &mut question_panel,
                                "Responder",
                                SubmitAnswerButton,
                                font,
                            );
                        } else {
                            spawn_option_buttons(&mut question_panel, &question, font);
                        }
                        spawn_timer_text(&mut question_panel, font);
                    }
                }
                TurnPhase::ExtraTurn => {
                    question_panel.spawn(ui_text(
                        "¡Casilla de DADO!",
                        34.0,
                        Color::srgb(1.0, 0.85, 0.40),
                        font,
                    ));
                    question_panel.spawn(ui_text(
                        &tr("{} ha caído en un dado y puede volver a tirar.")
                            .replace("{}", &state.player_name(state.current)),
                        22.0,
                        Color::WHITE,
                        font,
                    ));
                    question_panel
                        .spawn((
                            Node {
                                width: Val::Px(60.0),
                                height: Val::Px(60.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                        ))
                        .with_children(|die| {
                            spawn_dice_icon(die, 5);
                        });
                    spawn_child_button(question_panel, "Tirar de nuevo", ContinueButton, font);
                }
                TurnPhase::Taboo => {
                    spawn_taboo_card(
                        &mut question_panel,
                        state,
                        font,
                        &tr("TABÚ"),
                        &tr("{} describe la palabra a su equipo SIN decirla\nni decir ninguna de las prohibidas:").replace("{}", &state.player_name(state.current)),
                    );
                }
                TurnPhase::Final => {
                    spawn_taboo_card(
                        &mut question_panel,
                        state,
                        font,
                        &tr("RETO FINAL · 7ª Estrellita"),
                        &tr("{} ha vuelto al centro y se juega la 7ª Estrellita.\nDescribe la palabra sin decir las prohibidas:").replace("{}", &state.player_name(state.current)),
                    );
                }
                TurnPhase::Feedback => {
                    let correct = state.last_correct.unwrap_or(false);
                    let (label, color) = if state.taboo.is_some() {
                        // Feedback de una tarjeta Tabú.
                        (
                            if correct { tr("¡Acertada!") } else { tr("No acertada") },
                            if correct {
                                Color::srgb(0.30, 0.90, 0.40)
                            } else {
                                Color::srgb(0.95, 0.35, 0.35)
                            },
                        )
                    } else if correct {
                        (tr("¡Correcto!"), Color::srgb(0.30, 0.90, 0.40))
                    } else {
                        (tr("Incorrecto"), Color::srgb(0.95, 0.35, 0.35))
                    };
                    question_panel.spawn(ui_text(&label, 30.0, color, font));
                    if let Some(card) = state.taboo {
                        question_panel.spawn(ui_text(
                            &tr("La palabra era: {}").replace("{}", card.target),
                            22.0,
                            Color::WHITE,
                            font,
                        ));
                    } else if let Some(question) = state.question {
                        // En las abiertas se muestra la respuesta escrita; en
                        // las cerradas, la opción correcta.
                        let expected = if question.open {
                            question.answer
                        } else {
                            question.options[question.correct]
                        };
                        question_panel.spawn(ui_text(
                            &tr("Respuesta correcta: {expected}").replace("{expected}", expected),
                            20.0,
                            Color::WHITE,
                            font,
                        ));
                    }
                    spawn_child_button(question_panel, &tr("Continuar"), ContinueButton, font);
                }
                TurnPhase::Won => {
                    if let Some(winner) = state.winner {
                        question_panel.spawn(ui_text(
                            &tr("¡{} ha ganado la partida!").replace("{}", &state.player_name(winner)),
                            34.0,
                            Color::srgb(1.0, 0.85, 0.40),
                            font,
                        ));
                        question_panel.spawn(ui_text(
                            "Consiguió las 6 Estrellitas de color y superó\nel reto final en el centro (7ª Estrellita Tabú).",
                            20.0,
                            Color::WHITE,
                            font,
                        ));
                    }
                }
                _ => {}
            }
        });
}

/// Crea un texto con la fuente de interfaz.
fn ui_text(
    label: &str,
    size: f32,
    color: Color,
    font: &Handle<Font>,
) -> (Text, TextFont, TextColor) {
    (
        Text::new(tr(label)),
        TextFont {
            font: font.clone(),
            font_size: size,
            ..default()
        },
        TextColor(color),
    )
}

/// Botones con las 4 opciones de una pregunta cerrada.
fn spawn_option_buttons(
    parent: &mut ChildSpawnerCommands,
    question: &Question,
    font: &Handle<Font>,
) {
    for (index, option) in question.options.iter().enumerate() {
        parent
            .spawn((
                Button,
                Node {
                    width: Val::Px(560.0),
                    height: Val::Px(44.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.15, 0.18, 0.28)),
                BorderColor(Color::srgb(0.50, 0.55, 0.70)),
                BorderRadius::all(Val::Px(8.0)),
                OptionButton(index),
            ))
            .with_children(|option_node| {
                option_node.spawn(ui_text(option, 20.0, Color::WHITE, font));
            });
    }
}

/// Tarjeta Tabú real: palabra objetivo en grande, las 5 palabras prohibidas,
/// la pista, el temporizador y los botones de resultado (✓ acertada /
/// ✗ no acertada). La describe un jugador en voz alta a su equipo.
fn spawn_taboo_card(
    parent: &mut ChildSpawnerCommands,
    state: &BoardState,
    font: &Handle<Font>,
    title: &str,
    instruction: &str,
) {
    let Some(card) = state.taboo else {
        return;
    };
    parent.spawn(ui_text(title, 26.0, Category::Taboo.color(), font));
    parent.spawn(ui_text(instruction, 18.0, Color::WHITE, font));

    // Palabra objetivo (el centro de la tarjeta).
    parent
        .spawn((
            Node {
                width: Val::Px(380.0),
                height: Val::Px(64.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.55, 0.35, 0.80)),
            BorderColor(Color::srgb(1.0, 0.90, 0.60)),
            BorderRadius::all(Val::Px(12.0)),
        ))
        .with_children(|target_box| {
            target_box.spawn(ui_text(card.target, 34.0, Color::WHITE, font));
        });

    parent.spawn(ui_text(
        "No puedes decir:",
        18.0,
        Color::srgb(1.0, 0.70, 0.70),
        font,
    ));
    for word in card.forbidden {
        parent.spawn(ui_text(
            &format!("• {word}"),
            22.0,
            Color::srgb(1.0, 0.45, 0.45),
            font,
        ));
    }
    parent.spawn(ui_text(
        &tr("Pista: {}").replace("{}", card.hint),
        18.0,
        Color::srgb(0.70, 0.80, 1.0),
        font,
    ));

    spawn_timer_text(parent, font);

    // Botones de resultado.
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: Val::Px(16.0),
                ..default()
            },
        ))
        .with_children(|row| {
            spawn_child_button(row, &tr("✓ ¡Acertada!"), TabooGuessedButton, font);
            spawn_child_button(row, &tr("✗ No acertada"), TabooMissButton, font);
        });
}

/// Campo de respuesta escrita para las preguntas abiertas. Es un botón
/// enfocable: al hacer clic recibe el teclado (`typing_input`) y se dibuja
/// el cursor "▌" en su `Text`.
fn spawn_answer_input(parent: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    parent.spawn((
        Button,
        Node {
            width: Val::Px(560.0),
            height: Val::Px(46.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.08, 0.10, 0.18)),
        BorderColor(Color::srgb(0.60, 0.65, 0.80)),
        BorderRadius::all(Val::Px(8.0)),
        AnswerInput {
            text: String::new(),
            focused: false,
        },
        ui_text("▌", 22.0, Color::WHITE, font),
    ));
}

/// Texto del temporizador de 1 minuto; lo actualiza `tick_question_timer`
/// cada frame mientras la pregunta está en curso.
fn spawn_timer_text(parent: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    parent.spawn((
        TimerText,
        ui_text("⏱ 1:00", 20.0, Color::srgb(1.0, 0.60, 0.45), font),
    ));
}

/// Chip de la leyenda: un cuadrado del color + el nombre del tipo.
fn legend_chip(parent: &mut ChildSpawnerCommands, name: &str, color: Color, font: &Handle<Font>) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(4.0),
            ..default()
        })
        .with_children(|chip| {
            chip.spawn((
                Node {
                    width: Val::Px(13.0),
                    height: Val::Px(13.0),
                    ..default()
                },
                BackgroundColor(color),
                BorderColor(Color::srgb(0.05, 0.05, 0.08)),
                BorderRadius::all(Val::Px(4.0)),
            ));
            chip.spawn(ui_text(name, 14.0, Color::srgb(0.85, 0.88, 1.0), font));
        });
}

/// Crea un botón con texto centrado dentro de `parent`.
fn spawn_child_button(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    marker: impl Bundle,
    font: &Handle<Font>,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(220.0),
                height: Val::Px(46.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.20, 0.38, 0.66)),
            BorderColor(Color::srgb(0.60, 0.80, 1.0)),
            BorderRadius::all(Val::Px(10.0)),
            // Inherited: en Bevy 0.16 `Visible` se muestra aunque el padre esté
            // oculto (rompería los botones dentro del overlay de preguntas).
            Visibility::Inherited,
            marker,
        ))
        .with_children(|button| {
            button.spawn(ui_text(label, 22.0, Color::WHITE, font));
        });
}