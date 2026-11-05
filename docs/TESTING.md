# Accuracy fixtures

The project's rule: a change is accepted only if the game behaves *exactly* as before.

A fixture is a deterministic replay:
- **scripted input** (`tests/fixtures/F*.txt`): button/pointer events at fixed frame times,
- a **frame-counting clock** (`WP_SCRIPT_CLOCK=frames`, `WP_DETERMINISTIC=1`, single GX thread), so timing never depends on the PC,
- per frame, a **hash of the emulated state** written to `frames.txt`,
- every 60 frames, a **screenshot** `frameNNN.png`.

Two builds are equal on a fixture when `frames.txt` and every PNG are byte-identical.

## Running one

```
python tools/fixtures/run_fixture.py --install <launcher install folder> --script tests/fixtures/F23-boardisland.txt \
    --frames 19500 --name F23-before
# change something, rebuild, then
python tools/fixtures/run_fixture.py ... --name F23-after --gold fixture-runs/F23-before/observe
```

`RESULT F23-after ROWS_SAME png 325 diff 0` means exact. Frame counts are in `tests/fixtures/fixtures.toml`.
A run takes a few minutes (F23 ≈ 7 min on an 8-core desktop); fixtures run in parallel fine.

The original reference ("gold") runs came from the upstream C++ build. Gold output is derived from the game, so it
stays on your machine. Make your own gold set from a build you trust before changing anything.

## The 23 fixtures

`F1`–`F23` in `tests/fixtures/` (the file names say what each one plays: boot and menus, a board, Free Play pages,
every minigame page, 1-vs-3 and pair minigames, and the party modes Globe Trot, Swap Meet, Bingo and Board Game Island).

Known accepted differences against the C++ reference: F8 frame 26259 (a NaN sign in memory), plus NaN-only rows in
F10 and F12. They are listed in `tests/fixtures/fixtures.toml` and are not regressions.

## Faster checks while iterating

Use F23 + F17 + F20 + F8 (about 10 minutes in parallel). Run all 23 before a release.
