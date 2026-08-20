//! Práctica de leer y escribir (sección Lengua).
//!
//! Una sesión de 9 actividades mezcladas:
//! - **Leer (reconocimiento)**: se muestra una palabra y hay que elegir la
//!   idéntica entre 4 parecidas (con una letra cambiada).
//! - **Leer (comprensión)**: se muestra una pista y hay que elegir la
//!   palabra que corresponde.
//! - **Escribir**: se muestra una palabra y hay que teclearla (sin acentos
//!   ni mayúsculas hace falta: la comparación es tolerante).
//!
//! Cada respuesta da un feedback de 1,4 s (verde/rojo) y al final se muestra
//! el marcador con la opción de volver al menú de Lengua.

use bevy::prelude::*;
use rand::Rng;
use rand::seq::SliceRandom;

use super::{screen_background, spawn_button};
use crate::audio::{play_click, play_success, Sfx};
use crate::board::questions::normalize_answer;
use crate::game::GameState;
use crate::i18n::tr;

/// Número de actividades por sesión.
const ROUNDS: usize = 9;
/// Duración (s) del feedback antes de pasar a la siguiente actividad.
const FEEDBACK_SECONDS: f32 = 1.4;

/// Una actividad de la sesión.
enum ReadingRound {
    /// Buscar la palabra idéntica entre 4 parecidas.
    MatchWord {
        word: String,
        options: [String; 4],
        correct: usize,
    },
    /// Elegir la palabra que corresponde a una pista.
    ClueToWord {
        clue: String,
        options: [String; 4],
        correct: usize,
    },
    /// Escribir la palabra que se muestra.
    TypeWord { word: String },
}

/// Sesión de lectura/escritura activa.
#[derive(Resource)]
pub struct ReadingSession {
    rounds: Vec<ReadingRound>,
    index: usize,
    correct: usize,
    wrong: usize,
    selected: Option<usize>,
    feedback: bool,
    feedback_timer: f32,
    typed: String,
    done: bool,
}

impl ReadingSession {
    fn current(&self) -> &ReadingRound {
        &self.rounds[self.index]
    }

    /// Texto que se muestra como enunciado de la actividad actual.
    fn prompt_text(&self) -> String {
        match self.current() {
            ReadingRound::MatchWord { word, .. } => {
                tr("Busca la palabra: \"{}\"").replace("{}", word)
            }
            ReadingRound::ClueToWord { clue, .. } => clue.clone(),
            ReadingRound::TypeWord { word } => {
                tr("Escribe la palabra: \"{}\"").replace("{}", word)
            }
        }
    }

    /// La palabra correcta de la actividad actual (para el feedback).
    fn answer_text(&self) -> &str {
        match self.current() {
            ReadingRound::MatchWord { word, .. } => word,
            ReadingRound::ClueToWord { options, correct, .. } => &options[*correct],
            ReadingRound::TypeWord { word } => word,
        }
    }
}

// ---- Componentes de la UI --------------------------------------------------

/// Raíz de la pantalla de lectura/escritura.
#[derive(Component)]
pub struct ReadingUiRoot;

/// Campo de texto etiquetado por su función.
#[derive(Component)]
pub struct ReadingText(ReadingField);

#[derive(Clone, Copy, PartialEq, Eq)]
enum ReadingField {
    Title,
    Prompt,
    Typed,
    Progress,
    Feedback,
    ResultTitle,
    ResultDetail,
}

/// Texto de una opción (hijo de su botón).
#[derive(Component)]
pub struct ReadingOptionText(pub usize);

/// Botón de una opción (A/B/C/D).
#[derive(Component)]
pub struct ReadingOptionButton(pub usize);

/// Contenedor de resultados (oculto hasta terminar).
#[derive(Component)]
pub struct ReadingResultBox;

/// Botón de volver al menú de Lengua.
#[derive(Component)]
pub struct ReadingBackButton;

/// Plugin de la práctica de leer y escribir.
pub struct ReadingPlugin;

