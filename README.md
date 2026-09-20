# iron-hand-cad

Deterministic hands for CAD drawings: takes an entity reference, a verb and parameters, and returns a new drawing state produced by exact geometric operations.

Built as a tool to be handed to an agent. It contains no AI and no UI of its own.

## What it does

- **Operate** — apply a geometric edit (for example, change a hole diameter) to the entity a reference points at.
- **Return a new state** — the input model is never modified; the result is a new model state, ready to be diffed against the old one.

References come from [iron-scout-cad](https://github.com/iyulab/iron-scout-cad), which also produces the numeric diff used to verify an edit.

## What it is not

- Not a CAD application, and not a replacement for one. Final authoring belongs in a CAD application.
- Not a generator. It edits existing drawings; it does not create drawings from nothing.
- Not an ML library. It performs no inference — no generative editing, no LLM calls.
- Not a file writer. It operates on an in-memory entity model and returns one.
- Not a geometry kernel. It builds a command layer on top of existing geometry primitives.

## Status

Pre-implementation. No code yet. The design principles are settled and documented in [docs/principles.md](docs/principles.md); read that before proposing anything.

## License

MIT
