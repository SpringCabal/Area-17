# Area 17 WASM: how it is built

Two WASM modules, registered in `LuaRules/wasm/manifest.txt` and
`LuaUI/wasm/manifest.txt`:

- `LuaRules/wasm/area17-rules.wasm` (`wasm-src/rules-synced`): synced
  gameplay and the Core-CUS unit scripts;
- `LuaUI/wasm/area17-ui.wasm` (`wasm-src/ui`): the RmlUi HUD, input, camera,
  sound and the abduction-beam cone.

Both use `wasm-src/shared`, the data they must agree on: the tech tree and
its costs, the weapons and abilities, and the message names.

Unit, weapon, effect and sound definitions stay Lua data. The scenario's
initial units are generated from the Lua scenario file at build time
(`wasm-src/build.rs`).

## Rules module

Each gadget in `rules-synced/src/gadgets/` owns its state and implements its
own callins; only data several gadgets share lives in `state::Global`
(the UFO, economy, tech tree, queued input, session, fall immunity).

| Gadget | Does |
| --- | --- |
| `game_load` | spawns the scenario, respawns the UFO 105 frames after it dies |
| `unit_ufo_controller` | player input from the UI, UFO movement, aim and fire; the UFO's death: resource penalty and the alien that drops over the wreck |
| `unit_civilian_spawner` | buildings that keep spawning civilians and tanks; wandering and fleeing |
| `unit_drones` | carrier drones following the UFO |
| `unit_ufo_abduction` | the abduction beam: lifting, capture, rewards, the endcube win |
| `weapon_handlers` | each weapon def's behaviour from its custom params: the gravity beams' pull, napalm and black-hole areas |
| `unit_fall_damage` | ground impacts and attacker-less collision damage |
| `upgrades` | tech effects, shield regeneration, ability timers |
| `game_terrain` | building footprints and burning ground as terrain types |
| `unit_setup` | visibility, and mid/aim points, radius and height from the defs' custom params |
| `unit_scripts` | attaches each new unit's Core-CUS script; wakes the scripts in `GameFrame`, as Lua does |
| `unit_buildings` | explosion sound of destroyed buildings |
| `api_status` | hands the UFO script its aim, weapon and beam every frame; sends the HUD status to the UI when it changes |

Defs are resolved once (`defs.rs`), with the custom params the Lua gadgets
read (capture rewards, `abduct_mult`, geometry, weapon behaviour). Synced randomness comes from
`spring::random::synced`, the engine RNG Lua's `math.random` uses.

### Unit scripts

`cus_scripts/` ports `Scripts/*.lua`: the UFO (hull spin, gun aiming, weapon
pieces, God Gun), humans and the alien (walk and float keyframes from
`Scripts/animations/`, parachute landing), tanks, anti-air, drones,
buildings, the helipad and the wreck. The rules and the scripts share the
module, so rules code attaches scripts and calls their functions
(`abduction_float`, `alien_land`) as typed calls through
`with_cus_module_or_defer`; it never goes through the engine. Each unit's
script holds only its own kind's state.

## Rules and UI

- UI to rules: `SendLuaRulesMsg` strings, sent when the input changes:
  `movement|x|z`, `aimWeapon|x|y|z`, `fireWeapon[|x|y|z]` and `ceaseFire`,
  `changeWeapon|name`, `ability|name`, `unlock|name`, `upgrade|name`.
- Rules to UI: an `AREA17_STATUS|key=value|...` line with the HUD values,
  checked every 15 frames and sent when it changed, plus `AREA17_DRONE`
  events. The rules play the capture sounds themselves, as Lua did. The UI reads the frame and the UFO's position
  from the engine, and the beam state from the UFO script's `beam_enabled`
  unit rules param, as the Lua widgets did.

## UI module

Widgets in `ui/src/widgets/`: `api_sync` (status and events from the rules),
`gui_ufo_control` (keyboard and mouse), `gui_camera` (follows the UFO),
`gui_rml` (the HUD document), `gfx_volumes` (beam cone),
`gui_full_view` (full map view, like the original's `start_force`),
spring-addons' `HideDefaultInterface` for the engine UI the original hid, and
the shared `springcabal` crate's `MusicLoop` (the looping music).
Messages and their sending, movement keys, Ctrl+Q, the gallery mode, the def
cache and the test tallies also use that crate.

## Known gap

`unit_attributes.lua` scales a unit's MoveType speed and acceleration from
`selfMoveSpeedChange`/`selfMaxAccelerationChange`. The port does not do that
for the UFO: haste lengthens its move goal but leaves its engine speed limit
alone. Civilians' speed uses the ground MoveType setter.

## Tests

In the repository's `verification/` suites: `area17 static`,
`area17 headless` (the `headless-integration` build: weapons and their
per-def behaviour, upgrades and costs, shield, UFO death and respawn,
capture, fall and collision damage, civilians, haste, God Gun crater; plus a
Lua observer, `verification/fixtures/area17/scenario.lua`, staged into the
test copy only) and `area17 visual` (the `e2e` build clicks the HUD and sends
the UI's input, and screenshots are compared with goldens). The `shared`
crate's host tests run with `just test` in `wasm-src/`.


## Revised Lua parent

The port consists of two commits above the revised Lua game. Gameplay/UI comes
first with unchanged Lua scripts and a numeric bridge; the second installs
Core-CUS. Lua's slope-impact rebound and point-defense reload fixes are applied
in rules code. Movement speed comes from unchanged unit definitions, avoiding
an extra reduction when a tank resumes wandering. The Core-CUS tank alternates
barrels and tank/AA aim threads wait for their piece turns before reporting
readiness. Existing tests, assets, diagnostics and port choices are retained.