impl Plugin for ReadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::ReadingPractice), spawn_reading_ui)
            .add_systems(OnExit(GameState::ReadingPractice), cleanup_reading)
            .add_systems(
                Update,
                update_reading.run_if(in_state(GameState::ReadingPractice)),
            );
    }
}

/// Letras de las opciones.
const OPTION_LETTERS: [char; 4] = ['A', 'B', 'C', 'D'];

/// Banco de palabras con su pista (para lectura y escritura).
const WORDS: [(&str, &str); 34] = [
    ("casa", "Lugar donde vives"),
    ("sol", "Brilla en el cielo por el día"),
    ("luna", "Se ve en el cielo por la noche"),
    ("mar", "Agua salada muy grande"),
    ("gato", "Animal que maúlla"),
    ("perro", "Animal que ladra"),
    ("pato", "Animal que dice cuac"),
    ("pez", "Animal que nada en el agua"),
    ("pan", "Se come y se hace con harina"),
    ("mesa", "Mueble donde se come"),
    ("silla", "Mueble para sentarse"),
    ("flor", "Planta que huele muy bien"),
    ("árbol", "Planta grande con tronco"),
    ("rojo", "El color de la sangre"),
    ("azul", "El color del mar y del cielo"),
    ("agua", "Se bebe y es transparente"),
    ("libro", "Tiene páginas y se lee"),
    ("lápiz", "Sirve para escribir y dibujar"),
    ("pelota", "Se bota y se juega con ella"),
    ("reloj", "Marca las horas"),
    ("puerta", "Se abre para entrar"),
    ("ventana", "Deja pasar la luz"),
    ("nube", "Flota en el cielo y es blanca"),
    ("lluvia", "Cae del cielo y moja"),
    ("fuego", "Da calor y luz"),
    ("tierra", "El planeta donde vivimos"),
    ("viento", "Se nota pero no se ve"),
    ("caballo", "Animal que galopa"),
    ("vaca", "Animal que da leche"),
    ("gallina", "Animal que pone huevos"),
    ("león", "El rey de la selva"),
    ("tigre", "Felino con rayas"),
    ("jirafa", "Animal con el cuello largo"),
    ("elefante", "Animal grande con trompa"),
];

/// Banco de palabras en inglés (palabra, pista).
const WORDS_EN: [(&str, &str); 34] = [
    ("house", "Where you live"),
    ("sun", "It shines in the sky during the day"),
    ("moon", "You can see it in the sky at night"),
    ("sea", "Very big salty water"),
    ("cat", "An animal that meows"),
    ("dog", "An animal that barks"),
    ("duck", "An animal that says quack"),
    ("fish", "An animal that swims in water"),
    ("bread", "You eat it and it is made with flour"),
    ("table", "Furniture where you eat"),
    ("chair", "Furniture to sit on"),
    ("flower", "A plant that smells very nice"),
    ("tree", "A big plant with a trunk"),
    ("red", "The colour of blood"),
    ("blue", "The colour of the sea and the sky"),
    ("water", "You drink it and it is clear"),
    ("book", "It has pages and you read it"),
    ("pencil", "You use it to write and draw"),
    ("ball", "You bounce it and play with it"),
    ("clock", "It shows the hours"),
    ("door", "You open it to come in"),
    ("window", "It lets the light in"),
    ("cloud", "It floats in the sky and is white"),
    ("rain", "It falls from the sky and gets you wet"),
    ("fire", "It gives heat and light"),
    ("earth", "The planet where we live"),
    ("wind", "You can feel it but not see it"),
    ("horse", "An animal that gallops"),
    ("cow", "An animal that gives milk"),
    ("hen", "An animal that lays eggs"),
    ("lion", "The king of the jungle"),
    ("tiger", "A cat with stripes"),
    ("giraffe", "An animal with a long neck"),
    ("elephant", "A big animal with a trunk"),
];

