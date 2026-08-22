//! Juego de lectura en MAYÚSCULAS — para aprender a leer desde cero.
//!
//! Pedagogía: se empieza siempre en mayúscula porque no hay confusión b/d, p/q
//! y todas ocupan el mismo alto. Progresión:
//! - Nivel 1 (3 rondas): Vocales sueltas A E I O U — reconocer la letra grande.
//! - Nivel 2 (3 rondas): Sílabas directas MA, PA, SA, LA, TA — la base del método silábico.
//! - Nivel 3 (4 rondas): Palabras cortas en mayúscula con dibujo/pista — SOL, CASA, MESA...
//!
//! Cada ronda da feedback 1.3s y al final muestra marcador con opción de volver.

use bevy::prelude::*;
use rand::seq::SliceRandom;
use rand::Rng;

use super::{screen_background, spawn_button};
use crate::audio::{play_click, play_success, Sfx};
use crate::game::GameState;
use crate::i18n::tr;

// ---- Configuración ---------------------------------------------------------

const ROUNDS: usize = 10;
const FEEDBACK_SECONDS: f32 = 1.35;

// ---- Rondas ---------------------------------------------------------------

enum UpperRound {
    /// Vocal suelta: encuentra la misma letra entre 4.
    FindVowel { target: char, options: [char; 4], correct: usize },
    /// Sílaba directa: encuentra MA, PA...
    FindSyllable { target: String, options: [String; 4], correct: usize },
    /// Palabra corta en mayúscula con pista (emoji + texto).
    WordWithClue { word: String, clue: String, emoji: String, options: [String; 4], correct: usize },
}

// ---- Sesión ---------------------------------------------------------------

#[derive(Resource)]
pub struct UppercaseSession {
    rounds: Vec<UpperRound>,
    index: usize,
    correct: usize,
    wrong: usize,
    selected: Option<usize>,
    feedback: bool,
    feedback_timer: f32,
    done: bool,
}

impl UppercaseSession {
    fn current(&self) -> &UpperRound { &self.rounds[self.index] }
    fn prompt_text(&self) -> String {
        match self.current() {
            UpperRound::FindVowel { target, .. } => tr("Busca la letra: {L}").replace("{L}", &target.to_string()),
            UpperRound::FindSyllable { target, .. } => tr("Busca la sílaba: {S}").replace("{S}", target),
            UpperRound::WordWithClue { clue, emoji, .. } => format!("{emoji}  {clue}"),
        }
    }
    fn answer_text(&self) -> String {
        match self.current() {
            UpperRound::FindVowel { target, .. } => target.to_string(),
            UpperRound::FindSyllable { target, .. } => target.clone(),
            UpperRound::WordWithClue { word, .. } => word.clone(),
        }
    }
    fn big_display(&self) -> String {
        match self.current() {
            UpperRound::FindVowel { target, .. } => target.to_string(),
            UpperRound::FindSyllable { target, .. } => target.clone(),
            UpperRound::WordWithClue { word, emoji, .. } => format!("{word}\n{emoji}"),
        }
    }
}

// ---- UI ------------------------------------------------------------------

#[derive(Component)]
pub struct UppercaseUiRoot;
#[derive(Component)]
pub struct UpperText(UpperField);
#[derive(Clone, Copy, PartialEq, Eq)]
enum UpperField { Title, Prompt, BigLetter, Progress, Feedback, ResultTitle, ResultDetail }
#[derive(Component)]
pub struct UpperOptionText(pub usize);
#[derive(Component)]
pub struct UpperOptionButton(pub usize);
#[derive(Component)]
pub struct UpperResultBox;
#[derive(Component)]
pub struct UpperBackButton;

pub struct UppercasePlugin;
impl Plugin for UppercasePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::UppercasePractice), spawn_uppercase_ui)
            .add_systems(OnExit(GameState::UppercasePractice), cleanup_uppercase)
            .add_systems(Update, update_uppercase.run_if(in_state(GameState::UppercasePractice)));
    }
}

