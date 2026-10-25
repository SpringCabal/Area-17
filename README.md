# Area 17 WASM

The Rust/WASM version of Area 17. Gameplay, the unit scripts and the HUD are Rust modules; unit, weapon and effect definitions stay Lua data. [`summary.md`](summary.md) describes the modules, and [`original-vs-wasm.md`](original-vs-wasm.md) maps each file of the Lua original game to the code that replaces it. Build instructions are in the [development repository](https://github.com/gajop/springcabal-dev).

## Differences from the Lua version

- Object collisions without an attacker use unpaired damage at 2.5 times the reported damage and skip Lua's ally, same-type and feature immunities.
- The alien's parachute is hidden until its landing animation, and shows again when the alien lands after the beam dropped it; Lua showed it from the drop and hid it for good after the first landing. Cosmetic, left as is.

## Scoped SDK port

This worktree uses `spring-sdk` and the shared addon/CUS host. Helpers receive
`&Spring`; queued commands and Rml events get a fresh reference when they run.
Unit-script tasks use `UnitCtx::with_spring` for individual engine operations.
This remains a Wasm game; native game binaries are not built here.

Music uses the shared `springcabal::music::MusicLoop`. From the development
repository root, run
`springcabal-rs/verification/music-loop.sh` to check Lua and
Rust playback across several loops. It requires FFmpeg, clips a private copy
of the soundtrack and captures the engine's audio without using the speakers.

Verification on the SDK test engine:

- Linux and Wine: all 22 Rust integration checks and six Lua gameplay checks
  pass, with no engine errors or warnings.
- Technology UI fixture passes, including values, hover, purchase gates and
  its screenshot. The full visual scenario verifies the Rml callbacks, all
  four weapons/abilities and the rendered crater.
- Strict Clippy passes with default and all features; all six shared Rust
  tests pass. The normal installed Wasm modules match the source, and static
  verification passes.
- All three visual captures complete. Manual RMSE measurements are 0.41%,
  5.50% and 2.80%, within the existing 8% limit. The suite still rejects Mesa
  Ubuntu package revision `.4` versus the recorded `.2`; no expected images,
  limits or renderer checks changed.

Diagnostic builds explicitly enable the game's INFO log sections, so the
engine's default NOTICE filter does not hide the verification markers. Normal
builds keep their existing log configuration.

Use the parent development repository's `just verify area17` commands and
select the engine with `SPRING_ENGINE_DIR`. Packaging continues to use
independently built engine releases.
