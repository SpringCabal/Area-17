function widget:GetInfo()
  return {
    name      = "UFO Beam",
    desc      = "what it says above",
    author    = "ashdnazg",
    date      = "22 August 2015",
    license   = "GNU GPL, v2 or later",
    layer     = 10,
    enabled   = true  --  loaded by default?
  }
end

local ufoDef = UnitDefNames["ufo"]
local ufoDefID = ufoDef.id
-- Keep the renderer usable across engine API variants.  wantedHeight is the
-- exported value for cruiseAlt, while the other names cover older/newer APIs.
local maxHeight = tonumber(ufoDef.wantedHeight or ufoDef.cruiseAltitude or ufoDef.cruiseAlt) or 350

local CONE_WIDTH = 150

local ufoID

VFS.Include("LuaUI/Utilities/glVolumes.lua")


local lightData = {
	[0] = {
		radiusOscillation = 0.05,
		period = 15,
		color = {1, 0.9, 0, 0.6},
		alphaOscillation = 0
	},
	[1] = {
		radiusOscillation = 0.15,
		period = 3,
		color = {1, 0.9, 0, 1},
		alphaOscillation = -0.2
	}
}

function widget:UnitCreated(unitID, unitDefID, unitTeam)
	if unitDefID == ufoDefID then
		ufoID = unitID
	end
end

function widget:UnitDestroyed(unitID)
	if ufoID == unitID then
		ufoID = nil
	end
end

function widget:Initialize()
	for _, unitID in ipairs(Spring.GetAllUnits()) do
		local unitDefID = Spring.GetUnitDefID(unitID)
		widget:UnitCreated(unitID, unitDefID)
	end
end

function widget:DrawWorld()
	if not ufoID or not Spring.ValidUnitID(ufoID) then
		return
	end

	local ux, uy, uz = Spring.GetUnitViewPosition(ufoID)
	if not ux then
		return
	end
	local ground = math.max(0, Spring.GetGroundHeight(ux, uz))
	local height = math.max(1, uy - ground)
	local data = lightData[Spring.GetUnitRulesParam(ufoID, "beam_enabled") or 0]
	if not data then
		return
	end
	local abductionRadius = tonumber(Spring.GetGameRulesParam("abductionRadius")) or 70
	local radius = abductionRadius * height / maxHeight

	local ro, p, color, ao = data.radiusOscillation, data.period, {unpack(data.color)}, data.alphaOscillation
	radius = math.sin(Spring.GetGameFrame() / p) * ro * radius + radius
	color[4] = math.sin(Spring.GetGameFrame() / p) * ao + color[4]
	gl.Blending(GL.SRC_ALPHA, GL.ONE_MINUS_SRC_ALPHA)
	gl.DepthMask(false)
	-- Keep the transparent cone depth-tested so units and terrain can occlude
	-- it. This matches the original Lua and WASM renderers.
	gl.DepthTest(true)
	gl.Culling(true)
	gl.AlphaTest(GL.GREATER, 0)
	gl.Utilities.DrawMyCone(ux,ground,uz, height, radius, 10, color)
	gl.Culling(false)
	gl.AlphaTest(false)
	gl.DepthTest(false)
	gl.DepthMask(true)
	gl.Blending(false)
end
