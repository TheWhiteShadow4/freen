
require 'ficsit-api'
require 'utils_0-1'
require 'freen'

lu = require 'tests/luaunit'


Test_FIN_API = {} --class

function Test_FIN_API:TestClasses()
	-- Erstelle eine Testklasse mit der API
	local testComponent = defineClass({
		base = _Component,
		aliase = {"TestComponent"}
	}, function(p)
		p.flag = true
	end)

	local cls = findClass("TestComponent")
	-- Die eigentliche Klasse wird nicht von defineClass zurückgegeben,
	lu.assertNotEquals(cls, testComponent)
	-- enthält diese aber als Typ.
	lu.assertEquals(cls, testComponent.getType())
	
	local array = computer.getPCIDevices(cls)
	
	-- array ist ein Lazy Array, daher noch leer.
	lu.assertNil(array[nil])
	lu.assertNil(array["1"])
	lu.assertEquals(#array, 0)
	-- Zugriff auf ein Index führt zur Instanzierung.
	local inst = array[1]
	lu.assertNotNil(inst, nil)
	lu.assertEquals(#array, 1)
	-- Die neue Instanz wird im Array gespeichert und ändert sich nicht.
	lu.assertEquals(array[1], inst)
	-- Der Konstuktor wurde aufgerufen. Unser flag ist true.
	lu.assertTrue(inst.flag)
	-- Da die Klasse von Component erbt, hat sie eine ID.
	lu.assertNotNil(inst.id)

	local inst2 = computer.getPCIDevices(cls)[1]
	-- Die neue Instanz ist keine Kopie von inst.
	lu.assertNotEquals(inst2, inst)
	-- Der Hash unterscheidet sich nicht.
	lu.assertNumber(inst.hash)
	lu.assertEquals(inst2.hash, inst.hash)
end

function Test_FIN_API:TestComponents()
	-- Erstelle eine Komponente aus der Testklasse
	local uid = component.findComponent(findClass("TestComponent"))[1]
	-- Hier wird ein String zurückgegeben.
	lu.assertString(uid)
	-- mit der ID lässt sich die Komponente finden
	local comp = component.proxy(uid)
	lu.assertTable(comp)
	lu.assertString(comp.id)
	-- Die Komponente befindet sich nun im Netzwerk
	local c2 = Network[uid]
	lu.assertEquals(c2, comp)
	
	-- Die API erlaubt Komponenten aus beliebige Namen zu generieren.
	local ids = component.findComponent("Blub")
	local blub = ids[1]
	local blob = ids[2]
	lu.assertString(blub)
	lu.assertString(blob)
	lu.assertNotEquals(blub, blob)
	-- Proxy akzeptiert ein Array und erstellt entsprechend viele Komponenten
	local proxies = component.proxy({blub, blob})
	lu.assertEquals(proxies[1].id, blub)
	lu.assertEquals(proxies[2].id, blob)
	-- Alle komponenten haben ein Set an Funktionen
	lu.assertFunction(proxies[1].getPowerConnectors)
	lu.assertFunction(proxies[1].getFactoryConnectors)
	lu.assertFunction(proxies[1].getPipeConnectors)
	lu.assertFunction(proxies[1].getInventories)
	lu.assertFunction(proxies[1].getNetworkConnectors)
	-- component.proxy erlaubt auch Lazy Arrays
	comp = component.proxy(component.findComponent("Blub"))[1]
	lu.assertNotNil(comp, nil)
end

function Test_FIN_API:TestGPU()
	-- GPU
	local cls = findClass("GPUT1")
	lu.assertEquals(cls.name, "GPU_T1_C")
	local gpu = computer.getPCIDevices(cls)[1]
	local w, h = gpu:getSize()
	lu.assertEquals(w, 120)
	lu.assertEquals(h, 40)
	-- Die Größe des Buffers entspricht der, des GPU Objekts.
	gpu:setSize(100, 30)
	local buf = gpu:getBuffer()
	local bw, bh = buf:getSize()
	lu.assertEquals(bw, 100)
	-- Large Screen
	cls = findClass("Screen")
	lu.assertEquals(cls.name, "Freen") -- Ohne Freen: Build_Screen_C
end

function Test_FIN_API:TestGPU2()
	local gpu = computer.getPCIDevices(findClass("GPUT1"))[1]
	local screen = computer.getPCIDevices(findClass("Screen"))[1]
	gpu:bindScreen(screen)
	local w,h = gpu:getSize()

	-- Aufgrund der asynchroner Initialisierung des Fensters kann es sein,
	-- dass der Buffer schon vor flush gelesen wird.
	-- Dieses Fall ist normalerweise tollerierbar.
	gpu:fill(0, 0, w, h, ' ')
	gpu:setForeground(1, 0.5, 0, 1)
	gpu:setText(0, 0, "♥Hallo♥")
	gpu:flush()
	event.pull(0.5)
	
	local buffer = gpu:getBuffer()
	local t,f,b = buffer:get(0, 0)
	lu.assertEquals(t, "♥")
	lu.assertEquals(f.r, 1)
	lu.assertEquals(f.g, 0.5)
	lu.assertEquals(f.b, 0)
	buffer:set(0, 0, "♦", 1, 0)
	gpu:setBuffer(buffer)
	gpu:flush()
	event.pull(0.1)
	--screen:close()
end

function Test_FIN_API:TestEvents()
	local comp = component.proxy(component.findComponent("Blub")[1])
	-- Wir überschreiben den Dummy mit dem Event Handler
	event.listen(comp)
	-- Event manuell auslösen
	comp:_fire("test", 42)
	-- Das Event kann nun ausgelesen werden
	local e,c,a = event.pull()
	lu.assertEquals(e, "test")
	lu.assertEquals(c, comp)
	lu.assertEquals(a, 42)
	-- Der zweite Aufruf gib nil zurück.
	e,c,a = event.pull()
	lu.assertNil(e)
	-- Ignorierte komponenten feuern nicht.
	event.ignore(comp)
	comp:_fire("test", 42)
	e,c,a = event.pull()
	lu.assertEquals(e, nil)
end

function Test_FIN_API:TestFilesystem()
	local dev1 = "4D92F19549A229990C9F5CBEFC69D414"
	local dev2 = "C963C1914AAB87A8558D4C9E324CCC99"
	
	lu.assertError(filesystem.initFileSystem, "")
	
	filesystem.initFileSystem("/dev")
	lu.assertFalse(filesystem.mount(dev1, "/"))
	lu.assertFalse(filesystem.mount("/dev"..dev1, "/"))
	lu.assertTrue(filesystem.mount("dev/"..dev1, "/"))
	-- Korrekte Syntax aber bereits gemaunted
	lu.assertFalse(filesystem.mount("/dev/"..dev1, "/"))
	-- Bereits verwendeter Mounpoint
	lu.assertFalse(filesystem.mount("/dev/"..dev2, "/"))
	lu.assertTrue(filesystem.mount("dev/"..dev2, "/u"))
	
	lu.assertTrue(filesystem.exists("/u/test.dat"))
	lu.assertTrue(filesystem.exists("/test.txt"))
	
	lu.assertTrue(filesystem.unmount("dev/"..dev2))
	
	lu.assertTrue(filesystem.isFile("test.txt"))
	-- Nicht mehr gemounted
	lu.assertFalse(filesystem.isFile("/u/test.txt"))
	lu.assertFalse(filesystem.isDir("/test.txt"))
	
	lu.assertTrue(filesystem.createDir("/ordner2"))
	lu.assertTrue(filesystem.rename("/ordner2", "/ordner3"))
	lu.assertTrue(filesystem.isDir("/ordner3"))
	lu.assertTrue(filesystem.remove("/ordner3"))
	local e = event.pull()
	lu.assertEquals(e, "FileSystemUpdate")
end

function Test_FIN_API:TestNetwork()
	local cards = component.proxy(component.findComponent(findClass("NetworkCard")))
	local sender = cards[1]
	local reciever = cards[3] -- mal was neues^^
	local port = 1
	reciever:open(port)
	-- Reciever muss angegeben werden.
	lu.assertError(sender.send, nil, port, "Hallo 1")
	-- Port darf nicht negativ sein.
	lu.assertError(sender.send, "", -1, "Hallo 1")
	-- Freen interessiert der Empänger nicht, wir geben ihn trotzdem an.
	sender:send(reciever.id, port, "Hallo 2", "M\02")
	local e,c,s,p,m1,m2 = event.pull(0.1)
	lu.assertEquals(e, "NetworkMessage")
	lu.assertEquals(c, reciever)
	lu.assertEquals(s, sender.id)
	lu.assertEquals(p, port)
	lu.assertEquals(m1, "Hallo 2")
	lu.assertEquals(m2, "M\02")
	reciever:close(port)
end

local runner = lu.LuaUnit.new()
os.exit( runner:runSuite() )