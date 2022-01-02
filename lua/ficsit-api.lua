lfs = require "lfs"
ffi = require("ffi")
ffi.cdef[[
void Sleep(int ms);
]]

function class(base, init)
	local c = {}
	if not init and type(base) == 'function' then
		init = base
		base = nil
	elseif type(base) == 'table' then
		for i,v in pairs(base) do c[i] = v end
		c._base = base
	end
	c.__index = c
	local mt = {}
	mt.__call = function(class_tbl, ...)
		local obj = {}
		setmetatable(obj,c)
		if init then
			init(obj,...)
		else 
		if base and base.init then
		base.init(obj, ...)
		end
	end
		return obj
	end
	c.init = init
	c.is_a = function(self, klass)
		local m = getmetatable(self)
		while m do 
			if m == klass then return true end
			m = m._base
		end
		return false
	end
	setmetatable(c, mt)
	return c
end

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
		local o = c.new and c:new() or {}
		setmetatable(o, c)
		c.__index = c
		c.__tostring = function(self) return self.getType().name end
		c.getType = function() return cls end
		return o
	end
	for _,n in pairs(cls.aliase) do
		classes[n] = cls
	end
	cls.name = cls.aliase[1]
	return c
end

local table_keys = function(t)
	local keys={}
	local n=0
	for k,_ in pairs(t) do
		n=n+1
		keys[n]=k
	end
	return keys
end

function lazyArray(func, ...)
	local a = {}
	local args = {...}
	setmetatable(a, {
		__index = function(a, i)
			if i > 0 then
				local obj = func(table.unpack(args))
				a[i] = obj
				return obj
			else
				return nil
			end
		end
	})
	return a
end

function findClass(str)
	return classes[str]
end

Network = Network or {}
ALIASES = {}

local function newUID()
	local id = ""
	for i = 1,32 do
		r = math.random(0, 15)
		id = id..(r > 9 and string.char(r + 55) or r)
	end
	return id
end

local Component = class(function(p)
	p.id = newUID()
end)

function Component:__tostring() return self.id end

--- Erstellt ein Array mit einer neuen virtuellen Netzwerk Komponente mit zufällig generierter Id.
--- Die so erstellte Komponente wird dem Netzwerk hinzugefügt.
function createComponentIds(query, nick)
	local id = newUID()
	-- Komponente erstellen und ins Netzwerk einfügen.
	local comp
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
	if nick ~= nil then
		if ALIASES[nick] == nil then ALIASES[nick] = {} end
		table.insert(ALIASES[nick], comp)
	end
	return {id}
end

computer = {
	getInstance = function() end,
	beep = function(pitch) end,
	stop = function() os.exit() end,
	panic = function(error)
		print(error)
		computer.stop()
	end,
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
		return lazyArray(cls.instantiate)
	end
}

component = {
	proxy = function(ids)
		if type(ids) == 'table' then
			ret = {}
			for _,id in pairs(ids) do
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
		elseif ALIASES[query] ~= nil then
			ids = {}
			for k,v in pairs(ALIASES[query]) do
				table.insert(ids, v.id)
			end
			return ids
		elseif query == "" then
			return table_keys(Network)
		else
			return lazyArray(createComponentIds, query, nil)
		end
	end
}

local EVENT_QUEUE = {}

function queueEvent(type, comp, ...)
	table.insert(EVENT_QUEUE, {type, comp, ...})
end

event = {
	listen = function(c) end,
	listening = function() return {} end,
	ignore = function(c) end,
	ignoreAll = function() end,
	clear = function() end,
	pull = function(n)
		if #EVENT_QUEUE > 0 then
			return table.unpack(table.remove(EVENT_QUEUE))
		else
			if n ~= nil and n > 0 then ffi.C.Sleep(n*1000.0) end
		end
	end
}

FS_ROOT = "drives/"
local ROOT_DEVICE = nil
local DRIVES = {}

local function startsWith(str, c)
	if (#str < #c) then return false end
    return string.sub(str, 1, #c) == c
end

local function stripPath(path)
	if path == "/" then return path end
	path = string.gsub(path..'/', '/+$', '/')
	path = string.gsub(path, '^/+', '')
	return path
end

local function findFile(name)
	for k,v in pairs(DRIVES) do
		if startsWith(name, k) then
			local path = '/'..v..string.sub(name, #k+1, #name)
			return string.gsub(lfs.currentdir(), "\\", "/")..path
		end
	end
	return nil
end

FileSystem = class(Component, function(p, device)
	p.device = device
	p.mounted = false
end)

filesystem = {
	initFileSystem = function(path)
		if ROOT_DEVICE ~= nil then return false end
		path = stripPath(path)
		if path == "/" then error("Empty device is not allowed.", 2) end
		ROOT_DEVICE = path
	end,
	makeFileSystem = function(type, name) end,
	removeFileSystem = function(name) end,
	mount = function(device, mountPoint)
		device = stripPath(device)
		if startsWith(device, ROOT_DEVICE) then
			local drive = string.sub(device, #ROOT_DEVICE+1, #device)
			if drive ~= nil then
				drive = FS_ROOT..drive
				mountPoint = string.gsub(mountPoint, '^/', '')
				DRIVES[mountPoint] = drive
				return os.rename(drive, drive) and true or false
			end
			return false
		end
	end,
	open = function(path, mode)
		path = string.gsub(path, '^//', '/')
		local file = findFile(path)
		return io.open(file, mode)
	end,
	createDir = function(path) end,
	remove = function(path) end,
	move = function(from, to) end,
	rename = function(path, name) end,
	childs = function(path)
		if ROOT_DEVICE == nil then error("no device at path found", 2) end
		local list = {}
		local entry
		for entry in lfs.dir(FS_ROOT) do
			if entry ~= "." and entry ~= ".." then
				table.insert(list, '/'..entry)
			end
		end
		return list
	end,
	exists = function(path) end,
	isFile = function(path) end,
	isDir = function(path) end,
	doFile = function(path) end,
	loadFile = function(path) end,
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

GPUT1Buffer = {}

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

function GPUT1Buffer:new(o)
	o = o or {}
	setmetatable(o, self)
	self.__index = self
	return o
end

FINComputerGPU = {
	_width=120,
	_height=40
}

function FINComputerGPU:new()
	--print("FINComputerGPU:new()", dump(self, 1))
	local o = {
		screen=nil,
		buffer=GPUT1Buffer:new({self._width, self._height}),
		fg = {r=1.0,g=1.0,b=1.0,a=1.0},
		bg = {r=0.0,g=0.0,b=0.0,a=1.0},
	}
	return o
end

function FINComputerGPU:bindScreen(screen)
	self.screen = screen
end
function FINComputerGPU:setSize(w, h)
	local oldW, oldH = self.buffer:getSize()
	self.buffer:setSize(w, h)
	queueEvent(ScreenSizeChanged, self, oldW, oldH)
end
function FINComputerGPU:getSize()
	return self.buffer:getSize()
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
function FINComputerGPU:fill(x, y, w, h, str) self.buffer:fill(x, y, w, h, str) end
function FINComputerGPU:setText(x, y, str) self.buffer:setText(x, y, str) end

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