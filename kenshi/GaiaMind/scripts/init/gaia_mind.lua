-- GaiaMind: click a squad member, the world thinks as them, they speak.
-- File mailbox only (no sockets): this script writes inbox.txt, kenshi_eyes.py
-- answers in outbox.txt. Both live in %LOCALAPPDATA%\Gaia\kenshi.

local dir = (os.getenv("LOCALAPPDATA") or ".") .. "\\Gaia\\kenshi\\"
local inbox = dir .. "inbox.txt"
local outbox = dir .. "outbox.txt"
local taken = dir .. "outbox.taken"

local speakers = {}
local last_poll = 0
local POLL_S = 1.0

local function clean(s)
    return (tostring(s or ""):gsub("[\t\r\n]", " "))
end

Events.on("Character::_NV_select", function(self)
    local ok, name = pcall(function() return self:getName() end)
    if not ok or not name or name == "" then return end
    speakers[name] = self
    local f = io.open(inbox, "a")
    if f then
        f:write("select\t", clean(name), "\t", os.time(), "\n")
        f:close()
    end
end)

Events.on("MyGUI::Gui::eventFrameStart", function(t)
    local now = os.clock()
    if now - last_poll < POLL_S then return end
    last_poll = now
    if not os.rename(outbox, taken) then return end
    local f = io.open(taken, "r")
    if not f then return end
    for line in f:lines() do
        local who, words = line:match("^([^\t]+)\t(.+)$")
        local c = who and speakers[who]
        if c then
            pcall(function() c:sayALine(words, true) end)
        end
    end
    f:close()
    os.remove(taken)
end)
