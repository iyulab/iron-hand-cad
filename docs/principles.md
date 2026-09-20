# Principles

> This document describes the rules currently in force. A sentence here that is wrong is a bug.

## 1. Determinism is non-negotiable

The same model, reference, verb and parameters produce the same new state. The library never calls an inference endpoint, never embeds a model, and never guesses.

An operation that cannot be carried out exactly — the reference does not resolve, the parameters violate a constraint, the result would be ambiguous — is **refused with a reason**. It is not approximated. A value that cannot be established is returned as **"unknown"**, a first-class value, not filled with a default.

*What this costs:* an interaction that a generative editor would complete "well enough" will sometimes be refused here. That is intended — in a drawing, two millimetres off is wrong, not close.

## 2. The input is never modified

Every operation returns a new model state. The original model, and any file it came from, are untouched. The library has no file-writing path.

What changed between the old and the new state is established by a numeric diff (see [iron-scout-cad](https://github.com/iyulab/iron-scout-cad)), not by this library describing its own work.

*What this costs:* callers that want a modified file must serialize the new state themselves.

## 3. Provenance and confidence travel with every entity

The entity model carries, for every entity: a **reference ID**, a **provenance**, and a **confidence** (including "unknown").

- Entities produced by an operation are marked as such in their provenance.
- Confidence never rises on its own. An entity derived from a low-confidence entity is not more trustworthy than its source.
- The library does not need to know *why* an entity has low confidence. It acts on the marker alone.

## 4. One reference scheme

This library accepts entity references exactly as [iron-scout-cad](https://github.com/iyulab/iron-scout-cad) issues them and does not define a reference or coordinate system of its own. If "here" can mean two different things on the two sides, the design is wrong.

## 5. A small, fixed set of verbs

The public surface is a small fixed set of verbs extended through parameters, with schemas loadable on demand. A new editing capability is expressed with existing verbs and new parameters first — a verb per capability is how a tool turns into a menu. A new verb is proposed only with evidence that the existing ones cannot express it.

*What this costs:* some edits will be more awkward to express than a dedicated verb would make them.

## 6. Domain neutrality

The library knows geometry and CAD entities. It does not know *why* an edit is being made. Concepts that only one consumer needs — business terms, workflow steps, labels — belong in that consumer's adapter, including when they are dressed in generic-sounding names.

The test: *would a third party using this library for the first time need the same thing in the same place?* If not, it does not belong here.

## 7. Trade-off order

When goals collide, the earlier one wins:

> API simplicity › coverage › development speed › backward compatibility

Determinism is not on this list because it is never traded. Licensing is not on this list because it is a constraint: the dependency tree of this crate is permissive-only (MIT / Apache-2.0 / BSD).

## 8. Compatibility

The crate is in 0.x. When a more correct design is found, a breaking change is the normal way to adopt it; it is not deferred for migration cost. Breaking changes bump the minor version. The major version is not bumped without an explicit maintainer decision.

## 9. Contributing

| Just do it | Propose first | Discuss before any work |
|---|---|---|
| Tests · bug fixes and refactors that leave the public API unchanged · docs | Public API changes · new dependencies · **new verbs** · changing the meaning of, or removing, an entity-model field | Anything that adds inference · anything in "What it is not" · copyleft dependencies · changes to the provenance/confidence contract, including new confidence values |

Adding an entity type or field is fine without prior discussion; mention it in the change description. If it is unclear which column a change falls in, treat it as the stricter one.

Tests for things the library **must not do** — modifying its input, raising a confidence, letting a wrong edit pass as correct, failing silently — are required, and their failure count is always zero. No test decides pass/fail by comparing rendered images.
