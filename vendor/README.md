# vendor/printpdf

printpdf 0.7.0, unmodified except for one determinism fix. MIT licensed —
see `printpdf/LICENSE`.

Two collections were serialized straight out of a `HashMap`. Rust's default
hasher is randomly seeded per process, so identical input produced different
bytes on every run. Entry order in a PDF resource dictionary and in the page
annotation array carries no meaning, so rendering is unaffected either way.

`src/xobject.rs` — image XObjects. Hit any document with two or more code
blocks, tables or alerts, since each background panel mints its own XObject:

    -use std::collections::HashMap;
    +use std::collections::BTreeMap;
    -    objects: HashMap<String, XObject>,
    +    objects: BTreeMap<String, XObject>,

`src/link_annotation.rs` — link annotations, found only after fixing the first
one: a document with no links but many images went stable while this README,
which has both, kept varying.

    -use std::collections::HashMap;
    +use std::collections::BTreeMap;
    -    link_annotations: HashMap<String, LinkAnnotation>,
    +    link_annotations: BTreeMap<String, LinkAnnotation>,
    -    type IntoIter = std::collections::hash_map::IntoIter<String, LinkAnnotation>;
    +    type IntoIter = std::collections::btree_map::IntoIter<String, LinkAnnotation>;

Other `HashMap`s in the crate were checked and left alone: `all_graphics_states`,
`patterns` and `page_id_to_obj` are never iterated, and `glyph_ids` feeds a
`BTreeMap` that re-sorts it. `bookmarks` is iterated but mdx never sets any.

Wired in through `[patch.crates-io]` in the workspace `Cargo.toml`, because
printpdf is reached transitively via genpdfi, which pins 0.7.

## Why vendored rather than upstreamed

printpdf is at 0.12 and already fixed this — it is all `BTreeMap` now — but
0.12 is a ground-up rewrite that no genpdf-family crate has been ported to, so
a backport to 0.7 would not be accepted. genpdfi's repository is **archived**,
so there is no upstream to ask for a bump either.

Revisit if any genpdf-family crate ports to printpdf 0.12; this directory can
then be deleted outright.

Only the files reachable from the build are kept: `src/`, `LICENSE`, and the
four assets referenced by `include_str!`/`include_bytes!`.

# vendor/genpdfi

genpdfi 0.2.7, unmodified except for one added primitive. Apache-2.0 OR MIT —
see `genpdfi/LICENSES/`.

`Area` exposed `draw_line`, a *stroked* polyline, and nothing that fills a
path. `Area.layer` is private with no accessor and `Context` carries only the
font cache, so an `Element` had no way to reach printpdf's drawing API — which
does support filled Bézier paths. That left mdx rasterizing every code block
and table background: a flat-colour PNG per panel, written to a temp file,
decoded and embedded as an image XObject.

Added `Area::fill_polygon` plus the `Layer::add_filled_shape` it calls, both
modelled directly on the existing `draw_line`/`add_line_shape` pair and going
through the same private `position()`/`transform_position()` mapping. They emit
`printpdf::Polygon { mode: PaintMode::Fill }`.

## Why vendored rather than upstreamed

genpdfi's repository is **archived** — no issues, no releases. There is
nowhere to send this.

Revisit if a maintained genpdf-family crate appears; `fill_polygon` is a
generic addition that would be worth offering upstream if one does.

Only `src/`, `Cargo.toml`, `README.md` and `LICENSES/` are kept.
