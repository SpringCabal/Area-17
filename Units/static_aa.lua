local Static_AA = Unit:New {
    footprintX			    = 6,
	footprintZ			    = 6,
	buildCostMetal          = 100,
	mass				    = 10e5,
    maxDamage               = 1500,
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
    yardMap                 = "oooooo oooooo oooooo oooooo oooooo oooooo",

	category               = "land armed",

    name                = "Static AA",
    script              = "static_aa.lua",

	 customParams        = {
		modelradius        = [[30]],
		midposoffset       = [[0 50 0]],

		terrainblock_x     = 64,
		terrainblock_z     = 64,
    },

	-- Hitbox
	collisionVolumeOffsets = "0 0 0",
	collisionVolumeScales  = "70 76 70",
	collisionVolumeType    = "cylY",

	sfxtypes               = {
		explosiongenerators = {
		  [[custom:explosion]],
		},
		pieceExplosionGenerators = { 
			"missiletrailsmall", 
		}, 
	},

	weapons                = {
		{
			def                = "beamLaser",
			onlyTargetCategory = "ufo",
		},
	},

	weaponDefs             = {
		beamLaser = {
			name                    = "Pulse laser",
			areaOfEffect            = 2,
			avoidFeature            = false,
			avoidFriendly           = false,
			avoidNeutral            = false,
			avoidGround             = false,
			coreThickness           = 0,
			craterBoost             = 0,
			craterMult              = 0,

			damage                  = {
				default = 15,
			},

			explosionGenerator      = [[custom:genericshellexplosion-small-red]],
			interceptedByShieldType = 1,
			noSelfDamage            = true,
			range                   = 2000,
			reloadtime              = 0.01,
			rgbColor                = "0.8 0.2 0.2",
			soundTrigger            = true,
			thickness               = 5,
			tolerance               = 5000,
			turret                  = true,
			weaponType              = "LaserCannon",
			weaponVelocity          = 2500,
			soundstart  			= "pulse.wav",
		},
	},
}

local Strong_AA = Static_AA:New {
	name                = "Strong AA",
	maxDamage           = 4500,
	
	footprintX			    = 12,
	footprintZ			    = 12,
	yardMap                 = "oooooooooooo oooooooooooo oooooooooooo oooooooooooo oooooooooooo oooooooooooo oooooooooooo oooooooooooo oooooooooooo oooooooooooo oooooooooooo oooooooooooo",
	collisionVolumeOffsets  = '0 50 0',
	collisionVolumeScales   = '100 84 100',
	collisionVolumeType     = 'cylY',
	customParams        = {
		modelradius        = [[80]],
		midposoffset       = [[0 50 0]],

		terrainblock_x     = 150,
		terrainblock_z     = 150,
    },
	sfxtypes               = {
		explosiongenerators = {
		  [[custom:explosion]],
		},
		pieceExplosionGenerators = { 
			"missiletrailsmall", 
		}, 
	},
	weaponDefs             = {
		beamLaser = {
			explosionGenerator      = [[custom:genericshellexplosion-large-purple]],
			name                    = "Beam Laser",
			coreThickness           = 0.2,
			craterBoost             = 0,
			craterMult              = 0,

			damage                  = {
				default = 50,
			},
			rgbColor                = "0.5 0.1 0.9",
			thickness               = 15,
		},
	},
}

return lowerkeys({
    Static_AA       = Static_AA,
	Strong_AA 	    = Strong_AA,
})
