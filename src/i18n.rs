//! Internacionalización (Fase 9): soporte de idiomas Español / English / Français.
//!
//! El idioma activo se guarda en `settings.json` y se carga al arrancar.
//! Se usa un estático global para que cualquier helper de UI (`ui_text`,
//! `spawn_button`, ...) pueda traducir etiquetas sin tener que recibir el
//! idioma por parámetro en todos los sitios.
//!
//! - [`tr`] traduce una cadena clave (español) al idioma activo; si no hay
//!   traducción devuelve la propia clave (fallback al español).
//! - [`t3`] devuelve una de las tres variantes según el idioma activo.
//!
//! Las preguntas no pasan por esta tabla: cada banco (`questions.rs`,
//! `trivia.rs`, ...) tiene sus propias variantes por idioma y se selecciona
//! con [`language`].

use std::sync::atomic::{AtomicU8, Ordering};

use serde::{Deserialize, Serialize};

/// Idiomas soportados por el juego.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Language {
    /// Español (idioma por defecto).
    Es,
    /// English.
    En,
    /// Français.
    Fr,
}

impl Default for Language {
    fn default() -> Self {
        Language::Es
    }
}

impl Language {
    /// Nombre localizado del idioma (para el selector).
    pub fn name(self) -> &'static str {
        match self {
            Language::Es => "Español",
            Language::En => "English",
            Language::Fr => "Français",
        }
    }
}

/// Índice del idioma activo (0 = Es, 1 = En, 2 = Fr).
static CURRENT_LANG: AtomicU8 = AtomicU8::new(0);

/// Cambia el idioma activo.
pub fn set_language(lang: Language) {
    CURRENT_LANG.store(lang as u8, Ordering::Relaxed);
}

/// Idioma activo.
pub fn language() -> Language {
    match CURRENT_LANG.load(Ordering::Relaxed) {
        1 => Language::En,
        2 => Language::Fr,
        _ => Language::Es,
    }
}

/// Devuelve la variante correcta entre (español, inglés, francés) según el
/// idioma activo. Útil para cadenas dinámicas que no están en la tabla.
#[allow(dead_code)]
pub fn t3(es: &str, en: &str, fr: &str) -> String {
    match language() {
        Language::Es => es.to_string(),
        Language::En => en.to_string(),
        Language::Fr => fr.to_string(),
    }
}

/// Traduce una cadena clave (español) al idioma activo usando la tabla
/// [`translate`]. Si la clave no está en la tabla devuelve la clave tal cual
/// (fallback a español).
pub fn tr(es: &str) -> String {
    match language() {
        Language::Es => es.to_string(),
        other => translate(es, other)
            .map(str::to_string)
            .unwrap_or_else(|| es.to_string()),
    }
}

