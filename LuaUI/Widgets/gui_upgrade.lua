function widget:GetInfo()
	return {
		name    = 'Upgrade widget for spaceships',
		desc    = '',
		author  = 'gajop',
		date    = 'August, 2015',
		license = 'GPL',
        layer = 0,
		enabled = true,
	}
end

local ufoID
local ufoDefID = UnitDefNames["ufo"].id

require('keysym.lua')
local UPGRADE_KEY = KEYSYMS.T

local vsx, vsy
local landTextColor = {1, 1, 1, 1}
local landText = "LAND"
local landTextSize = 30
local updateUI
local upgratdeAvailable = false

local Chili, screen0

function widget:Initialize()
    Chili = WG.Chili
    screen0 = Chili.Screen0
    vsx, vsy = Spring.GetViewGeometry()
    for _, unitID in pairs(Spring.GetAllUnits()) do
        self:UnitCreated(unitID, Spring.GetUnitDefID(unitID), Spring.GetUnitTeam(unitID))
    end
	VFS.Include("LuaRules/Utilities/tech.lua")
	WG.Tech = Tech
end

function widget:UnitCreated(unitID, unitDefID, unitTeam)
	if unitDefID == ufoDefID then
		ufoID = unitID
	end
end

local window
function widget:KeyPress(key, mods, isRepeat)
    if ufoID and key == UPGRADE_KEY then
        ToggleUpgradeUI()
    end
end

function widget:Update()
	UpdateUpgradeUI()
end

function ToggleUpgradeUI()
	if not window then
		Spring.PlaySoundFile("sounds/click.wav", 1, "ui")
		SpawnUpgradeUI()
	else
		if window then
			window:Dispose()
			window = nil
		end
	end
end

function widget:GameFrame()
	upgradeAvailable = false
	for name, _ in pairs(Tech.GetTechTree()) do
		if Tech.CanUnlock(name) then
			upgradeAvailable = true
			return
		end
	end
end

function UpdateUpgradeUI()
	if updateUI == nil then
		local w = 350
		local imgUpgrade = Chili.Image:New {
			file = "UI/upgrade.png",
			x = 0, width = "100%",
			y = 0, height = "100%",
			keepAspect = false,
		}
		updateUI = Chili.Button:New {
			x = (vsx - w) / 2,
			y = 0,
			width = w,
			height = 100,
			parent = screen0,
			backgroundColor = { 1,1,1,0 },
			focusColor = { 0,0,0,0 },
			caption = "",
			children = { imgUpgrade },
			OnClick = { function()
				ToggleUpgradeUI()
			end},
		}
		local lblTitle = Chili.Label:New {
			x = 82,
			y = 20,
			fontsize = 25,
			caption = "[T] Technology",
			parent = updateUI,
		}
		local lblUnlockAvailable = Chili.Label:New {
			x = 85,
			--y = 50,
			y = 20,
			fontsize = 20,
			caption = "* unlock available *",
			parent = updateUI,
			font = {
				color = { 1, 1, 1, 1},
				outline = true,
				shadow = false,
			},
		}
		updateUI.lblUnlockAvailable = lblUnlockAvailable
		updateUI.lblTitle = lblTitle
		updateUI.imgUpgrade = imgUpgrade
	end
	local research = Spring.GetGameRulesParam("research") or 0
	if upgradeAvailable and not updateUI.lblUnlockAvailable.visible then
		updateUI.lblUnlockAvailable:Show()
	elseif not upgradeAvailable and updateUI.lblUnlockAvailable.visible then
		updateUI.lblUnlockAvailable:Hide()
		updateUI.imgUpgrade.color = { 0.3, 0.3, 0.3, 1 }
		updateUI.imgUpgrade.color[4] = 1
		updateUI.lblTitle:SetCaption("\255\170\170\170" .. "[T] Technology" .. "\b")
	end
	if upgradeAvailable then
		local v = 0.8 + math.sin(os.clock() * 10) / 3.14 * 0.6
		local v256 = string.char(math.floor(v * 255))
		updateUI.lblUnlockAvailable:SetCaption("\255" .. v256 .. v256 .. v256 .. "* unlock available *\b")
		updateUI.imgUpgrade.color = { 1, 1, 1, 1 }
		updateUI.lblTitle:SetCaption("\255\0\204\255" .. "[T] Technology" .. "\b")
		--updateUI.lblUnlockAvailable.font.color = { v, v, v, 1 }
	end
end

