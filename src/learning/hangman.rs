//! Juego del ahorcado (sección Lengua).
//!
//! Hay que adivinar una palabra letra a letra antes de que se complete el
//! muñeco (6 fallos). Se juega con el teclado físico (A-Z) o con el teclado
//! en pantalla (A-Z, Ñ y vocales con tilde). Al ganar o perder se puede pedir
//! una palabra nueva o volver al menú de Lengua.

use bevy::prelude::*;
use rand::seq::SliceRandom;

use super::{screen_background, spawn_button, ui_text};
use crate::audio::{play_click, play_success, Sfx};
use crate::game::GameState;
use crate::i18n::tr;

/// Fallos máximos antes de perder.
const MAX_WRONG: usize = 6;

/// Letras del teclado en pantalla (incluye Ñ y vocales con tilde).
const ALPHABET: [char; 32] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'Ñ', 'O', 'P', 'Q', 'R',
    'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'Á', 'É', 'Í', 'Ó', 'Ú',
];

/// Etapas del muñeco del ahorcado (0 = sin fallos … 6 = ahorcado).
const HANGMAN_STAGES: [&str; 7] = [
    "  +---+\n  |   |\n      |\n      |\n      |\n=======",
    "  +---+\n  |   |\n  O   |\n      |\n      |\n=======",
    "  +---+\n  |   |\n  O   |\n  |   |\n      |\n=======",
    "  +---+\n  |   |\n  O   |\n /|   |\n      |\n=======",
    "  +---+\n  |   |\n  O   |\n /|\\ |\n      |\n=======",
    "  +---+\n  |   |\n  O   |\n /|\\ |\n /    |\n=======",
    "  +---+\n  |   |\n  O   |\n /|\\ |\n / \\ |\n=======",
];

/// Una palabra del banco con la pista de su categoría.
struct HangmanWord {
    word: &'static str,
    category: &'static str,
}

/// Sesión de ahorcado activa.
#[derive(Resource)]
pub struct HangmanSession {
    word: String,
    category: &'static str,
    revealed: Vec<bool>,
    guessed: Vec<char>,
    wrong: usize,
    won: bool,
    lost: bool,
}

impl HangmanSession {
    /// Crea una sesión nueva con una palabra al azar.
    fn new() -> Self {
        let mut rng = rand::thread_rng();
        let pick = words_bank().choose(&mut rng).expect("banco no vacío");
        HangmanSession {
            word: pick.word.to_string(),
            category: pick.category,
            revealed: vec![false; pick.word.chars().count()],
            guessed: Vec::new(),
            wrong: 0,
            won: false,
            lost: false,
        }
    }

    /// Intenta una letra: actualiza el progreso y los fallos.
    fn guess(&mut self, letter: char) {
        if self.won || self.lost {
            return;
        }
        let upper = letter.to_uppercase().next().unwrap_or(letter);
        if self.guessed.contains(&upper) {
            return;
        }
        self.guessed.push(upper);
        let mut found = false;
        for (ch, revealed) in self.word.chars().zip(self.revealed.iter_mut()) {
            if ch.to_uppercase().next() == Some(upper) {
                *revealed = true;
                found = true;
            }
        }
        if found {
            if self.revealed.iter().all(|r| *r) {
                self.won = true;
            }
        } else {
            self.wrong += 1;
            if self.wrong >= MAX_WRONG {
                self.lost = true;
            }
        }
    }
}

// ---- Componentes de la UI --------------------------------------------------

/// Raíz de la pantalla del ahorcado.
#[derive(Component)]
pub struct HangmanUiRoot;

/// Campo de texto etiquetado por su función.
#[derive(Component)]
pub struct HangmanText(HangmanField);

#[derive(Clone, Copy, PartialEq, Eq)]
enum HangmanField {
    Title,
    Category,
    Drawing,
    Word,
    WrongLetters,
    Message,
}

/// Botón de una letra del teclado en pantalla.
#[derive(Component)]
pub struct HangmanLetterButton(pub char);

/// Botón de pedir una palabra nueva.
#[derive(Component)]
pub struct HangmanNewButton;

