//! Plugin principal del juego y máquina de estados.

use bevy::prelude::*;

use crate::audio::SfxPlugin;
use crate::board::BoardPlugin;
use crate::camera::ThirdPersonCameraPlugin;
use crate::classic::ClassicPlugin;
use crate::fx::FxPlugin;
use crate::hud::HudPlugin;
use crate::learning::LearningPlugin;
use crate::menu::MenuPlugin;
use crate::pause::PausePlugin;
use crate::player::PlayerPlugin;
use crate::save::SavePlugin;
use crate::settings::SettingsPlugin;
use crate::world::WorldPlugin;

/// Evento que pide reiniciar el mundo de exploración (jugador a la salida,
/// puertas abiertas y sesiones de diálogo/cuestionario limpias). Lo dispara
/// el botón "Reiniciar partida" del menú de pausa y lo atienden los sistemas
/// de `WorldPlugin`, `PlayerPlugin` y `SavePlugin`.
#[derive(Event)]
pub struct RestartWorld;

/// Evento que pide restaurar el mundo desde una partida guardada (posición
/// del jugador y estado de las puertas). Lo dispara el botón "Continuar" del
/// menú principal.
#[derive(Event)]
pub struct RestoreWorld;

/// Evento que pide escribir `savegame.json` con el estado actual (progreso,
/// posición y puertas). Lo atiende `SavePlugin::save_system`.
#[derive(Event)]
pub struct SaveGameRequested;

/// Estados principales del juego.
///
/// `MainMenu`, `BoardSetup` y `BoardGame` corresponden al modo tablero
/// (estilo Trivial); `Playing` es la exploración libre del colegio,
/// `Paused` su menú de pausa y `Settings` la pantalla de ajustes.
/// `LearningMenu` es el centro de la zona de aprendizaje con sus secciones
/// (`LanguageMenu`, `MathMenu`, `ScienceMenu` y `MemoryMenu`) y los juegos
/// de cada sección.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[allow(dead_code)] // `Quiz` se reserva para el modo tablero (fase posterior).
pub enum GameState {
    /// Menú principal del juego.
    #[default]
    MainMenu,
    /// El jugador explora el colegio libremente.
    Playing,
    /// Configuración de la partida de tablero.
    BoardSetup,
    /// Partida de tablero en curso.
    BoardGame,
    /// Cuestionario en curso.
    Quiz,
    /// Centro de la zona de aprendizaje (Lengua, Matemáticas, Ciencias, Memoria).
    LearningMenu,
    /// Sección de Lengua: leer y escribir, ortografía y ahorcado.
    LanguageMenu,
    /// Sección de Matemáticas: operaciones y cálculo mental.
    MathMenu,
    /// Sección de Ciencias: ciencias naturales y geografía.
    ScienceMenu,
    /// Práctica de lectura y escritura.
    ReadingPractice,
    /// Práctica de ortografía (elegir la palabra bien escrita).
    SpellingPractice,
    /// Juego del ahorcado.
    HangmanGame,
    /// Sinónimos (Lengua).
    SynonymsPractice,
    /// Anagramas (Lengua).
    AnagramPractice,
    /// Vocabulario (Lengua).
    VocabPractice,
    /// Práctica de sumar/restar/multiplicar/dividir.
    MathPractice,
    /// Práctica de cálculo mental con temporizador.
    MentalPractice,
    /// Juego "Mayor, menor o igual" (comparar números).
    ComparePractice,
    /// Fracciones (Matemáticas).
    FractionsPractice,
    /// Geometría (Matemáticas).
    GeometryPractice,
    /// Problemas de texto (Matemáticas).
    WordProblemsPractice,
    /// Adivina el número (1-100).
    GuessNumberGame,
    /// Cuestionario de ciencias naturales.
    SciencePractice,
    /// Cuestionario de geografía de España.
    GeographyPractice,
    /// Menú de juegos clásicos (multijuegos).
    ClassicMenu,
    /// Tres en raya.
    TicTacToeGame,
    /// Conecta 4.
    Connect4Game,
    /// Hundir la flota.
    BattleshipGame,
    /// Menú de los juegos de memoria.
    MemoryMenu,
    /// Juego de memoria (emparejar tarjetas) en curso.
    MemoryGame,
    /// Juego de memoria de secuencia (repetir colores) en curso.
    MemorySequence,
    /// Juego en pausa (menú de pausa abierto).
    Paused,
    /// Pantalla de ajustes (sensibilidad y volumen).
    Settings,
}

/// Orquesta todos los plugins del juego.
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_event::<RestartWorld>()
            .add_event::<RestoreWorld>()
            .add_event::<SaveGameRequested>()
            .add_plugins((
                WorldPlugin,
                PlayerPlugin,
                ThirdPersonCameraPlugin,
                MenuPlugin,
                PausePlugin,
                SavePlugin,
                HudPlugin,
                SettingsPlugin,
                SfxPlugin,
                FxPlugin,
                BoardPlugin,
                LearningPlugin,
                ClassicPlugin,
            ));
    }
}