ffi = require("ffi")

ffi.cdef[[
typedef struct { float red, green, blue, alpha; } color;
typedef struct { uint32_t width, height; } size;
typedef struct { const char *ch; color fg, bg; } cell;
typedef struct { const char *e; uint64_t c; int32_t a1; int32_t a2; int32_t a3; } event;
uintptr_t new_event_handler();
uintptr_t graphic_handle(uint32_t w, uint32_t h);
void bind_screen(uintptr_t g, uintptr_t s);
event pull(uintptr_t g, float t);
uintptr_t create(uint32_t w, uint32_t h, uint32_t f, uintptr_t h);
void destroy(uintptr_t s);
void set_size(uintptr_t g, uint32_t width, uint32_t height);
void set_location(uintptr_t s, int32_t x, int32_t y);
void foreground(uintptr_t g, float r, float g, float b, float a);
void background(uintptr_t g, float r, float g, float b, float a);
void fill(uintptr_t g, int32_t x, int32_t y, int32_t w, int32_t h, const char *ch);
void write_text(uintptr_t g, int32_t x, int32_t y, const char *ch);
void write(uintptr_t g, int32_t x, int32_t y, const char *ch);
void flush(uintptr_t g);
uintptr_t get_buffer(uintptr_t g);
void set_buffer(uintptr_t g, uintptr_t b);
size buf_size(uintptr_t b);
void buf_resize(uintptr_t b, uint32_t w, uint32_t h);
void buf_copy(uintptr_t b, int32_t x, int32_t y, uintptr_t b2);
uintptr_t buf_clone(uintptr_t b);
void buf_fill(uintptr_t b, int32_t x, int32_t y, int32_t w, int32_t h, const char *ch, color fg, color bg);
void buf_write(uintptr_t b, int32_t x, int32_t y, const char *ch, color fg, color bg);
cell buf_get(uintptr_t b, uint32_t x, uint32_t y);
]]
local libDir = debug.getinfo(1).source:match("@?(.*/)")
--freen = ffi.load(libDir.."freen.dll")
local freen = ffi.load("G:\\Projekte\\Satisfactory\\Freen\\target\\i686-pc-windows-msvc\\debug\\freen.dll")

FREEN = {
	fontsize = 24
}

local SCREEN_CACHE = {}

local eventHandler = freen.new_event_handler()

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
	freen.set_size(self._handle, w, h)
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
	freen.write_text(self._handle, x, y, str)
end

function FINComputerGPU:bindScreen(screen)
	self.screen = screen
	freen.bind_screen(self._handle, self.screen:handle(self))
end

-- Freen Exklusive Funktion
function FINComputerGPU:setLocation(x, y)
	freen.set_location(self.screen:handle(self), x, y)
end

local org_event_pull = event.pull
event.pull = function(n)
	evt = org_event_pull()
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
	local buffer = GPUT1Buffer:new()
	buffer._handle = freen.get_buffer(self._handle)
	return buffer
end

function FINComputerGPU:setBuffer(buffer)
	freen.set_buffer(self._handle, buffer._handle)
end

function GPUT1Buffer:setSize(w, h)
	freen.buf_resize(self._handle, w, h)
end

function GPUT1Buffer:getSize(x, y, w, h, c, fg, bg)
	size = freen.buf_size(self._handle)
	return size.width, size.height
end

function GPUT1Buffer:fill(x, y, w, h, c, fg, bg)
	freen.buf_fill(self._handle, x, y, w, h, c, fg, bg)
end

function GPUT1Buffer:setText(x, y, txt, fg, bg)
	freen.buf_write(self._handle, x, y, txt, fg, bg)
end

function GPUT1Buffer:copy(x, y, buffer)
	freen.buf_copy(self._handle, x, y, buffer._handle)
end

function GPUT1Buffer:get(x, y, buffer)
	local cell = freen.buf_get(self._handle, x, y)
	if cell == nil then return nil end
	return tostring(cell.ch), cell.fg, cell.bg
end

function GPUT1Buffer:setRaw(x, y, c, fg, bg)
	freen.buf_copy(self._handle, x, y, c, fg, bg)
end

function GPUT1Buffer:clone()
	local clone = GPUT1Buffer:new()
	clone._handle = freen.buf_clone(self._handle)
	return clone
end

function FINComputerGPU:new()
	--print("new FINComputerGPU:new()", self._width, self._height)
	local o = {
		screen=nil,
		fg = {r=1.0,g=1.0,b=1.0,a=1.0},
		bg = {r=0.0,g=0.0,b=0.0,a=1.0}
	}
	o._handle = freen.graphic_handle(self._width, self._height)
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
