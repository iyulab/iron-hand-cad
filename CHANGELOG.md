# Changelog

Notable changes to this project are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versioning follows
[Semantic Versioning](https://semver.org/). While the version is 0.x, a breaking change
bumps the minor version.

## [Unreleased]

## [0.8.0] - 2026-10-07

### Changed

- Built on `iron-diff-cad` 0.8.0 and `uncad-model` 0.8.0.

## [0.7.0] - 2026-10-07

### Changed

- Built on `iron-diff-cad` 0.7.0 and `uncad-model` 0.7.0.

## [0.6.0] - 2026-10-07

### Changed

- Built on `uncad-model` 0.6.0.
- Built on `iron-diff-cad` 0.6.0.

## [0.5.0] - 2026-10-05

### Changed

- Built on `uncad-model` 0.5.0: a table's cells are fields like any other
  (`grid.rows[r].cells[c].text`); as with a dimension, setting one does not redraw what the
  table's block draws.
- Built on `iron-diff-cad` 0.5.0.

## [0.4.0] - 2026-10-04

### Changed

- Built on `uncad-model` 0.4.0 (a multileader's line type and content; the header's drawing
  identifiers), so it edits drawings of that model; its tests check edits with `iron-diff-cad`
  0.4.0.

### Fixed

- `set` refuses `id`, `origin`, `confidence` and `source_handle` in a nested entity's `common`
  block too (an INSERT's attributes), not only in the entity's own: an edit could rewrite an
  attribute's reference ID, a field a diff does not compare, so the edit went unseen.

## [0.3.0] - 2026-10-02

### Changed

- Built on `uncad-model` 0.3.0 (a drawing's `header`; a multileader's leader roots), so
  it edits drawings of that model; its tests check edits with `iron-diff-cad` 0.3.0.

## [0.2.0] - 2026-09-29

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
