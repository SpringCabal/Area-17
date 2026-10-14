include "constants.lua"


local barrels = {
	piece "muzzleLeft",
	piece "muzzleRight"
}

local pieces = {
	piece "tubeLeft",
	piece "tubeRight",
	piece "turret",
	piece "base",
}

local barrels = {piece "muzzleLeft",
				piece "muzzleRight"}
local turret = piece 'turret'
local tubeLeft = piece "tubeLeft"
local tubeRight = piece "tubeRight"

local SIG_AIM = 2


local currentBarrel = #barrels
function script.Create()

end

function script.QueryWeapon(num)

	return barrels[currentBarrel]
end

function script.AimFromWeapon(num)
	return barrels[currentBarrel]
end

function script.AimWeapon(num, heading, pitch)
	Signal(SIG_AIM)
	SetSignalMask(SIG_AIM)

	Turn(turret, z_axis, heading, math.rad(100))
 	Turn(tubeLeft, x_axis, -pitch, math.rad(50))
	Turn(tubeRight, x_axis, -pitch, math.rad(50))
	WaitForTurn(turret, z_axis)
 	WaitForTurn(tubeLeft, x_axis)
	WaitForTurn(tubeRight, x_axis)
	return true
end

function script.Shot(num)
	currentBarrel = currentBarrel % #barrels + 1
	local x, y, z = Spring.GetUnitPosition(unitID)
	Spring.PlaySoundFile("sounds/rocket.wav", 70, x, y, z)
end

function script.Killed(recentDamage, maxHealth)
	local flags = SFX.FIRE+SFX.SMOKE+SFX.NO_HEATCLOUD+SFX.EXPLODE_ON_HIT;
	for i=1,#pieces do
		EmitSfx(pieces[i],1024);
		Explode(pieces[i],flags);
		Sleep(math.random(400));
	end
	return 0
end
