# Area 17 WASM

The Rust/WASM version of Area 17. Gameplay, the unit scripts and the HUD are Rust modules; unit, weapon and effect definitions stay Lua data. [`summary.md`](summary.md) describes the modules, and [`original-vs-wasm.md`](original-vs-wasm.md) maps each file of the Lua original game to the code that replaces it. Build instructions are in the [development repository](https://github.com/gajop/springcabal-dev).

## Differences from the Lua version

- Object collisions without an attacker use unpaired damage at 2.5 times the reported damage and skip Lua's ally, same-type and feature immunities.
- The alien's parachute is hidden until its landing animation, and shows again when the alien lands after the beam dropped it; Lua showed it from the drop and hid it for good after the first landing. Cosmetic, left as is.
