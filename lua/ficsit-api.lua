ffi = require("ffi")
ffi.cdef[[
void Sleep(int ms);
]]

local classes = {}

function typeof(t, c)
	return t ~= nil and t.__index == c
end

local Class = {
	__tostring = function(self) return "Class" end
}
function Class:getType() return self end
Class.__index = Class

--- Definiert ein Klassen Objekt für einen Typen und fügt dieses zur Lookuptable hinzu.
--- Die Klasse muss ein Feld 'aliase' besitzen, indem eine Liste von Namen für die Lookuptable stehen.
--- Zudem können weitere Reflektion Felder wie internalName oder displayName angegeben werden.
function defineClass(c, cls)
	setmetatable(cls, Class)
	cls.instantiate = function()
		o = {}
		setmetatable(o, c)
		c.__index = c
		c.__tostring = function(self) return self.getType().name end
		c.getType = function() return cls end
		if o.new then o:new() end
		return o
	end
	for _,n in pairs(cls.aliase) do
		classes[n] = cls
	end
	cls.name = cls.aliase[1]
	return c
end

function findClass(str)
	return classes[str]
end

Network = Network or {}
NickTable = {}

local Component = {
	__tostring = function(o) return o.id end
}

--- Erstellt ein Array mit einer neuen virtuellen Netzwerk Komponente mit zufällig generierter Id.
--- Die so erstellte Komponente wird dem Netzwerk hinzugefügt.
function createComponentIds(query, nick)
	id = ""
	for i = 1,32 do
		r = math.random(0, 15)
		id = id..(r > 9 and string.char(r + 55) or r)
	end
	-- Komponente erstellen und ins Netzwerk einfügen.
	if getmetatable(query) == Class then
		comp = query:instantiate()
	else
		comp = {}
	end
	for i,v in pairs(Actor) do
		comp[i] = v
	end
	comp.id = id
	comp.nick = nick
	
	Network[id] = comp
	if nick ~= nil then NickTable[nick] = comp end
	return {id}
end

computer = {
	getInstance = function() end,
	beep = function(pitch) end,
	stop = function() os.exit() end,
	panic = function(error) end,
	reset = function() end,
	skip = function() end,
	getEEPROM = function() end,
	setEEPROM = function(code) end,
	time = function() end,
	millis = function() return os.clock() end,
	getPCIDevices = function(type)
		if type == nil then return {} end
		cls = classes[type.name] 
		if cls == nil then return {} end
		return {cls:instantiate()}
	end
}

component = {
	proxy = function(ids)
		if ids[1] ~= nil then
			ret = {}
			for _,id in pairs(c) do
				table.insert(ret, component.proxy(id))
			end
			return ret
		else
			if type(ids) ~= "string" then error("id is not a string") end
			return Network[ids]
		end
	end,
	findComponent = function(query)
		if query == nil then return {nil} end
		
		if Network[query] ~= nil then
			return Network[query].id
		elseif NickTable[query] ~= nil then
			return NickTable[query].id
		else
			return createComponentIds(query, "virtual")
		end
	end
}

event = {
	listen = function(c) end,
	listening = function() return {} end,
	ignore = function(c) end,
	ignoreAll = function() end,
	clear = function() end,
	pull = function(n)
		if n ~= nil and n > 0 then ffi.C.Sleep(n*1000.0) end
	end
}

filesystem = {
	initFileSystem = function(path) end,
	makeFileSystem = function(type, name) end,
	removeFileSystem = function(name) end,
	mount = function(device, mountPoint) end,
	open = function(path, mode) end,
	createDir = function(path) end,
	remove = function(path) end,
	move = function(from, to) end,
	rename = function(path, name) end,
	childs = function(path) end,
	exists = function(path) end,
	isFile = function(path) end,
	isDir = function(path) end,
	doFile = function(path) end,
	loadFile = function(path) end
}

Actor = {
	location = {0, 0, 0},
	scale = {1, 1, 1},
	rotation = {0, 0, 0},
	powerConnectors = {},
	factoryConnectors = {},
	pipeConnectors = {},
	inventories = {},
	networkConnectors = {},
}

function Actor:getPowerConnectors()
	return {}
end

function Actor:getFactoryConnectors()
	return {}
end

function Actor:getPipeConnectors()
	return {}
end

function Actor:getInventories()
	return {}
end

function Actor:getNetworkConnectors()
	return {}
end

GPUT1Buffer = {
	width=120,
	height=30
}

function GPUT1Buffer:setSize(w, h)
	self.width = w
	self.height = h
end

function GPUT1Buffer:getSize()
	return self.width, self.height
end

function GPUT1Buffer:get(x, y, c, fg, bg) end
function GPUT1Buffer:set(x, y, c, fg, bg) end
function GPUT1Buffer:setText(x, y, txt, fg, bg) end
function GPUT1Buffer:fill(x, y, w, h, c, fg, bg) end
function GPUT1Buffer:setRaw(x, y, c, fg, bg) end
function GPUT1Buffer:copy(x, y, buffer, textMode, fgMode, bgMode) end
function GPUT1Buffer:clone()
	return GPUT1Buffer:new({width=self.width, height=self.height})
end

function GPUT1Buffer:new()
	o = o or {}
	setmetatable(o, self)
	self.__index = self
	return o
end

FINComputerGPU = {
	screen=nil,
	buffer=GPUT1Buffer:new(),
	fg = {r=1.0,g=1.0,b=1.0,a=1.0},
	bg = {r=0.0,g=0.0,b=0.0,a=1.0},
}
function FINComputerGPU:bindScreen(screen)
	self.screen = screen
end
function FINComputerGPU:setSize(w, h)
	self.buffer:setSize(w, h)
end
function FINComputerGPU:getSize()
	return self.buffer.getSize()
end
function FINComputerGPU:setForeground(r, g, b, a)
	self.fg.r = r
	self.fg.g = g
	self.fg.b = b
	self.fg.a = a
end
function FINComputerGPU:setBackground(r, g, b, a)
	self.bg.r = r
	self.bg.g = g
	self.bg.b = b
	self.bg.a = a
end
function FINComputerGPU:setBuffer(buffer)
	if typeof(buffer, GPUT1Buffer) then
		self.buffer = GPUT1Buffer:new({width=0,height=0})
	else
		self.buffer = buffer
	end
end

function FINComputerGPU:getScreen() return self.screen end
function FINComputerGPU:getBuffer() return self.buffer end
function FINComputerGPU:flush() end
function FINComputerGPU:fill(x, y, w, h, str) end
function FINComputerGPU:setText(x, y, str) buffer:setText(x, y, str) end

defineClass(FINComputerGPU, {
	aliase = {"GPU_T1_C", "GPUT1"},
	displayName = "Computer GPU T1"
})

FINComputerScreen = defineClass({}, {
	aliase = {"Build_Screen_C", "Screen"},
	displayName = "Large Screen"
})

Powerpol = defineClass({}, {
	aliase = {"Build_PowerPoleMk1_C"}
})

function Actor:getPowerConnectors()
	return {PowerConnection:new({owner=self})}
end

PowerConnection = {
	owner = nil,
	connections = 0,
	maxConnections = 4
}
function PowerConnection:new(o)
	o = o or {}
	setmetatable(o, self)
	self.__index = self
	return o
end

function PowerConnection:getPower()
	return nil
end