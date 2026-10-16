-- Numeric entry points for the gameplay/UI port. Scripts/ stays unchanged.
function gadget:GetInfo()
    return {name = 'WASM Lua script bridge', layer = -1, enabled = true}
end
if not gadgetHandler:IsSyncedCode() then return false end
local registered = {}
local weaponNames = {'incendiaryBeamLaser', 'gravityBeam', 'pulseLaser', 'blackHoleGun'}
local weaponSlots = {1,5,19,17}
local function register(unitID)
    if registered[unitID] then return end
    local env = Spring.UnitScript.GetScriptEnv(unitID)
    if not env then return end
    registered[unitID] = true
    local walking = false
    local function expose(name, fn)
        Spring.UnitScript.UpdateCallIn(unitID, name, function(...)
            return Spring.UnitScript.CallAsUnit(unitID, fn, ...)
        end)
    end
    expose('WasmAbductionFloat', function()
        walking = false
        if env.Abduction_float then env.Abduction_float() end
    end)
    expose('WasmAlienLand', function()
        if env.Alien_land then env.Alien_land() end
    end)
    expose('WasmHumanWalks', function() return walking and 1 or 0 end)
    for _, name in ipairs({'StartMoving', 'StopMoving'}) do
        local original = env.script[name]
        if original then
            local isWalking = name == 'StartMoving'
            expose(name, function(...)
                walking = isWalking
                return original(...)
            end)
        end
    end
    if env.script.AimWeapons then
        local lastWeapon, lastBeam
        expose('WasmUfoControl', function(x,y,z,weapon,beam)
            if weapon ~= lastWeapon then
                env.script.SetCurrentWeapon(weaponNames[weapon])
                Spring.SetUnitRulesParam(unitID,'active_weapon_slot',weaponSlots[weapon] or 0)
                lastWeapon = weapon
            end
            env.script.AimWeapons(x,y,z)
            if beam ~= lastBeam then
                if beam == -1 then env.script.StartIndependence()
                else
                    if lastBeam == -1 then env.script.StopIndependence() end
                    env.script.SetBeamEnabled(beam == 1)
                end
                lastBeam = beam
            end
        end)
    end
end
function gadget:UnitCreated(unitID) register(unitID) end
function gadget:UnitDestroyed(unitID) registered[unitID] = nil end
function gadget:Initialize()
    for _, unitID in ipairs(Spring.GetAllUnits()) do register(unitID) end
end
-- Catch a unit whose Lua Create callback was still being installed at UnitCreated.
function gadget:GameFrame()
    for _, unitID in ipairs(Spring.GetAllUnits()) do register(unitID) end
end
