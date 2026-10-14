if not gadgetHandler:IsSyncedCode() then
	return
end
--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

function gadget:GetInfo()
	return {
		name 	= "UFO Controller",
		desc	= "Handles UFO control input.",
		author	= "Google Frog",
		date	= "22 August 2015",
		license	= "GNU GPL, v2 or later",
		layer	= 10,
		enabled = true
	}
end

-------------------------------------------------------------------
-------------------------------------------------------------------

local sqrt = math.sqrt

local Vector = Spring.Utilities.Vector

-------------------------------------------------------------------
-------------------------------------------------------------------

local ufoUnitDefID = UnitDefNames["ufo"].id
local alienUnitDefID = UnitDefNames["alien"].id
local ufoID
local alienID

local movementMessage
local weaponMessage
local ufoMoving

local aimx, aimy, aimz = 0, 0, 0

-------------------------------------------------------------------
-------------------------------------------------------------------
-- Movement Functions

local function GiveClampedMoveGoal(unitID, x, z, radius)
	radius = radius or 16
	local cx, cz = Spring.Utilities.ClampPosition(x, z)
	local cy = Spring.GetGroundHeight(cx, cz)
	--Spring.MarkerAddPoint(cx, cy, cz)
	Spring.SetUnitMoveGoal(unitID, cx, cy, cz, radius, nil, false) -- The last argument is whether the goal is raw
	return true
end

local function MoveUFO(unitID, x, z, range, radius)
	if not (unitID and Spring.ValidUnitID(unitID)) then
		return
	end
	local speed = Spring.GetUnitRulesParam(ufoID, "selfMoveSpeedChange") or 1
	range = (range or 1000)*speed

	local ux, uy, uz = Spring.GetUnitPosition(unitID)

	local moveVec = Vector.Norm(range, {x, z})

	GiveClampedMoveGoal(unitID, moveVec[1] + ux, moveVec[2] + uz, radius)
end

-------------------------------------------------------------------
-------------------------------------------------------------------

function gadget:UnitCreated(unitID, unitDefID)
	if ufoUnitDefID == unitDefID then
		ufoID = unitID
		Spring.GiveOrderToUnit(unitID, CMD.IDLEMODE, {0}, {}) --no land

		Spring.SetGameRulesParam("ufo_scare_radius", 500)
	end
end

function gadget:UnitDestroyed(unitID, unitDefID)
	if ufoID == unitID then
		-- The replacement UFO starts without these effects; retain only cooldowns.
		for _, abilityName in pairs(GG.Tech.GetAbilities()) do
			Spring.SetGameRulesParam(abilityName .. "Duration", 0)
		end
		Spring.SetGameRulesParam("ufo_scare_radius", 500)
		local x, y, z = Spring.GetUnitPosition(unitID)
		delayedCall = {Spring.GetGameFrame() + 10, {x,y,z}} -- we should really use GG.delayCall API
		ufoID = nil
	end
end

local function callScript(unitID, funcName, args)
	local env = unitID and Spring.UnitScript.GetScriptEnv(unitID)
	local func = env and env[funcName]
	if func then
		Spring.UnitScript.CallAsUnit(unitID,func, args)
	end
end