/// Botón de volver al menú de Lengua.
#[derive(Component)]
pub struct HangmanBackButton;

/// Plugin del juego del ahorcado.
pub struct HangmanPlugin;

impl Plugin for HangmanPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::HangmanGame), spawn_hangman_ui)
            .add_systems(OnExit(GameState::HangmanGame), cleanup_hangman)
            .add_systems(
                Update,
                update_hangman.run_if(in_state(GameState::HangmanGame)),
            );
    }
}

/// Crea un texto del campo indicado.
fn hangman_text(
    parent: &mut ChildSpawnerCommands,
    field: HangmanField,
    text: &str,
    size: f32,
    color: Color,
    font: &Handle<Font>,
) {
    parent.spawn((
        HangmanText(field),
        Text::new(text.to_string()),
        TextFont {
            font: font.clone(),
            font_size: size,
            ..default()
        },
        TextColor(color),
        TextLayout {
            linebreak: LineBreak::WordBoundary,
            ..default()
        },
    ));
}

/// Crea el botón de una letra del teclado en pantalla.
fn letter_button(
    parent: &mut ChildSpawnerCommands,
    letter: char,
    font: &Handle<Font>,
) {
    parent
        .spawn((
            Button,
            HangmanLetterButton(letter),
            Node {
                width: Val::Px(42.0),
                height: Val::Px(42.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.18, 0.22, 0.34)),
            BorderColor(Color::srgb(0.50, 0.55, 0.70)),
            BorderRadius::all(Val::Px(6.0)),
            // Inherited: en Bevy 0.16 `Visible` se muestra aunque el padre
            // esté oculto.
            Visibility::Inherited,
        ))
        .with_children(|button| {
            button.spawn(ui_text(&letter.to_string(), 18.0, Color::WHITE, font));
        });
}

/// Construye la pantalla del ahorcado.
fn spawn_hangman_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands.insert_resource(HangmanSession::new());
    commands
        .spawn((
            HangmanUiRoot,
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
                        width: Val::Px(760.0),
                        padding: UiRect::axes(Val::Px(28.0), Val::Px(20.0)),
                        row_gap: Val::Px(10.0),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.07, 0.09, 0.18, 0.96)),
                    BorderRadius::all(Val::Px(16.0)),
                ))
                .with_children(|panel| {
                    hangman_text(panel, HangmanField::Title, "AHORCADO", 28.0, Color::srgb(0.95, 0.75, 0.90), &font);
                    hangman_text(panel, HangmanField::Category, "", 18.0, Color::srgb(0.85, 0.90, 1.0), &font);

                    panel
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(28.0),
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with_children(|row| {
                            hangman_text(row, HangmanField::Drawing, "", 18.0, Color::srgb(0.70, 0.85, 1.0), &font);
                            hangman_text(row, HangmanField::Word, "", 44.0, Color::WHITE, &font);
                        });

                    hangman_text(panel, HangmanField::WrongLetters, "", 18.0, Color::srgb(1.0, 0.60, 0.60), &font);
                    hangman_text(panel, HangmanField::Message, "", 22.0, Color::srgb(0.40, 0.90, 0.50), &font);

                    // Teclado en pantalla.
                    panel
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            flex_wrap: FlexWrap::Wrap,
                            width: Val::Px(760.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(6.0),
                            row_gap: Val::Px(6.0),
                            ..default()
                        })
                        .with_children(|keyboard| {
                            for letter in ALPHABET {
                                letter_button(keyboard, letter, &font);
                            }
                        });

                    panel
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(16.0),
                            ..default()
                        })
                        .with_children(|row| {
                            spawn_button(row, "Nueva palabra", HangmanNewButton, &font);
                            spawn_button(row, "Volver a Lengua", HangmanBackButton, &font);
                        });
                });
        });
}

/// Destruye la pantalla y la sesión al salir.
fn cleanup_hangman(mut commands: Commands, roots: Query<Entity, With<HangmanUiRoot>>) {
    for root in &roots {
        commands.entity(root).despawn();
    }
    commands.remove_resource::<HangmanSession>();
}

