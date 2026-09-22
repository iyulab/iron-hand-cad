# iron-hand-cad

Deterministic hands for CAD drawings: takes an entity reference, a verb and parameters, and returns a new drawing state produced by exact geometric operations.

Built as a tool to be handed to an agent. It contains no AI and no UI of its own.

## What it does

- **Operate** — apply a geometric edit (for example, change a hole diameter) to the entity a reference points at.
- **Return a new state** — the input model is never modified; the result is a new model state, ready to be diffed against the old one.

References come from [iron-scout-cad](https://github.com/iyulab/iron-scout-cad). Whether an edit did what was intended is established by [iron-diff-cad](https://github.com/iyulab/iron-diff-cad). Models are expressed in [uncad-model](https://github.com/iyulab/uncad-model).

## What it is not

- Not a CAD application, and not a replacement for one. Final authoring belongs in a CAD application.
- Not a generator. It edits existing drawings; it does not create drawings from nothing.
- Not an ML library. It performs no inference — no generative editing, no LLM calls.
- Not a file writer. It operates on an in-memory entity model and returns one.
- Not a geometry kernel. It builds a command layer on top of existing geometry primitives.

## Status

0.x. One verb, `set(&db, target, path, value)`: the drawing with exactly that field of exactly that entity changed, or a `Refusal` that names why not. Fields are named the way the model's JSON form names them (`radius`, `center.x`, `common.layer`), which is also the vocabulary the diff reports changed fields in -- the edit and its verification speak one language. The rules are in [docs/principles.md](docs/principles.md); the verb set is [docs/verbs.md](docs/verbs.md). Read both before proposing anything.

```rust
let before: uncad_model::CadDatabase = /* from a parser, or from its JSON */;
let hole = /* an entity reference, from iron-scout-cad */;
let after = iron_hand_cad::set(&before, hole, "radius", serde_json::json!(6.0))?;
let set = iron_diff_cad::diff(&before, &after, iron_diff_cad::DiffOptions::default());
// exactly one MODIFIED entry, exactly one field: `radius`, delta 1.0
```

## License

MIT
