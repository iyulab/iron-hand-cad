//! `set` on real drawings, verified the only accepted way: by the numeric
//! diff of the state before and the state after. The golden models in
//! `tests/golden/` are the *before* states; no parser is involved.

use iron_diff_cad::{diff, Change, DiffOptions, Matching, Verdict};
use iron_hand_cad::{set, Refusal};
use serde_json::json;
use uncad_model::model::{Confidence, Entity, EntityId};
use uncad_model::CadDatabase;

fn g1() -> CadDatabase {
    serde_json::from_str(include_str!("golden/g1.expected.json"))
        .expect("the golden model deserializes")
}

fn g2() -> CadDatabase {
    serde_json::from_str(include_str!("golden/g2.expected.json"))
        .expect("the golden model deserializes")
}

/// The reference IDs of G1's four holes, in file order.
fn holes(db: &CadDatabase) -> Vec<EntityId> {
    db.entities
        .iter()
        .filter_map(|e| match e {
            Entity::Circle(c) => Some(c.common.id),
            _ => None,
        })
        .collect()
}

/// The one MODIFIED entry of a change set that holds nothing else.
fn the_only_modified(before: &CadDatabase, after: &CadDatabase) -> iron_diff_cad::Modified {
    let set = diff(before, after, DiffOptions::default());
    assert_eq!(set.matching, Matching::Reference);
    assert_eq!(set.changes.len(), 1, "{:?}", set.changes);
    match &set.changes[0] {
        Change::Modified(m) => m.clone(),
        other => panic!("expected MODIFIED, got {other:?}"),
    }
}

#[test]
fn a_hole_diameter_change_is_exactly_that_field_of_exactly_that_entity() {
    let before = g1();
    let hole = holes(&before)[0];
    let after = set(&before, hole, "radius", json!(6.0)).unwrap();

    let m = the_only_modified(&before, &after);
    assert_eq!(m.id, hole);
    assert_eq!(m.entity_type, "CIRCLE");
    assert_eq!(m.confidence, Confidence::High);
    assert_eq!(m.fields.len(), 1, "{:?}", m.fields);
    assert_eq!(m.fields[0].path, "radius");
    assert_eq!(m.fields[0].delta, Some(1.0));
    assert_eq!(m.fields[0].verdict, Verdict::Beyond);
}

#[test]
fn the_path_given_is_the_path_the_diff_reports() {
    let before = g1();
    let hole = holes(&before)[1];
    for (path, value) in [
        ("center.x", json!(21.5)),
        ("common.layer.data", json!("HOLES-2")),
        ("common.color_index", json!(3)),
    ] {
        let after = set(&before, hole, path, value).unwrap();
        let m = the_only_modified(&before, &after);
        assert_eq!(m.fields.len(), 1, "{path}: {:?}", m.fields);
        assert_eq!(m.fields[0].path, path);
    }
}

#[test]
fn a_whole_point_can_be_set_and_only_the_moved_coordinates_are_reported() {
    let before = g1();
    let hole = holes(&before)[2];
    let after = set(
        &before,
        hole,
        "center",
        json!({"x": 25.0, "y": 80.0, "z": 0.0}),
    )
    .unwrap();
    let m = the_only_modified(&before, &after);
    let paths: Vec<&str> = m.fields.iter().map(|f| f.path.as_str()).collect();
    assert_eq!(paths, ["center.x"]);
}

#[test]
fn an_entity_inside_a_nested_block_definition_is_edited_in_place() {
    let before = g2();
    let line = before
        .tables
        .block_records
        .values()
        .flat_map(|b| b.entities.iter())
        .find_map(|e| match e {
            Entity::Line(l) => Some(l.common.id),
            _ => None,
        })
        .expect("G2 draws a line inside its innermost block");
    let after = set(&before, line, "end_point.x", json!(12.0)).unwrap();
    let m = the_only_modified(&before, &after);
    assert_eq!(m.id, line);
    assert_eq!(m.entity_type, "LINE");
    assert_eq!(m.fields.len(), 1, "{:?}", m.fields);
    assert_eq!(m.fields[0].path, "end_point.x");
}

