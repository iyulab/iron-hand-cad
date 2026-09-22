//! The same drawing, reference, field and value must give the same new
//! state, byte for byte, every time -- and the input must come out untouched.

use iron_hand_cad::set;
use serde_json::json;
use uncad_model::model::{Entity, EntityId};
use uncad_model::CadDatabase;

/// With a handful of entries in a leaked hash set, two consecutive
/// identical orders are already unlikely; this many leave no realistic
/// chance of a false pass.
const RUNS: usize = 24;

fn g1() -> CadDatabase {
    serde_json::from_str(include_str!("golden/g1.expected.json"))
        .expect("the golden model deserializes")
}

fn first_hole(db: &CadDatabase) -> EntityId {
    db.entities
        .iter()
        .find_map(|e| match e {
            Entity::Circle(c) => Some(c.common.id),
            _ => None,
        })
        .expect("G1 has a hole")
}

#[test]
fn repeated_edits_are_byte_identical() {
    let db = g1();
    let hole = first_hole(&db);
    let first = serde_json::to_string(&set(&db, hole, "radius", json!(6.0)).unwrap()).unwrap();
    for run in 1..RUNS {
        let again = serde_json::to_string(&set(&db, hole, "radius", json!(6.0)).unwrap()).unwrap();
        assert_eq!(first, again, "run {run}: the new state changed");
    }
}

#[test]
fn repeated_refusals_are_byte_identical() {
    let db = g1();
    let hole = first_hole(&db);
    let first = serde_json::to_string(&set(&db, hole, "radius", json!("6")).unwrap_err()).unwrap();
    for run in 1..RUNS {
        let again =
            serde_json::to_string(&set(&db, hole, "radius", json!("6")).unwrap_err()).unwrap();
        assert_eq!(first, again, "run {run}: the refusal changed");
    }
}

#[test]
fn the_input_is_never_modified() {
    let db = g1();
    let before = serde_json::to_string(&db).unwrap();
    let hole = first_hole(&db);
    let _ = set(&db, hole, "radius", json!(6.0));
    let _ = set(&db, hole, "radius", json!(0.0));
    let _ = set(&db, EntityId::new(u64::MAX), "radius", json!(6.0));
    assert_eq!(serde_json::to_string(&db).unwrap(), before);
}