/// Muestra la palabra con las letras adivinadas.
fn word_display(session: &HangmanSession) -> String {
    let mut out = String::new();
    for (ch, revealed) in session.word.chars().zip(session.revealed.iter()) {
        if *revealed {
            out.push(ch.to_uppercase().next().unwrap_or(ch));
        } else {
            out.push('_');
        }
        out.push(' ');
    }
    out.trim_end().to_string()
}

/// Gestiona la sesión del ahorcado.
fn update_hangman(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    sfx: Res<Sfx>,
    session: Option<ResMut<HangmanSession>>,
    mut texts: Query<
        (&HangmanText, &mut Text),
        (Without<HangmanLetterButton>, Without<HangmanNewButton>, Without<HangmanBackButton>),
    >,
    mut letter_colors: Query<
        (&HangmanLetterButton, &mut BackgroundColor),
        Without<HangmanText>,
    >,
    letter_clicks: Query<
        (&Interaction, &HangmanLetterButton),
        (Changed<Interaction>, Without<HangmanText>),
    >,
    new_clicks: Query<&Interaction, (Changed<Interaction>, With<HangmanNewButton>)>,
    close_clicks: Query<&Interaction, (Changed<Interaction>, With<HangmanBackButton>)>,
) {
    let mut session = match session {
        Some(session) => session,
        None => {
            commands.insert_resource(HangmanSession::new());
            return;
        }
    };

    // Escape: volver al menú de Lengua.
    if keys.just_pressed(KeyCode::Escape) {
        commands.set_state(GameState::LanguageMenu);
        return;
    }

    // Botón "Volver a Lengua".
    if close_clicks.single().map_or(false, |i| *i == Interaction::Pressed) {
        commands.set_state(GameState::LanguageMenu);
        return;
    }

    // "Nueva palabra": reinicia la sesión (incluso a mitad de partida).
    if new_clicks.single().map_or(false, |i| *i == Interaction::Pressed) {
        *session = HangmanSession::new();
        play_click(&mut commands, &sfx);
    }

    // Entrada de letras: teclado físico (A-Z) o clic en el teclado en pantalla.
    let mut letter: Option<char> = None;
    for (interaction, button) in &letter_clicks {
        if *interaction == Interaction::Pressed {
            letter = Some(button.0);
            break;
        }
    }
    if letter.is_none() {
        for code in [
            KeyCode::KeyA, KeyCode::KeyB, KeyCode::KeyC, KeyCode::KeyD,
            KeyCode::KeyE, KeyCode::KeyF, KeyCode::KeyG, KeyCode::KeyH,
            KeyCode::KeyI, KeyCode::KeyJ, KeyCode::KeyK, KeyCode::KeyL,
            KeyCode::KeyM, KeyCode::KeyN, KeyCode::KeyO, KeyCode::KeyP,
            KeyCode::KeyQ, KeyCode::KeyR, KeyCode::KeyS, KeyCode::KeyT,
            KeyCode::KeyU, KeyCode::KeyV, KeyCode::KeyW, KeyCode::KeyX,
            KeyCode::KeyY, KeyCode::KeyZ,
        ]
        .iter()
        {
            if keys.just_pressed(*code) {
                if let Some(name) = key_name(*code) {
                    letter = Some(name);
                }
                break;
            }
        }
    }
    let before_wrong = session.wrong;
    let before_won = session.won;
    if let Some(letter) = letter {
        session.guess(letter);
        if session.won && !before_won {
            play_success(&mut commands, &sfx);
        } else if session.wrong > before_wrong {
            play_click(&mut commands, &sfx);
        }
    }

    // 1) Textos.
    for (field, mut text) in &mut texts {
        match field.0 {
            HangmanField::Category => {
                *text = Text::new(tr("Pista: {}").replace("{}", session.category));
            }
            HangmanField::Drawing => {
                *text = Text::new(HANGMAN_STAGES[session.wrong].to_string());
            }
            HangmanField::Word => {
                *text = Text::new(word_display(&session));
            }
            HangmanField::WrongLetters => {
                let upper_word: String = session.word.to_uppercase();
                let wrong: String = session
                    .guessed
                    .iter()
                    .filter(|c| !upper_word.contains(**c))
                    .map(|c| *c)
                    .collect();
                *text = Text::new(if wrong.is_empty() {
                    tr("Fallos: -")
                } else {
                    tr("Fallos: {}").replace("{}", &wrong)
                });
            }
            HangmanField::Message => {
                *text = Text::new(if session.won { tr("¡Has ganado!") } else if session.lost { tr("La palabra era: {}").replace("{}", &session.word.to_uppercase()) } else { String::new() });
            }
            _ => {}
        }
    }

    // 2) Colores de las letras del teclado en pantalla.
    for (button, mut bg) in &mut letter_colors {
        let used = session.guessed.contains(&button.0);
        *bg = BackgroundColor(if used {
            Color::srgb(0.10, 0.12, 0.20)
        } else {
            Color::srgb(0.18, 0.22, 0.34)
        });
    }
}

