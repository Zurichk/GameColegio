//! Banco de preguntas del tablero en **francés** (traducción completa del
//! banco original de español). Contiene las mismas 1.826 preguntas: cerradas
//! y abiertas, por categoría y dificultad.
//!
//! Se genera por partes (cada parte un módulo) y se combina en una sola lista
//! con `questions_fr()`.

use super::questions::Question;
use std::sync::OnceLock;

mod part1;
mod part2;
mod part3;
mod part4;
mod part5;

/// Devuelve el banco completo en francés (combina las cinco partes).
pub fn questions_fr() -> &'static [Question] {
    static CACHE: OnceLock<Vec<Question>> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            let mut v = Vec::with_capacity(
                part1::QUESTIONS_FR_1.len()
                    + part2::QUESTIONS_FR_2.len()
                    + part3::QUESTIONS_FR_3.len()
                    + part4::QUESTIONS_FR_4.len()
                    + part5::QUESTIONS_FR_5.len(),
            );
            v.extend_from_slice(part1::QUESTIONS_FR_1);
            v.extend_from_slice(part2::QUESTIONS_FR_2);
            v.extend_from_slice(part3::QUESTIONS_FR_3);
            v.extend_from_slice(part4::QUESTIONS_FR_4);
            v.extend_from_slice(part5::QUESTIONS_FR_5);
            v
        })
        .as_slice()
}