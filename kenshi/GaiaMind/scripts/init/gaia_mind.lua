-- GaiaMind: the squad thinks and speaks through Gaia.
-- File mailbox only (no sockets): this script writes inbox.txt, kenshi_eyes.py
-- answers in outbox.txt. Both live in %LOCALAPPDATA%\Gaia\kenshi.
--
-- inbox lines (Lua -> Gaia):
--   select <name> <ts>                       player clicked a squad member
--   event  <kind> <who> <detail> <ts>        hit | ko | death | faction
-- outbox lines (Gaia -> Lua):
--   <name> <words>                           that squad member says the line
--   @gaia  <words>                           shown in the game's message log

local dir = (os.getenv("LOCALAPPDATA") or ".") .. "\\Gaia\\kenshi\\"
local inbox = dir .. "inbox.txt"
local outbox = dir .. "outbox.txt"
local taken = dir .. "outbox.taken"

local speakers = {}
local last_poll, last_roster = 0, -100
local POLL_S, ROSTER_S, HIT_COOLDOWN_S = 1.0, 5.0, 15.0
local last_hit = {}

local function clean(s)
    return (tostring(s or ""):gsub("[\t\r\n]", " "))
end

local function post(...)
    local f = io.open(inbox, "a")
    if not f then return end
    local parts = { ... }
    for i = 1, #parts do parts[i] = clean(parts[i]) end
    f:write(table.concat(parts, "\t"), "\t", os.time(), "\n")
    f:close()
end

local function name_of(c)
    local ok, n = pcall(function() return c:getName() end)
    return ok and n and n ~= "" and n or nil
end

local function is_ours(c)
    local ok, yes = pcall(function() return c:isPlayerCharacter() end)
    return ok and yes
end

local function faction_of(c)
    local ok, n = pcall(function() return c:getFaction().name end)
    return ok and n or "?"
end

local function refresh_roster()
    pcall(function()
        for _, c in ipairs(getGameWorld().player.playerCharacters) do
            local n = name_of(c)
            if n then speakers[n] = c end
        end
    end)
end

Events.on("Character::_NV_select", function(self)
    local name = name_of(self)
    if not name then return end
    speakers[name] = self
    post("select", name)
end)

Events.on("Character::_NV_hitByMeleeAttack", function(self, cutDir, damage, attacker, attack, comboID)
    if not is_ours(self) then return end
    local name = name_of(self)
    local now = os.clock()
    if not name or now - (last_hit[name] or -HIT_COOLDOWN_S) < HIT_COOLDOWN_S then return end
    last_hit[name] = now
    speakers[name] = self
    local who = attacker and name_of(attacker) or "someone"
    post("event", "hit", name, who .. " (" .. (attacker and faction_of(attacker) or "?") .. ")")
end)

Events.on("MedicalSystem::knockout", function(self, skill)
    local c = self.me
    if not c or not is_ours(c) then return end
    local name = name_of(c)
    if name then post("event", "ko", name, "") end
end)

Events.on("Character::declareDead", function(self)
    if not is_ours(self) then return end
    local name = name_of(self)
    if name then
        speakers[name] = nil
        post("event", "death", name, faction_of(self))
    end
end)

Events.on("PlayerInterface::encounterFaction", function(self, faction)
    local ok, n = pcall(function() return faction.name end)
    if ok and n then post("event", "faction", "", n) end
end)

Events.on("MyGUI::Gui::eventFrameStart", function(t)
    local now = os.clock()
    if now - last_roster >= ROSTER_S then
        last_roster = now
        refresh_roster()
    end
    if now - last_poll < POLL_S then return end
    last_poll = now
    if not os.rename(outbox, taken) then return end
    local f = io.open(taken, "r")
    if not f then return end
    for line in f:lines() do
        local who, words = line:match("^([^\t]+)\t(.+)$")
        if who == "@gaia" then
            pcall(function() getGameWorld():showPlayerAMessage(words, true) end)
        elseif who and speakers[who] then
            pcall(function() speakers[who]:sayALine(words, true) end)
        end
    end
    f:close()
    os.remove(taken)
end)