/// Devuelve la letra mayúscula correspondiente a una tecla física A-Z.
fn key_name(code: KeyCode) -> Option<char> {
    let letter = match code {
        KeyCode::KeyA => 'A',
        KeyCode::KeyB => 'B',
        KeyCode::KeyC => 'C',
        KeyCode::KeyD => 'D',
        KeyCode::KeyE => 'E',
        KeyCode::KeyF => 'F',
        KeyCode::KeyG => 'G',
        KeyCode::KeyH => 'H',
        KeyCode::KeyI => 'I',
        KeyCode::KeyJ => 'J',
        KeyCode::KeyK => 'K',
        KeyCode::KeyL => 'L',
        KeyCode::KeyM => 'M',
        KeyCode::KeyN => 'N',
        KeyCode::KeyO => 'O',
        KeyCode::KeyP => 'P',
        KeyCode::KeyQ => 'Q',
        KeyCode::KeyR => 'R',
        KeyCode::KeyS => 'S',
        KeyCode::KeyT => 'T',
        KeyCode::KeyU => 'U',
        KeyCode::KeyV => 'V',
        KeyCode::KeyW => 'W',
        KeyCode::KeyX => 'X',
        KeyCode::KeyY => 'Y',
        KeyCode::KeyZ => 'Z',
        _ => return None,
    };
    Some(letter)
}

// ---- Banco de palabras -----------------------------------------------------

