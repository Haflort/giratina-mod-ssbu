# Giratina for Ridley - plugin

Source code of `libgiratina_skin.nro`, the Skyline plugin that comes with the
"Giratina (Altered & Origin Forme) over Ridley" skin for Super Smash Bros. Ultimate (Ridley costumes c04 and c05).

## What it does
Visual only - no changes to damage, knockback or movement.
- Every match starts in Altered Forme; the down taunt switches between Altered and Origin Forme
  (black smoke, darkness, red eye glint, burst and roar).
- Final Smash: Giratina vanishes into a rift in reality and the cutscene plays in the Distortion World
  (the cutscene itself is in the mod's model, motion and camera files). Afterwards he stays in Origin Forme.

It only touches Ridley when he is wearing costume 5 or 6 (c04/c05).

## Requirements (on the Switch)
- Skyline and ARCropolis
- Smashline 2 with nro_hook: https://github.com/HDR-Development/smashline/releases
  (HDR already includes them)

## Building
1. Install Rust and cargo-skyline: https://github.com/jam1garner/cargo-skyline
2. `cargo skyline build --release`
3. The plugin is `target/aarch64-skyline-switch/release/libgiratina_skin.nro`; copy it to
   `sd:/atmosphere/contents/01006A800016E000/romfs/skyline/plugins/`.

## AI disclosure
This code was written with AI assistance from Claude (Anthropic, https://claude.ai): Claude generated,
debugged and refactored the code; the author directed the work and tested it in game.
