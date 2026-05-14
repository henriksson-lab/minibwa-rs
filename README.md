# minibwa-rs










## Future Optimizations To Maybe Undo

Some memory-layout changes were made to close RSS gaps with the original C
implementation. They are worth revisiting if they do not also help speed:

- `mb_bseq1_t.qual` and `mb_bseq1_t.comment` currently use a thin optional
  owned string. This reduces RSS when those fields are absent, but
  `Option<Box<str>>` is more idiomatic Rust and may be preferable if speed is
  neutral or worse. A recent retry of `Option<Box<str>>` was slower on the
  yeast speed fixture, so keep the thin representation for now.
- `mb_extra_t` is packed to match C's flexible-array header closely. This saves
  RSS, but it is less idiomatic than a clearer Rust representation and should
  stay only if the memory win is important enough.
- Per-read hit lists use a thin raw owned buffer instead of `Vec<mb_hit_t>` in
  the batch table. This saves header space; `Vec` would be simpler if the RSS
  gap is no longer a priority.

When optimizing further, prefer speed improvements first. Keep RSS-specific raw
layout changes only when they do not interfere with hot-path throughput or when
matching C memory use is the explicit goal.
