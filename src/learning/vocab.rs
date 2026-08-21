//! Vocabulario (Lengua) — traduce la palabra al otro idioma.
//!
//! 10 rondas: se muestra palabra en ES y hay que elegir traducción EN o FR
//! según el idioma activo. Si idioma es ES, traduce a EN; si es EN, a ES;
//! si es FR, a ES. 4 opciones, feedback 1,4s, trilingüe.

use bevy::prelude::*;
use rand::seq::SliceRandom;

use super::{screen_background, spawn_button};
use crate::audio::{play_click, play_success, Sfx};
use crate::game::GameState;
use crate::i18n::{language, Language, tr};

const ROUNDS: usize = 10;
const FEEDBACK_SECONDS: f32 = 1.4;

#[derive(Clone, Copy)]
struct VocabEntry {
    es: &'static str,
    en: &'static str,
    fr: &'static str,
}

struct VocabRound {
    prompt: String,
    options: [String; 4],
    correct: usize,
}

#[derive(Resource)]
pub struct VocabSession {
    rounds: Vec<VocabRound>,
    index: usize,
    correct: usize,
    wrong: usize,
    selected: Option<usize>,
    feedback: bool,
    feedback_timer: f32,
    done: bool,
}

#[derive(Component)]
pub struct VocabUiRoot;
#[derive(Component)]
pub struct VocabText(VocabField);
#[derive(Clone, Copy, PartialEq, Eq)]
enum VocabField {
    Title,
    Question,
    Progress,
    Feedback,
    ResultTitle,
    ResultDetail,
}
#[derive(Component)]
pub struct VocabOptionText(pub usize);
#[derive(Component)]
pub struct VocabOptionButton(pub usize);
#[derive(Component)]
pub struct VocabResultBox;
#[derive(Component)]
pub struct VocabBackButton;