const OPTION_LETTERS: [char; 4] = ['A','B','C','D'];
const VOWELS: [char; 5] = ['A','E','I','O','U'];
const SYLLABLES: [&str; 12] = ["MA","PA","SA","LA","TA","ME","PE","SE","MI","PI","MO","PO"];
const WORDS: [(&str, &str, &str); 20] = [
    ("SOL", "Brilla en el cielo ☀️", "☀️"),
    ("LUNA", "Sale por la noche 🌙", "🌙"),
    ("CASA", "Donde vives 🏠", "🏠"),
    ("MESA", "Donde comes 🍽️", "🪑"),
    ("GATO", "Hace MIAU 🐱", "🐱"),
    ("PERRO", "Hace GUAU 🐶", "🐶"),
    ("PATO", "Hace CUAC 🦆", "🦆"),
    ("PEZ", "Nada en el agua 🐟", "🐟"),
    ("PAN", "Se hace con harina 🍞", "🍞"),
    ("FLOR", "Huele muy bien 🌸", "🌸"),
    ("ARBOL", "Tiene tronco y hojas 🌳", "🌳"),
    ("LIBRO", "Se lee 📚", "📚"),
    ("LAPIZ", "Para escribir ✏️", "✏️"),
    ("PELOTA", "Se bota ⚽", "⚽"),
    ("MAMA", "Te quiere mucho 👩", "👩"),
    ("PAPA", "Te cuida 👨", "👨"),
    ("AGUA", "Se bebe 💧", "💧"),
    ("LECHE", "La da la vaca 🥛", "🥛"),
    ("NUBE", "Blanca en el cielo ☁️", "☁️"),
    ("AVION", "Vuela ✈️", "✈️"),
];

fn upper_text(parent: &mut ChildSpawnerCommands, field: UpperField, text: &str, size: f32, font: &Handle<Font>) {
    parent.spawn((
        UpperText(field),
        Text::new(text.to_string()),
        TextFont { font: font.clone(), font_size: size, ..default() },
        TextColor(Color::WHITE),
        TextLayout { linebreak: LineBreak::WordBoundary, ..default() },
        Node { max_width: Val::Px(720.0), ..default() },
    ));
}

fn build_rounds() -> Vec<UpperRound> {
    let mut rng = rand::thread_rng();
    let mut rounds = Vec::with_capacity(ROUNDS);
    // 3 vocales
    for _ in 0..3 {
        let target = VOWELS[rng.gen_range(0..VOWELS.len())];
        let mut opts = vec![target];
        while opts.len() < 4 {
            let c = VOWELS[rng.gen_range(0..VOWELS.len())];
            if !opts.contains(&c) { opts.push(c); }
        }
        opts.shuffle(&mut rng);
        let correct = opts.iter().position(|&c| c==target).unwrap();
        rounds.push(UpperRound::FindVowel { target, options: opts.try_into().unwrap(), correct });
    }
    // 3 sílabas
    for _ in 0..3 {
        let target = SYLLABLES[rng.gen_range(0..SYLLABLES.len())].to_string();
        let mut opts = vec![target.clone()];
        while opts.len() < 4 {
            let s = SYLLABLES[rng.gen_range(0..SYLLABLES.len())].to_string();
            if !opts.contains(&s) { opts.push(s); }
        }
        opts.shuffle(&mut rng);
        let correct = opts.iter().position(|o| o==&target).unwrap();
        rounds.push(UpperRound::FindSyllable { target, options: opts.try_into().unwrap(), correct });
    }
    // 4 palabras
    for _ in 0..4 {
        let (w,c,e) = WORDS[rng.gen_range(0..WORDS.len())];
        let mut opts = vec![w.to_string()];
        while opts.len() < 4 {
            let (ow,_,_) = WORDS[rng.gen_range(0..WORDS.len())];
            if !opts.contains(&ow.to_string()) { opts.push(ow.to_string()); }
        }
        opts.shuffle(&mut rng);
        let correct = opts.iter().position(|o| o==w).unwrap();
        rounds.push(UpperRound::WordWithClue { word: w.to_string(), clue: c.to_string(), emoji: e.to_string(), options: opts.try_into().unwrap(), correct });
    }
    rounds.shuffle(&mut rng);
    rounds
}

