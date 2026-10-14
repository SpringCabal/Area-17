-- DISABLED
function widget:GetInfo()
	return {
		name    = 'Spaceship land and upgrade widget',
		desc    = '',
		author  = 'gajop',
		date    = 'August, 2015',
		license = 'GPL',
        layer = 0,
		enabled = false,
	}
end

local ufoID
local ufoDefID = UnitDefNames["ufo"].id
local helipadDefID = UnitDefNames["helipad"].id
local helipadID
local canLand

local LANDING_RADIUS = 200

require('keysym.lua')
local SPACE = KEYSYMS.SPACE

local vsx, vsy
local landTextColor = {1, 1, 1, 1}
local landText = "LAND"
local landTextSize = 30

local Chili, screen0

function widget:Initialize()
    Chili = WG.Chili
    screen0 = Chili.Screen0
    vsx, vsy = Spring.GetViewGeometry()
    for _, unitID in pairs(Spring.GetAllUnits()) do
        self:UnitCreated(unitID, Spring.GetUnitDefID(unitID), Spring.GetUnitTeam(unitID))
    end
end

function widget:DrawScreen()
    if not canLand then
        return
    end
    gl.PushMatrix()
        gl.Color(landTextColor[1], landTextColor[2], landTextColor[3], landTextColor[4])
        local fw = gl.GetTextWidth(landText) * landTextSize
        gl.Text(landText, (vsx - fw) / 2,  vsy / 2 - 400, landTextSize)
    gl.PopMatrix()
end

function widget:GameFrame()
    canLand = false
    if ufoID and Spring.ValidUnitID(ufoID) and not Spring.GetUnitStates(ufoID).autoland then
        local x, _, z = Spring.GetUnitPosition(ufoID)
        local units = Spring.GetUnitsInCylinder(x, z, LANDING_RADIUS)
        for _, unitID in pairs(units) do
            if Spring.GetUnitDefID(unitID) == helipadDefID then
                canLand = true
                helipadID = unitID
            end
        end
    end
end

function widget:UnitCreated(unitID, unitDefID, unitTeam)
	if unitDefID == ufoDefID then
		ufoID = unitID
	end
end

local window
function widget:KeyPress(key, mods, isRepeat)
    if ufoID and key == SPACE then
        if canLand then
            --Spring.GiveOrderToUnit(ufoID, CMD.SETBASE, {}, helipadID)
            Spring.GiveOrderToUnit(ufoID, CMD.IDLEMODE, {1}, {})
            SpawnUpgradeUI()
        else
            Spring.GiveOrderToUnit(ufoID, CMD.IDLEMODE, {0}, {})
            if window then
                window:Dispose()
                window = nil
            end
        end
    end
end

function SpawnUpgradeUI()
    local _, maxHealth = Spring.GetUnitHealth(ufoID)
    local maxHealthOrig = maxHealth

    local btnClose = Chili.Button:New {
        width = 100,
        height = 40,
        right = 10,
        bottom = 10,
        caption = "Take off",
        OnClick = { function() 
            window:Dispose()
            window = nil
            Spring.GiveOrderToUnit(ufoID, CMD.IDLEMODE, {0}, {})
        end },
    }
    local btnApply = Chili.Button:New {
        width = 100,
        height = 40,
        x = 10,
        bottom = 10,
        caption = "Upgrade",
        OnClick = { function() 
            Spring.SendLuaRulesMsg('upgrade|maxhealth|' .. maxHealth)
            maxHealthOrig = maxHealth
        end },
    }
    local lblTitle = Chili.Label:New {
        x = 10,
        y = 10,
        fontsize = 30,
        caption = "Upgrade ship",
    }
    local lblHealthTitle = Chili.Label:New {
        x = 10,
        y = 70,
        caption = "Health",
    }
    local lblHealthValue = Chili.Label:New {
        x = 140,
        y = 70,
        caption = tostring(maxHealth),
    }
    local btnIncreaseHealth = Chili.Button:New {
        x = 180,
        y = 60,
        width = 50,
        caption = "+",
        OnClick = { function()
            maxHealth =  maxHealth + 100
            lblHealthValue:SetCaption(tostring(maxHealth))
        end},
    }
    local btnDecreaseHealth = Chili.Button:New {
        x = 180,
        y = 80,
        width = 50,
        caption = "-",
        OnClick = { function()
            if maxHealth <= maxHealthOrig then
                return
            end
            maxHealth = maxHealth - 100
            lblHealthValue:SetCaption(tostring(maxHealth))
        end},
    }
    window = Chili.Window:New {
        parent = screen0,
        x = 100,
        width = 400,
        y = "50%",
        height = "30%",
        draggable = false,
        resizable = false,
        children = {
            btnClose,
            btnApply,
            lblTitle,
            lblHealthTitle,
            lblHealthValue,
            btnIncreaseHealth,
            btnDecreaseHealth
        },
    }
end