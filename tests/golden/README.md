# Golden cases

Copies of what the `uncad-model` repository's golden writer produces, one expected model per case:

| File | What it is |
|---|---|
| `<case>.expected.json` | The model a reader must produce from that case's synthetic drawing; the spec is the oracle |

Cases here: `g1` (a general part with four holes and a title block -- the *before* state of the hole-diameter edit), `g2` (blocks nested three deep -- an entity inside a block definition). The tests read these models directly -- no parser is involved -- edit them with this crate's verbs, and check the numeric diff of before and after.

The files are generated, not hand-written. To regenerate after a change to the writer or the spec, from a checkout of `uncad-model`:

```
cargo run -p uncad-model-golden --example write_case -- <case> <case>.dxf <case>.expected.json
```

and copy the JSON here. A tree that carries both repositories side by side checks that the copies have not drifted from the writer.
