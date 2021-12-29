ffi = require("ffi")

ffi.cdef[[
typedef struct { uint8_t red, green, blue, alpha; } rgb_color;
typedef struct { const char *e; uint32_t c; int32_t a1; int32_t a2; int32_t a3; } event;
int newEventHandler();
event pull(uintptr_t n, float t);
int create(uintptr_t w, uint32_t h, uint32_t f, uintptr_t h);
void destroy(uintptr_t n);
void setSize(uintptr_t n, uint32_t width, uint32_t height);
void foreground(uintptr_t n, float r, float g, float b, float a);
void background(uintptr_t n, float r, float g, float b, float a);
void fill(uintptr_t n, int32_t x, int32_t y, int32_t w, int32_t h, const char *ch);
void writeText(uintptr_t n, int32_t x, int32_t y, const char *ch);
void write(uintptr_t n, int32_t x, int32_t y, const char *ch);
void flush(uintptr_t n);
]]
local freen = ffi.load("G:\\Projekte\\Rust\\freen\\target\\i686-pc-windows-msvc\\debug\\freen.dll")

FONT_SIZE = 24

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
	self.screen.width = w
	self.screen.height = h
	if (self.screen._handle == nil) then
		self.screen._handle = freen.create(w, h, FONT_SIZE, eventHandler)
	end
end

function FINComputerGPU:getSize()
	return self.screen.width, self.screen.height
end

function FINComputerGPU:setForeground(r, g, b, a)
	freen.foreground(self.screen._handle, r, g, b, a)
end

function FINComputerGPU:setBackground(r, g, b, a)
	freen.background(self.screen._handle, r, g, b, a)
end

function FINComputerGPU:fill(x, y, w, h, ch)
	freen.fill(self.screen._handle, x, y, w, h, ch)
end

function FINComputerGPU:setText(x, y, str)
	freen.writeText(self.screen._handle, x, y, str)
end

function FINComputerGPU:flush()
	if self.screen._handle ~= nil then
		freen.flush(self.screen._handle)
	end
end

event.pull = function(n)
	n = n or 0.0
	evt = freen.pull(eventHandler, n)
	if (evt.c == 0) then
		return nil
	else
		return ffi.string(evt.e), evt.c, evt.a1, evt.a2, evt.a3
	end
end

Freen = {
	_handle = nil,
	width = 0,
	height = 0,	
}

function Freen:new()
	table.insert(SCREEN_CACHE, self)
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

defineClass(Freen, {
	aliase = {"Freen", "Screen", "FINComputerScreen"},
	displayName = "<3"
})
