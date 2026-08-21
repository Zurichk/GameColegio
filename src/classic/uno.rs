//! Uno 2-4 jugadores — colores, números, +2, bloqueo.

use bevy::prelude::*;
use rand::seq::SliceRandom;

use crate::game::GameState;
use crate::learning::screen_background;
use crate::i18n::tr;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Color { Red, Green, Blue, Yellow }

#[derive(Clone, Copy, PartialEq, Eq)]
enum Card { Number(Color, u8), Plus2(Color), Skip(Color) }

impl Card {
    fn can_play_on(&self, top: Card) -> bool {
        match (self, top) {
            (Card::Number(c,n), Card::Number(tc,tn)) => c==tc || n==tn,
            (Card::Plus2(c), Card::Plus2(tc)) => c==tc,
            (Card::Skip(c), Card::Skip(tc)) => c==tc,
            (Card::Number(c,_), Card::Plus2(tc)) => c==tc,
            (Card::Number(c,_), Card::Skip(tc)) => c==tc,
            (Card::Plus2(c), Card::Number(tc,_)) => c==tc,
            (Card::Skip(c), Card::Number(tc,_)) => c==tc,
            _ => false,
        }
    }
    fn label(&self) -> String {
        match self {
            Card::Number(c,n) => format!("{:?} {}", c, n),
            Card::Plus2(c) => format!("{:?} +2", c),
            Card::Skip(c) => format!("{:?} Skip", c),
        }
    }
}

#[derive(Resource, Clone)]
struct UnoSession {
    hands: Vec<Vec<Card>>,
    deck: Vec<Card>,
    discard: Card,
    turn: usize,
    players: usize,
    winner: Option<usize>,
}

impl UnoSession {
    fn new(players: usize) -> Self {
        let mut deck = Vec::new();
        for &c in [Color::Red, Color::Green, Color::Blue, Color::Yellow].iter() {
            for n in 0..=9 { deck.push(Card::Number(c,n)); }
            deck.push(Card::Plus2(c)); deck.push(Card::Skip(c));
        }
        deck.shuffle(&mut rand::thread_rng());
        let mut hands = vec![Vec::new(); players];
        for _ in 0..7 { for p in 0..players { if let Some(card)=deck.pop() { hands[p].push(card); } } }
        let discard = deck.pop().unwrap_or(Card::Number(Color::Red, 5));
        Self { hands, deck, discard, turn: 0, players, winner: None }
    }
}

#[derive(Component)]
struct UnoUiRoot;
#[derive(Component)]
struct UnoText(UnoField);
#[derive(Clone, Copy, PartialEq, Eq)]
enum UnoField { Title, Discard, Status, Hand }
#[derive(Component)]
struct UnoCardButton(usize);
#[derive(Component)]
struct UnoDrawButton;
#[derive(Component)]
struct UnoBackButton;

pub struct UnoPlugin;
impl Plugin for UnoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::UnoGame), spawn_uno)
            .add_systems(OnExit(GameState::UnoGame), cleanup_uno)
            .add_systems(Update, update_uno.run_if(in_state(GameState::UnoGame)));
    }
}