/// Palabras del ahorcado — 72 palabras.
const WORDS: [HangmanWord; 72] = [
    // Animales
    HangmanWord { word: "perro", category: "Animal" },
    HangmanWord { word: "gato", category: "Animal" },
    HangmanWord { word: "león", category: "Animal" },
    HangmanWord { word: "tigre", category: "Animal" },
    HangmanWord { word: "elefante", category: "Animal" },
    HangmanWord { word: "jirafa", category: "Animal" },
    HangmanWord { word: "mariposa", category: "Animal" },
    HangmanWord { word: "delfín", category: "Animal" },
    HangmanWord { word: "tortuga", category: "Animal" },
    HangmanWord { word: "loro", category: "Animal" },
    HangmanWord { word: "cebra", category: "Animal" },
    HangmanWord { word: "caballo", category: "Animal" },
    // Frutas
    HangmanWord { word: "manzana", category: "Fruta" },
    HangmanWord { word: "plátano", category: "Fruta" },
    HangmanWord { word: "naranja", category: "Fruta" },
    HangmanWord { word: "fresa", category: "Fruta" },
    HangmanWord { word: "uva", category: "Fruta" },
    HangmanWord { word: "pera", category: "Fruta" },
    HangmanWord { word: "sandía", category: "Fruta" },
    HangmanWord { word: "melón", category: "Fruta" },
    HangmanWord { word: "limón", category: "Fruta" },
    HangmanWord { word: "kiwi", category: "Fruta" },
    HangmanWord { word: "cereza", category: "Fruta" },
    HangmanWord { word: "ciruela", category: "Fruta" },
    // Colegio
    HangmanWord { word: "lápiz", category: "Colegio" },
    HangmanWord { word: "cuaderno", category: "Colegio" },
    HangmanWord { word: "pizarra", category: "Colegio" },
    HangmanWord { word: "profesor", category: "Colegio" },
    HangmanWord { word: "mochila", category: "Colegio" },
    HangmanWord { word: "recreo", category: "Colegio" },
    HangmanWord { word: "biblioteca", category: "Colegio" },
    HangmanWord { word: "pupitre", category: "Colegio" },
    HangmanWord { word: "examen", category: "Colegio" },
    HangmanWord { word: "patio", category: "Colegio" },
    HangmanWord { word: "maestra", category: "Colegio" },
    HangmanWord { word: "libro", category: "Colegio" },
    // Naturaleza
    HangmanWord { word: "montaña", category: "Naturaleza" },
    HangmanWord { word: "río", category: "Naturaleza" },
    HangmanWord { word: "bosque", category: "Naturaleza" },
    HangmanWord { word: "mar", category: "Naturaleza" },
    HangmanWord { word: "desierto", category: "Naturaleza" },
    HangmanWord { word: "volcán", category: "Naturaleza" },
    HangmanWord { word: "nube", category: "Naturaleza" },
    HangmanWord { word: "arcoíris", category: "Naturaleza" },
    HangmanWord { word: "estrella", category: "Naturaleza" },
    HangmanWord { word: "luna", category: "Naturaleza" },
    HangmanWord { word: "viento", category: "Naturaleza" },
    HangmanWord { word: "lluvia", category: "Naturaleza" },
    // Transporte
    HangmanWord { word: "avión", category: "Transporte" },
    HangmanWord { word: "tren", category: "Transporte" },
    HangmanWord { word: "barco", category: "Transporte" },
    HangmanWord { word: "bicicleta", category: "Transporte" },
    HangmanWord { word: "autobús", category: "Transporte" },
    HangmanWord { word: "camión", category: "Transporte" },
    HangmanWord { word: "helicóptero", category: "Transporte" },
    HangmanWord { word: "coche", category: "Transporte" },
    // Casa
    HangmanWord { word: "ventana", category: "Casa" },
    HangmanWord { word: "puerta", category: "Casa" },
    HangmanWord { word: "cocina", category: "Casa" },
    HangmanWord { word: "espejo", category: "Casa" },
    HangmanWord { word: "lámpara", category: "Casa" },
    HangmanWord { word: "sofá", category: "Casa" },
    HangmanWord { word: "cama", category: "Casa" },
    HangmanWord { word: "silla", category: "Casa" },
    // Ciudad
    HangmanWord { word: "puente", category: "Ciudad" },
    HangmanWord { word: "parque", category: "Ciudad" },
    HangmanWord { word: "hospital", category: "Ciudad" },
    HangmanWord { word: "escuela", category: "Ciudad" },
    HangmanWord { word: "mercado", category: "Ciudad" },
    HangmanWord { word: "plaza", category: "Ciudad" },
    HangmanWord { word: "calle", category: "Ciudad" },
    HangmanWord { word: "semáforo", category: "Ciudad" },
];

/// Banco de palabras del ahorcado según el idioma activo.
fn words_bank() -> &'static [HangmanWord] {
    match crate::i18n::language() {
        crate::i18n::Language::En => &WORDS_EN,
        crate::i18n::Language::Fr => &WORDS_FR,
        crate::i18n::Language::Es => &WORDS,
    }
}

