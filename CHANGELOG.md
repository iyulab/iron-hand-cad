# Changelog

Notable changes to this project are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versioning follows
[Semantic Versioning](https://semver.org/). While the version is 0.x, a breaking change
bumps the minor version.

## [Unreleased]

### Changed

- **Breaking:** Built on the current `uncad-model` API. Field paths follow its JSON form, in
  which a polyline vertex is a point and a bulge: pass `vertices[i].point.x` (and
  `vertices[i].bulge`) to `set` where `vertices[i].x` was passed before.
- **Breaking:** `Refusal` is `#[non_exhaustive]`, so that a later version can name a new
  reason without breaking callers. A `match` on it needs a wildcard arm.

## [0.1.0] - 2026-09-22

Initial release. One verb over an [uncad-model](https://github.com/iyulab/uncad-model)
drawing: `set(&db, target, path, value)` returns the drawing with exactly that field of exactly
that entity changed, or a `Refusal` that names why not. Fields are named the way the model's
JSON form names them (`radius`, `center.x`, `common.layer`); identity and provenance fields are
never editable. The input is never modified, and the same input gives the same output.
