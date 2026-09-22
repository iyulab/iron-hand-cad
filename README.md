# iron-hand-cad

Deterministic hands for CAD drawings: takes an entity reference, a verb and parameters, and returns a new drawing state produced by exact geometric operations.

Built as a tool to be handed to an agent. It contains no AI and no UI of its own.

## What it does

- **Operate** — apply a geometric edit (for example, change a hole diameter) to the entity a reference points at.
- **Return a new state** — the input model is never modified; the result is a new model state.

Models are expressed in the [uncad-model](https://github.com/iyulab/uncad-model) entity model, and
entity references in the form [iron-scout-cad](https://github.com/iyulab/iron-scout-cad) resolves
them to.

## What it is not

- Not a CAD application, and not a replacement for one. Final authoring belongs in a CAD application.
- Not a generator. It edits existing drawings; it does not create drawings from nothing.
- Not an ML library. It performs no inference — no generative editing, no LLM calls.
- Not a file writer. It operates on an in-memory entity model and returns one.
- Not a geometry kernel. It builds a command layer on top of existing geometry primitives.

## Status

0.x. One verb, `set(&db, target, path, value)`: the drawing with exactly that field of exactly
that entity changed, or a `Refusal` that names why not. Fields are named the way the model's JSON
form names them (`radius`, `center.x`, `common.layer`). The rules are in
[docs/principles.md](docs/principles.md); the verb set is [docs/verbs.md](docs/verbs.md).

```rust
let before: uncad_model::CadDatabase = /* from a parser, or from its JSON */;
let hole = /* an entity reference */;
let after = iron_hand_cad::set(&before, hole, "radius", serde_json::json!(6.0))?;
// `after` differs from `before` in exactly that field of exactly that entity
```

## License

MIT
