ffi = require("ffi")

ffi.cdef[[
typedef struct { const char id[16]; uintptr_t h; } uid_handle;
typedef struct { float r, g, b, a; } color;
typedef struct { uint32_t width, height; } size;
typedef struct { const char ch[4]; size_t l; color fg, bg; } cell;
typedef struct { const char *e; const char c[16]; int32_t a1; int32_t a2; int32_t a3; const char *x; } event;
uintptr_t new_event_handler();
uintptr_t graphic_handle(uint32_t w, uint32_t h);
void bind_screen(uintptr_t g, uintptr_t s);
event pull(uintptr_t g, float t);
uid_handle create_screen(uint32_t f, uintptr_t h);
void destroy_screen(uintptr_t s);
void set_size(uintptr_t g, uint32_t width, uint32_t height);
void set_location(uintptr_t s, int32_t x, int32_t y);
void foreground(uintptr_t g, color c);
void background(uintptr_t g, color c);
void fill(uintptr_t g, int32_t x, int32_t y, int32_t w, int32_t h, const char *ch);
void write_text(uintptr_t g, int32_t x, int32_t y, const char *ch);
void write(uintptr_t g, int32_t x, int32_t y, const char *ch);
void flush(uintptr_t g);
uintptr_t get_buffer(uintptr_t g);
void set_buffer(uintptr_t g, uintptr_t b);
size buf_size(uintptr_t b);
void buf_resize(uintptr_t b, uint32_t w, uint32_t h);
void buf_copy(uintptr_t b, int32_t x, int32_t y, uintptr_t b2, uint8_t tbm, uint8_t fbm, uint8_t bbm);
uintptr_t buf_clone(uintptr_t b);
void buf_fill(uintptr_t b, int32_t x, int32_t y, int32_t w, int32_t h, const char *ch, color fg, color bg);
void buf_write(uintptr_t b, int32_t x, int32_t y, const char *ch, color fg, color bg);
void buf_set(uintptr_t b, int32_t x, int32_t y, const char *ch, color fg, color bg);
cell buf_get(uintptr_t b, uint32_t x, uint32_t y);
uid_handle create_network(uint16_t i, uintptr_t h);
bool open_port(uintptr_t n, uint16_t p);
void close_port(uintptr_t n, uint16_t p);
void close_all_ports(uintptr_t n);
void send_message(uintptr_t n, const char *ch, uint16_t p, const char *ch, size_t len);
void broadcast_message(uintptr_t n, uint16_t p, const char *ch, uint32_t len);
]]
local libDir = debug.getinfo(1).source:match("@?(.*/)")
--freen = ffi.load(libDir.."freen.dll")
local freen = ffi.load("G:\\Projekte\\Satisfactory\\Freen\\target\\i686-pc-windows-msvc\\debug\\freen.dll")

FREEN = {
	fontsize = 24,
	portStart = 10000
}

local SCREEN_CACHE = {}

local eventHandler = freen.new_event_handler()

function __parseUID(c_id)
	return ffi.string(c_id, 16)
end

local org_event_pull = event.pull
event.pull = function(n)
	local evt = {org_event_pull()}
	if (evt[1] ~= nil) then
		return unpack(evt)
	end
	n = n or 0.0
	evt = freen.pull(eventHandler, n)
	if (evt.e == nil) then
		return nil
	else
		local comp = nil
		if evt.c ~= nil then
			comp = component.proxy(__parseUID(evt.c))
		end
		if evt.x ~= nil then
			local data = ffi.string(evt.x, evt.a3)
			return ffi.string(evt.e), comp, "", evt.a1, data
		else
			return ffi.string(evt.e), comp, evt.a1, evt.a2, evt.a3
		end
	end
end

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
	if g ~= nil then
		r = {r, g, b, a}
	end
	freen.foreground(self._handle, r)
end

function FINComputerGPU:setBackground(r, g, b, a)
	if g ~= nil then
		r = {r, g, b, a}
	end
	freen.background(self._handle, r)
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
	freen.bind_screen(self._handle, screen._handle)
	self.screen = screen