/// Banco de palabras en francés (palabra, piste).
const WORDS_FR: [(&str, &str); 34] = [
    ("maison", "L'endroit où tu habites"),
    ("soleil", "Il brille dans le ciel le jour"),
    ("lune", "On la voit dans le ciel la nuit"),
    ("mer", "De très grande eau salée"),
    ("chat", "Un animal qui miaule"),
    ("chien", "Un animal qui aboie"),
    ("canard", "Un animal qui fait coin-coin"),
    ("poisson", "Un animal qui nage dans l'eau"),
    ("pain", "On le mange et il est fait avec de la farine"),
    ("table", "Le meuble où l'on mange"),
    ("chaise", "Le meuble pour s'asseoir"),
    ("fleur", "Une plante qui sent très bon"),
    ("arbre", "Une grande plante avec un tronc"),
    ("rouge", "La couleur du sang"),
    ("bleu", "La couleur de la mer et du ciel"),
    ("eau", "On la boit et elle est transparente"),
    ("livre", "Il a des pages et on le lit"),
    ("crayon", "Il sert à écrire et à dessiner"),
    ("ballon", "On le fait rebondir et on joue avec"),
    ("horloge", "Elle indique les heures"),
    ("porte", "On l'ouvre pour entrer"),
    ("fenêtre", "Elle laisse passer la lumière"),
    ("nuage", "Il flotte dans le ciel et il est blanc"),
    ("pluie", "Elle tombe du ciel et elle mouille"),
    ("feu", "Il donne de la chaleur et de la lumière"),
    ("terre", "La planète où nous vivons"),
    ("vent", "On le sent mais on ne le voit pas"),
    ("cheval", "Un animal qui galope"),
    ("vache", "Un animal qui donne du lait"),
    ("poule", "Un animal qui pond des œufs"),
    ("lion", "Le roi de la jungle"),
    ("tigre", "Un félin avec des rayures"),
    ("girafe", "Un animal avec un long cou"),
    ("éléphant", "Un grand animal avec une trompe"),
];

/// Crea un texto del campo indicado (con su etiqueta).
fn reading_text(
    parent: &mut ChildSpawnerCommands,
    field: ReadingField,
    text: &str,
    size: f32,
    font: &Handle<Font>,
) {
    parent.spawn((
        ReadingText(field),
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
            max_width: Val::Px(700.0),
            ..default()
        },
    ));
}