#[test]
fn every_copy_of_a_model_space_entity_is_edited_so_the_state_stays_consistent() {
    let before = g1();
    let hole = holes(&before)[3];
    let after = set(&before, hole, "radius", json!(4.5)).unwrap();
    let copies: Vec<&Entity> = after
        .entities
        .iter()
        .chain(
            after
                .tables
                .block_records
                .values()
                .flat_map(|b| b.entities.iter()),
        )
        .filter(|e| e.common().id == hole)
        .collect();
    assert!(copies.len() >= 2, "G1 lists model space twice");
    assert!(copies.iter().all(|c| *c == copies[0]));
    for c in copies {
        let Entity::Circle(c) = c else {
            panic!("a hole")
        };
        assert_eq!(c.radius, 4.5);
    }
}

#[test]
fn confidence_never_rises_and_provenance_is_untouched() {
    let mut before = g1();
    let hole = holes(&before)[0];
    for e in before.entities.iter_mut().chain(
        before
            .tables
            .block_records
            .values_mut()
            .flat_map(|b| b.entities.iter_mut()),
    ) {
        if let Entity::Circle(c) = e {
            if c.common.id == hole {
                c.common.confidence = Confidence::Low;
            }
        }
    }
    let after = set(&before, hole, "radius", json!(6.0)).unwrap();
    let edited = after
        .entities
        .iter()
        .find(|e| e.common().id == hole)
        .unwrap()
        .common();
    let original = before
        .entities
        .iter()
        .find(|e| e.common().id == hole)
        .unwrap()
        .common();
    assert_eq!(edited.confidence, Confidence::Low);
    assert_eq!(edited.origin, original.origin);
    assert_eq!(edited.source_handle, original.source_handle);
    assert_eq!(edited.id, original.id);
    let m = the_only_modified(&before, &after);
    assert_eq!(m.confidence, Confidence::Low);
    assert_eq!(m.fields.len(), 1, "{:?}", m.fields);
}

#[test]
fn refusals_name_their_reason_and_change_nothing() {
    let before = g1();
    let hole = holes(&before)[0];
    let cases: Vec<(EntityId, &str, serde_json::Value, Refusal)> = vec![
        (
            EntityId::new(u64::MAX),
            "radius",
            json!(6.0),
            Refusal::NoSuchEntity {
                id: EntityId::new(u64::MAX),
            },
        ),
        (
            hole,
            "radius..x",
            json!(6.0),
            Refusal::MalformedPath {
                path: "radius..x".into(),
            },
        ),
        (
            hole,
            "diameter",
            json!(12.0),
            Refusal::NoSuchField {
                id: hole,
                entity_type: "CIRCLE".into(),
                path: "diameter".into(),
            },
        ),
        (
            hole,
            "radius",
            json!("6"),
            Refusal::WrongKind {
                path: "radius".into(),
                expected: "number".into(),
                given: "string".into(),
            },
        ),
        (
            hole,
            "radius",
            json!(0.0),
            Refusal::Constraint {
                path: "radius".into(),
                rule: "radius must be positive".into(),
            },
        ),
        (
            hole,
            "radius",
            json!(-1.0),
            Refusal::Constraint {
                path: "radius".into(),
                rule: "radius must be positive".into(),
            },
        ),
    ];
    for (id, path, value, expected) in cases {
        assert_eq!(set(&before, id, path, value).unwrap_err(), expected);
    }
    for path in [
        "type",
        "common",
        "common.id",
        "common.origin",
        "common.confidence",
        "common.source_handle",
        "common.source_handle.data",
    ] {
        match set(&before, hole, path, json!(1)) {
            Err(Refusal::NotEditable { path: p, .. }) => assert_eq!(p, path),
            other => panic!("{path}: expected NOT_EDITABLE, got {other:?}"),
        }
    }
    // A structurally wrong value: a point without `z` is not a Point3D.
    match set(&before, hole, "center", json!({"x": 1.0, "y": 2.0})) {
        Err(Refusal::Invalid { path, .. }) => assert_eq!(path, "center"),
        other => panic!("expected INVALID, got {other:?}"),
    }
    // G1's title block is an INSERT with attributes; an attribute's text
    // height is a text height.
    let text = before
        .entities
        .iter()
        .find_map(|e| match e {
            Entity::Attrib(a) => Some(a.common.id),
            _ => None,
        })
        .expect("G1's title block has attributes");
    assert!(matches!(
        set(&before, text, "text_height", json!(0.0)),
        Err(Refusal::Constraint { .. })
    ));
}

