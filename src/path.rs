//! Field paths in the model's JSON form: `radius`, `center.x`,
//! `vertices[2].y`, `common.layer.data`. A path names a node of an entity's
//! JSON tree; segments are object keys separated by `.`, each optionally
//! followed by one or more `[n]` array indices.

use serde_json::Value;

/// One step into the tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Segment {
    Key(String),
    Index(usize),
}

/// Parses `path` into its segments. `None` for a path that is not well
/// formed (empty, an empty key, an index that is not a number).
pub fn parse(path: &str) -> Option<Vec<Segment>> {
    if path.is_empty() {
        return None;
    }
    let mut out = Vec::new();
    for part in path.split('.') {
        let (key, rest) = match part.find('[') {
            Some(i) => (&part[..i], &part[i..]),
            None => (part, ""),
        };
        if key.is_empty() || key.contains(']') {
            return None;
        }
        out.push(Segment::Key(key.to_string()));
        let mut rest = rest;
        while !rest.is_empty() {
            let close = rest.find(']')?;
            if !rest.starts_with('[') {
                return None;
            }
            let index: usize = rest[1..close].parse().ok()?;
            out.push(Segment::Index(index));
            rest = &rest[close + 1..];
        }
    }
    Some(out)
}

/// The node `segments` name in `root`, mutably, or `None` when any step
/// finds nothing.
pub fn get_mut<'a>(root: &'a mut Value, segments: &[Segment]) -> Option<&'a mut Value> {
    let mut node = root;
    for segment in segments {
        node = match segment {
            Segment::Key(k) => node.as_object_mut()?.get_mut(k)?,
            Segment::Index(i) => node.as_array_mut()?.get_mut(*i)?,
        };
    }
    Some(node)
}

/// The name of the last key of a path (`radius` for `holes[2].radius`),
/// which is what the constraint table is keyed on.
pub fn last_key(segments: &[Segment]) -> Option<&str> {
    segments.iter().rev().find_map(|s| match s {
        Segment::Key(k) => Some(k.as_str()),
        Segment::Index(_) => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_parse_into_keys_and_indices() {
        assert_eq!(
            parse("vertices[2].y").unwrap(),
            [
                Segment::Key("vertices".into()),
                Segment::Index(2),
                Segment::Key("y".into())
            ]
        );
        assert_eq!(parse("radius").unwrap(), [Segment::Key("radius".into())]);
        assert_eq!(
            parse("common.layer.data").unwrap(),
            [
                Segment::Key("common".into()),
                Segment::Key("layer".into()),
                Segment::Key("data".into())
            ]
        );
    }

    #[test]
    fn malformed_paths_are_rejected_not_repaired() {
        for bad in ["", ".", "a..b", "a[", "a[x]", "a]", "[0]", "a[0"] {
            assert!(parse(bad).is_none(), "{bad:?} parsed");
        }
    }

    #[test]
    fn the_last_key_skips_indices() {
        let p = parse("vertices[2]").unwrap();
        assert_eq!(last_key(&p), Some("vertices"));
    }
}