/// Crea el texto de una opción (hijo de su botón).
fn reading_option_text(
    parent: &mut ChildSpawnerCommands,
    index: usize,
    size: f32,
    font: &Handle<Font>,
) {
    parent.spawn((
        ReadingOptionText(index),
        Text::new(String::new()),
        TextFont {
            font: font.clone(),
            font_size: size,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

/// Construye la pantalla de lectura/escritura.
fn spawn_reading_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands.insert_resource(ReadingSession {
        rounds: build_rounds(),
        index: 0,
        correct: 0,
        wrong: 0,
        selected: None,
        feedback: false,
        feedback_timer: 0.0,
        typed: String::new(),
        done: false,
    });
    commands
        .spawn((
            ReadingUiRoot,
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
                        width: Val::Px(720.0),
                        padding: UiRect::axes(Val::Px(28.0), Val::Px(24.0)),
                        row_gap: Val::Px(12.0),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.07, 0.09, 0.18, 0.96)),
                    BorderRadius::all(Val::Px(16.0)),
                ))
                .with_children(|panel| {
                    reading_text(panel, ReadingField::Title, "Leer y escribir", 28.0, &font);
                    reading_text(panel, ReadingField::Prompt, "", 24.0, &font);

                    // Opciones A/B/C/D.
                    for index in 0..4 {
                        panel
                            .spawn((
                                Button,
                                ReadingOptionButton(index),
                                Node {
                                    width: Val::Px(640.0),
                                    height: Val::Px(44.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border: UiRect::all(Val::Px(2.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.15, 0.18, 0.28)),
                                BorderColor(Color::srgb(0.50, 0.55, 0.70)),
                                BorderRadius::all(Val::Px(8.0)),
                            ))
                            .with_children(|option| {
                                reading_option_text(option, index, 20.0, &font);
                            });
                    }

                    reading_text(panel, ReadingField::Typed, "", 26.0, &font);
                    reading_text(panel, ReadingField::Progress, "", 17.0, &font);
                    reading_text(panel, ReadingField::Feedback, "", 22.0, &font);

                    // Resultados (ocultos hasta terminar).
                    panel
                        .spawn((
                            ReadingResultBox,
                            Node {
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                row_gap: Val::Px(10.0),
                                ..default()
                            },
                            Visibility::Hidden,
                        ))
                        .with_children(|results| {
                            reading_text(results, ReadingField::ResultTitle, "", 26.0, &font);
                            reading_text(results, ReadingField::ResultDetail, "", 20.0, &font);
                            spawn_button(results, "Volver a Lengua", ReadingBackButton, &font);
                        });
                });
        });
}

/// Destruye la pantalla y la sesión al salir.
fn cleanup_reading(mut commands: Commands, roots: Query<Entity, With<ReadingUiRoot>>) {
    for root in &roots {
        commands.entity(root).despawn();
    }
    commands.remove_resource::<ReadingSession>();
}

/// Construye las 9 actividades de una sesión (mezcladas).
fn build_rounds() -> Vec<ReadingRound> {
    let mut rng = rand::thread_rng();
    let mut rounds = Vec::with_capacity(ROUNDS);
    for _ in 0..3 {
        rounds.push(match_round(&mut rng));
    }
    for _ in 0..3 {
        rounds.push(clue_round(&mut rng));
    }
    for _ in 0..3 {
        rounds.push(type_round(&mut rng));
    }
    rounds.shuffle(&mut rng);
    rounds
}

/// Variantes de una palabra cambiando una letra por una vocal (para las
/// opciones parecidas de la actividad de reconocimiento).
fn similar_variants(word: &str, rng: &mut impl Rng) -> Vec<String> {
    let chars: Vec<char> = word.chars().collect();
    let mut candidates: Vec<String> = Vec::new();
    for i in 0..chars.len() {
        for vowel in ['a', 'e', 'i', 'o', 'u'] {
            if chars[i] == vowel {
                continue;
            }
            let mut copy = chars.clone();
            copy[i] = vowel;
            let variant: String = copy.into_iter().collect();
            if variant != word && !candidates.contains(&variant) {
                candidates.push(variant);
            }
        }
    }
    candidates.shuffle(rng);
    candidates.truncate(3);
    candidates
}

/// Actividad de reconocimiento: buscar la palabra idéntica.
fn match_round(rng: &mut impl Rng) -> ReadingRound {
    let bank = words_bank();
    let (word, _) = bank[rng.gen_range(0..bank.len())];
    let mut options = vec![word.to_string()];
    options.extend(similar_variants(word, rng));
    options.shuffle(rng);
    let correct = options.iter().position(|o| o == word).unwrap_or(0);
    ReadingRound::MatchWord {
        word: word.to_string(),
        options: options.try_into().unwrap_or_else(|_| {
            // No debería ocurrir: siempre hay al menos 4 opciones.
            [String::new(), String::new(), String::new(), String::new()]
        }),
        correct,
    }
}

/// Actividad de comprensión: elegir la palabra que corresponde a la pista.
fn clue_round(rng: &mut impl Rng) -> ReadingRound {
    let bank = words_bank();
    let (word, clue) = bank[rng.gen_range(0..bank.len())];
    let mut options = vec![word.to_string()];
    while options.len() < 4 {
        let (other, _) = bank[rng.gen_range(0..bank.len())];
        if !options.contains(&other.to_string()) {
            options.push(other.to_string());
        }
    }
    options.shuffle(rng);
    let correct = options.iter().position(|o| o == word).unwrap_or(0);
    ReadingRound::ClueToWord {
        clue: clue.to_string(),
        options: options.try_into().unwrap_or_else(|_| {
            [String::new(), String::new(), String::new(), String::new()]
        }),
        correct,
    }
}

/// Actividad de escritura: teclear la palabra que se muestra.
fn type_round(rng: &mut impl Rng) -> ReadingRound {
    let bank = words_bank();
    let (word, _) = bank[rng.gen_range(0..bank.len())];
    ReadingRound::TypeWord {
        word: word.to_string(),
    }
}

/// Banco de palabras (palabra, pista) según el idioma activo.
fn words_bank() -> &'static [(&'static str, &'static str)] {
    match crate::i18n::language() {
        crate::i18n::Language::En => &WORDS_EN,
        crate::i18n::Language::Fr => &WORDS_FR,
        crate::i18n::Language::Es => &WORDS,
    }
}

