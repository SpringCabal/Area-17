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
local ufoDefID   = UnitDefNames["ufo"].id
local alienDefID = UnitDefNames["alien"].id

local baseShieldRegen = 1.5
local maxShieldPower = 1000
local delayedUpdate

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
	
	-- TEST resources, comment these lines out on release
--      Spring.SetGameRulesParam("biomass", 500000)
--      Spring.SetGameRulesParam("research", 50000)
--      Spring.SetGameRulesParam("metal", 0)
end

function gadget:GameFrame()
	if not ufoID then
		return
	end
	if delayedUpdate then
		for name, tech in pairs(GG.Tech.GetTechTree(name)) do
			local tech = GG.Tech.GetTech(name)
			if tech.level > 0 then
				UpdateTech(name)
			end
		end
		delayedUpdate = false
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

function UpdateTech(name)
	local tech = GG.Tech.GetTech(name)
	
	local _, value = GG.Tech.GetTechTooltip(name)
	local multiplier = (100 + value) / 100
	if name == "armor" then
		if not ufoID then
			return
		end
		local newMaxHealth = UnitDefs[ufoDefID].health * multiplier
		local hp, maxHP = Spring.GetUnitHealth(ufoID)
		local ratio = hp / maxHP
		Spring.SetUnitMaxHealth(ufoID, newMaxHealth)
		Spring.SetUnitHealth(ufoID, ratio * newMaxHealth) --scale current HP
	elseif name == "antiMissilePointDefense" then
		Spring.SetUnitRulesParam(ufoID, "PDisUnlocked", 1)
		GG.SetPDReload(multiplier)
	elseif name == "incendiaryBeamLaser" then
		GG.SetFireMults(multiplier, multiplier)
		GG.SetFireBlockingMult(multiplier)
	elseif name == "blackHoleGun" then
		GG.SetBlackHoleMults(multiplier, multiplier)
	elseif name == "gravityBeam" then
		GG.SetGravityBeamMult(multiplier)
	elseif name == "pulseLaser" then
		GG.SetPulseLaserShots(tech.level)
	elseif name == "carrierDrones" then
		GG.UpdateDroneParameters(tech.level + 1, 5 - tech.level)
	elseif name == "coneUpgrade" then
		GG.UpdateAbductionParameters(70 * multiplier, 7 * multiplier, multiplier)
	end
end

function gadget:UnitCreated(unitID, unitDefID, unitTeam)
	if unitDefID == ufoDefID then
		ufoID = unitID
		
		delayedUpdate = true
	end
end

function gadget:UnitDestroyed(unitID, unitDefID, unitTeam)
	if ufoID == unitID then
		ufoID = nil
		local alienUdef = UnitDefs[alienDefID]
		local biomass = Spring.GetGameRulesParam("biomass") or 0
		local research = Spring.GetGameRulesParam("research") or 0
		local metal = Spring.GetGameRulesParam("metal") or 0
		Spring.SetGameRulesParam("biomass", math.max(0, biomass - (alienUdef.customParams.biomass or 0)))
		Spring.SetGameRulesParam("research", math.max(0, research - (alienUdef.customParams.research or 0)))
		Spring.SetGameRulesParam("metal", math.max(0, metal - (alienUdef.customParams.metal or 0)))
	end
end

function HandleLuaMessage(msg)
	local msg_table = explode('|', msg)
	if msg_table[1] == 'unlock' then
		local name = msg_table[2]
		if not GG.Tech.UnlockTech(name) then
			Spring.Log("tech", LOG.ERROR, "Something went wrong unlocking tech: " .. name)
			return
		end
		if name == "carrierDrones" then
			GG.UpdateDroneParameters(1, 5)
		elseif name == "antiMissilePointDefense" then
			Spring.SetUnitRulesParam(ufoID, "PDisUnlocked", 1)
		end
	elseif msg_table[1] == 'upgrade' then
		local name = msg_table[2]
		
		if not GG.Tech.UpgradeTech(name) then
			Spring.Log("tech", LOG.ERROR, "Something went wrong upgrading tech: " .. name)
			return
		end
		
		UpdateTech(name)
	end
end

function gadget:RecvLuaMsg(msg)
	HandleLuaMessage(msg)
end

