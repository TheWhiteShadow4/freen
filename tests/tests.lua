
require 'ficsit-api'
require 'utils_0-1'
--require 'freen'

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
	-- Die eigentliche Klasse wird nicht von defineClass zurückgegeben.
	lu.assertNotEquals(cls, testComponent)
	-- Enthält diese aber als Typ
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
	-- Der Konstuktor wurde aufgerufen.
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
	--proxies[1]:getPowerConnectors()
	
	--print(dump(proxies[1]:getPowerConnectors()))
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
	lu.assertEquals(cls.name, "Build_Screen_C")
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
	
end

local runner = lu.LuaUnit.new()
os.exit( runner:runSuite() )