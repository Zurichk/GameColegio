//! Sección Multijuegos / Juegos Clásicos — hub y juegos de reunión.
//!
//! - Hub (`ClassicMenu`) con 3 juegos: Tres en Raya, Conecta 4, Hundir la Flota.
//! - Cada juego es un overlay de UI a pantalla completa (como `learning`).

pub mod battleship;
pub mod connect4;
pub mod tictactoe;

use bevy::prelude::*;

use crate::game::GameState;
use crate::learning::{screen_background, spawn_button};
use crate::i18n::tr;

/// Plugin de la sección clásicos.
pub struct ClassicPlugin;

impl Plugin for ClassicPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            tictactoe::TicTacToePlugin,
            connect4::Connect4Plugin,
            battleship::BattleshipPlugin,
        ))
        .add_systems(OnEnter(GameState::ClassicMenu), spawn_classic_menu)
        .add_systems(OnExit(GameState::ClassicMenu), despawn_classic_menu)
        .add_systems(Update, classic_menu_input.run_if(in_state(GameState::ClassicMenu)));
    }
}

#[derive(Component)]
pub struct ClassicMenuUi;
#[derive(Component)]
pub struct TicTacToeButton;
#[derive(Component)]
pub struct Connect4Button;
#[derive(Component)]
pub struct BattleshipButton;
#[derive(Component)]
pub struct ClassicBackButton;

fn spawn_classic_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands
        .spawn((
            ClassicMenuUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(12.0),
                ..default()
            },
            screen_background(),
            Visibility::Visible,
        ))
        .with_children(|root| {
            root.spawn((
                Text::new(tr("JUEGOS CLÁSICOS")),
                TextFont { font: font.clone(), font_size: 56.0, ..default() },
                TextColor(Color::srgb(1.0, 0.85, 0.40)),
            ));
            root.spawn((
                Text::new(tr("Reúnete y juega — 2 jugadores o contra la CPU")),
                TextFont { font: font.clone(), font_size: 22.0, ..default() },
                TextColor(Color::srgb(0.85, 0.90, 1.0)),
            ));
            root.spawn(Node { height: Val::Px(14.0), ..default() });
            spawn_button(root, "Tres en Raya", TicTacToeButton, &font);
            spawn_button(root, "Conecta 4", Connect4Button, &font);
            spawn_button(root, "Hundir la Flota (5×5)", BattleshipButton, &font);
            root.spawn(Node { height: Val::Px(14.0), ..default() });
            spawn_button(root, "Volver al menú principal", ClassicBackButton, &font);
        });
}

fn despawn_classic_menu(mut commands: Commands, roots: Query<Entity, With<ClassicMenuUi>>) {
    for root in &roots {
        commands.entity(root).despawn();
    }
}

fn classic_menu_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    interactions: Query<(&Interaction, Option<&TicTacToeButton>, Option<&Connect4Button>, Option<&BattleshipButton>, Option<&ClassicBackButton>), Changed<Interaction>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::MainMenu);
        return;
    }
    for (interaction, ttt, c4, bs, back) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if ttt.is_some() {
            next_state.set(GameState::TicTacToeGame);
        } else if c4.is_some() {
            next_state.set(GameState::Connect4Game);
        } else if bs.is_some() {
            next_state.set(GameState::BattleshipGame);
        } else if back.is_some() {
            next_state.set(GameState::MainMenu);
        }
    }
}