pub struct VocabPlugin;
impl Plugin for VocabPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::VocabPractice), spawn_vocab_ui)
            .add_systems(OnExit(GameState::VocabPractice), cleanup_vocab)
            .add_systems(Update, update_vocab.run_if(in_state(GameState::VocabPractice)));
    }
}
const OPTION_LETTERS: [char; 4] = ['A', 'B', 'C', 'D'];
fn vocab_text(parent: &mut ChildSpawnerCommands, field: VocabField, text: &str, size: f32, font: &Handle<Font>) {
    parent.spawn((VocabText(field), Text::new(text.to_string()), TextFont { font: font.clone(), font_size: size, ..default() }, TextColor(Color::WHITE), TextLayout { linebreak: LineBreak::WordBoundary, ..default() }, Node { max_width: Val::Px(700.0), ..default() }));
}
fn vocab_option_text(parent: &mut ChildSpawnerCommands, index: usize, size: f32, font: &Handle<Font>) {
    parent.spawn((VocabOptionText(index), Text::new(String::new()), TextFont { font: font.clone(), font_size: size, ..default() }, TextColor(Color::WHITE)));
}
fn spawn_vocab_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands.insert_resource(VocabSession { rounds: build_rounds(), index: 0, correct: 0, wrong: 0, selected: None, feedback: false, feedback_timer: 0.0, done: false });
    commands
        .spawn((VocabUiRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, screen_background(), Visibility::Visible, ZIndex(30)))
        .with_children(|overlay| {
            overlay.spawn((Node { flex_direction: FlexDirection::Column, width: Val::Px(680.0), padding: UiRect::axes(Val::Px(28.0), Val::Px(24.0)), row_gap: Val::Px(14.0), align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.07, 0.09, 0.18, 0.96)), BorderRadius::all(Val::Px(16.0)))).with_children(|panel| {
                vocab_text(panel, VocabField::Title, "", 28.0, &font);
                vocab_text(panel, VocabField::Question, "", 30.0, &font);
                for index in 0..4 { panel.spawn((Button, VocabOptionButton(index), Node { width: Val::Px(600.0), height: Val::Px(46.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.15, 0.18, 0.28)), BorderColor(Color::srgb(0.50, 0.55, 0.70)), BorderRadius::all(Val::Px(8.0)))).with_children(|o| vocab_option_text(o, index, 21.0, &font)); }
                vocab_text(panel, VocabField::Progress, "", 17.0, &font);
                vocab_text(panel, VocabField::Feedback, "", 22.0, &font);
                panel.spawn((VocabResultBox, Node { flex_direction: FlexDirection::Column, align_items: AlignItems::Center, row_gap: Val::Px(10.0), ..default() }, Visibility::Hidden)).with_children(|r| { vocab_text(r, VocabField::ResultTitle, "", 26.0, &font); vocab_text(r, VocabField::ResultDetail, "", 20.0, &font); spawn_button(r, "Volver a Lengua", VocabBackButton, &font); });
            });
        });
}
fn cleanup_vocab(mut commands: Commands, roots: Query<Entity, With<VocabUiRoot>>) {
    for root in &roots { commands.entity(root).despawn(); }
    commands.remove_resource::<VocabSession>();
}
fn build_rounds() -> Vec<VocabRound> {
    let mut rng = rand::thread_rng();
    let mut rounds = Vec::with_capacity(ROUNDS);
    let lang = language();
    let mut pool: Vec<VocabEntry> = VOCAB.to_vec();
    pool.shuffle(&mut rng);
    for entry in pool.into_iter().take(ROUNDS) {
        let (prompt, correct_answer) = match lang {
            Language::Es => (format!("\"{}\" en inglés es...", entry.es), entry.en),
            Language::En => (format!("\"{}\" in Spanish is...", entry.en), entry.es),
            Language::Fr => (format!("\"{}\" en espagnol est...", entry.fr), entry.es),
        };
        // distractores: otras traducciones
        let mut distractors: Vec<&str> = VOCAB.iter().filter(|e| e.es != entry.es).map(|e| match lang { Language::Es => e.en, Language::En => e.es, Language::Fr => e.es }).collect();
        distractors.shuffle(&mut rng);
        let mut options = vec![correct_answer.to_string()];
        for d in distractors.into_iter().take(3) {
            if !options.contains(&d.to_string()) {
                options.push(d.to_string());
            }
        }
        options.shuffle(&mut rng);
        let correct = options.iter().position(|o| o == correct_answer).unwrap_or(0);
        rounds.push(VocabRound { prompt, options: options.try_into().unwrap(), correct });
    }
    rounds
}
const OPTION_NEUTRAL: Color = Color::srgb(0.15, 0.18, 0.28);
const OPTION_DIM: Color = Color::srgb(0.10, 0.12, 0.20);
const OPTION_CORRECT: Color = Color::srgb(0.15, 0.42, 0.25);
const OPTION_WRONG: Color = Color::srgb(0.50, 0.20, 0.20);
fn update_vocab(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut commands: Commands,
    sfx: Res<Sfx>,
    session: Option<ResMut<VocabSession>>,
    mut texts: Query<(&VocabText, &mut Text, &mut TextColor, &mut Visibility), (Without<VocabOptionText>, Without<VocabOptionButton>, Without<VocabResultBox>)>,
    mut option_texts: Query<(&VocabOptionText, &mut Text), (Without<VocabText>, Without<VocabOptionButton>, Without<VocabResultBox>)>,
    mut option_colors: Query<(&VocabOptionButton, &mut BackgroundColor), Without<VocabText>>,
    option_clicks: Query<(&Interaction, &VocabOptionButton), (Changed<Interaction>, Without<VocabText>)>,
    mut result_box: Query<&mut Visibility, (With<VocabResultBox>, Without<VocabText>, Without<VocabOptionButton>)>,
    close_clicks: Query<&Interaction, (Changed<Interaction>, With<VocabBackButton>)>,
) {
    let dt = time.delta_secs();
    let mut session = match session { Some(s) => s, None => { commands.insert_resource(VocabSession { rounds: build_rounds(), index: 0, correct: 0, wrong: 0, selected: None, feedback: false, feedback_timer: 0.0, done: false }); return; } };
    if keys.just_pressed(KeyCode::Escape) { commands.set_state(GameState::LanguageMenu); return; }
    if session.done {
        let close = close_clicks.single().map_or(false, |i| *i == Interaction::Pressed) || keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::KeyQ);
        if close { commands.set_state(GameState::LanguageMenu); return; }
        for (field, mut text, mut color, mut vis) in &mut texts {
            match field.0 {
                VocabField::ResultTitle => { *text = Text::new(tr(if session.correct >= ROUNDS / 2 { "¡Muy bien!" } else { "¡Sigue practicando!" })); *color = TextColor(if session.correct >= ROUNDS / 2 { Color::srgb(0.40, 0.90, 0.50) } else { Color::srgb(0.95, 0.55, 0.30) }); *vis = Visibility::Visible; }
                VocabField::ResultDetail => { *text = Text::new(tr("Aciertos: {} · Fallos: {}  de {} palabras").replace("{}", &session.correct.to_string()).replace("{}", &session.wrong.to_string()).replace("{}", &ROUNDS.to_string())); *vis = Visibility::Visible; }
                _ => {}
            }
        }
        if let Ok(mut vis) = result_box.single_mut() { *vis = Visibility::Visible; }
        return;
    }
    if session.feedback {
        session.feedback_timer -= dt;
        for (button, mut bg) in &mut option_colors { let q = &session.rounds[session.index]; *bg = BackgroundColor(if button.0 == q.correct { OPTION_CORRECT } else if Some(button.0) == session.selected { OPTION_WRONG } else { OPTION_DIM }); }
        if session.feedback_timer <= 0.0 { session.feedback = false; session.selected = None; session.index += 1; if session.index >= ROUNDS { session.done = true; return; } }
    } else {
        let mut chosen: Option<usize> = None;
        for (interaction, button) in &option_clicks { if *interaction == Interaction::Pressed { chosen = Some(button.0); break; } }
        if chosen.is_none() { for (index, code) in [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4].iter().enumerate() { if keys.just_pressed(*code) { chosen = Some(index); break; } } }
        if let Some(index) = chosen { let q = &session.rounds[session.index]; if index == q.correct { session.correct += 1; play_success(&mut commands, &sfx); } else { session.wrong += 1; } session.selected = Some(index); session.feedback = true; session.feedback_timer = FEEDBACK_SECONDS; }
        for (_button, mut bg) in &mut option_colors { *bg = BackgroundColor(OPTION_NEUTRAL); }
    }
    let question = &session.rounds[session.index];
    for (field, mut text, mut color, mut vis) in &mut texts {
        match field.0 {
            VocabField::Title => { *text = Text::new(tr("VOCABULARIO")); *color = TextColor(Color::srgb(0.80, 0.75, 1.0)); *vis = Visibility::Visible; }
            VocabField::Question => { *text = Text::new(question.prompt.clone()); *vis = Visibility::Visible; }
            VocabField::Progress => { *text = Text::new(tr("Palabra {}/{}  ·  Aciertos: {}  ·  Fallos: {}").replace("{}", &(session.index + 1).to_string()).replace("{}", &ROUNDS.to_string()).replace("{}", &session.correct.to_string()).replace("{}", &session.wrong.to_string())); *vis = Visibility::Visible; }
            VocabField::Feedback => { if session.feedback { let ok = session.selected == Some(question.correct); if ok { *text = Text::new(tr("¡Correcto!")); *color = TextColor(Color::srgb(0.40, 0.90, 0.50)); } else { *text = Text::new(tr("Incorrecto — era {}) {}").replace("{}", &OPTION_LETTERS[question.correct].to_string()).replace("{}", &question.options[question.correct])); *color = TextColor(Color::srgb(0.95, 0.40, 0.40)); } *vis = Visibility::Visible; } else { *vis = Visibility::Hidden; } }
            _ => {}
        }
    }
    for (field, mut text) in &mut option_texts { *text = Text::new(format!("{}) {}", OPTION_LETTERS[field.0], question.options[field.0])); }
    for (interaction, _button) in &option_clicks { if *interaction == Interaction::Pressed { play_click(&mut commands, &sfx); break; } }
}