end

-- Freen Exklusive Funktion
function FINComputerGPU:setLocation(x, y)
	freen.set_location(self.screen._handle, x, y)
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

function GPUT1Buffer:getSize()
	size = freen.buf_size(self._handle)
	return size.width, size.height
end

function GPUT1Buffer:fill(x, y, w, h, c, fg, bg)
	if type(fg) == 'number' then fg = {fg, fg, fg, fg} end
	if type(bg) == 'number' then bg = {bg, bg, bg, bg} end
	freen.buf_fill(self._handle, x, y, w, h, c, fg, bg)
end

function GPUT1Buffer:setText(x, y, txt, fg, bg)
	if type(fg) == 'number' then fg = {fg, fg, fg, fg} end
	if type(bg) == 'number' then bg = {bg, bg, bg, bg} end
	freen.buf_write(self._handle, x, y, txt, fg, bg)
end

function GPUT1Buffer:copy(x, y, buffer, txtbm, fgbm, bgbm)
	freen.buf_copy(self._handle, x, y, buffer._handle, txtbm, fgbm, bgbm)
end

function GPUT1Buffer:set(x, y, txt, fg, bg)
	if type(fg) == 'number' then fg = {fg, fg, fg, fg} end
	if type(bg) == 'number' then bg = {bg, bg, bg, bg} end
	freen.buf_set(self._handle, x, y, txt, fg, bg)
end

function GPUT1Buffer:get(x, y, buffer)
	local cell = freen.buf_get(self._handle, x, y)
	if cell == nil then return nil end
	return ffi.string(cell.ch, cell.l), cell.fg, cell.bg
end

function GPUT1Buffer:setRaw(x, y, c, fg, bg)
	freen.buf_copy(self._handle, x, y, c, fg, bg)
end

function GPUT1Buffer:clone()
	local clone = GPUT1Buffer:new()
	clone._handle = freen.buf_clone(self._handle)
	return clone
end

function FINComputerGPU:init()
	self._width=120
	self._height=40
	self.screen=nil
	self.fg = {r=1.0,g=1.0,b=1.0,a=1.0}
	self.bg = {r=0.0,g=0.0,b=0.0,a=1.0}
	self._handle = freen.graphic_handle(self._width, self._height)
end

local Freen = defineClass({
	aliase = {"Freen", "Screen", "Build_Screen_C", "FINComputerScreen"},
	displayName = "Freen Window"
}, function(p)
	local c = freen.create_screen(FREEN.fontsize, eventHandler)
	p.id = __parseUID(c.id) -- ffi.string(c.id, 16)
	p._handle = c.h
	table.insert(SCREEN_CACHE, p)
end)

function Freen:close()
	if (self._handle ~= nil) then
		freen.destroy_screen(self._handle)
		self._handle = nil
		
		for idx,s in pairs(SCREEN_CACHE) do
			if s == self then
				table.remove(SCREEN_CACHE, idx)
				return
			end
		end
	end
end

NetworkCard = defineClass({
	base = _Component,
	aliase = {"NetworkCard", "NetworkCard_C"},
	displayName = "NetworkCard"
}, function (p)
	local c = freen.create_network(FREEN.portStart, eventHandler)
	p.id = ffi.string(c.id, 16)
	p._handle = c.h
	addNetworkComponent(p)
end)

function NetworkCard:open(port)
	freen.open_port(self._handle, port)
end

function NetworkCard:close(port)
	freen.close_port(self._handle, port)
end

function NetworkCard:closeAll()
	freen.close_all_ports(self._handle)
end

function NetworkCard:send(rec, port, msg)
	if rec == nil then error("reciever is nil") end
	freen.send_message(self._handle, rec, port, msg, #msg)
end

function NetworkCard:broadcast(port, msg)
	freen.broadcast_message(self._handle, port, msg, #msg)
end