local techMapping = {}
function SpawnUpgradeUI()
    local btnClose = Chili.Button:New {
        width = 100,
        height = 40,
        right = 10,
        bottom = 10,
        caption = "Close",
        OnClick = { function() 
            window:Dispose()
            window = nil
        end },
    }
    local lblTitle = Chili.Label:New {
        x = 10,
        y = 10,
        fontsize = 30,
        caption = "TECH TREE",
		font = {
			color = { 0, 0.8, 1, 0.7},
		},
    }
	local imgShip = Chili.Image:New {
		file = "UI/wireframe.png",
		x = 200,
		y = 0,
		width = 200,
		height = 100,
	}

    local children = { btnClose, lblTitle, imgShip }
    x, y = 10, 50
    for name, tech in pairs(Tech.GetTechTree()) do
        local btnTech, imgTech, lblTech, imgTechUnlocked
        local enabled = false
        local file
        if tech.enabled then
            file = tech.iconPath
        else
            file = tech.iconDisabledPath
        end
        imgTech = Chili.Image:New {
            margin = {0, 0, 0, 0},
            x = 7,
            y = 7,
            width = 45,
            height = 45,
            file = file,
        }
		imgTechLocked = Chili.Image:New {
            margin = {0, 0, 0, 0},
            bottom = 5,
			x = 2,
            width = 12,
            height = 12,
            file = "UI/key.png",
			keepAspect = false,
        }
		local lvlCaption
		if tech.level == tech.maxLevel then
			lvlCaption = "\255\0\255\0" .. tech.level .. "/" .. tech.maxLevel .. "\b"
		elseif tech.level > 0 then
			lvlCaption = "\255\0\255\0" .. tech.level .. "\b/" .. tech.maxLevel
		else
			lvlCaption = tech.level .. "/" .. tech.maxLevel
		end
        lblTech = Chili.Label:New {
            right = 3,
            bottom = 3,
            caption = lvlCaption,
            align = "right",
        }
        btnTech = Chili.Button:New {
            caption = "",
            x = x + (tech.x or 60),
            y = y + (tech.y or 60),
            width = 60,
            height = 60,
            tooltip = Tech.GetTechTooltip(name),
            padding = {0, 0, 0, 0},
            itemPadding = {0, 0, 0, 0},
            backgroundColor = {0.5, 0.5, 0.5, 1},
			lockedColor = { 0, 0.8, 1, 1},
			completedColor = { 0.4, 1, 0.4, 1}, 
            children = {lblTech, imgTechLocked, imgTech},
            OnClick = { function(ctrl, x, y, button)
                if button == 1 then
                    UpgradeUnlockTech(name)
                end
            end},
        }
		btnTech.origFocusColor = btnTech.focusColor
		if not tech.enabled then
			btnTech.focusColor = btnTech.backgroundColor
		elseif tech.locked then
			btnTech.focusColor = btnTech.lockedColor
		else
			if tech.level == tech.maxLevel then
				btnTech.backgroundColor = btnTech.completedColor
				btnTech.focusColor = btnTech.completedColor
			end
			imgTechLocked:Hide()
		end
        techMapping[name] = { btnTech = btnTech, imgTech = imgTech, lblTech = lblTech, imgTechLocked = imgTechLocked }
        table.insert(children, btnTech)
    end
    window = Chili.Window:New {
        parent = screen0,
        x = 200,
        width = 400,
        bottom = 100,
        height = 500,
        draggable = false,
        resizable = false,
        children = children,
		OnDispose = { function() 
			Spring.PlaySoundFile("sounds/click.wav", 1, "ui")
		end},
    }
end

function UpgradeUnlockTech(name)
	local tech = Tech.GetTech(name)
	if tech.locked then
		UnlockTech(name)
	else
		UpgradeTech(name)
	end
end

function UnlockTech(name)
	local tech = Tech.GetTech(name)
	if not tech.enabled then
		return false
	end
	local unlocked, enabledTechs = Tech.UnlockTech(name)
	if not unlocked then
		return false
	end
	Spring.SendLuaRulesMsg('unlock|' .. name)
	
	Spring.PlaySoundFile("sounds/select.wav", 1, "ui")
	techMapping[name].imgTechLocked:Hide()
	local btnTech = techMapping[name].btnTech
	btnTech.focusColor = btnTech.origFocusColor
	local tooltip, value = Tech.GetTechTooltip(name)
	techMapping[name].btnTech.tooltip = tooltip
	
	-- enable techs that depend on it
	for _, enabledTechName in pairs(enabledTechs) do
		local comps = techMapping[enabledTechName]
    	comps.imgTech.file = Tech.GetTech(enabledTechName).iconPath
		comps.btnTech.focusColor = comps.btnTech.lockedColor
    end
end

function UpgradeTech(name)
	local upgraded = Tech.UpgradeTech(name)
	if not upgraded then
		return false
	end
	local tech = Tech.GetTech(name)
	Spring.PlaySoundFile("sounds/click.wav", 1, "ui")
	if tech.level < tech.maxLevel then
		techMapping[name].lblTech:SetCaption("\255\0\255\0" .. tech.level .. "\b/" .. tech.maxLevel)
	else
		local btnTech = techMapping[name].btnTech
		btnTech.backgroundColor = btnTech.completedColor
		btnTech.focusColor = btnTech.completedColor
		techMapping[name].lblTech:SetCaption("\255\0\255\0" .. tech.level .. "/" .. tech.maxLevel .. "\b")
	end
	local tooltip, value = Tech.GetTechTooltip(name)
	techMapping[name].btnTech.tooltip = tooltip

	Spring.SendLuaRulesMsg('upgrade|' .. name)
end
