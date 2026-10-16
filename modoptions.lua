-- Wiki: http://springrts.com/wiki/Modoptions.lua

--  Custom Options Definition Table format

--  NOTES:
--  - using an enumerated table lets you specify the options order

--
--  These keywords must be lowercase for LuaParser to read them.
--
--  key:      the string used in the script.txt
--  name:     the displayed name
--  desc:     the description (could be used as a tooltip)
--  type:     the option type
--  def:      the default value;
--  min:      minimum value for number options
--  max:      maximum value for number options
--  step:     quantization step, aligned to the def value
--  maxlen:   the maximum string length for string options
--  items:    array of item strings for list options
--  scope:    "all", "player", "team", "allyteam"      <<< not supported yet >>>


--------------------------------------------------------------------------------
--------------------------------------------------------------------------------
local options = {
	{
		key = "test_mode",
		name = "Headless integration tests",
		desc = "Runs the Area 17 automated integration scenario.",
		type = "bool",
		def = false,
	},
	{
		key = "visual_test",
		name = "Off-screen visual smoke test",
		desc = "Captures Area 17 screenshots for the Xvfb verification runner.",
		type = "bool",
		def = false,
	},
}

return options
