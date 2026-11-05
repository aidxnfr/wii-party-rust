# Contributing

1. **Never commit anything from the game**: no disc files, extracted assets, generated code (`generated/`),
   screenshots/recordings of fixture output, or saves. `.gitignore` covers the usual places; check `git status`.
2. **Exactness first.** Any change to the runtime, hardware, renderer or generator must keep the fixtures
   byte-identical (`docs/TESTING.md`). Performance work counts only if it is exact.
3. **Fix generators, not generated code.** If translated code is wrong, fix `recompiler/rust/cpp2rs.py` or
   `recompiler/dsp/*` and regenerate.
4. **Keep provenance.** Code ported from wiiparty-recomp or Dolphin keeps a short "port of <file>" note and its
   SPDX line; add new third-party code to `THIRD_PARTY_NOTICES.md`.
5. **Pins live in one place** (`docs/UPDATING.md`).
6. Rust style: `cargo fmt` for new code; existing ported code keeps its compact form to stay diff-able against the
   C++ it came from.
