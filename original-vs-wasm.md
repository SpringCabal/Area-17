# Area 17 original-to-WASM comparison

The left side is a file of the Lua original, `Area-17.sdd`; the right side
is the active WASM implementation or the reason a file remains Lua/unused.

## Synced LuaRules gadgets

| Original file | WASM counterpart | Result |
| --- | --- | --- |
| `api_custom_unit_shaders.lua` | none | Optional shader registration; not needed by gameplay |
| `dbg_dev_commands.lua` | none | Development-only commands intentionally excluded |
| `game_end.lua` | `abduction.rs` | Original gadget is effectively a no-op; endcube capture publishes win state |
| `game_load.lua` | `build.rs`, `scenario.rs` | Scenario data is generated at compile time and created through typed calls |
| `game_spawn.lua` | `scenario.rs` | Initial scenario creation is owned by Rust |
| `lups_cloak_fx.lua` | engine cloak state from `upgrades.rs` | Core cloak behavior is ported; optional LUPS decoration is not |
| `lups_wrapper.lua` | none | Optional LUPS integration, not gameplay |
| `typemap_control.lua` | none | Debug/map typemap control, not gameplay |
| `ui_chonsole_load.lua` | none | Console compatibility helper, not gameplay |
| `unit_attributes.lua` | `definitions::set_move_speed_multiplier`, `set_gunship_attributes` | Aircraft and ground units receive Lua's live attribute speeds; definitions stay unchanged |
| `unit_buildings.lua` | `gadgets/unit_buildings.rs` | Explosive-building destruction behavior ported |
| `unit_centeroffset.lua` | `definitions.rs` | Area 17 collision-volume geometry configured from typed definitions |
| `unit_civilian_spawner.lua` | `civilians.rs` | Initial populations, activation, wandering, replenishment, and flee behavior ported |
| `unit_drones.lua` | `drones.rs` | Tech-gated spawning and fight/move orders ported |
| `unit_fall_damage.lua` | `fall_damage.rs` | Collision damage, bounce, outside-map damage, and immunity ported |
| `unit_script.lua` | `rules-synced/src/cus_scripts/` | Empty Lua compatibility loader; model callbacks are implemented by Rust Core-CUS |
| `unit_spawner.lua` | `scenario.rs`, `gadgets/unit_ufo_controller.rs` | UFO tracking and delayed respawn |
| `unit_ufo_abduction.lua` | `abduction.rs` | Beam filtering, floating, capture values, healing, effects, and endcube win ported |
| `unit_ufo_controller.lua` | `ufo_controller.rs` | Movement, aim, targets, firing, weapon changes, abilities, and teleport ported |
| `upgrades.lua` | `upgrades.rs`, `state.rs` | Dependencies, costs, levels, resources, shield, armor, cone, and weapon effects ported |
| `weapon_area_damage.lua` | `weapons.rs` | Incendiary and black-hole persistent areas, falloff, damage, effects, and expiry ported |
| `weapon_impulse.lua` | `weapons.rs`, `fall_damage.rs` | Area 17 impulse profiles, mass/movement scaling, drag, and unsticking ported |
| `weapon_multshot.lua` | `weapon_multishot.rs` | Pulse projectiles and point-defense reload changes ported with explicit slot mapping |

## LuaRules utilities

| Original file | WASM counterpart | Result |
| --- | --- | --- |
| `ClampPosition.lua` | `definitions::clamp_xz` | Typed semantic helper |
| `UnitRendering.lua` | none | Rendering helper not required by active rules |
| `explodeMessage.lua` | `ufo_controller.rs` parser | Ported for Area 17 messages |
| `tablefunctions.lua` | none | Lua table convenience functions no longer needed |
| `tech.lua` | `state.rs`, `upgrades.rs`, `widgets/api_sync.rs` | Tech data and transitions ported as typed state |
| `unitTypeChecker.lua` | `definitions.rs` helpers | Ported for the Area 17 unit set |
| `utilities.lua` | `definitions.rs`, `weapons.rs`, `gadgets/game_terrain.rs` | Used behaviors split into semantic helpers |
| `vector.lua` | local typed vector helpers | Ported where used; no generic Lua table-vector API retained |

## Lua unit scripts

| Original file | WASM counterpart | Result |
| --- | --- | --- |
| `Scripts/ufo.lua` | `rules-synced/src/cus_scripts/ufo.rs` | Core-CUS piece animation, aiming, weapon queries, firing, and death effects |
| `Scripts/human.lua` + `Scripts/animations/*` | `rules-synced/src/cus_scripts/human.rs` + `keyframes.rs` | Core-CUS walking, floating, landing, parachute, and death behavior |
| `Scripts/drone.lua`, `Scripts/tank.lua`, `Scripts/static_aa.lua` | `rules-synced/src/cus_scripts/props.rs` | Core-CUS animation, weapon callins, sounds, and death behavior |
| `Scripts/building.lua`, `helipad.lua`, `ufo_wreck.lua`, `endcube.lua` | `rules-synced/src/cus_scripts/props.rs` | Core-CUS visibility, activation, and destruction behavior |

