function widget:GetInfo()
	return {
		name 	= "Movement Control",
		desc	= "Controls unit movement directly with keyboard.",
		author	= "Google Frog",
		date	= "22 August 2015",
		license	= "GNU GPL, v2 or later",
		layer	= 20, 
		enabled = true
	}
end

-------------------------------------------------------------------
-------------------------------------------------------------------

require('keysym.lua')
local controls = VFS.Include("LuaHandler/Utilities/game_controls.lua")
local UP = KEYSYMS.UP
local DOWN = KEYSYMS.DOWN
local LEFT = KEYSYMS.LEFT
local RIGHT = KEYSYMS.RIGHT
local W = KEYSYMS.W
local S = KEYSYMS.S
local A = KEYSYMS.A
local D = KEYSYMS.D
local PAGEUP = KEYSYMS.PAGEUP
local PAGEDOWN = KEYSYMS.PAGEDOWN
local N_0 = KEYSYMS.N_0
local N_9 = KEYSYMS.N_9

local ufoID
local ufoDefID = UnitDefNames["ufo"].id
local trackedUFOID

local cloak = 0

local weapons, abilities
local currentWeapon

local aimWeapons = {
	["incendiaryBeamLaser"] = true,
	["gravityBeam"] = true,
}

-------------------------------------------------------------------
-------------------------------------------------------------------
-- Camera

local LOOK_FORWARDS = 12

local UFO_START_X = 14000
local UFO_START_Z = 1600

local AVERAGE_SPEEDS = 12
local curIndex = 1
local lastXSpeed = {average = 0}
local lastYSpeed = {average = 0}
local lastZSpeed = {average = 0}

local mouseControl = false
local keyControl = false

local lx, ly, lz = UFO_START_X, 0, UFO_START_Z

local cx, cy, cz = UFO_START_X, 0, UFO_START_Z

local function UpdateSpeed(newVal, values, default)
	-- Keep the same unscaled movement history and operation order as the
	-- shared Rust smoother. Scaling before accumulation changes f32 rounding.
	values.average = values.average + (newVal - (values[curIndex] or 0))
	values[curIndex] = newVal
	return values.average/AVERAGE_SPEEDS
end

local function SetCameraPosition(x, y, z, transition)
	local newState = {
		px = x,
		py = y + 25,
		pz = z,
		mode = 1,
		flipped = -1,
		fov = 45,
		height = 2000,
		angle = 0.4,
	}
	Spring.SetCameraState(newState, transition)
end

local function GetCameraTarget()
	if not ufoID or not Spring.ValidUnitID(ufoID) or Spring.GetUnitIsDead(ufoID) then
		return nil
	end

	local x, y, z = Spring.GetUnitViewPosition(ufoID)
	if not x then
		return nil
	end

	local vx, vy, vz = Spring.GetUnitVelocity(ufoID)
	vx, vy, vz = vx or 0, vy or 0, vz or 0
	return x + vx * LOOK_FORWARDS, y + vy * LOOK_FORWARDS, z + vz * LOOK_FORWARDS
end

local function ResetCameraToUFO()
	if Spring.GetGameRulesParam("devMode") == 1 then
		return false
	end

	local x, y, z = GetCameraTarget()
	if not x then
		return false
	end

	-- Start the smoothing history at the UFO rather than at the map origin.
	lx, ly, lz = x, y, z
	cx, cy, cz = x, y, z
	curIndex = 1
	lastXSpeed = {average = 0}
	lastYSpeed = {average = 0}
	lastZSpeed = {average = 0}
	SetCameraPosition(cx, cy, cz, 0)
	Spring.SelectUnitArray({})
	trackedUFOID = ufoID
	return true
end

-- Follow the UFO explicitly. The engine's `track on` command is not reliable
-- during startup and can leave the camera at the map's default corner.
local function UpdateCamera()
	if Spring.GetGameRulesParam("devMode") == 1 then
		return
	end

	local x, y, z = GetCameraTarget()
	if not x then
		return
	end

	-- Apparent unit speed.
	local sx, sy, sz = x - lx, y - ly, z - lz
	lx, ly, lz = x, y, z

	sx = UpdateSpeed(sx, lastXSpeed)
	sy = UpdateSpeed(sy, lastYSpeed)
	sz = UpdateSpeed(sz, lastZSpeed)

	curIndex = curIndex + 1
	if curIndex > AVERAGE_SPEEDS then
		curIndex = 1
	end

	-- A small amount of smoothing keeps the camera movement comfortable while
	-- preserving the original Lua camera's velocity lead.
	cx, cy, cz = cx + sx, cy + sy, cz + sz
	SetCameraPosition(cx, cy, cz, 0.4)
end
-------------------------------------------------------------------
-------------------------------------------------------------------

local function MovementControl()
	local x, z = 0, 0
	
	if Spring.GetKeyState(A) or Spring.GetKeyState(LEFT) then
		x = x - 1
	end
	if Spring.GetKeyState(D) or Spring.GetKeyState(RIGHT) then
		x = x + 1
	end
	if Spring.GetKeyState(W) or Spring.GetKeyState(UP) then
		z = z - 1
	end
	if Spring.GetKeyState(S) or Spring.GetKeyState(DOWN) then
		z = z + 1
	end
	
	Spring.SendLuaRulesMsg('movement|' .. x .. '|' .. z)
end