/// Palabras del ahorcado en inglés — 72 palabras.
const WORDS_EN: [HangmanWord; 72] = [
    // Animals
    HangmanWord { word: "dog", category: "Animal" },
    HangmanWord { word: "cat", category: "Animal" },
    HangmanWord { word: "lion", category: "Animal" },
    HangmanWord { word: "tiger", category: "Animal" },
    HangmanWord { word: "elephant", category: "Animal" },
    HangmanWord { word: "giraffe", category: "Animal" },
    HangmanWord { word: "butterfly", category: "Animal" },
    HangmanWord { word: "dolphin", category: "Animal" },
    HangmanWord { word: "turtle", category: "Animal" },
    HangmanWord { word: "parrot", category: "Animal" },
    HangmanWord { word: "zebra", category: "Animal" },
    HangmanWord { word: "horse", category: "Animal" },
    // Fruit
    HangmanWord { word: "apple", category: "Fruit" },
    HangmanWord { word: "banana", category: "Fruit" },
    HangmanWord { word: "orange", category: "Fruit" },
    HangmanWord { word: "strawberry", category: "Fruit" },
    HangmanWord { word: "grape", category: "Fruit" },
    HangmanWord { word: "pear", category: "Fruit" },
    HangmanWord { word: "watermelon", category: "Fruit" },
    HangmanWord { word: "melon", category: "Fruit" },
    HangmanWord { word: "lemon", category: "Fruit" },
    HangmanWord { word: "kiwi", category: "Fruit" },
    HangmanWord { word: "cherry", category: "Fruit" },
    HangmanWord { word: "plum", category: "Fruit" },
    // School
    HangmanWord { word: "pencil", category: "School" },
    HangmanWord { word: "notebook", category: "School" },
    HangmanWord { word: "blackboard", category: "School" },
    HangmanWord { word: "teacher", category: "School" },
    HangmanWord { word: "backpack", category: "School" },
    HangmanWord { word: "break", category: "School" },
    HangmanWord { word: "library", category: "School" },
    HangmanWord { word: "desk", category: "School" },
    HangmanWord { word: "exam", category: "School" },
    HangmanWord { word: "playground", category: "School" },
    HangmanWord { word: "classroom", category: "School" },
    HangmanWord { word: "book", category: "School" },
    // Nature
    HangmanWord { word: "mountain", category: "Nature" },
    HangmanWord { word: "river", category: "Nature" },
    HangmanWord { word: "forest", category: "Nature" },
    HangmanWord { word: "sea", category: "Nature" },
    HangmanWord { word: "desert", category: "Nature" },
    HangmanWord { word: "volcano", category: "Nature" },
    HangmanWord { word: "cloud", category: "Nature" },
    HangmanWord { word: "rainbow", category: "Nature" },
    HangmanWord { word: "star", category: "Nature" },
    HangmanWord { word: "moon", category: "Nature" },
    HangmanWord { word: "wind", category: "Nature" },
    HangmanWord { word: "rain", category: "Nature" },
    // Transport
    HangmanWord { word: "plane", category: "Transport" },
    HangmanWord { word: "train", category: "Transport" },
    HangmanWord { word: "boat", category: "Transport" },
    HangmanWord { word: "bicycle", category: "Transport" },
    HangmanWord { word: "bus", category: "Transport" },
    HangmanWord { word: "truck", category: "Transport" },
    HangmanWord { word: "helicopter", category: "Transport" },
    HangmanWord { word: "car", category: "Transport" },
    // House
    HangmanWord { word: "window", category: "House" },
    HangmanWord { word: "door", category: "House" },
    HangmanWord { word: "kitchen", category: "House" },
    HangmanWord { word: "mirror", category: "House" },
    HangmanWord { word: "lamp", category: "House" },
    HangmanWord { word: "sofa", category: "House" },
    HangmanWord { word: "bed", category: "House" },
    HangmanWord { word: "chair", category: "House" },
    // City
    HangmanWord { word: "bridge", category: "City" },
    HangmanWord { word: "park", category: "City" },
    HangmanWord { word: "hospital", category: "City" },
    HangmanWord { word: "school", category: "City" },
    HangmanWord { word: "market", category: "City" },
    HangmanWord { word: "square", category: "City" },
    HangmanWord { word: "street", category: "City" },
    HangmanWord { word: "traffic light", category: "City" },
];

