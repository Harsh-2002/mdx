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