function gadget:GameFrame(frame)
	if delayedCall and delayedCall[1] == frame then
		delayedCall = delayedCall[2]
		local alienID = Spring.CreateUnit(alienUnitDefID, delayedCall[1], delayedCall[2] + 100, delayedCall[3], 0, 1)
		callScript(alienID, "Abduction_float")
		
		delayedCall = nil
		delayedCall2 = { Spring.GetGameFrame() + 10, alienID }
	end
	if delayedCall2 and delayedCall2[1] == frame then
		local alienID = delayedCall2[2]
		callScript(alienID, "Alien_land")
		delayedCall2 = nil
	end
	if ufoID then
		local x, y, z = Spring.GetUnitPosition(ufoID)

		if aimx then
			local env = Spring.UnitScript.GetScriptEnv(ufoID)
			if env then
				Spring.UnitScript.CallAsUnit(ufoID, env.script.AimWeapons, aimx + x, aimy + y, aimz + z)
			end
		end
		if weaponMessage and weaponMessage.frame + 2 <= frame then
			Spring.SetUnitTarget(ufoID, nil)
			weaponMessage = false
		end

		if (movementMessage and movementMessage.frame + 2 > frame) then
			MoveUFO(ufoID, movementMessage.x, movementMessage.z)
			ufoMoving = true
		else

			local vx, _, vz = Spring.GetUnitVelocity(ufoID)
			if vx then
				local speed = Vector.AbsVal(vx, vz)
				if ufoMoving or (speed > 6) then
					MoveUFO(ufoID, vx, vz, 20)
					ufoMoving = false
				end

				if weaponMessage then
					if weaponMessage.x then
						Spring.SetUnitTarget(ufoID, weaponMessage.x, weaponMessage.y, weaponMessage.z)
					else
						Spring.SetUnitTarget(ufoID, aimx + x, aimy + y, aimz + z)
					end
				end
			end

			movementMessage = false
		end


		Spring.SetGameRulesParam("ufo_x", x)
		Spring.SetGameRulesParam("ufo_y", y)
		Spring.SetGameRulesParam("ufo_z", z)

		-- decrease all cooldowns
		local abilities = GG.Tech.GetAbilities()
		for _, abilityName in pairs(abilities) do
			local cd = Spring.GetGameRulesParam(abilityName .. "CD") or 0
			cd = math.max(0, cd - 1)
			Spring.SetGameRulesParam(abilityName .. "CD", cd)
		end
		-- decrease all active durations
		for _, abilityName in pairs(abilities) do
			local dur = Spring.GetGameRulesParam(abilityName .. "Duration") or 0
			if dur == 1 then
				DisableAbility(abilityName)
			end
			dur = math.max(0, dur - 1)
			Spring.SetGameRulesParam(abilityName .. "Duration", dur)
		end
	end
end

-- disable abilities here
function DisableAbility(abilityName)
	if abilityName == "cloak" then
		Spring.SetUnitCloak(ufoID, false)
		Spring.SetGameRulesParam("ufo_scare_radius", 500)
	elseif abilityName == "haste" then
		Spring.SetUnitRulesParam(ufoID, "selfMoveSpeedChange", 1)
		Spring.SetUnitRulesParam(ufoID, "selfMaxAccelerationChange", 1)
		GG.UpdateUnitAttributes(ufoID)
	elseif abilityName == "independenceDayGun" then
		local env = Spring.UnitScript.GetScriptEnv(ufoID)
		if env then
			Spring.UnitScript.CallAsUnit(ufoID, env.script.StopIndependence)
		end
	end
end

function gadget:Initialize()
	for _, unitID in ipairs(Spring.GetAllUnits()) do
		local unitDefID = Spring.GetUnitDefID(unitID)
		gadget:UnitCreated(unitID, unitDefID)
	end
end

-------------------------------------------------------------------
-------------------------------------------------------------------
-- Aim Functions

local function UpdateUfoAim(x, y, z)
	local ux, uy, uz = Spring.GetUnitPosition(ufoID)
	x, y, z = x - ux, y - uy, z - uz

	local diff = {x - aimx, y - aimy, z - aimz}
	local absVal = Vector.AbsVal(diff)

	if absVal < 40 then
		aimx, aimy, aimz = x, y, z
	else
		diff = Vector.Norm(40, diff)
		aimx, aimy, aimz = aimx + diff[1], aimy + diff[2], aimz + diff[3]
	end
end

local function Teleport(distance)
	local x, y, z = Spring.GetUnitPosition(ufoID)
	local vx, vy, vz = Spring.GetUnitVelocity(ufoID)

	if not (x and vx) then
		return
	end

	Spring.SpawnCEG("teleportOut", x, y, z, 0, 1, 0, 12, 12)
	
	local height = Spring.GetGroundHeight(x, z)
	y = y - height

	vx, vz = Vector.Norm(distance, vx, vz)
	x, z = Spring.Utilities.ClampPosition(vx + x, vz + z)

	height = Spring.GetGroundHeight(x, z)
	y = y + height

	Spring.SetUnitPosition(ufoID, x, y, z)
	
	Spring.SpawnCEG("teleportIn", x, y, z, 0, 1, 0, 12, 12)
end

