//! Anagramas (Lengua) — ordena las letras para formar la palabra.
//!
//! 10 rondas: se muestra palabra desordenada y 4 opciones. Trilingüe.

use bevy::prelude::*;
use rand::seq::SliceRandom;

use super::{screen_background, spawn_button};
use crate::audio::{play_click, play_success, Sfx};
use crate::game::GameState;
use crate::i18n::tr;

const ROUNDS: usize = 10;
const FEEDBACK_SECONDS: f32 = 1.4;

#[derive(Clone, Copy)]
struct AnagramQuestion {
    scrambled: &'static str,
    options: [&'static str; 4],
    correct: usize,
}

#[derive(Resource)]
pub struct AnagramSession {
    rounds: Vec<AnagramQuestion>,
    index: usize,
    correct: usize,
    wrong: usize,
    selected: Option<usize>,
    feedback: bool,
    feedback_timer: f32,
    done: bool,
}

#[derive(Component)]
pub struct AnagramUiRoot;
#[derive(Component)]
pub struct AnagramText(AnagramField);
#[derive(Clone, Copy, PartialEq, Eq)]
enum AnagramField {
    Title,
    Question,
    Progress,
    Feedback,
    ResultTitle,
    ResultDetail,
}
#[derive(Component)]
pub struct AnagramOptionText(pub usize);
#[derive(Component)]
pub struct AnagramOptionButton(pub usize);
#[derive(Component)]
pub struct AnagramResultBox;
#[derive(Component)]
pub struct AnagramBackButton;

pub struct AnagramPlugin;
impl Plugin for AnagramPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::AnagramPractice), spawn_anagram_ui)
            .add_systems(OnExit(GameState::AnagramPractice), cleanup_anagram)
            .add_systems(Update, update_anagram.run_if(in_state(GameState::AnagramPractice)));
    }
}
const OPTION_LETTERS: [char; 4] = ['A', 'B', 'C', 'D'];
fn anagram_text(parent: &mut ChildSpawnerCommands, field: AnagramField, text: &str, size: f32, font: &Handle<Font>) {
    parent.spawn((
        AnagramText(field),
        Text::new(text.to_string()),
        TextFont { font: font.clone(), font_size: size, ..default() },
        TextColor(Color::WHITE),
        TextLayout { linebreak: LineBreak::WordBoundary, ..default() },
        Node { max_width: Val::Px(700.0), ..default() },
    ));
}
fn anagram_option_text(parent: &mut ChildSpawnerCommands, index: usize, size: f32, font: &Handle<Font>) {
    parent.spawn((AnagramOptionText(index), Text::new(String::new()), TextFont { font: font.clone(), font_size: size, ..default() }, TextColor(Color::WHITE)));
}
fn spawn_anagram_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands.insert_resource(AnagramSession { rounds: build_rounds(), index: 0, correct: 0, wrong: 0, selected: None, feedback: false, feedback_timer: 0.0, done: false });
    commands
        .spawn((
            AnagramUiRoot,
            Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
            screen_background(),
            Visibility::Visible,
            ZIndex(30),
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    Node { flex_direction: FlexDirection::Column, width: Val::Px(680.0), padding: UiRect::axes(Val::Px(28.0), Val::Px(24.0)), row_gap: Val::Px(14.0), align_items: AlignItems::Center, ..default() },
                    BackgroundColor(Color::srgba(0.07, 0.09, 0.18, 0.96)),
                    BorderRadius::all(Val::Px(16.0)),
                ))
                .with_children(|panel| {
                    anagram_text(panel, AnagramField::Title, "", 28.0, &font);
                    anagram_text(panel, AnagramField::Question, "", 30.0, &font);
                    for index in 0..4 {
                        panel.spawn((Button, AnagramOptionButton(index), Node { width: Val::Px(600.0), height: Val::Px(46.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.15, 0.18, 0.28)), BorderColor(Color::srgb(0.50, 0.55, 0.70)), BorderRadius::all(Val::Px(8.0)))).with_children(|o| anagram_option_text(o, index, 21.0, &font));
                    }
                    anagram_text(panel, AnagramField::Progress, "", 17.0, &font);
                    anagram_text(panel, AnagramField::Feedback, "", 22.0, &font);
                    panel.spawn((AnagramResultBox, Node { flex_direction: FlexDirection::Column, align_items: AlignItems::Center, row_gap: Val::Px(10.0), ..default() }, Visibility::Hidden)).with_children(|r| { anagram_text(r, AnagramField::ResultTitle, "", 26.0, &font); anagram_text(r, AnagramField::ResultDetail, "", 20.0, &font); spawn_button(r, "Volver a Lengua", AnagramBackButton, &font); });
                });
        });
}
fn cleanup_anagram(mut commands: Commands, roots: Query<Entity, With<AnagramUiRoot>>) {
    for root in &roots { commands.entity(root).despawn(); }
    commands.remove_resource::<AnagramSession>();
}
fn build_rounds() -> Vec<AnagramQuestion> {
    let mut rng = rand::thread_rng();
    bank().choose_multiple(&mut rng, ROUNDS).copied().collect()
}
fn bank() -> &'static [AnagramQuestion] {
    match crate::i18n::language() {
        crate::i18n::Language::En => &ANAGRAMS_EN,
        crate::i18n::Language::Fr => &ANAGRAMS_FR,
        _ => &ANAGRAMS,
    }
}
const OPTION_NEUTRAL: Color = Color::srgb(0.15, 0.18, 0.28);
const OPTION_DIM: Color = Color::srgb(0.10, 0.12, 0.20);
const OPTION_CORRECT: Color = Color::srgb(0.15, 0.42, 0.25);
const OPTION_WRONG: Color = Color::srgb(0.50, 0.20, 0.20);
fn update_anagram(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut commands: Commands,
    sfx: Res<Sfx>,
    session: Option<ResMut<AnagramSession>>,
    mut texts: Query<(&AnagramText, &mut Text, &mut TextColor, &mut Visibility), (Without<AnagramOptionText>, Without<AnagramOptionButton>, Without<AnagramResultBox>)>,
    mut option_texts: Query<(&AnagramOptionText, &mut Text), (Without<AnagramText>, Without<AnagramOptionButton>, Without<AnagramResultBox>)>,
    mut option_colors: Query<(&AnagramOptionButton, &mut BackgroundColor), Without<AnagramText>>,
    option_clicks: Query<(&Interaction, &AnagramOptionButton), (Changed<Interaction>, Without<AnagramText>)>,
    mut result_box: Query<&mut Visibility, (With<AnagramResultBox>, Without<AnagramText>, Without<AnagramOptionButton>)>,
    close_clicks: Query<&Interaction, (Changed<Interaction>, With<AnagramBackButton>)>,
) {
    let dt = time.delta_secs();
    let mut session = match session { Some(s) => s, None => { commands.insert_resource(AnagramSession { rounds: build_rounds(), index: 0, correct: 0, wrong: 0, selected: None, feedback: false, feedback_timer: 0.0, done: false }); return; } };
    if keys.just_pressed(KeyCode::Escape) { commands.set_state(GameState::LanguageMenu); return; }
    if session.done {
        let close = close_clicks.single().map_or(false, |i| *i == Interaction::Pressed) || keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::KeyQ);
        if close { commands.set_state(GameState::LanguageMenu); return; }
        for (field, mut text, mut color, mut vis) in &mut texts {
            match field.0 {
                AnagramField::ResultTitle => { *text = Text::new(tr(if session.correct >= ROUNDS / 2 { "¡Muy bien!" } else { "¡Sigue practicando!" })); *color = TextColor(if session.correct >= ROUNDS / 2 { Color::srgb(0.40, 0.90, 0.50) } else { Color::srgb(0.95, 0.55, 0.30) }); *vis = Visibility::Visible; }
                AnagramField::ResultDetail => { *text = Text::new(tr("Aciertos: {} · Fallos: {}  de {} palabras").replace("{}", &session.correct.to_string()).replace("{}", &session.wrong.to_string()).replace("{}", &ROUNDS.to_string())); *vis = Visibility::Visible; }
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
            AnagramField::Title => { *text = Text::new(tr("ANAGRAMAS")); *color = TextColor(Color::srgb(0.80, 0.75, 1.0)); *vis = Visibility::Visible; }
            AnagramField::Question => { *text = Text::new(format!("Ordena: \"{}\"", question.scrambled)); *vis = Visibility::Visible; }
            AnagramField::Progress => { *text = Text::new(tr("Palabra {}/{}  ·  Aciertos: {}  ·  Fallos: {}").replace("{}", &(session.index + 1).to_string()).replace("{}", &ROUNDS.to_string()).replace("{}", &session.correct.to_string()).replace("{}", &session.wrong.to_string())); *vis = Visibility::Visible; }
            AnagramField::Feedback => { if session.feedback { let ok = session.selected == Some(question.correct); if ok { *text = Text::new(tr("¡Correcto!")); *color = TextColor(Color::srgb(0.40, 0.90, 0.50)); } else { *text = Text::new(tr("Incorrecto — era {}) {}").replace("{}", &OPTION_LETTERS[question.correct].to_string()).replace("{}", &question.options[question.correct])); *color = TextColor(Color::srgb(0.95, 0.40, 0.40)); } *vis = Visibility::Visible; } else { *vis = Visibility::Hidden; } }
            _ => {}
        }
    }
    for (field, mut text) in &mut option_texts { *text = Text::new(format!("{}) {}", OPTION_LETTERS[field.0], question.options[field.0])); }
    for (interaction, _button) in &option_clicks { if *interaction == Interaction::Pressed { play_click(&mut commands, &sfx); break; } }
}