/// Busca la traducción de `es` en el idioma indicado.
fn translate(es: &str, lang: Language) -> Option<&'static str> {
    // Tabla de traducciones. Las claves son siempre el texto en español.
    // Mantener ordenadas alfabéticamente para facilitar el mantenimiento.
    let (en, fr) = match es {
        "  ¡Pareja!" => ("  Match!", "  Paire !"),
        "¡Correcto!" => ("Correct!", "Correct !"),
        "¡Cuentas claras!" => ("Great job!", "Bravo !"),
        "¡Has ganado!" => ("You won!", "Tu as gagné !"),
        "¡Memoria increíble!" => ("Amazing memory!", "Mémoire incroyable !"),
        "¡Mira con atención..." => ("Watch carefully...", "Regarde attentivement..."),
        "¡Ahora repite la secuencia!" => ("Now repeat the sequence!", "Maintenant répète la séquence !"),
        "¡Cuidado! Vuelve a mirar..." => ("Careful! Watch again...", "Attention ! Regarde encore..."),
        "¡Muy bien!" => ("Very good!", "Très bien !"),
        "¡Sigue practicando!" => ("Keep practicing!", "Continue de t'entraîner !"),
        "¡Elige la dirección!" => ("Choose a direction!", "Choisis une direction !"),
        "¡Salida del centro!" => ("Leave the center!", "Sortie du centre !"),
        "¡Casilla de DADO!" => ("DICE square!", "Case DÉ !"),
        "¡{} ha ganado la partida!" => ("{} won the game!", "{} a gagné la partie !"),
        "7ª Estrellita (TABÚ)" => ("7th Star (TABOO)", "7e Étoile (TABOU)"),
        "¡Acertada!" => ("Guessed!", "Trouvée !"),
        "No acertada" => ("Not guessed", "Pas trouvée"),
        "No puedes decir:" => ("You cannot say:", "Tu ne peux pas dire :"),
        "{} describe la palabra a su equipo SIN decirla\nni decir ninguna de las prohibidas:" => ("{} describes the word to the team WITHOUT saying it\nor any of the forbidden words:", "{} décrit le mot à son équipe SANS le dire\nni dire un des mots interdits :"),
        "{} ha vuelto al centro y se juega la 7ª Estrellita.\nDescribe la palabra sin decir las prohibidas:" => ("{} is back in the center playing for the 7th Star.\nDescribe the word without saying the forbidden ones:", "{} est revenu au centre et joue la 7e Étoile.\nDécris le mot sans dire les mots interdits :"),
        "{} está en el centro y elige la Estrellita a la que quiere ir.\nEl dado ({}) cuenta las casillas: la Estrellita está a 4 del centro." => ("{} is in the center and picks the Star to go to.\nThe dice ({}) counts the squares: the Star is 4 from the center.", "{} est au centre et choisit l'Étoile vers laquelle aller.\nLe dé ({}) compte les cases : l'Étoile est à 4 du centre."),
        "{} lanzó un {}.\nElige hacia dónde mover su ficha (cuenta {} casillas) para reunir las Estrellitas en el orden que quieras:" => ("{} rolled a {}.\nChoose which way to move the pawn ({} squares) to collect the Stars in any order you like:", "{} a lancé un {}.\nChoisis où déplacer son pion ({} cases) pour réunir les Étoiles dans l'ordre que tu veux :"),
        "{} ha caído en un dado y puede volver a tirar." => ("{} landed on a dice and can roll again.", "{} est tombé sur un dé et peut relancer."),
        "TABÚ" => ("TABOO", "TABOU"),
        "✓ ¡Acertada!" => ("✓ Guessed!", "✓ Trouvée !"),
        "✗ No acertada" => ("✗ Not guessed", "✗ Pas trouvée"),
        "Aciertos: {} · Fallos: {}  de {} actividades" => ("Correct: {} · Wrong: {}  out of {} activities", "Réussites : {} · Erreurs : {}  sur {} activités"),
        "Aciertos: {} · Fallos: {}  de {} frases" => ("Correct: {} · Wrong: {}  out of {} sentences", "Réussites : {} · Erreurs : {}  sur {} phrases"),
        "Aciertos: {} · Fallos: {}  de {} operaciones" => ("Correct: {} · Wrong: {}  out of {} operations", "Réussites : {} · Erreurs : {}  sur {} opérations"),
        "Aciertos: {} · Fallos: {}  de {} preguntas" => ("Correct: {} · Wrong: {}  out of {} questions", "Réussites : {} · Erreurs : {}  sur {} questions"),
        "Actividad {}/{}  ·  Aciertos: {}  ·  Fallos: {}" => ("Activity {}/{}  ·  Correct: {}  ·  Wrong: {}", "Activité {}/{}  ·  Réussites : {}  ·  Erreurs : {}"),
        "Agua, comida y aire" => ("Water, food and air", "De l'eau, de la nourriture et de l'air"),
        "Arte y Literatura" => ("Art and Literature", "Arts et littérature"),
        "Ahorcado" => ("Hangman", "Le pendu"),
        "AJUSTES" => ("SETTINGS", "RÉGLAGES"),
        "Ajustes" => ("Settings", "Réglages"),
        "Animal con el cuello largo" => ("Animal with a long neck", "Animal au long cou"),
        "Animal grande con trompa" => ("Big animal with a trunk", "Grand animal avec une trompe"),
        "Animal que da leche" => ("Animal that gives milk", "Animal qui donne du lait"),
        "Animal que dice cuac" => ("Animal that says quack", "Animal qui fait coin-coin"),
        "Animal que galopa" => ("Animal that gallops", "Animal qui galope"),
        "Animal que ladra" => ("Animal that barks", "Animal qui aboie"),
        "Animal que maúlla" => ("Animal that meows", "Animal qui miaule"),
        "Animal que nada en el agua" => ("Animal that swims in water", "Animal qui nage dans l'eau"),
        "Animal que pone huevos" => ("Animal that lays eggs", "Animal qui pond des œufs"),
        "Animal" => ("Animal", "Animal"),
        "Aprende jugando en tu colegio" => ("Learn while playing at your school", "Apprends en jouant dans ton école"),
        "Aula de Historia" => ("History Classroom", "Classe d'Histoire"),
        "Aula de Informática" => ("Computer Room", "Salle informatique"),
        "Aula de Matemáticas" => ("Maths Classroom", "Classe de Maths"),
        "Busca la palabra: \"{}\"" => ("Find the word: \"{}\"", "Trouve le mot : \"{}\""),
        "Cálculo mental" => ("Mental maths", "Calcul mental"),
        "CÁLCULO MENTAL" => ("MENTAL MATHS", "CALCUL MENTAL"),
        "Cancelar" => ("Cancel", "Annuler"),
        "Ciencias naturales" => ("Natural science", "Sciences naturelles"),
        "CIENCIAS NATURALES" => ("NATURAL SCIENCE", "SCIENCES NATURELLES"),
        "Ciencias" => ("Science", "Sciences"),
        "CENTRO · SALIDA" => ("CENTER · START", "CENTRE · DÉPART"),
        "Comenzar partida" => ("Start game", "Commencer la partie"),
        "Colegio" => ("School", "École"),
        "Consiguió las 6 Estrellitas de color y superó\nel reto final en el centro (7ª Estrellita Tabú)." => ("They got the 6 colored Stars and passed\nthe final challenge in the center (7th Taboo Star).", "Il/Elle a obtenu les 6 Étoiles de couleur et a réussi\nle défi final au centre (7e Étoile Tabou)."),
        "Continuar" => ("Continue", "Continuer"),
        "Dado" => ("Dice", "Dé"),
        "Dado: -" => ("Dice: -", "Dé : -"),
        "Dado: {dice}" => ("Dice: {dice}", "Dé : {dice}"),
        "Derecha →" => ("Right →", "Droite →"),
        "Desaparece" => ("It disappears", "Il disparaît"),
        "Dificultad" => ("Difficulty", "Difficulté"),
        "Dividir" => ("Divide", "Diviser"),
        "E — Abrir puerta" => ("E — Open door", "E — Ouvrir la porte"),
        "E — Abrir/Cerrar puerta" => ("E — Open/Close door", "E — Ouvrir/Fermer la porte"),
        "E — Cerrar puerta" => ("E — Close door", "E — Fermer la porte"),
        "Elige una sección" => ("Choose a section", "Choisis une section"),
        "Encuentra las parejas iguales" => ("Find the matching pairs", "Trouve les paires identiques"),
        "Escribe aquí…" => ("Type here...", "Écris ici..."),
        "Escribe la palabra: \"{}\"" => ("Type the word: \"{}\"", "Écris le mot : \"{}\""),
        "Escribe tu respuesta:" => ("Type your answer:", "Écris ta réponse :"),
        "Escribir" => ("Write", "Écrire"),
        "E — Hablar · Q — Cuestionario" => ("E — Talk · Q — Quiz", "E — Parler · Q — Questionnaire"),
        "Espacio / Clic — Continuar" => ("Space / Click — Continue", "Espace / Clic — Continuer"),
        "Explorar el colegio" => ("Explore the school", "Explorer l'école"),
        "Fallos: -" => ("Wrong: -", "Erreurs : -"),
        "Fallos: {}" => ("Wrong: {}", "Erreurs : {}"),
        "Frase {}/{}  ·  Aciertos: {}  ·  Fallos: {}" => ("Sentence {}/{}  ·  Correct: {}  ·  Wrong: {}", "Phrase {}/{}  ·  Réussites : {}  ·  Erreurs : {}"),
        "GAMECOLEGIO" => ("GAMECOLEGIO", "GAMECOLEGIO"),
        "Geografía de España" => ("Geography of Spain", "Géographie de l'Espagne"),
        "GEOGRAFÍA DE ESPAÑA" => ("GEOGRAPHY OF SPAIN", "GÉOGRAPHIE DE L'ESPAGNE"),
        "Geografía" => ("Geography", "Géographie"),
        "Ciencia y Naturaleza" => ("Science and Nature", "Sciences et nature"),
        "Guardar partida" => ("Save game", "Sauvegarder la partie"),
        "Historia" => ("History", "Histoire"),
        "¡Hola! Soy el profesor de Matemáticas." => ("Hi! I am the Maths teacher.", "Bonjour ! Je suis le professeur de maths."),
        "Para ganar la Estrellita azul, domina las tablas, los porcentajes y la geometría." => ("To win the blue Star, master the tables, percentages and geometry.", "Pour gagner l'Étoile bleue, maîtrise les tables, les pourcentages et la géométrie."),
        "En el tablero te preguntaré sumas, ecuaciones y algo de álgebra." => ("On the board I will ask you sums, equations and a bit of algebra.", "Sur le plateau je te poserai des additions, des équations et un peu d'algèbre."),
        "¡Mucha suerte! Y recuerda: la práctica hace al maestro." => ("Good luck! And remember: practice makes perfect.", "Bonne chance ! Et souviens-toi : c'est en forgeant qu'on devient forgeron."),
        "Bienvenido a clase de Historia." => ("Welcome to History class.", "Bienvenue en cours d'Histoire."),
        "Fechas, batallas, imperios y personajes... ¡mi asignatura favorita!" => ("Dates, battles, empires and famous people... my favourite subject!", "Des dates, des batailles, des empires et des personnages... ma matière préférée !"),
        "La Estrellita naranja espera a quien sepa de Roma, Egipto y Grecia." => ("The orange Star awaits those who know about Rome, Egypt and Greece.", "L'Étoile orange attend ceux qui connaissent Rome, l'Égypte et la Grèce."),
        "¡Estudia bien y nos vemos en el tablero!" => ("Study hard and see you on the board!", "Étudie bien et on se retrouve sur le plateau !"),
        "¡Hola! Yo llevo la clase de Informática." => ("Hi! I run the Computing class.", "Bonjour ! Je m'occupe du cours d'informatique."),
        "Mira los ordenadores de los pupitres: aquí se aprende haciendo." => ("Look at the computers on the desks: here you learn by doing.", "Regarde les ordinateurs des bureaux : ici on apprend en faisant."),
        "Si aciertas mis preguntas de hardware, redes y código, la Estrellita verde será tuya." => ("If you get my hardware, network and code questions right, the green Star will be yours.", "Si tu réussis mes questions de matériel, de réseaux et de code, l'Étoile verte sera à toi."),
        "¡Que no te pille desprevenido lo de los lenguajes de programación!" => ("Don't let programming languages catch you off guard!", "Que les langages de programmation ne te prennent pas au dépourvu !"),
        "rojo" => ("red", "rouge"),
        "verde" => ("green", "vert"),
        "azul" => ("blue", "bleu"),
        "amarillo" => ("yellow", "jaune"),
        "Hoy hace mucho ______." => ("It is very ______ today.", "Il fait très ______ aujourd'hui."),
        "Idioma" => ("Language", "Langue"),
        "Incorrecto" => ("Wrong", "Incorrect"),
        "Incorrecto — era {}) {}" => ("Wrong — it was {}) {}", "Incorrect — c'était {}) {}"),
        "Incorrecto — era: {}" => ("Wrong — it was: {}", "Incorrect — c'était : {}"),
        "Incorrecto — la correcta es: {}" => ("Wrong — the right answer is: {}", "Incorrect — la bonne réponse est : {}"),
        "Informática" => ("Computing", "Informatique"),
        "Juegos de memoria" => ("Memory games", "Jeux de mémoire"),
        "Jugar otra vez" => ("Play again", "Rejouer"),
        "Jugador {}" => ("Player {}", "Joueur {}"),
        "Jugadores: 2" => ("Players: 2", "Joueurs : 2"),
        "Jugadores: {count}" => ("Players: {count}", "Joueurs : {count}"),
        "La palabra era: {}" => ("The word was: {}", "Le mot était : {}"),
        "Lanzar dado" => ("Roll dice", "Lancer le dé"),
        "Leer y escribir" => ("Reading and writing", "Lire et écrire"),
        "Leer, escribir y jugar con las palabras" => ("Read, write and play with words", "Lire, écrire et jouer avec les mots"),
        "Leer" => ("Read", "Lire"),
        "Lengua" => ("Language", "Langue"),
        "Marca las horas" => ("It tells the time", "Il indique l'heure"),
        "Matemáticas" => ("Maths", "Mathématiques"),
        "MATEMÁTICAS" => ("MATHS", "MATHÉMATIQUES"),
        "Memoria de secuencia" => ("Sequence memory", "Mémoire de séquence"),
        "Memoria" => ("Memory", "Mémoire"),
        "Menú" => ("Menu", "Menu"),
        "Menú principal" => ("Main menu", "Menu principal"),
        "MODO TABLERO" => ("BOARD MODE", "MODE PLATEAU"),
        "Modo Tablero" => ("Board mode", "Mode plateau"),
        "Mueble donde se come" => ("Furniture to eat on", "Meuble sur lequel on mange"),
        "Mueble para sentarse" => ("Furniture to sit on", "Meuble pour s'asseoir"),
        "Multiplicar" => ("Multiply", "Multiplier"),
        "Naturaleza, cuerpo humano y geografía" => ("Nature, the human body and geography", "Nature, corps humain et géographie"),
        "Naturaleza" => ("Nature", "Nature"),
        "No coinciden" => ("They don't match", "Elles ne correspondent pas"),
        "Nueva palabra" => ("New word", "Nouveau mot"),
        "Operación {}/{}  ·  Aciertos: {}  ·  Fallos: {}" => ("Operation {}/{}  ·  Correct: {}  ·  Wrong: {}", "Opération {}/{}  ·  Réussites : {}  ·  Erreurs : {}"),
        "Operaciones y cálculo mental" => ("Operations and mental maths", "Opérations et calcul mental"),
        "Ortografía" => ("Spelling", "Orthographe"),
        "ORTOGRAFÍA" => ("SPELLING", "ORTHOGRAPHE"),
        "Parejas: {} · Movimientos: {} · Tiempo: {} s" => ("Pairs: {} · Moves: {} · Time: {} s", "Paires : {} · Coups : {} · Temps : {} s"),
        "Parejas: {}/{} · Movimientos: {} · Tiempo: {} s" => ("Pairs: {}/{} · Moves: {} · Time: {} s", "Paires : {}/{} · Coups : {} · Temps : {} s"),
        "Secuencia de {} colores · Tiempo: {} s · Pulsaciones: {}" => ("Sequence of {} colors · Time: {} s · Presses: {}", "Séquence de {} couleurs · Temps : {} s · Pressions : {}"),
        "Parejas de formas (8)" => ("Shape pairs (8)", "Paires de formes (8)"),
        "Parejas de formas" => ("Shape pairs", "Paires de formes"),
        "Parejas de letras (6)" => ("Letter pairs (6)", "Paires de lettres (6)"),
        "Parejas de letras" => ("Letter pairs", "Paires de lettres"),
        "Parejas de números (8)" => ("Number pairs (8)", "Paires de nombres (8)"),
        "Parejas de números" => ("Number pairs", "Paires de nombres"),
        "Parejas de palabras (6)" => ("Word pairs (6)", "Paires de mots (6)"),
        "Parejas de palabras" => ("Word pairs", "Paires de mots"),
        "Parejas mixtas (10)" => ("Mixed pairs (10)", "Paires mixtes (10)"),
        "Parejas mixtas" => ("Mixed pairs", "Paires mixtes"),
        "Partida guardada" => ("Game saved", "Partie sauvegardée"),
        "✓ Partida guardada" => ("✓ Game saved", "✓ Partie sauvegardée"),
        "Pasillo" => ("Hallway", "Couloir"),
        "Patio" => ("Schoolyard", "Cour de récréation"),
        "PAUSA" => ("PAUSE", "PAUSE"),
        "Pista: {}" => ("Hint: {}", "Indice : {}"),
        "Planta grande con tronco" => ("Big plant with a trunk", "Grande plante avec un tronc"),
        "Planta que huele muy bien" => ("Plant that smells very nice", "Plante qui sent très bon"),
        "Pregunta {}/{}  ·  Aciertos: {}  ·  Fallos: {}" => ("Question {}/{}  ·  Correct: {}  ·  Wrong: {}", "Question {}/{}  ·  Réussites : {}  ·  Erreurs : {}"),
        "Primeros pasos" => ("First steps", "Premiers pas"),
        "Reanudar" => ("Resume", "Reprendre"),
        "Recepción" => ("Reception", "Accueil"),
        "Reiniciar partida" => ("Restart game", "Recommencer la partie"),
        "Responder" => ("Answer", "Répondre"),
        "Respuesta correcta: {expected}" => ("Correct answer: {expected}", "Bonne réponse : {expected}"),
        "Restar" => ("Subtract", "Soustraire"),
        "RETO FINAL · 7ª Estrellita" => ("FINAL CHALLENGE · 7th Star", "DÉFI FINAL · 7e Étoile"),
        "Ronda {}/{} · Secuencia: {} · Pulsaciones: {}" => ("Round {}/{} · Sequence: {} · Presses: {}", "Manche {}/{} · Séquence : {} · Pressions : {}"),
        "Ronda {}/{} · Aciertos: {} · Fallos: {}" => ("Round {}/{} · Correct: {} · Wrong: {}", "Manche {}/{} · Réussites : {} · Échecs : {}"),
        "¿QUÉ SIGNO VA ENTRE LOS DOS NÚMEROS?" => ("WHICH SIGN GOES BETWEEN THE TWO NUMBERS?", "QUEL SIGNE VA ENTRE LES DEUX NOMBRES ?"),
        "Incorrecto — el signo correcto era: {}" => ("Incorrect — the right sign was: {}", "Incorrect — le bon signe était : {}"),
        "Mayor, menor o igual" => ("Greater, less or equal", "Plus grand, plus petit ou égal"),
        "Aciertos: {} · Fallos: {} — Nota: {}" => ("Correct: {} · Wrong: {} — Grade: {}", "Réussites : {} · Échecs : {} — Note : {}"),
        "SALIDA" => ("START", "DÉPART"),
        "Salir del juego" => ("Quit game", "Quitter le jeu"),
        "Salir" => ("Quit", "Quitter"),
        "Se abre para entrar" => ("It opens to go in", "Elle s'ouvre pour entrer"),
        "Se acabó el tiempo — era {}) {}" => ("Time's up — it was {}) {}", "Temps écoulé — c'était {}) {}"),
        "Se bebe y es transparente" => ("You drink it and it is clear", "On la boit et elle est transparente"),
        "Se bota y se juega con ella" => ("You bounce it and play with it", "On la fait rebondir et on joue avec"),
        "Se come y se hace con harina" => ("You eat it and it is made with flour", "On la mange et elle est faite avec de la farine"),
        "Se convierte en hielo" => ("It turns into ice", "Il se transforme en glace"),
        "Se convierte en vapor" => ("It turns into steam", "Il se transforme en vapeur"),
        "Se nota pero no se ve" => ("You can feel it but not see it", "On le sent mais on ne le voit pas"),
        "Se ve en el cielo por la noche" => ("You see it in the sky at night", "On le voit dans le ciel la nuit"),
        "Sensibilidad del ratón" => ("Mouse sensitivity", "Sensibilité de la souris"),
        "Sirve para escribir y dibujar" => ("You use it to write and draw", "Elle sert à écrire et à dessiner"),
        "Sí, salir" => ("Yes, quit", "Oui, quitter"),
        "Sumar" => ("Add", "Additionner"),
        "Tabú" => ("Taboo", "Tabou"),
        "Tiene páginas y se lee" => ("It has pages and you read it", "Il a des pages et on le lit"),
        "Tiempo: {} s" => ("Time: {} s", "Temps : {} s"),
        "Tirar de nuevo" => ("Roll again", "Relancer"),
        "Tu respuesta: {display}" => ("Your answer: {display}", "Ta réponse : {display}"),
        "Turno: {}" => ("Turn: {}", "Tour : {}"),
        "Turno: Jugador 1" => ("Turn: Player 1", "Tour : Joueur 1"),
        "✓ {subject}" => ("✓ {subject}", "✓ {subject}"),
        "10 · Sobresaliente" => ("10 · Outstanding", "10 · Très bien"),
        "6,7 · Notable" => ("6,7 · Good", "6,7 · Bien"),
        "3,3 · Suspenso" => ("3,3 · Fail", "3,3 · Échec"),
        "0 · Suspenso" => ("0 · Fail", "0 · Échec"),
        "Cerrar" => ("Close", "Fermer"),
        "¡Asignatura superada!" => ("Subject passed!", "Matière réussie !"),
        "Asignatura no superada" => ("Subject not passed", "Matière non réussie"),
        "Aciertos: {} · Fallos: {}  —  Nota: {}" => ("Correct: {} · Wrong: {}  —  Grade: {}", "Réussites : {} · Erreurs : {}  —  Note : {}"),
        "Cuestionario de {}" => ("{} quiz", "Questionnaire de {}"),
        "Volumen" => ("Volume", "Volume"),
        "Volver" => ("Back", "Retour"),
        "Volver a Ciencias" => ("Back to Science", "Retour aux Sciences"),
        "Volver a Juegos de memoria" => ("Back to Memory games", "Retour aux Jeux de mémoire"),
        "Volver a la zona de aprendizaje" => ("Back to the learning zone", "Retour à la zone d'apprentissage"),
        "Volver a Lengua" => ("Back to Language", "Retour à la Langue"),
        "Volver a Matemáticas" => ("Back to Maths", "Retour aux Mathématiques"),
        "Volver al menú principal" => ("Back to main menu", "Retour au menu principal"),
        "Zona de aprendizaje" => ("Learning zone", "Zone d'apprentissage"),
        "¿Seguro que quieres salir del juego?" => ("Are you sure you want to quit the game?", "Es-tu sûr de vouloir quitter le jeu ?"),
        "¿Seguro que quieres salir?" => ("Are you sure you want to quit?", "Es-tu sûr de vouloir quitter ?"),
        "← Izquierda" => ("← Left", "← Gauche"),
        "⏱ {}:{:02}" => ("⏱ {}:{:02}", "⏱ {}:{:02}"),
        "⏱ 1:00" => ("⏱ 1:00", "⏱ 1:00"),
        _ => return None,
    };
    match lang {
        Language::En => Some(en),
        Language::Fr => Some(fr),
        Language::Es => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Los tests mutan el idioma global y se ejecutan en paralelo: un mutex
    // serializa el acceso para que no se pisen entre sí.
    static LANG_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn default_is_spanish() {
        let _guard = LANG_LOCK.lock().unwrap();
        assert_eq!(language(), Language::Es);
        assert_eq!(tr("Volver"), "Volver");
    }

    #[test]
    fn translates_known_strings() {
        let _guard = LANG_LOCK.lock().unwrap();
        set_language(Language::En);
        assert_eq!(tr("Volver"), "Back");
        assert_eq!(tr("AJUSTES"), "SETTINGS");
        assert_eq!(tr("Turno: {}"), "Turn: {}");

        set_language(Language::Fr);
        assert_eq!(tr("Volver"), "Retour");
        assert_eq!(tr("AJUSTES"), "RÉGLAGES");
    }

    #[test]
    fn falls_back_to_spanish_for_unknown() {
        let _guard = LANG_LOCK.lock().unwrap();
        set_language(Language::En);
        assert_eq!(tr("cadena inexistente"), "cadena inexistente");
        set_language(Language::Es);
    }

    #[test]
    fn t3_picks_the_active_language() {
        let _guard = LANG_LOCK.lock().unwrap();
        set_language(Language::Es);
        assert_eq!(t3("a", "b", "c"), "a");
        set_language(Language::En);
        assert_eq!(t3("a", "b", "c"), "b");
        set_language(Language::Fr);
        assert_eq!(t3("a", "b", "c"), "c");
        set_language(Language::Es);
    }
}