/// Convierte una tecla a su carácter (minúsculas, dígitos y espacio).
fn key_char(key: &KeyCode) -> Option<char> {
    use KeyCode::*;
    Some(match key {
        KeyA => 'a',
        KeyB => 'b',
        KeyC => 'c',
        KeyD => 'd',
        KeyE => 'e',
        KeyF => 'f',
        KeyG => 'g',
        KeyH => 'h',
        KeyI => 'i',
        KeyJ => 'j',
        KeyK => 'k',
        KeyL => 'l',
        KeyM => 'm',
        KeyN => 'n',
        KeyO => 'o',
        KeyP => 'p',
        KeyQ => 'q',
        KeyR => 'r',
        KeyS => 's',
        KeyT => 't',
        KeyU => 'u',
        KeyV => 'v',
        KeyW => 'w',
        KeyX => 'x',
        KeyY => 'y',
        KeyZ => 'z',
        Digit0 => '0',
        Digit1 => '1',
        Digit2 => '2',
        Digit3 => '3',
        Digit4 => '4',
        Digit5 => '5',
        Digit6 => '6',
        Digit7 => '7',
        Digit8 => '8',
        Digit9 => '9',
        Space => ' ',
        _ => return None,
    })
}

/// Colores de los botones de opción.
const OPTION_NEUTRAL: Color = Color::srgb(0.15, 0.18, 0.28);
const OPTION_DIM: Color = Color::srgb(0.10, 0.12, 0.20);
const OPTION_CORRECT: Color = Color::srgb(0.15, 0.42, 0.25);
const OPTION_WRONG: Color = Color::srgb(0.50, 0.20, 0.20);

