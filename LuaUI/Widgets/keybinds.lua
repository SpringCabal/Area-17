function widget:GetInfo()
  return {
    name      = "Keybinds",
    desc      = "",
    author    = "Bluestone",
    date      = "in the future",
    license   = "GPL-v2",
    layer     = -10000,
    enabled   = true,
  }
end

local bindText, mouseText
local Chili, screen0
local children = {}
local x,y,h

local purple = "\255\255\10\255"
local white = "\255\255\255\255"

function SetBindings()
    local binds = { --real keybinds

        "Any+pause  pause",

        --"Alt+b  debug",
        --"--Alt+v  debugcolvol",

		"ctrl+q reloadforce",

		"f12 screenshot",

		"Any+up moveforward ",
		"Any+left moveleft ",
		"Any+right moveright",
		"Any+down moveback",

		--"Any+w moveforward ",
		--"Any+a moveleft ",
		--"Any+d moveright",
		--"Any+s moveback",

		--"Any+pageup moveup",
		--"Any+pagedown movedown",
    }

    for _,binding in pairs(binds) do
        Spring.SendCommands("bind ".. binding)
    end
end

function widget:Initialize()
    local devMode = true
    if devMode then
        Spring.SendCommands("unbindkeyset w")
        Spring.SendCommands("unbindkeyset s")
		Spring.SendCommands("unbindkeyset c")
	else
        Spring.SendCommands("unbindkeyset enter chat") --because because.
    end
    SetBindings()

    bindText = { -- keybinds told to player
        --purple .. "Q : " .. white .. "swap pull / push",
        --purple .. "A : " .. white .. "stop shooting",
        --purple .. "W : " .. white .. "jump (+ left mouse)",
        --purple .. "S : " .. white .. "stop shooting & moving",
    }

    mouseText = {
--         purple .. "Left click: " .. white .. "Shoot",
-- 		purple .. "1-9: " .. white .. "Switch weapon",
        --purple .. "Scroll Wheel : " .. white .. "Zoom",
		purple .. "WASD or Arrows: " .. white .. "Move",
		purple .. "Left click : " .. white .. "Shoot",
		purple .. "T : " .. white .. "Technology Panel",
		purple .. "1 - 9 : " .. white .. "Switch weapon",
        purple .. "Ctrl+Q : " .. white .. "Quit",
    }


    if (not WG.Chili) then
		return
	end
	Chili = WG.Chili
	screen0 = Chili.Screen0

    MakeBindingText()
end


function MakeBindingText()
    if (not WG.Chili) then
		return
	end

    for _,child in pairs(children) do
        screen0:RemoveChild(child)
    end


    h = 20
    y = h*(#bindText + #mouseText)
    x = 10

    for _,text in ipairs(mouseText) do
        AddLine(text,x,y)
        y = y - h
    end
    for _,text in ipairs(bindText) do
        AddLine(text,x,y)
        y = y - h
    end
end

function  AddLine(text,x,y,h)
    children[#children+1] = Chili.Label:New{
		 right = x,
		 align = "right",
         bottom = y,
         parent = screen0,
         caption = text,
    }
end
