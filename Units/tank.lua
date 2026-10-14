
local Tank = Unit:New {
	--Internal settings
	objectName             = "tank.dae",
    name                   = "tank",
    unitName               = "tank",
    script                 = "tank" .. ".lua",

	sfxtypes               = {
		explosiongenerators = {
		  [[custom:explosion]],
		},
		pieceExplosionGenerators = { 
			"missiletrailsmall", 
		}, 
	},

	-- Movement
	acceleration           = 0.5,
	brakeRate              = 0.5,
	maxVelocity            = 30,
	turnRate               = 300,
	footprintX             = 2,
	footprintZ             = 2,

    movementClass          = "Bot1x1",
	turnInPlace            = true,
	turnInPlaceSpeedLimit  = 1.6,
	turnInPlaceAngleLimit  = 90,

	upright                = true,

	customParams           = {
        -- Capture resources (all capturable units should have these fields defined)
        -- Use OOP maybe?
        biomass            = 5000,
        research           = 0,
        metal              = 0, -- not to be confused with engine metal

		modelradius        = [[30]],
		midposoffset       = [[0 25 0]],

		abduct_mult        = 0.2,
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

	movestate              = 2,

	collisionVolumeOffsets = "0 0 8",
	collisionVolumeScales  = "50 50 70",
	collisionVolumeType    = "cylZ",

	-- Attributes
	category               = "land armed",

	mass                   = 100,
	maxDamage              = 5000,
	autoHeal               = 5,
	idleAutoHeal           = 5,
	idleTime               = 60,

	-- Economy
	buildCostEnergy        = 100,
	buildCostMetal         = 100,
	buildTime              = 100,
	maxWaterDepth          = 0,
	maxSlope               = 55,

	weapons                = {
		{
			def                = "missile",
			onlyTargetCategory = "ufo",
		},
	},

	weaponDefs             = {

		missile = {
			name                    = "AA Missile",
			areaOfEffect            = 24,
			burst                   = 2,
			burstrate               = 0.7,
			canattackground         = false,
			--cegTag                  = [[missiletrailbluebig]],
			cegTag                   = "missiletrailsmall",
			craterBoost             = 0,
			craterMult              = 0,
			cylinderTargeting       = 3,

			collisionsize = 20,
			size					= 500,
			targetable              = 1,

			damage                  = {
				default = 200,
			},

			rgbColor                = "0 0.6 1",
			explosionGenerator      = [[custom:genericshellexplosion-medium]],
			fireStarter             = 70,
			flightTime              = 1.6,
			impactOnly              = true,
			impulseBoost            = 0,
			impulseFactor           = 0.4,
			interceptedByShieldType = 2,
			leadLimit               = 0,
			lineOfSight             = true,
			model                   = [[missile.dae]],
			noSelfDamage            = true,
			range                   = 1720,
			reloadtime              = 10,
			renderType              = 1,
			smokeTrail              = false,
			--soundHit                = [[explosion/ex_med11]],
			startVelocity           = 700,
			--texture2                = [[AAsmoketrail]],
			tracks                  = true,
			turnRate                = 70000,
			turret                  = true,
			weaponAcceleration      = 800,
			weaponTimer             = 5,
			weaponType              = "MissileLauncher",
			weaponVelocity          = 1800,
		},
	},
}

local Small_Tank = Tank:New {
	name                = "small_tank",
	objectName          = "tank_2.dae",
	mass                = 60,
	maxDamage           = 1000,
	maxVelocity         = 180,
	acceleration        = 1,
	brakeRate           = 1,
	weaponDefs             = {
		missile = {
			size					= 3,
			reloadtime              = 3,
			damage = {
				default = 35,
			},
			range                   = 720,
			rgbColor                = "1 1 0",
			weaponVelocity          = 4000,
		}
	},
	sfxtypes               = {
		explosiongenerators = {
		  [[custom:explosion]],
		},
		pieceExplosionGenerators = { 
			"missiletrailsmall", 
		}, 
	},
	customParams           = {
        -- Capture resources (all capturable units should have these fields defined)
        -- Use OOP maybe?
        biomass            = 1000,
		abduct_mult        = 0.6,
	},
}

return lowerkeys({
    Tank        = Tank,
	Small_Tank = Small_Tank,
})
