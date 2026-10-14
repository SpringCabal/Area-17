if not gadgetHandler:IsSyncedCode() then
	return
end
--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

function gadget:GetInfo()
	return {
		name 	= "UFO Abduction",
		desc	= "Implements main abduction weapon of the UFO.",
		author	= "Google Frog (help from xponen impulse float toggle)",
		date	= "22 August 2015",
		license	= "GNU GPL, v2 or later",
		layer	= 10,
		enabled = true
	}
end

-------------------------------------------------------------------
-------------------------------------------------------------------

local GRAVITY = Game.gravity/30/30
local RAD_PER_ROT = (math.pi/(2^15))

local ufoUnitDefID = UnitDefNames["ufo"].id
local alienUnitDefID = UnitDefNames["alien"].id
local ufoID

local endcubeDefID = UnitDefNames["endcube"].id
local helipadDefID = UnitDefNames["helipad"].id

local noAbduct = {
	[UnitDefNames["drone"].id] = true,
	[UnitDefNames["helipad"].id] = true,
	[UnitDefNames["building1"].id] = true,
	[UnitDefNames["building2"].id] = true,
	[UnitDefNames["building3"].id] = true,
	[UnitDefNames["building4"].id] = true,
	[UnitDefNames["building4bigtank"].id] = true,
	[UnitDefNames["static_aa"].id] = true,
	[UnitDefNames["strong_aa"].id] = true,
	--[UnitDefNames["alien"].id] = true,
	[UnitDefNames["ufo_wreck"].id] = true,
}

local Vector = Spring.Utilities.Vector

-------------------------------------------------------------------
-------------------------------------------------------------------

local abductionRadius = 70
local abductionSpeed = 6
local abductionAccelMult = 1

local function UpdateAbductionParameters(radius, speed, accel)
	abductionRadius = radius
	abductionSpeed = speed
	abductionAccelMult = accel
	Spring.SetGameRulesParam("abductionRadius", radius)
end

-------------------------------------------------------------------
-------------------------------------------------------------------

local function callScript(unitID, funcName, args)
	local env = unitID and Spring.UnitScript.GetScriptEnv(unitID)
	local func = env and env[funcName]
	if func then
		Spring.UnitScript.CallAsUnit(unitID,func, args)
	end
end

local function FloatUnitInDirection(unitID, x, y, z, uvx, uvz, speed, hAccel, vAccel)

	local dx, dy, dz = Spring.GetUnitVelocity(unitID)

	local vx, vy, vz = Vector.Norm(speed, x, y, z)

	local dyCorrection = math.min(vAccel, vy + GRAVITY - dy)

	local dxCorrection = vx - dx + uvx
	local dzCorrection = vz - dz + uvz

	if Vector.AbsVal(dxCorrection, dzCorrection) > hAccel then
		dxCorrection, dzCorrection = Vector.Norm(hAccel, dxCorrection, dzCorrection)
	end

	local headingInRadian = Spring.GetUnitHeading(unitID)*RAD_PER_ROT --get current heading
	Spring.SetUnitRotation(unitID, 0, -headingInRadian, 0) --restore current heading. This force unit to stay upright/prevent tumbling.TODO: remove negative sign if Spring no longer mirror input anymore
	Spring.AddUnitImpulse(unitID, 0, -4, 0) --Note: -4/+4 hax is for impulse capacitor  (Spring 91 only need -1/+1, Spring 94 require at least -4/+4). TODO: remove -4/+4 hax if no longer needed
	Spring.AddUnitImpulse(unitID, dxCorrection, 4 + dyCorrection, dzCorrection)
end

local function SetAbductionArea(ax, ay, az, vx, vz, grabDistance, radius, speed)
	if Spring.GetUnitRulesParam(ufoID, "beam_enabled") == -1 then
		return
	end
	local units = Spring.GetUnitsInCylinder(ax, az, radius)

    local enabled = false
	for i = 1, #units do
		local unitID = units[i]
        local unitDefID = Spring.GetUnitDefID(unitID)
		if unitID ~= ufoID and not noAbduct[unitDefID] then
			local _,_,_,ux,uy,uz = Spring.GetUnitPosition(unitID, true)
			local unitHeight = Spring.GetUnitHeight(unitID)

			if ay - uy < grabDistance + unitHeight*2 then
                local udef = UnitDefs[unitDefID]
                local biomass = Spring.GetGameRulesParam("biomass") or 0
                local research = Spring.GetGameRulesParam("research") or 0
                local metal = Spring.GetGameRulesParam("metal") or 0
                Spring.SetGameRulesParam("biomass", biomass + (udef.customParams.biomass or 0))
                Spring.SetGameRulesParam("research", research + (udef.customParams.research or 0))
                Spring.SetGameRulesParam("metal", metal + (udef.customParams.metal or 0))
				local hp, maxHP = Spring.GetUnitHealth(ufoID)
				hp = hp + maxHP * 0.05
				Spring.SetUnitHealth(ufoID, math.min(hp, maxHP))
				if unitDefID == alienUnitDefID then
					Spring.PlaySoundFile("sounds/yay.wav", 100, 'sfx')
				else
					Spring.PlaySoundFile("sounds/swoop.wav", 50, ux, uy, uz, 'sfx')
				end
				Spring.DestroyUnit(unitID, false, true)
                if unitDefID == endcubeDefID then
                    Spring.SetGameRulesParam("gameOver", 1)
                    Spring.SetGameRulesParam("gameWon", 1)
                end
			else
                enabled = true
				local mult = 1
				local udef = UnitDefs[unitDefID]
				if udef and udef.customParams and udef.customParams.abduct_mult then
					mult = mult*tonumber(udef.customParams.abduct_mult)
				end
				
				FloatUnitInDirection(unitID, ax - ux, ay - uy, az - uz, vx, vz, speed*mult, 2*mult*abductionAccelMult, GRAVITY + 0.4*mult*abductionAccelMult)
				callScript(unitID, "Abduction_float")
			end
		end
	end
    local env = Spring.UnitScript.GetScriptEnv(ufoID)
    Spring.UnitScript.CallAsUnit(ufoID, env.script.SetBeamEnabled, enabled)
end


-------------------------------------------------------------------
-------------------------------------------------------------------

function gadget:UnitCreated(unitID, unitDefID)
	if ufoUnitDefID == unitDefID then
		ufoID = unitID
	end
end

function gadget:GameFrame(frame)

	if ufoID then
		local x,y,z = Spring.GetUnitPosition(ufoID)
		local vx, _, vz = Spring.GetUnitVelocity(ufoID)
		if not x then
			return
		end

		SetAbductionArea(x, y, z, vx, vz, 40, abductionRadius, abductionSpeed)
	end
end

function gadget:Initialize()
	GG.UpdateAbductionParameters = UpdateAbductionParameters
	
	UpdateAbductionParameters(70, 7, 1)
end
