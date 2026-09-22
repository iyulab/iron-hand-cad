//! Deterministic hands for a 2D CAD drawing in the [`uncad_model`] entity
//! model: [`set`] takes a drawing, an entity reference, a field and a value,
//! and returns the drawing with exactly that field of exactly that entity
//! changed -- or a [`Refusal`] saying why it could not.
//!
//! Nothing here guesses. A reference that resolves to nothing, a field the
//! entity does not have, a value of the wrong kind, a value that violates a
//! constraint the model itself does not enforce (a radius of zero) -- each is
//! refused with a reason, never approximated. The input is never modified;
//! the result is a new state, and what changed between the two is
//! established by `iron-diff-cad`, not by this crate describing its own
//! work. The same input gives the same output, byte for byte.
//!
//! Fields are named the way the model's JSON form names them (`radius`,
//! `center.x`, `vertices[2].y`, `common.layer`), which is also the way the
//! diff names the fields it reports: the edit and its verification speak
//! one vocabulary. The verb set of this crate is in `docs/verbs.md`.

#![forbid(unsafe_code)]

mod path;
mod set;

pub use set::{set, Refusal};

pub use uncad_model::CadDatabase;