/// Palabras del ahorcado en francés — 72 palabras.
const WORDS_FR: [HangmanWord; 72] = [
    // Animaux
    HangmanWord { word: "chien", category: "Animal" },
    HangmanWord { word: "chat", category: "Animal" },
    HangmanWord { word: "lion", category: "Animal" },
    HangmanWord { word: "tigre", category: "Animal" },
    HangmanWord { word: "éléphant", category: "Animal" },
    HangmanWord { word: "girafe", category: "Animal" },
    HangmanWord { word: "papillon", category: "Animal" },
    HangmanWord { word: "dauphin", category: "Animal" },
    HangmanWord { word: "tortue", category: "Animal" },
    HangmanWord { word: "perroquet", category: "Animal" },
    HangmanWord { word: "zèbre", category: "Animal" },
    HangmanWord { word: "cheval", category: "Animal" },
    // Fruits
    HangmanWord { word: "pomme", category: "Fruit" },
    HangmanWord { word: "banane", category: "Fruit" },
    HangmanWord { word: "orange", category: "Fruit" },
    HangmanWord { word: "fraise", category: "Fruit" },
    HangmanWord { word: "raisin", category: "Fruit" },
    HangmanWord { word: "poire", category: "Fruit" },
    HangmanWord { word: "pastèque", category: "Fruit" },
    HangmanWord { word: "melon", category: "Fruit" },
    HangmanWord { word: "citron", category: "Fruit" },
    HangmanWord { word: "kiwi", category: "Fruit" },
    HangmanWord { word: "cerise", category: "Fruit" },
    HangmanWord { word: "prune", category: "Fruit" },
    // École
    HangmanWord { word: "crayon", category: "École" },
    HangmanWord { word: "cahier", category: "École" },
    HangmanWord { word: "tableau", category: "École" },
    HangmanWord { word: "professeur", category: "École" },
    HangmanWord { word: "cartable", category: "École" },
    HangmanWord { word: "récréation", category: "École" },
    HangmanWord { word: "bibliothèque", category: "École" },
    HangmanWord { word: "bureau", category: "École" },
    HangmanWord { word: "examen", category: "École" },
    HangmanWord { word: "cour", category: "École" },
    HangmanWord { word: "maîtresse", category: "École" },
    HangmanWord { word: "livre", category: "École" },
    // Nature
    HangmanWord { word: "montagne", category: "Nature" },
    HangmanWord { word: "rivière", category: "Nature" },
    HangmanWord { word: "forêt", category: "Nature" },
    HangmanWord { word: "mer", category: "Nature" },
    HangmanWord { word: "désert", category: "Nature" },
    HangmanWord { word: "volcan", category: "Nature" },
    HangmanWord { word: "nuage", category: "Nature" },
    HangmanWord { word: "arc-en-ciel", category: "Nature" },
    HangmanWord { word: "étoile", category: "Nature" },
    HangmanWord { word: "lune", category: "Nature" },
    HangmanWord { word: "vent", category: "Nature" },
    HangmanWord { word: "pluie", category: "Nature" },
    // Transport
    HangmanWord { word: "avion", category: "Transport" },
    HangmanWord { word: "train", category: "Transport" },
    HangmanWord { word: "bateau", category: "Transport" },
    HangmanWord { word: "vélo", category: "Transport" },
    HangmanWord { word: "bus", category: "Transport" },
    HangmanWord { word: "camion", category: "Transport" },
    HangmanWord { word: "hélicoptère", category: "Transport" },
    HangmanWord { word: "voiture", category: "Transport" },
    // Maison
    HangmanWord { word: "fenêtre", category: "Maison" },
    HangmanWord { word: "porte", category: "Maison" },
    HangmanWord { word: "cuisine", category: "Maison" },
    HangmanWord { word: "miroir", category: "Maison" },
    HangmanWord { word: "lampe", category: "Maison" },
    HangmanWord { word: "canapé", category: "Maison" },
    HangmanWord { word: "lit", category: "Maison" },
    HangmanWord { word: "chaise", category: "Maison" },
    // Ville
    HangmanWord { word: "pont", category: "Ville" },
    HangmanWord { word: "parc", category: "Ville" },
    HangmanWord { word: "hôpital", category: "Ville" },
    HangmanWord { word: "école", category: "Ville" },
    HangmanWord { word: "marché", category: "Ville" },
    HangmanWord { word: "place", category: "Ville" },
    HangmanWord { word: "rue", category: "Ville" },
    HangmanWord { word: "feu tricolore", category: "Ville" },
];