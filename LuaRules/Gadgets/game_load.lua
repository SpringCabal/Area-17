-- Copied/adapted from Scened

if not gadgetHandler:IsSyncedCode() then
    return
end
--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

function gadget:GetInfo()
    return {
        name = "Game load",
        desc = "Spawns units for the game.",
        author = "gajop",
        date = "August 2015",
        license = "GNU GPL, v2 or later",
        layer = 100,
        enabled = true
    }
end

local file = "LuaRules/Configs/spawn.lua"

function gadget:Initialize()
    if Game.gameName == "Scenario Editor Alien resources" then
        gadgetHandler:RemoveGadget()
        return
    end
    local data = loadstring(VFS.LoadFile(file))()
    self:load(data.units)
end

function gadget:setUnitProperties(unitId, unit)
    local x = math.sin(math.rad(unit.angle))
    local z = math.cos(math.rad(unit.angle))
    Spring.SetUnitDirection(unitId, x, 0, z)
    if unit.maxhealth ~= nil then
        Spring.SetUnitMaxHealth(unitId, unit.maxhealth)
    end
    if unit.health ~= nil then
        Spring.SetUnitHealth(unitId, unit.health)
    end
    if unit.tooltip ~= nil then
        Spring.SetUnitTooltip(unitId, unit.tooltip)
    end
    if unit.stockpile ~= nil then
        Spring.SetUnitStockpile(unitId, unit.stockpile)
    end
    if unit.experience ~= nil then
        Spring.SetUnitExperience(unitId, unit.experience)
    end
    if unit.fuel ~= nil and Spring.SetUnitFuel then
        Spring.SetUnitFuel(unitId, unit.fuel)
    end
    if unit.neutral ~= nil then
        Spring.SetUnitNeutral(unitId, unit.neutral)
    end
    if unit.alwaysVisible ~= nil then
        Spring.SetUnitAlwaysVisible(unitId, unit.alwaysVisible)
    end
    if unit.blocking ~= nil then
        Spring.SetUnitBlocking(unitId, unit.blocking)
    end
    if unit.losState ~= nil then
        Spring.SetUnitLosState(unitId, 0, unit.losState)
    end
    if unit.unitDefName == "house" then
        Spring.SetUnitAlwaysVisible(unitId, true)
        Spring.SetUnitNeutral(unitId, true)
    end
    if unit.rules ~= nil then
        for _, foo in pairs(unit.rules) do
            if type(foo) == "table" then
                for rule, value in pairs(foo) do
                    Spring.SetUnitRulesParam(unitId, rule, value)
                end
            end
        end
    end
    if unit.states ~= nil then
        local s = unit.states
        if s.cloak ~= nil then
            Spring.GiveOrderToUnit(unitId, CMD.INSERT, {0, CMD.CLOAK, 0, boolToNumber(s.cloak)}, {"alt"})
        end
        if s.firestate ~= nil then
            Spring.GiveOrderToUnit(unitId, CMD.INSERT, {0, CMD.FIRE_STATE, 0, s.firestate}, {"alt"})
        end
        if s.movestate ~= nil then
            Spring.GiveOrderToUnit(unitId, CMD.INSERT, {0, CMD.MOVE_STATE, 0, s.movestate}, {"alt"})
        end
        -- setting the active state doesn't work currently
        --[[
        if s.active ~= nil then
            Spring.GiveOrderToUnit(unitId, CMD.INSERT,
                { 0, CMD.IDLEMODE, 0, boolToNumber(s.active)},
                {"alt"}
            );
        end
        --]]
        if s["repeat"] ~= nil then
            Spring.GiveOrderToUnit(unitId, CMD.INSERT, {0, CMD.REPEAT, 0, boolToNumber(s["repeat"])}, {"alt"})
        end
    end
end

function isUnitCommand(command)
    if command.params ~= nil and #command.params ~= 1 then
        return false
    end
    local unitCommands = {
        "DEATHWAIT",
        "ATTACK",
        "GUARD",
        "REPAIR",
        "LOAD_UNITS",
        "UNLOAD_UNITS",
        "RECLAIM",
        "RESSURECT",
        "CAPTURE",
        "LOOPBACKATTACK"
    }
    for _, unitCommand in pairs(unitCommands) do
        if command.name == unitCommand then
            return true
        end
    end
    return false
end

function gadget:setUnitCommands(unitId, commands)
    for _, command in pairs(commands) do
        local params
        -- unit commands need to get the real unit ID
        if isUnitCommand(command) then
            params = {self:getSpringUnitId(command.params[1])}
        else
            params = command.params
        end
        if command.name ~= "BUILD_COMMAND" then
            Spring.GiveOrderToUnit(unitId, CMD[command.name], params, {"shift"})
        else
            Spring.GiveOrderToUnit(unitId, -UnitDefNames[command.buildUnitDef].id, params, {"shift"})
        end
    end
end

function gadget:loadUnit(unit)
    -- FIXME: figure out why this sometimes fails on load with a specific unit.id
    local unitId = Spring.CreateUnit(unit.unitDefName, unit.x, unit.y, unit.z, 0, unit.teamId, false, true)
    if unitId == nil then
        Spring.Log("scened", LOG.ERROR, "Failed to create the following unit: " .. table.show(unit))
        return
    end
    -- FIXME: this check is not usable until unit creation by ID is fixed
    if false and unit.id ~= nil and unit.id ~= unitId then
        Spring.Log(
            "scened",
            LOG.ERROR,
            "Created unit has different id: " .. tostring(unit.id) .. ", " .. tostring(unitId)
        )
    end
    self:setUnitProperties(unitId, unit)
    if unit.commands ~= nil then
        self:setUnitCommands(unitId, unit.commands)
    end
    return unitId
end

function gadget:load(units)
    self.unitIdCounter = 0
    -- load the units without the commands
    local unitCommands = {}
    for _, unit in pairs(units) do
        local commands = unit.commands
        unit.commands = nil
        local unitId = self:loadUnit(unit)
        if unitId then
            unitCommands[unitId] = commands
        end
    end
    -- load the commands
    for unitId, commands in pairs(unitCommands) do
        self:setUnitCommands(unitId, commands)
    end
end

function gadget:clear()
    for _, unitId in pairs(Spring.GetAllUnits()) do
        Spring.DestroyUnit(unitId, false, true)
        --self:removeUnit(unitId)
    end

    for unitId, _ in pairs(self.s2mUnitIdMapping) do
        self:removeUnit(unitId)
    end
    self.s2mUnitIdMapping = {}
    self.m2sUnitIdMapping = {}
    self.unitIdCounter = 0
end

function boolToNumber(bool)
    if bool then
        return 1
    else
        return 0
    end
end
