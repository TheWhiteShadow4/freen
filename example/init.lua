--- Freen Matrix Simulation

-- For running in IDE from project root. set LUA_PATH should solve it too.
package.path = package.path..';example/?.lua' --$DEV-ONLY$

require 'ficsit-api' --$DEV-ONLY$
require 'freen' --$DEV-ONLY$
require 'matrix'

local gpu = computer.getPCIDevices(findClass("GPUT1"))[1]
if not gpu then
	error("No GPU T1 found!")
end

-- Initializes a screen. The freen module creates a native component here.
local screen = computer.getPCIDevices(findClass("FINComputerScreen"))[1]
if not screen then
	-- Alternative, falls kein Large Screen angeschlossen ist.
	local comp = component.findComponent(findClass("Screen"))[1]
	if not comp then
		error("No Screen found!")
	end
	screen = component.proxy(comp)

end
-- On this point the window is created.
gpu:bindScreen(screen)

gpu:setSize(120, 40) -- This is also the default size.
w,h = gpu:getSize()

matrix = Matrix:new(gpu, {0, 1, 0, 1})

gpu:setBackground(0, 0, 0, 0)
gpu:setForeground(0, 1, 0, 1) 
gpu:fill(0,0,w,h," ")

while true do
	e = event.pull(0.02)
	-- Ends the loop when the window is closed. Only valid for freen.
	if e == "WindowClosed" then break end --$DEV-ONLY$
	matrix:tick()
	matrix:render()
	gpu:flush()
end