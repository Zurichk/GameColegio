//! Banco de preguntas del tablero en **inglés** (traducción completa del
//! banco original de español). Contiene las mismas 1.826 preguntas: cerradas
//! y abiertas, por categoría y dificultad.
//!
//! Se genera por partes (cada parte un módulo) y se combina en una sola lista
//! con `questions_en()`.

use super::questions::Question;
use std::sync::OnceLock;

mod part1;
mod part2;
mod part3;
mod part4;
mod part5;
mod part6;

/// Devuelve el banco completo en inglés (combina las seis partes).
pub fn questions_en() -> &'static [Question] {
    static CACHE: OnceLock<Vec<Question>> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            let mut v = Vec::with_capacity(
                part1::QUESTIONS_EN_1.len()
                    + part2::QUESTIONS_EN_2.len()
                    + part3::QUESTIONS_EN_3.len()
                    + part4::QUESTIONS_EN_4.len()
                    + part5::QUESTIONS_EN_5.len()
                    + part6::QUESTIONS_EN_6.len(),
            );
            v.extend_from_slice(part1::QUESTIONS_EN_1);
            v.extend_from_slice(part2::QUESTIONS_EN_2);
            v.extend_from_slice(part3::QUESTIONS_EN_3);
            v.extend_from_slice(part4::QUESTIONS_EN_4);
            v.extend_from_slice(part5::QUESTIONS_EN_5);
            v.extend_from_slice(part6::QUESTIONS_EN_6);
            v
        })
        .as_slice()
}