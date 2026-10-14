local armLeft = piece('armLeft');
local armRight = piece('armRight');
local footLeft = piece('footLeft');
local footRIght = piece('footRIght');
local handLeft = piece('handLeft');
local handRight = piece('handRight');
local head = piece('head');
local legLeft = piece('legLeft');
local legRight = piece('legRight');
local thighLeft = piece('thighLeft');
local thighRight = piece('thighRight');
local torso = piece('torso');
local parachute

local alienDefID = UnitDefNames["alien"].id
if Spring.GetUnitDefID(unitID) == alienDefID then
	parachute = piece('parachute');
end

local Animations = {};

local scriptEnv = {
	armLeft = armLeft,
	armRight = armRight,
	footLeft = footLeft,
	footRIght = footRIght,
	handLeft = handLeft,
	handRight = handRight,
	head = head,
	legLeft = legLeft,
	legRight = legRight,
	thighLeft = thighLeft,
	thighRight = thighRight,
	torso = torso,
	x_axis = x_axis,
	y_axis = y_axis,
	z_axis = z_axis,
}

Animations['stop'] = VFS.Include("Scripts/animations/human_stop.lua", scriptEnv);
Animations['walk'] = VFS.Include("Scripts/animations/human_walk.lua", scriptEnv);
Animations['float'] = VFS.Include("Scripts/animations/human_float.lua", scriptEnv);

local SIG_WALK =  tonumber("00001",2);
local SIG_IDLE =  tonumber("00010",2);

local floating = false;

function constructSkeleton(unit, piece, offset)
    if (offset == nil) then
        offset = {0,0,0};
    end

    local bones = {};
    local info = Spring.GetUnitPieceInfo(unit,piece);

    for i=1,3 do
        info.offset[i] = offset[i]+info.offset[i];
    end

    bones[piece] = info.offset;
    local map = Spring.GetUnitPieceMap(unit);
    local children = info.children;

    if (children) then
        for i, childName in pairs(children) do
            local childId = map[childName];
            local childBones = constructSkeleton(unit, childId, info.offset);
            for cid, cinfo in pairs(childBones) do
                bones[cid] = cinfo;
            end
        end
    end
    return bones;
end

function script.Create()
    local map = Spring.GetUnitPieceMap(unitID);
    local offsets = constructSkeleton(unitID,map.Scene, {0,0,0});

    for a,anim in pairs(Animations) do
        for i,keyframe in pairs(anim) do
            local commands = keyframe.commands;
            for k,command in pairs(commands) do
                -- commands are described in (c)ommand,(p)iece,(a)xis,(t)arget,(s)peed format
                -- the t attribute needs to be adjusted for move commands from blender's absolute values
                if (command.c == "move") then
                    local adjusted =  command.t - (offsets[command.p][command.a]);
                    Animations[a][i]['commands'][k].t = command.t - (offsets[command.p][command.a]);
                end
            end
        end
    end
    PlayAnimation('stop');
end

local animCmd = {['turn']=Turn,['move']=Move};
function PlayAnimation(animname)
    local anim = Animations[animname];
    for i = 1, #anim do
        local commands = anim[i].commands;
        for j = 1,#commands do
            local cmd = commands[j];
            animCmd[cmd.c](cmd.p,cmd.a,cmd.t,cmd.s);
        end
        if(i < #anim) then
            local t = anim[i+1]['time'] - anim[i]['time'];
            Sleep(t*33); -- sleep works on milliseconds
        end
    end
end

local function Walk()
	Signal(SIG_WALK)
	SetSignalMask(SIG_WALK)
	PlayAnimation("walk", true);
	while true do
		PlayAnimation("walk", false);
	end
end

local function Stop()
	Signal(SIG_WALK)
	SetSignalMask(SIG_WALK)
	PlayAnimation("stop",true)
end

local function Float()
	Signal(SIG_WALK)
	SetSignalMask(SIG_WALK)
	PlayAnimation("float", true);
	while floating do
		PlayAnimation("float", false);
	end
end

function Abduction_float()
	if not floating then
		floating = true
		local ux, uy, uz = Spring.GetUnitPosition(unitID)
		Spring.PlaySoundFile("sounds/scream.wav", 0.8, ux, uy, uz, 'sfx')
		StartThread(Float)
	end
end

function script.StartMoving()
	floating = false
	Signal(SIG_WALK);
	StartThread(Walk);
end

function script.StopMoving()
	floating = false
	Signal(SIG_WALK);
	StartThread(Stop);
end

local function ParachuteFall()
	Move(parachute, y_axis, 45, 15)
 	Move(parachute, z_axis, -60, 20)
	WaitForMove(parachute, y_axis)
	WaitForMove(parachute, z_axis)
	Hide(parachute)
	
	floating = false
end

function Alien_land()
	if Spring.GetUnitDefID(unitID) == alienDefID then
		StartThread(ParachuteFall)
	end
end

-- None of this works because it doesn't seem like it counts as a "falling unit"
-- function script.setSFXoccupy(num)
-- 	if floating and num == 4 and Spring.GetUnitDefID(unitID) == alienDefID then
-- 		floating = false
-- 		Hide(parachute)
-- 	end
-- end

function script.Landed()
	Alien_land()
end

function script.Killed(recentDamage, maxHealth)
	local flags = SFX.FIRE+SFX.SMOKE+SFX.NO_HEATCLOUD;
	Explode(armLeft,flags)
	Explode(armRight,flags)
	Explode(footLeft,flags)
	Explode(footRIght,flags)
	Explode(handLeft,flags)
	Explode(handRight,flags)
	Explode(head,flags)
	Explode(legLeft,flags)
	Explode(legRight,flags)
	Explode(thighLeft,flags)
	Explode(thighRight,flags)
	return 0
end
