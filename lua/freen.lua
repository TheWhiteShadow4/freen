ffi = require("ffi")

ffi.cdef[[
typedef struct { uint8_t red, green, blue, alpha; } rgb_color;
typedef struct { const char *e; uint64_t c; int32_t a1; int32_t a2; int32_t a3; } event;
int newEventHandler();
int graphicHandle(uint32_t w, uint32_t h);
void bindScreen(uintptr_t n, uintptr_t s);
event pull(uintptr_t n, float t);
int create(uint32_t w, uint32_t h, uint32_t f, uintptr_t h);
void destroy(uintptr_t n);
int getBuffer(uintptr_t n);
void setSize(uintptr_t n, uint32_t width, uint32_t height);
void setLocation(uintptr_t n, int32_t x, int32_t y);
void foreground(uintptr_t n, float r, float g, float b, float a);
void background(uintptr_t n, float r, float g, float b, float a);
void fill(uintptr_t n, int32_t x, int32_t y, int32_t w, int32_t h, const char *ch);
void writeText(uintptr_t n, int32_t x, int32_t y, const char *ch);
void write(uintptr_t n, int32_t x, int32_t y, const char *ch);
void flush(uintptr_t n);
]]
local libDir = debug.getinfo(1).source:match("@?(.*/)")
--freen = ffi.load(libDir.."freen.dll")
local freen = ffi.load("G:\\Projekte\\Satisfactory\\Freen\\target\\i686-pc-windows-msvc\\debug\\freen.dll")

FREEN = {
	fontsize = 24
}

local SCREEN_CACHE = {}

local eventHandler = freen.newEventHandler()

Screens = {}
--- Schließt alle Fenster
function Screens:close()
	for _,s in pairs(SCREEN_CACHE) do
		s:close()
	end
	SCREEN_CACHE = {}
end

function FINComputerGPU:setSize(w, h)
	self._width = w
	self._height = h
	freen.setSize(self._handle, w, h)
end

function FINComputerGPU:getSize(w, h)
	return self._width, self._height
end

function FINComputerGPU:setForeground(r, g, b, a)
	freen.foreground(self._handle, r, g, b, a)
end

function FINComputerGPU:setBackground(r, g, b, a)
	freen.background(self._handle, r, g, b, a)
end

function FINComputerGPU:flush()
	freen.flush(self._handle)
end

function FINComputerGPU:fill(x, y, w, h, ch)
	freen.fill(self._handle, x, y, w, h, ch)
end

function FINComputerGPU:setText(x, y, str)
	freen.writeText(self._handle, x, y, str)
end

function FINComputerGPU:bindScreen(screen)
	self.screen = screen
	freen.bindScreen(self._handle, self.screen:handle(self))
end

-- Freen Exklusive Funktion
function FINComputerGPU:setLocation(x, y)
	freen.setLocation(self.screen:handle(self), x, y)
end

local org_event_pull = event.pull
event.pull = function(n)
	evt = org_event_pull(n)
	if (evt ~= nil) then
		return evt
	end
	n = n or 0.0
	evt = freen.pull(eventHandler, n)
	if (evt.c == 0) then
		return nil
	else
		return ffi.string(evt.e), toUID(evt.c), evt.a1, evt.a2, evt.a3
	end
end

function FINComputerGPU:getBuffer()
	local buffer = GPUT1Buffer:new({self._width, self._height})
	buffer._handle = freen.getBuffer(self._handle)
	return buffer
end

function FINComputerGPU:new()
	--print("new FINComputerGPU:new()", self._width, self._height)
	local o = {
		screen=nil,
		fg = {r=1.0,g=1.0,b=1.0,a=1.0},
		bg = {r=0.0,g=0.0,b=0.0,a=1.0}
	}
	o._handle = freen.graphicHandle(self._width, self._height)
	return o
end

Freen = {}

function Freen:new()
	local f = {}
	setmetatable(f, self)
	self.__index = self
	table.insert(SCREEN_CACHE, f)
	return f
end

function Freen:handle(gpu)
	if (self._handle == nil) then
		self._handle = freen.create(gpu._width, gpu._height, FREEN.fontsize, eventHandler)
	end
	return self._handle
end

function Freen:close()
	if (self._handle ~= nil) then
		freen.close(self._handle)
		self._handle = nil
		
		for idx,s in pairs(SCREEN_CACHE) do
			if s == self then
				table.remove(SCREEN_CACHE, idx)
				return
			end
		end
	end
end

function toUID(uid)
	return uid
end

defineClass(Freen, {
	aliase = {"Freen", "Screen", "FINComputerScreen"},
	displayName = "<3"
})