local function WeaponControl()
	local mx, my, lmb, mmb, rmb = Spring.GetMouseState()
	if lmb then
		if aimWeapons[currentWeapon] then
			Spring.SendLuaRulesMsg('fireWeapon')
		else
			local _, pos = Spring.TraceScreenRay(mx, my, true)
			if pos then
				local x, y, z = pos[1], pos[2], pos[3]
				Spring.SendLuaRulesMsg('fireWeapon|' .. x .. '|' .. y .. '|' .. z )
			end
		end
	end
end

local lastx, lasty, lastz

local function AimingControl()
	local mx, my = Spring.GetMouseState()
	local _, pos = Spring.TraceScreenRay(mx, my, true)
	if pos then
		local x, y, z = pos[1], pos[2], pos[3]
		--if x ~= lastx or y ~= lasty or z ~= lastz then
			lastx, lasty, lastz = x, y, z
			Spring.SendLuaRulesMsg('aimWeapon|' .. x .. '|' .. y .. '|' .. z )
		--end
	end
end

-- handles weapon switching and abilities
function widget:KeyPress(key, mods, isRepeat)
	if controls.IsWriting() then
		return false
	end
	if ufoID then
		for _, abilityName in pairs(abilities) do
			local cd = Spring.GetGameRulesParam(abilityName .. "CD") or 0
			local tech = WG.Tech.GetTech(abilityName)
			if not tech.locked and Spring.GetKeyCode(tech.key) == key and cd == 0 then
				Spring.SendLuaRulesMsg('ability|' .. abilityName)
				return
			end
		end
		if key >= N_0 and key <= N_9 then
			local num = key - N_0
			local weaponName = weapons[num]
			if weaponName == nil or weaponName == currentWeapon or WG.Tech.GetTech(weaponName).locked then 
				return
			end
			WG.UI.SetAbilityEnabled(weaponName, true)
			if currentWeapon ~= nil then
				WG.UI.SetAbilityEnabled(currentWeapon, false)
			end
			currentWeapon = weaponName
			Spring.PlaySoundFile("sounds/select.wav")
			Spring.SendLuaRulesMsg('changeWeapon|' .. weaponName)
		end
		if key == LEFT or key == RIGHT or key == UP or key == DOWN or key == W or key == A or key == S or key == D then
			keyControl = true
			return true
		end
		
		if key == PAGEUP or key == PAGEDOWN then
			return true
		end
	end
	
end

function widget:KeyRelease(key)
	if not (Spring.GetKeyState(A) or Spring.GetKeyState(LEFT) or Spring.GetKeyState(D) or Spring.GetKeyState(RIGHT) or Spring.GetKeyState(W) or Spring.GetKeyState(UP) or Spring.GetKeyState(S) or Spring.GetKeyState(DOWN)) then
		keyControl = false
	end
end

function widget:GameFrame(n)
	-- disable all movement and weapon control when landed/landing (OLD STUFF)
	-- if not ufoID or Spring.GetUnitStates(ufoID).autoland then 
	
	if not ufoID then
		return
	end
	if controls.IsWriting() then
		Spring.SendLuaRulesMsg('movement|0|0')
		keyControl, mouseControl = false, false
		return
	end
	
	AimingControl()
	if keyControl then
		MovementControl()
	end
	
	if mouseControl then
		WeaponControl()
	end
end

function widget:Update()
	if ufoID and trackedUFOID ~= ufoID then
		ResetCameraToUFO()
	end
	UpdateCamera()
end

function widget:MousePress(mx, my, button)
	if controls.IsWriting() then
		return false
	end
	local alt, ctrl, meta, shift = Spring.GetModKeyState()
	if button == 1 and not Spring.IsAboveMiniMap(mx, my) then
		local _, pos = Spring.TraceScreenRay(mx, my, true)
		if pos then
			local x, y, z = pos[1], pos[2], pos[3]
			Spring.SendLuaRulesMsg('fireWeapon|' .. x .. '|' .. y .. '|' .. z )
			mouseControl = true
			return (Spring.GetGameRulesParam("devMode") ~= 1)
		end
	end
end

function widget:MouseRelease(mx, my, button)
	if button == 1 then
		mouseControl = false
	end
end

-------------------------------------------------------------------
-------------------------------------------------------------------
function widget:Initialize()
	if Game.gameName == "Scenario Editor Area 17" then
		widgetHandler:RemoveWidget()
		return
	end
	for _, unitID in pairs(Spring.GetAllUnits()) do
		self:UnitCreated(unitID, Spring.GetUnitDefID(unitID), Spring.GetUnitTeam(unitID))
	end
	weapons = WG.Tech.GetWeapons()
	abilities = WG.Tech.GetAbilities()
	
	local newState = {
		px = cx,
		py = cy + 25,
		pz = cz,
		mode = 1,
		flipped = -1,
		fov = 45,
		height = 2000,
		angle = 0.4,
	}
	Spring.SetCameraState(newState, 0.4)
	ResetCameraToUFO()
end

function widget:UnitCreated(unitID, unitDefID, unitTeam)
	if unitDefID == ufoDefID then
		ufoID = unitID
		if currentWeapon ~= nil then
			WG.UI.SetAbilityEnabled(currentWeapon, true)
			Spring.SendLuaRulesMsg('changeWeapon|' .. currentWeapon)
		end
		ResetCameraToUFO()
	end
end

function widget:UnitDestroyed(unitID, unitDefID, unitTeam)
	if ufoID == unitID then
		Spring.SendCommands("track off")
		ufoID = nil
		trackedUFOID = nil
	end
end
