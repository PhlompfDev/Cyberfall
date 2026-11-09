-- Cyberfall settings. Saved to settings.json, shown in Native Settings UI if it's installed.
-- Percentages are of Titanfall 2's values.

local defaults = {
  enabled = true,
  needsPerk = true,
  length = 100,
  speed = 100,
  kick = 100,
  tilt = 100,
  slideEnabled = true,
  slideSpeed = 100,
}

local settings = {}
for key, value in pairs(defaults) do
  settings[key] = value
end

local function load()
  local file = io.open("settings.json", "r")
  if not file then
    return
  end
  local ok, saved = pcall(json.decode, file:read("*a"))
  file:close()
  if ok and type(saved) == "table" then
    for key in pairs(defaults) do
      if type(saved[key]) == type(defaults[key]) then
        settings[key] = saved[key]
      end
    end
  end
end

local function save()
  local file = io.open("settings.json", "w+")
  if file then
    file:write(json.encode(settings))
    file:close()
  end
end

local function apply()
  Game.CyberfallWallSetSettings(settings.enabled, settings.needsPerk,
    settings.length / 100, settings.speed / 100, settings.kick / 100, settings.tilt / 100)
  Game.CyberfallSlideSetSettings(settings.slideEnabled, settings.slideSpeed / 100)
end

local function setter(key)
  return function(value)
    settings[key] = value
    save()
    apply()
  end
end

local function menu()
  local ui = GetMod("nativeSettings")
  if not ui then
    return
  end
  local tab = "/Cyberfall"
  local section = tab .. "/wallrunning"
  local sliding = tab .. "/sliding"
  if not ui.pathExists(tab) then
    ui.addTab(tab, "Cyberfall")
  end
  if ui.pathExists(section) then
    ui.removeSubcategory(section)
  end
  if ui.pathExists(sliding) then
    ui.removeSubcategory(sliding)
  end
  ui.addSubcategory(section, "Wallrunning")

  ui.addSwitch(section, "Wallrunning", "Turn wallrunning on or off.",
    settings.enabled, defaults.enabled, setter("enabled"))
  ui.addSwitch(section, "Requires Air Dash", "Only wallrun once you have the Air Dash perk in the Reflexes tree. Turn off to wallrun from the start.",
    settings.needsPerk, defaults.needsPerk, setter("needsPerk"))
  ui.addRangeFloat(section, "Wallrun length", "How long you can stay on one wall. 100% is Titanfall 2.",
    25, 200, 5, "%.0f%%", settings.length, defaults.length, setter("length"))
  ui.addRangeFloat(section, "Wallrun speed", "How fast you run along a wall. 100% is Titanfall 2.",
    25, 200, 5, "%.0f%%", settings.speed, defaults.speed, setter("speed"))
  ui.addRangeFloat(section, "Wall kick strength", "How hard jumping off a wall pushes you. 100% is Titanfall 2.",
    25, 200, 5, "%.0f%%", settings.kick, defaults.kick, setter("kick"))
  ui.addRangeFloat(section, "Camera tilt", "How far the camera leans away from the wall. 0% turns it off.",
    0, 200, 5, "%.0f%%", settings.tilt, defaults.tilt, setter("tilt"))

  ui.addSubcategory(sliding, "Sliding")
  ui.addSwitch(sliding, "Momentum slide", "Titanfall-style slide: keeps your speed, and crouching into a landing slides. Off is the base game's slide.",
    settings.slideEnabled, defaults.slideEnabled, setter("slideEnabled"))
  ui.addRangeFloat(sliding, "Slide speed", "How much speed a slide adds. 100% is Titanfall 2.",
    25, 200, 5, "%.0f%%", settings.slideSpeed, defaults.slideSpeed, setter("slideSpeed"))
end

registerForEvent("onInit", function()
  load()
  apply()
  menu()
end)