/// Gestiona la sesión: entrada de teclado, clics, feedback y resultados.
fn update_reading(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut commands: Commands,
    sfx: Res<Sfx>,
    session: Option<ResMut<ReadingSession>>,
    mut texts: Query<
        (&ReadingText, &mut Text, &mut TextColor, &mut Visibility),
        (
            Without<ReadingOptionText>,
            Without<ReadingOptionButton>,
            Without<ReadingResultBox>,
        ),
    >,
    mut option_texts: Query<
        (&ReadingOptionText, &mut Text),
        (
            Without<ReadingText>,
            Without<ReadingOptionButton>,
            Without<ReadingResultBox>,
        ),
    >,
    mut option_colors: Query<(&ReadingOptionButton, &mut BackgroundColor), Without<ReadingText>>,
    mut option_visibility: Query<(&ReadingOptionButton, &mut Visibility), Without<ReadingText>>,
    option_clicks: Query<
        (&Interaction, &ReadingOptionButton),
        (Changed<Interaction>, Without<ReadingText>),
    >,
    mut result_box: Query<
        &mut Visibility,
        (
            With<ReadingResultBox>,
            Without<ReadingText>,
            Without<ReadingOptionButton>,
        ),
    >,
    close_clicks: Query<&Interaction, (Changed<Interaction>, With<ReadingBackButton>)>,
) {
    let dt = time.delta_secs();
    let mut session = match session {
        Some(session) => session,
        // Defensa: si no hay sesión (p. ej. tras un cambio de estado raro),
        // se reconstruye para que la pantalla nunca se quede vacía.
        None => {
            commands.insert_resource(ReadingSession {
                rounds: build_rounds(),
                index: 0,
                correct: 0,
                wrong: 0,
                selected: None,
                feedback: false,
                feedback_timer: 0.0,
                typed: String::new(),
                done: false,
            });
            return;
        }
    };

    // Escape: volver al menú de Lengua.
    if keys.just_pressed(KeyCode::Escape) {
        commands.set_state(GameState::LanguageMenu);
        return;
    }

    // 1) Resultados: mostrar marcador y botón de volver.
    if session.done {
        let close = close_clicks.single().map_or(false, |i| *i == Interaction::Pressed)
            || keys.just_pressed(KeyCode::Enter)
            || keys.just_pressed(KeyCode::KeyQ);
        if close {
            commands.set_state(GameState::LanguageMenu);
            return;
        }
        for (field, mut text, mut color, mut vis) in &mut texts {
            match field.0 {
                ReadingField::ResultTitle => {
                    *text = Text::new(tr(if session.correct >= ROUNDS / 2 { "¡Muy bien!" } else { "¡Sigue practicando!" }));
                    *color = TextColor(if session.correct >= ROUNDS / 2 {
                        Color::srgb(0.40, 0.90, 0.50)
                    } else {
                        Color::srgb(0.95, 0.55, 0.30)
                    });
                    *vis = Visibility::Visible;
                }
                ReadingField::ResultDetail => {
                    *text = Text::new(tr("Aciertos: {} · Fallos: {}  de {} actividades").replace("{}", &session.correct.to_string()).replace("{}", &session.wrong.to_string()).replace("{}", &ROUNDS.to_string()));
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

    // 2) Feedback: cuenta atrás y colores de las opciones.
    if session.feedback {
        session.feedback_timer -= dt;
        for (button, mut bg) in &mut option_colors {
            let round = session.current();
            let correct = match round {
                ReadingRound::MatchWord { correct, .. } => *correct,
                ReadingRound::ClueToWord { correct, .. } => *correct,
                ReadingRound::TypeWord { .. } => 0,
            };
            *bg = BackgroundColor(if button.0 == correct {
                OPTION_CORRECT
            } else if Some(button.0) == session.selected {
                OPTION_WRONG
            } else {
                OPTION_DIM
            });
        }
        if session.feedback_timer <= 0.0 {
            session.feedback = false;
            session.selected = None;
            session.index += 1;
            if session.index >= ROUNDS {
                session.done = true;
                return;
            }
        }
    } else {
        // 3) Responder según el tipo de actividad.
        match session.current() {
            ReadingRound::TypeWord { word } => {
                let word = word.clone();
                // Escribir: teclear letras/dígitos/espacio y Enter para enviar.
                for key in keys.get_just_pressed() {
                    if *key == KeyCode::Backspace {
                        session.typed.pop();
                    } else if let Some(ch) = key_char(key) {
                        session.typed.push(ch);
                    }
                }
                if keys.just_pressed(KeyCode::Enter) {
                    let ok = normalize_answer(&session.typed) == normalize_answer(&word);
                    if ok {
                        session.correct += 1;
                    } else {
                        session.wrong += 1;
                    }
                    session.selected = None;
                    session.feedback = true;
                    session.feedback_timer = FEEDBACK_SECONDS;
                    if ok {
                        play_success(&mut commands, &sfx);
                    }
                }
            }
            _ => {
                // Elegir una opción con clic o teclas 1-4.
                let mut chosen: Option<usize> = None;
                for (interaction, button) in &option_clicks {
                    if *interaction == Interaction::Pressed {
                        chosen = Some(button.0);
                        break;
                    }
                }
                if chosen.is_none() {
                    for (index, code) in [
                        KeyCode::Digit1,
                        KeyCode::Digit2,
                        KeyCode::Digit3,
                        KeyCode::Digit4,
                    ]
                    .iter()
                    .enumerate()
                    {
                        if keys.just_pressed(*code) {
                            chosen = Some(index);
                            break;
                        }
                    }
                }
                if let Some(index) = chosen {
                    let correct = match session.current() {
                        ReadingRound::MatchWord { correct, .. } => *correct,
                        ReadingRound::ClueToWord { correct, .. } => *correct,
                        ReadingRound::TypeWord { .. } => 0,
                    };
                    if index == correct {
                        session.correct += 1;
                        play_success(&mut commands, &sfx);
                    } else {
                        session.wrong += 1;
                    }
                    session.selected = Some(index);
                    session.feedback = true;
                    session.feedback_timer = FEEDBACK_SECONDS;
                }
                for (_button, mut bg) in &mut option_colors {
                    *bg = BackgroundColor(OPTION_NEUTRAL);
                }
            }
        }
    }

    // 4) Textos de la actividad actual.
    let round = session.current();
    let is_type = matches!(round, ReadingRound::TypeWord { .. });
    for (field, mut text, mut color, mut vis) in &mut texts {
        match field.0 {
            ReadingField::Title => {
                *text = Text::new(if is_type {
                    "Escribir"
                } else {
                    "Leer"
                });
                *color = TextColor(Color::srgb(0.95, 0.85, 0.55));
                *vis = Visibility::Visible;
            }
            ReadingField::Prompt => {
                *text = Text::new(session.prompt_text());
                *vis = Visibility::Visible;
            }
            ReadingField::Typed => {
                if is_type {
                    let display = if session.typed.is_empty() {
                        tr("Escribe aquí…")
                    } else {
                        session.typed.clone()
                    };
                    *text = Text::new(tr("Tu respuesta: {display}").replace("{display}", &display));
                    *color = TextColor(Color::srgb(0.80, 0.95, 1.0));
                    *vis = Visibility::Visible;
                } else {
                    *vis = Visibility::Hidden;
                }
            }
            ReadingField::Progress => {
                *text = Text::new(tr("Actividad {}/{}  ·  Aciertos: {}  ·  Fallos: {}").replace("{}", &(session.index + 1).to_string()).replace("{}", &ROUNDS.to_string()).replace("{}", &session.correct.to_string()).replace("{}", &session.wrong.to_string()));
                *vis = Visibility::Visible;
            }
            ReadingField::Feedback => {
                if session.feedback {
                    let ok = if is_type {
                        normalize_answer(&session.typed) == normalize_answer(session.answer_text())
                    } else {
                        session.selected == Some(match round {
                            ReadingRound::MatchWord { correct, .. } => *correct,
                            ReadingRound::ClueToWord { correct, .. } => *correct,
                            ReadingRound::TypeWord { .. } => 0,
                        })
                    };
                    if ok {
                        *text = Text::new(tr("¡Correcto!"));
                        *color = TextColor(Color::srgb(0.40, 0.90, 0.50));
                    } else {
                        *text = Text::new(tr("Incorrecto — era: {}").replace("{}", session.answer_text()));
                        *color = TextColor(Color::srgb(0.95, 0.40, 0.40));
                    }
                    *vis = Visibility::Visible;
                } else {
                    *vis = Visibility::Hidden;
                }
            }
            _ => {}
        }
    }
    // Opciones: solo visibles en actividades de elegir.
    for (_button, mut vis) in &mut option_visibility {
        *vis = if is_type {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }
    // Textos de las opciones.
    if let ReadingRound::MatchWord { options, .. } | ReadingRound::ClueToWord { options, .. } =
        round
    {
        for (field, mut text) in &mut option_texts {
            *text = Text::new(format!(
                "{}) {}",
                OPTION_LETTERS[field.0],
                options[field.0]
            ));
        }
    }
    // Clic sonoro al pulsar una opción.
    for (interaction, _button) in &option_clicks {
        if *interaction == Interaction::Pressed {
            play_click(&mut commands, &sfx);
            break;
        }
    }
}