-------------------------------------------------------------------
-------------------------------------------------------------------
-- Handling messages

function HandleLuaMessage(msg)
	local msg_table = Spring.Utilities.ExplodeMessage('|', msg)
	if msg_table[1] == 'movement' then
		local x = tonumber(msg_table[2])
		local z = tonumber(msg_table[3])

		if x == 0 and z == 0 then
			movementMessage = false
		else
			movementMessage = {
				frame = Spring.GetGameFrame(),
				x = x,
				z = z
			}
		end
	end

	if msg_table[1] == 'aimWeapon' then
		local x = tonumber(msg_table[2])
		local y = tonumber(msg_table[3])
		local z = tonumber(msg_table[4])

		if ufoID then
			UpdateUfoAim(x,y,z)
		end
	end

	if msg_table[1] == 'fireWeapon' then
		local x = tonumber(msg_table[2])
		local y = tonumber(msg_table[3])
		local z = tonumber(msg_table[4])

		if ufoID then
			if x then
				Spring.SetUnitTarget(ufoID, x, y, z)

				weaponMessage = {
					frame = Spring.GetGameFrame(),
					x = x,
					y = y,
					z = z
				}
			else
				local ux, uy, uz = Spring.GetUnitPosition(ufoID)
				Spring.SetUnitTarget(ufoID, aimx + ux, aimy + uy, aimz + uz)

				weaponMessage = {
					frame = Spring.GetGameFrame(),
				}
			end
		end
	end



	-- Handle all abilities here. Abilities can only be turned on and last a certain duration.
	if msg_table[1] == 'ability' then
		if not ufoID then
			return
		end
		local abilityName = msg_table[2]
		local tech = GG.Tech.GetTech(abilityName)
		local duration = tech.ability.duration
		local cooldown = tech.ability.cooldown

		local _, multiplier = GG.Tech.GetTechTooltip(tech.name)

		Spring.SetGameRulesParam(abilityName .. "CD", cooldown)
		Spring.SetGameRulesParam(abilityName .. "Duration", duration)

		local x, y, z = Spring.GetUnitPosition(ufoID)
		if abilityName == "cloak" then
			Spring.SetGameRulesParam(abilityName .. "CD", cooldown * (1 - multiplier / 100))
			Spring.SetUnitCloak(ufoID, 4)
			Spring.PlaySoundFile("sounds/cloak.wav", 40, x, y, z, 'sfx')
			Spring.SetGameRulesParam("ufo_scare_radius", 0)
		elseif abilityName == "haste" then
			local baseHaste = 1.6
			local speedModifier = multiplier/100 + 1 -- Tech modifier
			local haste = baseHaste * speedModifier
			Spring.SetGameRulesParam(abilityName .. "Duration", duration * speedModifier)
			Spring.PlaySoundFile("sounds/haste.wav", 40, x, y, z, 'sfx')

			Spring.SetUnitRulesParam(ufoID, "selfMoveSpeedChange", haste)
			Spring.SetUnitRulesParam(ufoID, "selfMaxAccelerationChange", haste)
			GG.UpdateUnitAttributes(ufoID)
		elseif abilityName == "teleport" then
			local baseDistance = 800 -- elmo or w/e, customize
			local distanceModifier = multiplier/100 + 1 -- Tech modifier
			local distance = baseDistance * distanceModifier
			Spring.PlaySoundFile("sounds/teleport.wav", 40, x, y, z, 'sfx')
			Teleport(distance)
		elseif abilityName == "independenceDayGun" then
			local durationMod = multiplier/100 + 1 -- Tech modifier
			Spring.SetGameRulesParam(abilityName .. "Duration", duration * durationMod)
			local env = Spring.UnitScript.GetScriptEnv(ufoID)
			if env then
				Spring.UnitScript.CallAsUnit(ufoID, env.script.StartIndependence)
			end
		end
	end

	if msg_table[1] == 'changeWeapon' and ufoID then
		local env = Spring.UnitScript.GetScriptEnv(ufoID)
		Spring.UnitScript.CallAsUnit(ufoID, env.script.SetCurrentWeapon, msg_table[2])
	end
end

function gadget:RecvLuaMsg(msg)
	HandleLuaMessage(msg)
end

