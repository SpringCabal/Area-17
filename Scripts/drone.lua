include "constants.lua"

-- pieces
local head = piece('head');

function script.Create()
end

function script.QueryWeapon(num)
	return head
end

function script.AimFromWeapon(num) 
	return head
end

function script.AimWeapon(num, heading, pitch)
	return true
end

function script.Killed(recentDamage, maxHealth)
	return 0
end