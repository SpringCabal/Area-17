local Civilian = Unit:New {
	--Internal settings
	objectName             = "citizen.dae",
    name                   = "Civilian",
    unitName               = "civilian",
    script                 = "human.lua",
	
	-- Movement
	acceleration           = 0.5,
	brakeRate              = 0.5,
	-- Keep the original definition speed. The unit_attributes gadget changes
	-- the live ground move type to 2 elmos/second, and the WASM rules do the
	-- same through the typed MoveCtrl API. This definition value also provides
	-- the command-speed ceiling needed when the flee multiplier is applied.
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
        research           = 0,
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

local Civilian1 = Civilian:New {
	name				  = "Civilian1",
	objectName             = "citizen1.dae",
}

return lowerkeys({
    Civilian       = Civilian,
	Civilian1 	    = Civilian1,
})