#[test]
fn a_refusal_serializes_with_its_reason_as_the_tag() {
    let before = g1();
    let hole = holes(&before)[0];
    let refusal = set(&before, hole, "radius", json!("6")).unwrap_err();
    assert_eq!(
        serde_json::to_value(&refusal).unwrap(),
        json!({"reason": "WRONG_KIND", "path": "radius", "expected": "number", "given": "string"})
    );
}

#[test]
fn a_chain_of_edits_is_verified_by_one_diff_listing_every_field() {
    // Two edits to one entity, then one to another: the diff of the first and
    // last states lists exactly those three fields on exactly those two
    // entities, in the contract's order. The chain is the caller's; each
    // step is a plain `set`.
    let before = g1();
    let holes = holes(&before);
    let step1 = set(&before, holes[0], "radius", json!(6.0)).unwrap();
    let step2 = set(&step1, holes[0], "center.x", json!(22.0)).unwrap();
    let after = set(&step2, holes[1], "radius", json!(4.0)).unwrap();

    let set = diff(&before, &after, DiffOptions::default());
    assert_eq!(set.changes.len(), 2, "{:?}", set.changes);
    let modified: Vec<&iron_diff_cad::Modified> = set
        .changes
        .iter()
        .map(|c| match c {
            Change::Modified(m) => m,
            other => panic!("expected MODIFIED, got {other:?}"),
        })
        .collect();
    assert_eq!(modified[0].id, holes[0]);
    let paths: Vec<&str> = modified[0].fields.iter().map(|f| f.path.as_str()).collect();
    assert_eq!(paths, ["center.x", "radius"]);
    assert_eq!(modified[1].id, holes[1]);
    assert_eq!(modified[1].fields.len(), 1);
    assert_eq!(modified[1].fields[0].path, "radius");
    // Each intermediate state is untouched by the later steps.
    assert_eq!(the_only_modified(&before, &step1).fields[0].path, "radius");
}

#[test]
fn a_refusal_is_an_error_with_a_readable_message() {
    let before = g1();
    let hole = holes(&before)[0];
    let err: Box<dyn std::error::Error> =
        set(&before, hole, "radius", json!(0.0)).unwrap_err().into();
    assert_eq!(err.to_string(), "\"radius\": radius must be positive");
    let err = set(&before, hole, "diameter", json!(1.0)).unwrap_err();
    assert_eq!(err.to_string(), "CIRCLE has no field at \"diameter\"");
}

/// Model space is listed twice (the top level and its block record); when
/// the two copies of the target disagree the drawing contradicts itself,
/// and editing one copy, or both to one value, would pick a side. The edit
/// is refused and the drawing is returned to nobody changed.
#[test]
fn copies_that_disagree_are_refused_not_reconciled() {
    let mut before = g1();
    let hole = holes(&before)[0];
    let block_copy = before
        .tables
        .block_records
        .values_mut()
        .flat_map(|b| b.entities.iter_mut())
        .find(|e| e.common().id == hole)
        .expect("G1 lists model space in its block record too");
    let Entity::Circle(circle) = block_copy else {
        panic!("a hole")
    };
    circle.radius += 1.0;
    let untouched = before.clone();

    let refusal = set(&before, hole, "center.x", json!(0.0)).unwrap_err();
    assert_eq!(refusal, Refusal::InconsistentCopies { id: hole });
    assert_eq!(before, untouched);

    // Only the target's copies are compared: another entity still edits.
    let other = holes(&before)[1];
    assert!(set(&before, other, "radius", json!(6.0)).is_ok());
}

/// An edit changes an entity that is there; it never makes one. On a
/// drawing with no entities every target is absent, and the answer is the
/// refusal, not a drawing with something new in it.
#[test]
fn a_drawing_with_no_entities_gets_a_refusal_not_a_new_entity() {
    let mut empty = g1();
    let hole = holes(&empty)[0];
    empty.entities.clear();
    for block in empty.tables.block_records.values_mut() {
        block.entities.clear();
    }
    assert_eq!(
        set(&empty, hole, "radius", json!(6.0)).unwrap_err(),
        Refusal::NoSuchEntity { id: hole }
    );
}
