if not gadgetHandler:IsSyncedCode() then
	return
end
--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

function gadget:GetInfo()
	return {
		name 	= "Upgrades",
		desc	= "Spaceship upgrade.",
		author	= "gajop",
		date	= "August 2015",
		license	= "GNU GPL, v2 or later",
		layer	= 0,
		enabled = true
	}
end

local ufoID
local ufoDefID = UnitDefNames["ufo"].id

local baseShieldRegen = 1.5
local maxShieldPower = 1000

local function explode(div,str)
	if (div=='') then return 
		false 
	end
	local pos,arr = 0,{}
	-- for each divider found
	for st,sp in function() return string.find(str,div,pos,true) end do
		table.insert(arr,string.sub(str,pos,st-1)) -- Attach chars left of current divider
		pos = sp + 1 -- Jump past current divider
	end
	table.insert(arr,string.sub(str,pos)) -- Attach chars right of last divider
	return arr
end

function gadget:Initialize()
	Spring.SetGameRulesParam("biomass", 0)
    Spring.SetGameRulesParam("research", 0)
    Spring.SetGameRulesParam("metal", 0)
	
	-- TEST resources, uncomment on release
    Spring.SetGameRulesParam("biomass", 500000)
    Spring.SetGameRulesParam("research", 50000)
    Spring.SetGameRulesParam("metal", 0)
end

function gadget:GameFrame()
	if not ufoID then
		return
	end
	local shieldTech = GG.Tech.GetTech("shield")
	if not shieldTech.locked then
		local enabled, power = Spring.GetUnitShieldState(ufoID)
		local _, multiplier = GG.Tech.GetTechTooltip("shield")
		power = math.min(power + baseShieldRegen * (1 + multiplier/100), maxShieldPower)
		Spring.SetUnitShieldState(ufoID, -1, true, power)
		Spring.SetGameRulesParam("shieldPower", power)
	else
		Spring.SetUnitShieldState(ufoID, -1, false)
	end
end

function gadget:UnitCreated(unitID, unitDefID, unitTeam)
	if unitDefID == ufoDefID then
		ufoID = unitID
	end
end

function gadget:UnitDestroyed(unitID, unitDefID, unitTeam)
	if ufoID == unitID then
		ufoID = nil
	end
end

function HandleLuaMessage(msg)
	local msg_table = explode('|', msg)
	if msg_table[1] == 'unlock' then
		local name = msg_table[2]
		if not GG.Tech.UnlockTech(name) then
			Spring.Log("tech", LOG.ERROR, "Something went wrong unlocking tech: " .. name)
		end
	elseif msg_table[1] == 'upgrade' then
		local name = msg_table[2]
		if name == "armor" then
			local _, value = GG.Tech.GetTechTooltip(name)
			local newMaxHealth = UnitDefs[ufoDefID].health * (100 + value) / 100
			local hp, maxHP = Spring.GetUnitHealth(ufoID)
			local ratio = hp / maxHP
			Spring.SetUnitMaxHealth(ufoID, newMaxHealth)
			Spring.SetUnitHealth(ufoID, ratio * newMaxHealth) --scale current HP
		end
		if not GG.Tech.UpgradeTech(name) then
			Spring.Log("tech", LOG.ERROR, "Something went wrong upgrading tech: " .. name)
		end
	end
end

function gadget:RecvLuaMsg(msg)
	HandleLuaMessage(msg)
end