## LuaUI widgets and configuration

| Original file | WASM counterpart | Result |
| --- | --- | --- |
| `Configs/chilitip_conf.lua` | RmlUi stylesheet/document | Chili-only configuration replaced by RmlUi |
| `Configs/lupsFXs.lua`, `lupsUnitFXs.lua` | none | Optional LUPS configuration |
| `SystemWidgets/BlockUserWidgets.lua`, `Widgets/BlockUserWidgets.lua` | none | Unnecessary in isolated port |
| `Widgets/api_analytics.lua` | none | Optional analytics excluded |
| `Widgets/api_chili.lua` | RmlUi engine helpers | Chili replaced by RmlUi |
| `Widgets/cmd_customformations2.lua` | none | Generic formation widget, unrelated to Area 17 |
| `Widgets/cmd_default_set_target.lua` | `widgets/gui_ufo_control.rs` | Cursor target generated directly for UFO |
| `Widgets/cmd_handle_cmd_colors.lua` | none | Generic command-color widget |
| `Widgets/dbg_chili_inspector.lua`, `dbg_chili_profiler.lua` | none | Development-only widgets |
| `Widgets/gfx_deferred_rendering.lua` | engine rendering configuration | No game-specific Rust behavior |
| `Widgets/gfx_lups_manager.lua`, `gfx_lups_smokesignal.lua` | none | Optional LUPS effects |
| `Widgets/gfx_projectile_lights.lua` | engine weapon/effect rendering | No custom gameplay state |
| `Widgets/gfx_ufo_beam.lua` | `widgets/gfx_volumes.rs` | Beam position/state and volume draw ported |
| `Utilities/glVolumes.lua` | `widgets/gfx_volumes.rs` | Direct typed stencil-volume geometry port |
| `Widgets/gui_abilities.lua` | `rml/loadout.rs`, `widgets/gui_ufo_control.rs` | Unlocked ability rows, cooldowns, and keys ported |
| `Widgets/gui_chili_docking.lua`, `gui_chili_selections_and_cursortip.lua`, `gui_chiliguidemo.lua` | none | Generic Chili widgets replaced or irrelevant |
| `Widgets/gui_disable_camera_control.lua` | `widgets/gui_camera.rs` | Fixed camera geometry ported |
| `Widgets/gui_land.lua` | none | Generic landing helper not used |
| `Widgets/gui_ufo_control.lua` | `widgets/gui_ufo_control.rs`, `widgets/gui_camera.rs` | WASD/mouse control and camera follow ported |
| `Widgets/gui_upgrade.lua` | `rml/tech.rs`, `commands.rs` | Upgrade cards, dependencies, click actions, and status ported |
| `Widgets/keybinds.lua` | `widgets/gui_ufo_control.rs` | Key lookup and movement state ported |
| `Widgets/layout.lua` | `rml/`, `area17.rml`, `area17.rcss` | HUD layout ported to RmlUi |
| `Widgets/lups_wrapper.lua` | none | Optional LUPS wrapper |
| `Widgets/map_edge_extension.lua` | none | Optional map presentation |
| `Widgets/snd_music.lua` | `springcabal::music::MusicLoop` (`widgets/setup.rs`) | Looping music |
| `Widgets/start_force.lua` | `widgets/gui_full_view.rs` | Full map view |
| `Widgets/ui_chonsole_load.lua` | none | Console compatibility helper |
| `Widgets/ui_endgame.lua` | `rml/hud.rs` | End-game panel and terminal state ported |
| `Widgets/ui_floating_text.lua` | `rml/hud.rs` event queue | Scrolling/fading event text ported |
| `Widgets/ui_health.lua` | `rml/hud.rs` | Health/resource HUD ported |
| `Widgets/ui_hide_default_layout.lua` | `HideDefaultInterface` in `widgets/setup.rs` | Default engine layout hidden |
| `Widgets/ui_quitbutton.lua` | none | Optional quit control omitted |
| `Widgets/ui_shield.lua` | `rml/hud.rs` | Shield HUD ported |
| `Widgets/ui_stats.lua` | `rml/hud.rs` | Stats/resources HUD ported |
| `LuaUI/main.lua`, `LuaUI/config.lua` | `ui/src/lib.rs`, `rml/lifecycle.rs` | RmlUi environment and document lifecycle ported |

## Tests

The repository's `area17 static`, `area17 headless` and `area17 visual`
suites; see [`summary.md`](summary.md).


## Commit boundary

The gameplay/UI commit keeps `Scripts/` byte-for-byte identical to its Lua
parent. It retains the Lua loader and utilities, and registers numeric calls in
`LuaRules/Gadgets/wasm_script_bridge.lua`. The unit-script commit replaces this
adapter and the Lua scripts with Core-CUS, keeping gameplay callers behind
`script_bridge.rs`. Both commits build and run independently.
