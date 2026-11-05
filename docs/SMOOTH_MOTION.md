# Smooth motion (optional, off by default)

The game logic always runs at 60 Hz, as on the Wii. Smooth motion adds 1–4 extra frames between two game
frames for 120/180/240 Hz screens. Turn it on in the launcher (Settings) or the stats window; it switches live.

How it works (`crates/wprt-gx/src/backend.rs`, `Interp`):
1. Every frame's draws (vertices, state, clears and EFB copies) are recorded.
2. After the real frame is finished, the frame is replayed into private render targets with each object's
   clip-space positions blended between the previous and the current frame. EFB copies are replayed into private
   textures; effects that read last frame's copy get a snapshot taken before it was overwritten.
3. The host shows the in-between frames, then the real one, spread over the frame interval.

Objects are paired with the previous frame by pipeline, textures, vertex count and material constants. Look-alike
draws that appear three or more times (particles, coins), draws whose texture coordinates differ, non-rigid
motion and large jumps are not blended; they show their state from the nearest real frame.

Trade-offs: about 10 ms more input delay (the next real frame must exist before the in-between frames can be
drawn), and occasional small glitches where pairing fails. The game's own state is never touched: fixtures with
smooth motion on are still exact.

## Notes for a VR port

60 Hz logic is fine, but a headset needs new images at 72–120 Hz from the current head pose. The record/replay
machinery above is the right base: instead of blending two frames, replay the latest frame with the camera from
the current head pose (no extra latency) and twice per frame for the two eyes. 2D menus would go on a virtual panel.