const ANAGRAMS: [AnagramQuestion; 30] = [
    AnagramQuestion { scrambled: "asac", options: ["casa", "saca", "asca", "acsa"], correct: 0 },
    AnagramQuestion { scrambled: "los", options: ["sol", "los", "slo", "ols"], correct: 0 },
    AnagramQuestion { scrambled: "obut", options: ["tubo", "boto", "obut", "tobu"], correct: 0 },
    AnagramQuestion { scrambled: "orem", options: ["amor", "roma", "mora", "remo"], correct: 0 },
    AnagramQuestion { scrambled: "odra", options: ["roda", "ardo", "dora", "roda"], correct: 0 },
    AnagramQuestion { scrambled: "alob", options: ["bola", "loba", "balo", "albo"], correct: 0 },
    AnagramQuestion { scrambled: "ocra", options: ["arco", "cora", "roca", "ocra"], correct: 2 },
    AnagramQuestion { scrambled: "otag", options: ["gato", "toga", "gota", "tago"], correct: 0 },
    AnagramQuestion { scrambled: "orep", options: ["pero", "rope", "pore", "repo"], correct: 0 },
    AnagramQuestion { scrambled: "alpa", options: ["pala", "lapa", "pala", "alpa"], correct: 0 },
    AnagramQuestion { scrambled: "semo", options: ["mesa", "semo", "emos", "mase"], correct: 0 },
    AnagramQuestion { scrambled: "orel", options: ["lore", "role", "orel", "lero"], correct: 0 },
    AnagramQuestion { scrambled: "odna", options: ["noda", "dona", "onda", "ando"], correct: 0 },
    AnagramQuestion { scrambled: "arpa", options: ["rapa", "para", "arpa", "paar"], correct: 1 },
    AnagramQuestion { scrambled: "late", options: ["tela", "late", "leta", "etla"], correct: 0 },
    AnagramQuestion { scrambled: "osac", options: ["caso", "saco", "cosa", "osca"], correct: 2 },
    AnagramQuestion { scrambled: "onel", options: ["león", "lone", "enol", "onel"], correct: 0 },
    AnagramQuestion { scrambled: "orba", options: ["obra", "baro", "roba", "bora"], correct: 0 },
    AnagramQuestion { scrambled: "odal", options: ["lado", "dalo", "odal", "alod"], correct: 0 },
    AnagramQuestion { scrambled: "ocar", options: ["arco", "cora", "roca", "caro"], correct: 3 },
    AnagramQuestion { scrambled: "etar", options: ["arte", "reta", "tare", "rate"], correct: 0 },
    AnagramQuestion { scrambled: "dnae", options: ["nada", "dana", "anda", "dane"], correct: 2 },
    AnagramQuestion { scrambled: "ocna", options: ["cona", "noca", "cano", "anco"], correct: 0 },
    AnagramQuestion { scrambled: "alpa", options: ["pala", "lapa", "alpa", "paal"], correct: 0 },
    AnagramQuestion { scrambled: "sola", options: ["losa", "sola", "also", "olas"], correct: 3 },
    AnagramQuestion { scrambled: "anap", options: ["pana", "napa", "pana", "anap"], correct: 0 },
    AnagramQuestion { scrambled: "oter", options: ["reto", "tero", "rote", "orte"], correct: 0 },
    AnagramQuestion { scrambled: "oloc", options: ["loco", "colo", "ocol", "cool"], correct: 0 },
    AnagramQuestion { scrambled: "adar", options: ["rada", "dara", "arda", "adra"], correct: 0 },
    AnagramQuestion { scrambled: "orne", options: ["reno", "rone", "nero", "enro"], correct: 0 },
];
const ANAGRAMS_EN: [AnagramQuestion; 30] = [
    AnagramQuestion { scrambled: "esuoh", options: ["house", "shou e", "huso e", "esuoh"], correct: 0 },
    AnagramQuestion { scrambled: "nus", options: ["sun", "nus", "uns", "snu"], correct: 0 },
    AnagramQuestion { scrambled: "tac", options: ["cat", "act", "tac", "cta"], correct: 0 },
    AnagramQuestion { scrambled: "god", options: ["dog", "god", "gdo", "dgo"], correct: 0 },
    AnagramQuestion { scrambled: "elif", options: ["file", "life", "lief", "elif"], correct: 1 },
    AnagramQuestion { scrambled: "tae", options: ["eat", "tea", "ate", "eta"], correct: 1 },
    AnagramQuestion { scrambled: "pots", options: ["stop", "pots", "tops", "spot"], correct: 0 },
    AnagramQuestion { scrambled: "star", options: ["rats", "arts", "star", "tars"], correct: 2 },
    AnagramQuestion { scrambled: "elivo", options: ["olive", "voile", "viole", "elivo"], correct: 0 },
    AnagramQuestion { scrambled: "dlo", options: ["old", "dol", "lod", "dlo"], correct: 0 },
    AnagramQuestion { scrambled: "tae", options: ["tea", "eat", "ate", "eta"], correct: 0 },
    AnagramQuestion { scrambled: "nets", options: ["sent", "nest", "tens", "nets"], correct: 1 },
    AnagramQuestion { scrambled: "evil", options: ["vile", "veil", "evil", "live"], correct: 3 },
    AnagramQuestion { scrambled: "peal", options: ["leap", "pale", "peal", "plea"], correct: 0 },
    AnagramQuestion { scrambled: "save", options: ["vase", "save", "aves", "vaes"], correct: 1 },
    AnagramQuestion { scrambled: "tinsel", options: ["listen", "silent", "tinsel", "inlets"], correct: 0 },
    AnagramQuestion { scrambled: "dorm", options: ["word", "mord", "dorm", "mrod"], correct: 0 },
    AnagramQuestion { scrambled: "care", options: ["race", "care", "acre", "reac"], correct: 0 },
    AnagramQuestion { scrambled: "night", options: ["thing", "night", "thign", "ghint"], correct: 0 },
    AnagramQuestion { scrambled: "angel", options: ["angle", "angel", "glean", "lange"], correct: 0 },
    AnagramQuestion { scrambled: "stone", options: ["tones", "stone", "onset", "notes"], correct: 1 },
    AnagramQuestion { scrambled: "heart", options: ["earth", "heart", "hater", "rathe"], correct: 0 },
    AnagramQuestion { scrambled: "fired", options: ["fried", "fired", "drefi", "rifed"], correct: 0 },
    AnagramQuestion { scrambled: "spear", options: ["pares", "spear", "reaps", "spare"], correct: 3 },
    AnagramQuestion { scrambled: "regal", options: ["large", "regal", "glare", "lager"], correct: 0 },
    AnagramQuestion { scrambled: "least", options: ["steal", "least", "slate", "stale"], correct: 0 },
    AnagramQuestion { scrambled: "trace", options: ["crate", "trace", "react", "cater"], correct: 1 },
    AnagramQuestion { scrambled: "lemon", options: ["melon", "lemon", "lomen", "monel"], correct: 0 },
    AnagramQuestion { scrambled: "brid", options: ["bird", "brid", "drib", "rbid"], correct: 0 },
    AnagramQuestion { scrambled: "form", options: ["from", "form", "morf", "from"], correct: 0 },
];
const ANAGRAMS_FR: [AnagramQuestion; 30] = [
    AnagramQuestion { scrambled: "nosiam", options: ["maison", "aimons", "nosiam", "manios"], correct: 0 },
    AnagramQuestion { scrambled: "leios", options: ["soleil", "leios", "isole", "loise"], correct: 0 },
    AnagramQuestion { scrambled: "nuLe", options: ["lune", "nuel", "nuLe", "unel"], correct: 0 },
    AnagramQuestion { scrambled: "rem", options: ["mer", "rem", "erm", "mre"], correct: 0 },
    AnagramQuestion { scrambled: "tcha", options: ["chat", "tcha", "acht", "chta"], correct: 0 },
    AnagramQuestion { scrambled: "niche", options: ["chien", "chine", "niche", "chine"], correct: 0 },
    AnagramQuestion { scrambled: "rdanac", options: ["canard", "rdanac", "carnad", "nardac"], correct: 0 },
    AnagramQuestion { scrambled: "nosipos", options: ["poisson", "nosipos", "sonipos", "posinos"], correct: 0 },
    AnagramQuestion { scrambled: "napi", options: ["pain", "napi", "pina", "anip"], correct: 0 },
    AnagramQuestion { scrambled: "elbat", options: ["table", "elbat", "bleta", "tabel"], correct: 0 },
    AnagramQuestion { scrambled: "siahec", options: ["chaise", "siahec", "chiesa", "sahice"], correct: 0 },
    AnagramQuestion { scrambled: "ruelf", options: ["fleur", "ruelf", "fluer", "luref"], correct: 0 },
    AnagramQuestion { scrambled: "rebra", options: ["arbre", "rebra", "barre", "berra"], correct: 0 },
    AnagramQuestion { scrambled: "guero", options: ["rouge", "guero", "rugoe", "oguer"], correct: 0 },
    AnagramQuestion { scrambled: "uleb", options: ["bleu", "uleb", "lube", "bule"], correct: 0 },
    AnagramQuestion { scrambled: "uae", options: ["eau", "uae", "aue", "eau"], correct: 0 },
    AnagramQuestion { scrambled: "vlier", options: ["livre", "vlier", "vrile", "liver"], correct: 0 },
    AnagramQuestion { scrambled: "yonarc", options: ["crayon", "yonarc", "coryan", "narcoy"], correct: 0 },
    AnagramQuestion { scrambled: "lonabl", options: ["ballon", "lonabl", "banoll", "labonl"], correct: 0 },
    AnagramQuestion { scrambled: "ghorelo", options: ["horloge", "ghorelo", "logerho", "horelgo"], correct: 0 },
    AnagramQuestion { scrambled: "trope", options: ["porte", "trope", "ropte", "potre"], correct: 0 },
    AnagramQuestion { scrambled: "treenfe", options: ["fenêtre", "treenfe", "ferente", "tenrefe"], correct: 0 },
    AnagramQuestion { scrambled: "guane", options: ["nuage", "guane", "augne", "nguea"], correct: 0 },
    AnagramQuestion { scrambled: "upiel", options: ["pluie", "upiel", "lupie", "pieul"], correct: 0 },
    AnagramQuestion { scrambled: "ufe", options: ["feu", "ufe", "fue", "uef"], correct: 0 },
    AnagramQuestion { scrambled: "reter", options: ["terre", "reter", "erret", "terer"], correct: 0 },
    AnagramQuestion { scrambled: "tnev", options: ["vent", "tnev", "vten", "ntve"], correct: 0 },
    AnagramQuestion { scrambled: "velach", options: ["cheval", "velach", "chalve", "lache v"], correct: 0 },
    AnagramQuestion { scrambled: "vache", options: ["vache", "chave", "cache", "vache"], correct: 0 },
    AnagramQuestion { scrambled: "le pou", options: ["poule", "le pou", "pouel", "loupe"], correct: 0 },
];
