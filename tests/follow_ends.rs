//! Moving the ends a reading names: the paths `iron-scout-cad`'s `coincident_ends` gives for the ends
//! that sit where one end does are the paths `set` writes, and the numeric diff of the drawing before
//! and after lists exactly those fields -- nothing else moves.

use iron_diff_cad::{diff, Change, DiffOptions, Matching};
use iron_hand_cad::set;
use iron_scout_cad::{coincident_ends, Anchor};
use serde_json::{json, Value};
use uncad_model::model::EntityId;
use uncad_model::CadDatabase;

/// The first golden model, with three lines added to its model space: A runs to (10, 0), B starts at
/// that same point, C starts 0.05 mm from it, and D starts well away. The model is in millimetres.
fn drawing() -> CadDatabase {
    let mut doc: Value = serde_json::from_str(include_str!("golden/g1.expected.json"))
        .expect("the golden model deserializes");
    let common = doc["entities"][0]["common"].clone();
    let line = |id: &str, from: (f64, f64), to: (f64, f64)| {
        let mut common = common.clone();
        common["id"] = json!(id);
        common["source_handle"] = json!({"type": "ABSENT"});
        json!({
            "type": "LINE",
            "common": common,
            "start_point": {"x": from.0, "y": from.1, "z": 0.0},
            "end_point": {"x": to.0, "y": to.1, "z": 0.0},
        })
    };
    let entities = doc["entities"].as_array_mut().expect("entities");
    entities.push(line("9001", (0.0, 0.0), (10.0, 0.0)));
    entities.push(line("9002", (10.0, 0.0), (10.0, 10.0)));
    entities.push(line("9003", (10.05, 0.0), (20.0, 0.0)));
    entities.push(line("9004", (50.0, 50.0), (60.0, 50.0)));
    serde_json::from_value(doc).expect("the drawing deserializes")
}

fn id(text: &str) -> EntityId {
    serde_json::from_value(json!(text)).expect("a canonical ID")
}

#[test]
fn moving_the_ends_a_reading_names_changes_exactly_the_fields_it_named() {
    let before = drawing();
    let asked = id("9001");

    let found = coincident_ends(&before, asked, Anchor::End, 100).expect("the drawing has a unit");
    assert_eq!(found.end.path.as_deref(), Some("end_point"));
    let followers: Vec<_> = found.ends.iter().filter(|e| e.can_follow).collect();
    // B's start sits on the end; C's start is 0.05 mm away, inside the 0.1 mm gap; D is far.
    let entities: Vec<String> = followers
        .iter()
        .map(|e| {
            serde_json::to_value(e.entity)
                .unwrap()
                .as_str()
                .unwrap()
                .to_owned()
        })
        .collect();
    assert_eq!(entities, ["9002", "9003"], "{found:?}");

    // One straight move in x of the asked end and every end that can follow it, each written through
    // the path the reading gave, to the position the reading gave in the entity's own coordinates.
    let dx = 2.5;
    let mut after = before.clone();
    for end in std::iter::once(&found.end).chain(followers.iter().copied()) {
        let path = end.path.as_deref().expect("a line's end has a field path");
        after = set(
            &after,
            end.entity,
            path,
            json!({"x": end.local.x + dx, "y": end.local.y, "z": 0.0}),
        )
        .expect("the path the reading gave is a path set writes");
    }

    let changes = diff(
        &before,
        &after,
        DiffOptions {
            matching: Matching::Reference,
            ..DiffOptions::default()
        },
    );
    let mut got: Vec<(String, Vec<String>)> = changes
        .changes
        .iter()
        .map(|c| match c {
            Change::Modified(m) => (
                serde_json::to_value(m.id)
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_owned(),
                m.fields.iter().map(|f| f.path.clone()).collect(),
            ),
            other => panic!("expected MODIFIED, got {other:?}"),
        })
        .collect();
    got.sort();
    let expect = |id: &str, field: &str| (id.to_owned(), vec![format!("{field}.x")]);
    assert_eq!(
        got,
        [
            expect("9001", "end_point"),
            expect("9002", "start_point"),
            expect("9003", "start_point"),
        ],
        "{:?}",
        changes.changes
    );
}
