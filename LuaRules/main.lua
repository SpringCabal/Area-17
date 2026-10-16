-- Area 17 WASM keeps the engine's content-side unit-script gadget active,
-- while authoritative gameplay lives in the Core-WASM rules environment.
return VFS.Include("luagadgets/gadgets.lua", nil, VFS.BASE)
