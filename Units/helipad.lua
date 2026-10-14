local Helipad = Unit:New {
    customParams        = {
        radius = 0,
    },
    footprintX			    = 8,
	footprintZ			    = 8,
	buildCostMetal          = 100,
	mass				    = 10e5,
    maxDamage               = 10000,
    collisionVolumeScales   = '0 0 0',
    collisionVolumeType     = 'cylY',
    pushResistant           = true,
    blocking                = false,
    canMove                 = false,
    canGuard                = false,
    canPatrol               = false,
    canRepeat               = false,
    stealth				    = true,
	turnRate			    = 0,
	upright				    = true,
    sightDistance		    = 0,
    yardMap                 = "oooooooo oooooooo oooooooo oooooooo oooooooo oooooooo oooooooo oooooooo",

    name                = "Helipad",
    script              = "helipad.lua",
    onoffable               = true,
    isAirbase = true,
}

return lowerkeys({
    Helipad       = Helipad,
})
