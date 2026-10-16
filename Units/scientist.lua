local Scientist = Unit:New {
	--Internal settings
	objectName             = "scientist.dae",
    name                   = "Scientist",
    unitName               = "scientist",
    script                 = "human.lua",
	
	-- Movement
	acceleration           = 0.5,
	brakeRate              = 0.5,
	-- Matches the original definition. WASM applies the effective 2 elmos/s
	-- profile and the 2.5x flee profile through typed MoveCtrl calls.
	maxVelocity            = 40,
	turnRate               = 3000,
	footprintX             = 1,
	footprintZ             = 1,
	
    movementClass          = "Bot1x1",
	turnInPlace            = true,
	turnInPlaceSpeedLimit  = 1.6, 
	turnInPlaceAngleLimit  = 90,
	
	upright                = true,
	
	sfxtypes                     = {	
		pieceExplosionGenerators = { 
			"bloodytrail", 
		}, 
	},
	
	customParams           = {
		modelradius        = [[12]],
		midposoffset       = [[0 28 0]],
        -- Capture resources (all capturable units should have these fields defined)
        -- Use OOP maybe?
        biomass            = 100,
        research           = 200,
        metal              = 0, -- not to be confused with engine metal
	},
	
	-- Abiltiies
	builder                = false,
	canAttack              = true,
	canGuard               = true,
	canMove                = true,
	canPatrol              = true,
	canStop                = true,
	collide                = true,
	leaveTracks            = false, -- Todo, add tracks
	
	-- Hitbox
	collisionVolumeOffsets = "0 0 0",
	collisionVolumeScales  = "18 45 18",
	collisionVolumeType    = "cylY",
	
	-- Attributes
	category               = "land",
	
	mass                   = 100,
	maxDamage              = 100,
	autoHeal               = 5,
	idleAutoHeal           = 5,
	idleTime               = 60,
	
	-- Economy
	buildCostEnergy        = 100,
	buildCostMetal         = 100,
	buildTime              = 100,
	maxWaterDepth          = 0,
	maxSlope               = 55,
}

local Scientist2 = Scientist:New {
	name				  = "Scientist2",
	unitName               = "scientist2",
	objectName             = "scientist_2.dae",
}


return lowerkeys({
    Scientist       = Scientist,
	Scientist2 	    = Scientist2,
})
