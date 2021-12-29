--- Freen Matrix Simulation

require 'ficsit-api' --$DEV-ONLY$
require 'freen' --$DEV-ONLY$
require 'matrix'

local gpu = computer.getPCIDevices(findClass("GPUT1"))[1]
if not gpu then
	error("No GPU T1 found!")
end
-- Initialisiert ein Screen. Durch das freen Modul wird hier ein Freen Fenster erstellt.
local screen = computer.getPCIDevices(findClass("FINComputerScreen"))[1]
if not screen then
	-- Alternative, falls kein Large Screen angeschlossen ist.
	local comp = component.findComponent(findClass("Screen"))[1]
	if not comp then
		error("No Screen found!")
	end
	screen = component.proxy(comp)

end
gpu:bindScreen(screen)
-- Freen erwartet, dass vor dem Zeichen eine Größe festgelegt wird.
gpu:setSize(120, 40)
w,h = gpu:getSize()

matrix = Matrix:new(gpu)

gpu:setBackground(0, 0, 0, 0)
gpu:setForeground(0, 1, 0, 1) 
gpu:fill(0,0,w,h," ")

while true do
	event.pull(0.02)
	matrix:tick()
	matrix:render()
	gpu:flush()
end