local modinfo = {
	name = "Area 17",
	shortname = "ALIR",
	game = "Area 17",
	shortgame = "ALIR",
    version = "$VERSION",
	description = "Alien resources",
	
	modtype = "1",
	-- Single-player only: without this the engine binds a UDP socket and warns
	-- that nobody else can reach it. Nothing here is ever hosted or joined.
	onlyLocal = true,
	depend = {
		"cursors.sdz",
	}
}

return modinfo
