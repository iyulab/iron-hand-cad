//! The `set` verb: one field of one entity, to a given value.

use crate::path::{self, Segment};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uncad_model::model::{Entity, EntityId};
use uncad_model::CadDatabase;

/// Why an edit was not carried out. Every variant names the reason exactly;
/// none is a partial success.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Refusal {
    /// No entity of the drawing carries this reference.
    NoSuchEntity { id: EntityId },
    /// The drawing lists this entity more than once (model space lists its
    /// entities both at the top level and in its block record) and the
    /// copies disagree. The drawing is inconsistent; it is not edited.
    InconsistentCopies { id: EntityId },
    /// The path is not of the form `key`, `key.key`, `key[n].key`.
    MalformedPath { path: String },
    /// The entity has no field at this path.
    NoSuchField {
        id: EntityId,
        entity_type: String,
        path: String,
    },
    /// The field exists but is not something an edit may change: the type
    /// tag, or the identity and provenance markers in `common`.
    NotEditable { path: String, why: String },
    /// The value is of a different kind than the field holds.
    WrongKind {
        path: String,
        expected: String,
        given: String,
    },
    /// The value violates a constraint the model itself does not enforce.
    Constraint { path: String, rule: String },
    /// With the value in place the entity no longer deserializes as the
    /// model's type -- a structurally wrong value (a point without `z`).
    Invalid { path: String, detail: String },
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Refusal::NoSuchEntity { id } => write!(f, "no entity carries the reference {id:?}"),
            Refusal::InconsistentCopies { id } => write!(
                f,
                "the drawing lists entity {id:?} more than once and the copies disagree"
            ),
            Refusal::MalformedPath { path } => write!(f, "malformed field path {path:?}"),
            Refusal::NoSuchField {
                entity_type, path, ..
            } => write!(f, "{entity_type} has no field at {path:?}"),
            Refusal::NotEditable { path, why } => write!(f, "{path:?} is not editable: {why}"),
            Refusal::WrongKind {
                path,
                expected,
                given,
            } => write!(f, "{path:?} holds a {expected}, not a {given}"),
            Refusal::Constraint { path, rule } => write!(f, "{path:?}: {rule}"),
            Refusal::Invalid { path, detail } => {
                write!(f, "with {path:?} set, the entity is not valid: {detail}")
            }
        }
    }
}

impl std::error::Error for Refusal {}

/// Fields of `common` that say what an entity *is* and where it came from,
/// not what it looks like. An edit never touches them: the reference ID is
/// what the diff pairs the two states by, and the provenance markers are
/// the source's statement, not the editor's.
const PROTECTED_COMMON: [&str; 4] = ["id", "origin", "confidence", "source_handle"];

/// Numeric fields the model stores as plain `f64` but which are only
/// meaningful when positive. The model does not enforce this; an edit does.
const POSITIVE: [&str; 2] = ["radius", "text_height"];

/// The drawing with the field at `path` of the entity `target` set to
/// `value`, and nothing else changed. `db` is not modified.
///
/// `path` names the field the way the model's JSON form does (`radius`,
/// `center.x`, `vertices[2].y`, `common.layer`); `value` is the new value in
/// that same form. The field must exist and the value must be of its kind;
/// the type tag and the `common` identity and provenance fields (`id`,
/// `origin`, `confidence`, `source_handle`) are never editable. `radius`
/// and `text_height` must be positive.
pub fn set(
    db: &CadDatabase,
    target: EntityId,
    path: &str,
    value: Value,
) -> Result<CadDatabase, Refusal> {
    let segments = path::parse(path).ok_or_else(|| Refusal::MalformedPath {
        path: path.to_string(),
    })?;
    guard_editable(path, &segments)?;

    let copies: Vec<&Entity> = db
        .all_entities()
        .filter(|e| e.common().id == target)
        .collect();
    let Some(first) = copies.first() else {
        return Err(Refusal::NoSuchEntity { id: target });
    };
    if copies.iter().any(|c| c != first) {
        return Err(Refusal::InconsistentCopies { id: target });
    }

    let mut tree = serde_json::to_value(first).map_err(|e| Refusal::Invalid {
        path: path.to_string(),
        detail: e.to_string(),
    })?;
    let node = path::get_mut(&mut tree, &segments).ok_or_else(|| Refusal::NoSuchField {
        id: target,
        entity_type: first.type_name().to_string(),
        path: path.to_string(),
    })?;
    if !node.is_null() && kind(node) != kind(&value) {
        return Err(Refusal::WrongKind {
            path: path.to_string(),
            expected: kind(node).to_string(),
            given: kind(&value).to_string(),
        });
    }
    *node = value;
    if let Some(key) = path::last_key(&segments) {
        if POSITIVE.contains(&key) && node.as_f64().is_some_and(|v| v <= 0.0) {
            return Err(Refusal::Constraint {
                path: path.to_string(),
                rule: format!("{key} must be positive"),
            });
        }
    }
    let edited: Entity = serde_json::from_value(tree).map_err(|e| Refusal::Invalid {
        path: path.to_string(),
        detail: e.to_string(),
    })?;

    let mut out = db.clone();
    for e in out.all_entities_mut() {
        if e.common().id == target {
            *e = edited.clone();
        }
    }
    Ok(out)
}

fn guard_editable(path: &str, segments: &[Segment]) -> Result<(), Refusal> {
    let refuse = |why: &str| {
        Err(Refusal::NotEditable {
            path: path.to_string(),
            why: why.to_string(),
        })
    };
    match segments {
        [Segment::Key(k)] if k == "type" => refuse("the type tag says what the entity is"),
        [Segment::Key(k)] if k == "common" => {
            refuse("`common` as a whole carries the identity and provenance markers")
        }
        [Segment::Key(c), Segment::Key(k), ..]
            if c == "common" && PROTECTED_COMMON.contains(&k.as_str()) =>
        {
            refuse("identity and provenance are the source's statement, not the editor's")
        }
        _ => Ok(()),
    }
}

fn kind(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}