const VOCAB: [VocabEntry; 40] = [
    VocabEntry { es: "casa", en: "house", fr: "maison" },
    VocabEntry { es: "sol", en: "sun", fr: "soleil" },
    VocabEntry { es: "luna", en: "moon", fr: "lune" },
    VocabEntry { es: "agua", en: "water", fr: "eau" },
    VocabEntry { es: "libro", en: "book", fr: "livre" },
    VocabEntry { es: "gato", en: "cat", fr: "chat" },
    VocabEntry { es: "perro", en: "dog", fr: "chien" },
    VocabEntry { es: "escuela", en: "school", fr: "école" },
    VocabEntry { es: "profesor", en: "teacher", fr: "professeur" },
    VocabEntry { es: "amigo", en: "friend", fr: "ami" },
    VocabEntry { es: "familia", en: "family", fr: "famille" },
    VocabEntry { es: "ciudad", en: "city", fr: "ville" },
    VocabEntry { es: "montaña", en: "mountain", fr: "montagne" },
    VocabEntry { es: "río", en: "river", fr: "rivière" },
    VocabEntry { es: "bosque", en: "forest", fr: "forêt" },
    VocabEntry { es: "estrella", en: "star", fr: "étoile" },
    VocabEntry { es: "avión", en: "plane", fr: "avion" },
    VocabEntry { es: "tren", en: "train", fr: "train" },
    VocabEntry { es: "coche", en: "car", fr: "voiture" },
    VocabEntry { es: "manzana", en: "apple", fr: "pomme" },
    VocabEntry { es: "naranja", en: "orange", fr: "orange" },
    VocabEntry { es: "plátano", en: "banana", fr: "banane" },
    VocabEntry { es: "rojo", en: "red", fr: "rouge" },
    VocabEntry { es: "azul", en: "blue", fr: "bleu" },
    VocabEntry { es: "verde", en: "green", fr: "vert" },
    VocabEntry { es: "amarillo", en: "yellow", fr: "jaune" },
    VocabEntry { es: "mesa", en: "table", fr: "table" },
    VocabEntry { es: "silla", en: "chair", fr: "chaise" },
    VocabEntry { es: "puerta", en: "door", fr: "porte" },
    VocabEntry { es: "ventana", en: "window", fr: "fenêtre" },
    VocabEntry { es: "reloj", en: "clock", fr: "horloge" },
    VocabEntry { es: "pan", en: "bread", fr: "pain" },
    VocabEntry { es: "leche", en: "milk", fr: "lait" },
    VocabEntry { es: "cielo", en: "sky", fr: "ciel" },
    VocabEntry { es: "jardín", en: "garden", fr: "jardin" },
    VocabEntry { es: "puente", en: "bridge", fr: "pont" },
    VocabEntry { es: "caballo", en: "horse", fr: "cheval" },
    VocabEntry { es: "vaca", en: "cow", fr: "vache" },
    VocabEntry { es: "elefante", en: "elephant", fr: "éléphant" },
    VocabEntry { es: "chocolate", en: "chocolate", fr: "chocolat" },
];
