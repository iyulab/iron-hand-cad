# Verbs

> This document lists the verbs currently in force. A verb not listed here does not exist; adding one is a proposal (principles, section 5).

The set is deliberately small. A new editing capability is expressed with an existing verb and new parameters first.

## `set`

```rust
pub fn set(db: &CadDatabase, target: EntityId, path: &str, value: serde_json::Value)
    -> Result<CadDatabase, Refusal>
```

The drawing with the field at `path` of the entity `target` set to `value`, and nothing else changed. The input is not modified.

| Parameter | What it is |
|---|---|
| `target` | An entity reference, as [iron-scout-cad](https://github.com/iyulab/iron-scout-cad) issues them. Every copy of the entity in the drawing is edited (model space lists its entities both at the top level and in its block record), so the new state is consistent |
| `path` | The field, named the way the model's JSON form names it: `radius`, `center.x`, `vertices[2].point.y`, `vertices[2].bulge`, `common.layer`. This is the same vocabulary [iron-diff-cad](https://github.com/iyulab/iron-diff-cad) reports changed fields in, so the path given to `set` is the path the diff reports back |
| `value` | The new value, in the model's JSON form. It must be of the kind the field holds (a number for a number, an object for a point); a field that is currently `null` accepts any kind and is then checked by the model's own type |

### What is refused

Each refusal is a `Refusal` value that names its reason; an edit is never carried out partially.

| Reason | When |
|---|---|
| `NO_SUCH_ENTITY` | No entity of the drawing carries `target` |
| `INCONSISTENT_COPIES` | The drawing lists the entity more than once and the copies disagree -- the drawing is inconsistent, so it is not edited |
| `MALFORMED_PATH` | `path` is not of the form `key`, `key.key`, `key[n].key` |
| `NO_SUCH_FIELD` | The entity has no field at `path` |
| `NOT_EDITABLE` | `path` is the type tag, `common` as a whole, or one of `common.id` · `common.origin` · `common.confidence` · `common.source_handle`. Identity is what the diff pairs the two states by; provenance and confidence are the source's statement, and this library never raises a confidence |
| `WRONG_KIND` | `value` is of a different kind than the field holds |
| `CONSTRAINT` | `value` violates a constraint the model itself does not enforce. Currently: `radius` and `text_height` must be positive |
| `INVALID` | With `value` in place the entity no longer deserializes as the model's type -- for example a point without `z` |

### What it does not do

- It does not mark the edited entity as derived. The entity keeps its reference ID, provenance and source handle: it still came from where it came from, and *what changed* is the diff's statement, not this library's. An entity this library *creates* (there is no such verb yet) would carry `DERIVED` provenance and a fresh reference ID.
- It edits definitions, not instances. An entity that lives in a block definition is drawn once per reference to that block; setting a field of it changes every one of those places. The chain of references a hit was reached through (a `via` list from [iron-scout-cad](https://github.com/iyulab/iron-scout-cad)) says where the entity was seen, not which instance to edit -- there is no per-instance edit.
- It does not keep dependent geometry consistent. Setting a dimension's definition point does not move the dimension's text; a chain of edits is a sequence of `set` calls, each verified by its own diff.