fn spawn_uno(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands.insert_resource(UnoSession::new(2));
    commands
        .spawn((UnoUiRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, screen_background(), Visibility::Visible, ZIndex(30)))
        .with_children(|overlay| {
            overlay.spawn((Node { flex_direction: FlexDirection::Column, width: Val::Px(720.0), padding: UiRect::axes(Val::Px(20.0), Val::Px(16.0)), row_gap: Val::Px(10.0), align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.07, 0.09, 0.18, 0.96)), BorderRadius::all(Val::Px(16.0)))).with_children(|panel| {
                panel.spawn((UnoText(UnoField::Title), Text::new("UNO 2 JUGADORES"), TextFont { font: font.clone(), font_size: 28.0, ..default() }, TextColor(Color::srgb(0.95, 0.85, 0.40))));
                panel.spawn((UnoText(UnoField::Discard), Text::new("Descartes: "), TextFont { font: font.clone(), font_size: 22.0, ..default() }, TextColor(Color::WHITE)));
                panel.spawn((UnoText(UnoField::Status), Text::new("Tu turno"), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE)));
                panel.spawn((UnoText(UnoField::Hand), Text::new("Tu mano:"), TextFont { font: font.clone(), font_size: 16.0, ..default() }, TextColor(Color::WHITE)));
                panel.spawn(Node { flex_direction: FlexDirection::Row, flex_wrap: FlexWrap::Wrap, width: Val::Px(680.0), justify_content: JustifyContent::Center, column_gap: Val::Px(8.0), row_gap: Val::Px(8.0), ..default() }).with_children(|row| {
                    for i in 0..7 {
                        row.spawn((Button, UnoCardButton(i), Node { width: Val::Px(90.0), height: Val::Px(60.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(1.0)), ..default() }, BackgroundColor(Color::srgb(0.85, 0.80, 0.70)), BorderRadius::all(Val::Px(6.0)))).with_children(|b| { b.spawn((Text::new("".to_string()), TextFont { font: font.clone(), font_size: 14.0, ..default() }, TextColor(Color::BLACK))); });
                    }
                });
                panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(12.0), ..default() }).with_children(|row| {
                    row.spawn((Button, UnoDrawButton, Node { width: Val::Px(140.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.30, 0.40, 0.60)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Robar")), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE))); });
                    row.spawn((Button, UnoBackButton, Node { width: Val::Px(140.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Volver")), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE))); });
                });
            });
        });
}

fn cleanup_uno(mut commands: Commands, roots: Query<Entity, With<UnoUiRoot>>) {
    for root in &roots { commands.entity(root).despawn(); }
    commands.remove_resource::<UnoSession>();
}

fn update_uno(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut session: ResMut<UnoSession>,
    card_clicks: Query<(&Interaction, &UnoCardButton), (Changed<Interaction>, Without<UnoBackButton>)>,
    draw_clicks: Query<&Interaction, (Changed<Interaction>, With<UnoDrawButton>)>,
    back_clicks: Query<&Interaction, (Changed<Interaction>, With<UnoBackButton>)>,
    mut texts: Query<(&UnoText, &mut Text)>,
    mut card_query: Query<(&UnoCardButton, &mut BackgroundColor, &Children)>,
    mut card_texts: Query<&mut Text, Without<UnoText>>,
) {
    if keys.just_pressed(KeyCode::Escape) || back_clicks.single().map_or(false, |i| *i == Interaction::Pressed) { commands.set_state(GameState::ClassicMenu); return; }
    if session.winner.is_some() { return; }
    let mut played = false;
    for (interaction, btn) in &card_clicks {
        if *interaction == Interaction::Pressed {
            let idx = btn.0;
            if idx < session.hands[0].len() {
                let card = session.hands[0][idx];
                if card.can_play_on(session.discard) {
                    session.discard = card;
                    session.hands[0].remove(idx);
                    if session.hands[0].is_empty() { session.winner = Some(0); }
                    played = true;
                    break;
                }
            }
        }
    }
    if draw_clicks.single().map_or(false, |i| *i == Interaction::Pressed) && !played {
        if let Some(card) = session.deck.pop() { session.hands[0].push(card); }
        played = true;
    }
    if played && session.winner.is_none() {
        // CPU turno simple: juega primera carta jugable o roba
        session.turn = 1;
        if let Some(pos) = session.hands[1].iter().position(|c| c.can_play_on(session.discard)) {
            let card = session.hands[1].remove(pos);
            session.discard = card;
            if session.hands[1].is_empty() { session.winner = Some(1); }
        } else if let Some(card) = session.deck.pop() {
            session.hands[1].push(card);
        }
        session.turn = 0;
    }
    for (field, mut text) in &mut texts {
        match field.0 {
            UnoField::Discard => { *text = Text::new(format!("Descartes: {}", session.discard.label())); }
            UnoField::Status => { *text = Text::new(if let Some(w)=session.winner { if w==0 {"¡Ganas tú!".to_string()} else {"¡Gana CPU!".to_string()} } else { "Tu turno — toca carta que encaje o Roba".to_string() }); }
            UnoField::Hand => { *text = Text::new(format!("Tu mano ({} cartas)", session.hands[0].len())); }
            _ => {}
        }
    }
    for (btn, mut bg, children) in &mut card_query {
        let can = btn.0 < session.hands[0].len() && session.hands[0][btn.0].can_play_on(session.discard);
        *bg = BackgroundColor(if can { Color::srgb(0.25, 0.55, 0.25) } else { Color::srgb(0.50, 0.50, 0.50) });
        for child in children.iter() {
            if let Ok(mut text) = card_texts.get_mut(child) {
                if btn.0 < session.hands[0].len() {
                    *text = Text::new(session.hands[0][btn.0].label());
                } else {
                    *text = Text::new("".to_string());
                }
            }
        }
    }
}