fn spawn_uppercase_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands.insert_resource(UppercaseSession {
        rounds: build_rounds(),
        index: 0, correct: 0, wrong: 0, selected: None, feedback: false, feedback_timer: 0.0, done: false,
    });
    commands.spawn((
        UppercaseUiRoot,
        Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
        screen_background(), Visibility::Visible, ZIndex(30),
    )).with_children(|overlay| {
        overlay.spawn((
            Node { flex_direction: FlexDirection::Column, width: Val::Px(760.0), padding: UiRect::axes(Val::Px(28.0), Val::Px(24.0)), row_gap: Val::Px(12.0), align_items: AlignItems::Center, ..default() },
            BackgroundColor(Color::srgba(0.07,0.09,0.18,0.96)), BorderRadius::all(Val::Px(16.0)),
        )).with_children(|panel| {
            upper_text(panel, UpperField::Title, "LEER EN MAYÚSCULAS", 28.0, &font);
            panel.spawn((
                UpperText(UpperField::BigLetter),
                Text::new("A"),
                TextFont { font: font.clone(), font_size: 96.0, ..default() },
                TextColor(Color::srgb(1.0,0.90,0.40)),
                Node { min_height: Val::Px(110.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
            ));
            upper_text(panel, UpperField::Prompt, "", 22.0, &font);
            for i in 0..4 {
                panel.spawn((
                    Button, UpperOptionButton(i),
                    Node { width: Val::Px(640.0), height: Val::Px(52.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() },
                    BackgroundColor(Color::srgb(0.15,0.18,0.28)), BorderColor(Color::srgb(0.50,0.55,0.70)), BorderRadius::all(Val::Px(10.0)),
                )).with_children(|opt| {
                    opt.spawn((UpperOptionText(i), Text::new(""), TextFont { font: font.clone(), font_size: 28.0, ..default() }, TextColor(Color::WHITE)));
                });
            }
            upper_text(panel, UpperField::Progress, "", 16.0, &font);
            upper_text(panel, UpperField::Feedback, "", 22.0, &font);
            panel.spawn((UpperResultBox, Node { flex_direction: FlexDirection::Column, align_items: AlignItems::Center, row_gap: Val::Px(10.0), ..default() }, Visibility::Hidden)).with_children(|res| {
                upper_text(res, UpperField::ResultTitle, "", 26.0, &font);
                upper_text(res, UpperField::ResultDetail, "", 18.0, &font);
                spawn_button(res, "Volver a Lengua", UpperBackButton, &font);
            });
        });
    });
}

fn cleanup_uppercase(mut commands: Commands, roots: Query<Entity, With<UppercaseUiRoot>>) {
    for r in &roots { commands.entity(r).despawn(); }
    commands.remove_resource::<UppercaseSession>();
}

const OPT_NEUTRAL: Color = Color::srgb(0.15,0.18,0.28);
const OPT_DIM: Color = Color::srgb(0.10,0.12,0.20);
const OPT_OK: Color = Color::srgb(0.15,0.42,0.25);
const OPT_BAD: Color = Color::srgb(0.50,0.20,0.20);

fn update_uppercase(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut commands: Commands,
    sfx: Res<Sfx>,
    session: Option<ResMut<UppercaseSession>>,
    mut texts: Query<(&UpperText, &mut Text, &mut TextColor, &mut Visibility)>,
    mut opt_texts: Query<(&UpperOptionText, &mut Text), (Without<UpperText>, Without<UpperOptionButton>, Without<UpperResultBox>)>,
    mut opt_bg: Query<(&UpperOptionButton, &mut BackgroundColor), Without<UpperText>>,
    mut opt_vis: Query<(&UpperOptionButton, &mut Visibility), Without<UpperText>>,
    clicks: Query<(&Interaction, &UpperOptionButton), (Changed<Interaction>, Without<UpperText>)>,
    mut result_box: Query<&mut Visibility, (With<UpperResultBox>, Without<UpperText>, Without<UpperOptionButton>)>,
    back: Query<&Interaction, (Changed<Interaction>, With<UpperBackButton>)>,
) {
    let dt = time.delta_secs();
    let mut session = match session { Some(s) => s, None => { commands.insert_resource(UppercaseSession { rounds: build_rounds(), index:0, correct:0, wrong:0, selected:None, feedback:false, feedback_timer:0.0, done:false }); return; } };

    if keys.just_pressed(KeyCode::Escape) { commands.set_state(GameState::LanguageMenu); return; }

    if session.done {
        let close = back.single().map_or(false, |i| *i==Interaction::Pressed) || keys.just_pressed(KeyCode::Enter);
        if close { commands.set_state(GameState::LanguageMenu); return; }
        for (field, mut txt, mut col, mut vis) in &mut texts {
            match field.0 {
                UpperField::ResultTitle => { *txt = Text::new(if session.correct >= ROUNDS/2 { "¡Muy bien!" } else { "¡Sigue practicando!" }); *col = TextColor(if session.correct >= ROUNDS/2 { Color::srgb(0.40,0.90,0.50)} else {Color::srgb(0.95,0.55,0.30)}); *vis=Visibility::Visible; },
                UpperField::ResultDetail => { *txt = Text::new(format!("Aciertos: {} · Fallos: {} de {}", session.correct, session.wrong, ROUNDS)); *vis=Visibility::Visible; },
                _=>{}
            }
        }
        if let Ok(mut v) = result_box.single_mut() { *v=Visibility::Visible; }
        return;
    }

    if session.feedback {
        session.feedback_timer -= dt;
        let correct = match session.current() {
            UpperRound::FindVowel { correct, .. } => *correct,
            UpperRound::FindSyllable { correct, .. } => *correct,
            UpperRound::WordWithClue { correct, .. } => *correct,
        };
        for (btn, mut bg) in &mut opt_bg {
            *bg = BackgroundColor(if btn.0==correct {OPT_OK} else if Some(btn.0)==session.selected {OPT_BAD} else {OPT_DIM});
        }
        if session.feedback_timer <= 0.0 {
            session.feedback=false; session.selected=None; session.index+=1;
            if session.index >= ROUNDS { session.done=true; return; }
        }
    } else {
        let mut chosen: Option<usize> = None;
        for (inter, btn) in &clicks { if *inter==Interaction::Pressed { chosen=Some(btn.0); break; } }
        if chosen.is_none() {
            for (idx, code) in [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4].iter().enumerate() {
                if keys.just_pressed(*code) { chosen=Some(idx); break; }
            }
        }
        if let Some(idx) = chosen {
            let correct = match session.current() {
                UpperRound::FindVowel { correct, .. } => *correct,
                UpperRound::FindSyllable { correct, .. } => *correct,
                UpperRound::WordWithClue { correct, .. } => *correct,
            };
            if idx==correct { session.correct+=1; play_success(&mut commands, &sfx); } else { session.wrong+=1; }
            session.selected=Some(idx); session.feedback=true; session.feedback_timer=FEEDBACK_SECONDS;
            play_click(&mut commands, &sfx);
        }
        for (_btn, mut bg) in &mut opt_bg { *bg=BackgroundColor(OPT_NEUTRAL); }
    }

    // Textos
    let round = session.current();
    for (field, mut txt, mut col, mut vis) in &mut texts {
        match field.0 {
            UpperField::Prompt => { *txt=Text::new(session.prompt_text()); *vis=Visibility::Visible; },
            UpperField::BigLetter => { *txt=Text::new(session.big_display()); *vis=Visibility::Visible; },
            UpperField::Progress => { *txt=Text::new(format!("Actividad {}/{}  ·  Aciertos: {}  ·  Fallos: {}", session.index+1, ROUNDS, session.correct, session.wrong)); *vis=Visibility::Visible; },
            UpperField::Feedback => {
                if session.feedback {
                    let ok = session.selected == Some(match round { UpperRound::FindVowel { correct, .. }=>*correct, UpperRound::FindSyllable { correct, .. }=>*correct, UpperRound::WordWithClue { correct, .. }=>*correct });
                    if ok { *txt=Text::new("¡Correcto!"); *col=TextColor(Color::srgb(0.40,0.90,0.50)); } else { *txt=Text::new(format!("Era: {}", session.answer_text())); *col=TextColor(Color::srgb(0.95,0.40,0.40)); }
                    *vis=Visibility::Visible;
                } else { *vis=Visibility::Hidden; }
            },
            _=>{}
        }
    }
    for (_btn, mut vis) in &mut opt_vis { *vis=Visibility::Visible; }
    match round {
        UpperRound::FindVowel { options, .. } => {
            for (field, mut txt) in &mut opt_texts {
                *txt = Text::new(format!("{}) {}", OPTION_LETTERS[field.0], options[field.0]));
            }
        },
        UpperRound::FindSyllable { options, .. } => {
            for (field, mut txt) in &mut opt_texts {
                *txt = Text::new(format!("{}) {}", OPTION_LETTERS[field.0], options[field.0]));
            }
        },
        UpperRound::WordWithClue { options, .. } => {
            for (field, mut txt) in &mut opt_texts {
                *txt = Text::new(format!("{}) {}", OPTION_LETTERS[field.0], options[field.0]));
            }
        },
    }
    for (inter, _) in &clicks { if *inter==Interaction::Pressed { play_click(&mut commands, &sfx); break; } }
}
