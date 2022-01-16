ffi = require("ffi")

ffi.cdef[[
typedef struct { const char id[16]; uintptr_t h; } uid_handle;
typedef struct { float r, g, b, a; } color;
typedef struct { uint32_t width, height; } size;
typedef struct { const char ch[4]; size_t l; color fg, bg; } cell;
typedef struct { const char *val; size_t len; bool num; } param;
typedef struct { const char *val; size_t len; } array;
typedef struct { const char *e; const char cmp[16]; size_t len; param p[8]; } signal;
uintptr_t new_event_handler();
uintptr_t graphic_handle(uint32_t w, uint32_t h);
void bind_screen(uintptr_t g, uintptr_t s);
signal pull(uintptr_t g, float t);
uid_handle create_screen(uint32_t f, uintptr_t h);
void destroy_screen(uintptr_t s);
void set_size(uintptr_t g, uint32_t w, uint32_t h);
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
uid_handle create_network(uint16_t i, size_t b, uintptr_t h);
bool open_port(uintptr_t n, uint16_t p);
void close_port(uintptr_t n, uint16_t p);
void close_all_ports(uintptr_t n);
void send_message(uintptr_t n, const char *ch, uint16_t p, param[?], size_t len);
void broadcast_message(uintptr_t n, uint16_t p, param[?], size_t len);
uintptr_t create_filesystem(const char *p, const char *n, uintptr_t h);
bool mount(uintptr_t fs, const char *d, const char *m);
bool unmount(uintptr_t fs, const char *d);
bool fs_exists(uintptr_t fs, const char *f);
bool fs_is_file(uintptr_t fs, const char *f);
bool fs_is_dir(uintptr_t fs, const char *f);
bool fs_remove(uintptr_t fs, const char *f);
bool fs_rename(uintptr_t fs, const char *f, const char *t);
bool fs_create_dir(uintptr_t fs, const char *f);
array fs_childs(uintptr_t fs, const char *f);
]]
local libDir = debug.getinfo(1).source:match("@?(.*\\)")
--freen = ffi.load(libDir.."freen.dll")
local freen = ffi.load("G:\\Satisfactory\\Freen\\target\\i686-pc-windows-msvc\\debug\\freen.dll")

--[[
Freen Konfigurations Objekt
--]]
FREEN = {
	fontsize = 24,
	portStart = 10000,
	maxNetworkArgs = 7,
	networkBuffer = bit.lshift(1, 16), --64kb
}

--- FFI Datantyp zur Übergabe von generischen Parametern.
local param = ffi.metatype("param", {})
--- Cache für offene Freen Displays.
local SCREEN_CACHE = {}
--- Der Eventhandler verarbeitet Signale von nativen Freen Componenten.
local eventHandler = freen.new_event_handler()

--- Konvertiert eine interne Komponenten ID
function __parseUID(c_id)
	return ffi.string(c_id, 16)
end

local function extract_signal_params(sig)
	local args = {}
	for i = 0,(sig.len-1) do
		local p = sig.p[i]
		local v = ffi.string(p.val, p.len)
		if p.num then v = tonumber(v) end
		table.insert(args, v)
	end
	return args
end

local org_event_pull = event.pull
event.pull = function(n)
	local sig = {org_event_pull()}
	if (sig[1] ~= nil) then
		return unpack(sig)
	end
	n = n or 0.0
	sig = freen.pull(eventHandler, n)
	if (sig.e == nil) then
		return nil
	else
		local args = extract_signal_params(sig)
		local comp = nil
		if sig.cmp ~= nil then
			comp = component.proxy(__parseUID(sig.cmp))
		end
		return ffi.string(sig.e), comp, table.unpack(args)
	end
end

--- Schließt alle Fenster
function FREEN:close()
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
	local buffer = GPUT1Buffer:new({})
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
	p.id = __parseUID(c.id)
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

--[[
Implementierung einer Netzwerkkarte mit nativem UDP Sockets.
Erlaubt den Transport von Daten zwischen verschiedenen Prozessen.
Es erfolgt ein Offset Mappings zwischen Freen Ports und der nativen Socket Ports.
Die Größe der Datenpakete ist standardmäßig auf 64kb limitiert. 

Aktuell können Daten ausschließlich als Strings übertragen werden.
--]]
NetworkCard = defineClass({
	base = _Component,
	aliase = {"NetworkCard_C", "NetworkCard"},
	displayName = "NetworkCard"
}, function (p)
	local c = freen.create_network(FREEN.portStart, FREEN.networkBuffer, eventHandler)
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

local function network_data(args)
	if #args > FREEN.maxNetworkArgs then error("Too many arguments", 3) end
	local array = ffi.new("param[?]", #args)
	for i,a in pairs(args) do
		-- TODO: Alle Argumente werden momentan von freen als String gesendet.
		--local n = type(a) == 'number'
		a = tostring(a)
		array[i-1] = param(a, #a, false)
	end
	return array
end

function NetworkCard:send(rec, port, ...)
	--print("send", ...)
	if rec == nil then error("reciever is nil") end
	
	local args = {...}
	local array = network_data(args)
	freen.send_message(self._handle, rec, tonumber(port), array, #args)
end

function NetworkCard:broadcast(port, ...)
	--print("broadcast", ...)
	local args = {...}
	local array = network_data(args)
	freen.broadcast_message(self._handle, tonumber(port), array, #args)
end

local freen_fs = nil
local function check_fs()
	if freen_fs == nil then error("Filesystem not initialized.", 3) end
end

filesystem.initFileSystem = function(path)
	if path == "" then error("Empty device is not allowed.", 2) end
	freen_fs = freen.create_filesystem(FS_ROOT, path, eventHandler)
end

filesystem.mount = function(device, mountPoint)
	check_fs()
	return freen.mount(freen_fs, device, mountPoint)
end

filesystem.unmount = function(device)
	check_fs()
	return freen.unmount(freen_fs, device)
end

filesystem.exists = function(file)
	check_fs()
	return freen.fs_exists(freen_fs, file)
end

filesystem.isFile = function(file)
	check_fs()
	return freen.fs_is_file(freen_fs, file)
end

filesystem.isDir = function(dir)
	check_fs()
	return freen.fs_is_dir(freen_fs, dir)
end

filesystem.createDir = function(dir)
	check_fs()
	return freen.fs_create_dir(freen_fs, dir)
end

filesystem.rename = function(from, to)
	check_fs()
	return freen.fs_rename(freen_fs, from, to)
end

filesystem.remove = function(file)
	check_fs()
	return freen.fs_remove(freen_fs, file)
end