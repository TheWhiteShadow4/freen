# Freen

Dieses Paket erlaubt dir, Lua Programme für die Satisfactory Mod Ficsit Network (FIN), teilweise außerhalb des Spiels entwickeln und zu testen.

Aktuell enthält es folgende Komponenten:
- Ficsit-api.lua
	Enthält Mocks und Utilities zur Kompatiblität mit der FIN API.
- Freen
	Erweiterung mit nativem Support für die GPU und Netzwerk API.

## Features

- `findClass()`
- computer
- component
- event
- filesystem
- GPU und Buffer (mit freen)
- Netzwerk (mit freen)

Die globale Funktion `findClass()` enthält bereits einige implementierte Klassen.
Weitere können mit `defineClass()` zum Testen angelegt werden.

```lua
MyClass = defineClass({
	aliase = {"MyClass"},
	displayName = "My Class"
}, function(p)
	p.value = 5
end)
...
id = component.findComponent(findClass("MeineKlasse"))[1]
comp = component.proxy(id)
print(comp.value) --> 5
```
Komponenten können auch über `addNetworkComponent()` dem Netzwerk hinzugefügt werden und anschließend über ihre ID gefunden werden.
Dynamische Komponenten erhalten eine zufällige ID die von Start zu Start unterschidlich sein kann.

## Vorraussetzungen

- Lua Interpreter mit FFI benötigt. z.B luaJIT
- Node.js zum Erstellen der Bundle Dateien.
- Rust Compiler zum kompilieren von Freen.

## Freen dll Bauen

Für einige Releases wird eine fertig kompilierte Windows 32bit dll mitgeliefert.
Auf abweichenden Systemen muss diese allerdings gebaut werden.

Die Bibliothek kann über das Rust Werkzeug Cargo gebaut werden.
In der Regel handelt es sich bei Lua Interpretern um 32bit Versionen, was für den Bau von Freen berücksichtigt werden muss.

Beispiel mit rustup Toolchain:
```
cargo build --target=i686-pc-windows-msvc;
```

Die fertige dll muss in der freen.lua entsprechend verlinkt werden.

## BuildSkript build.js
Das Repository verfügt über ein Node.js Buildskript für FIN Lua Programme.
Dieses integriert Module zu einer fertigen Datei, die entweder per Copy-Paste oder über die Dateisystem-API von FIN hochgeladen werden kann.

Um damit Module von außerhalb des Projekts zu laden, kann der Pfad im Attribut libs in der package.json angegeben oder über die Umgebungsvariable LUA_PATH gesetzte werden.

### Freen Einbinden
Für die Integration werden zwei Includes benötigt:

```lua
require 'ficsit-api' --$DEV-ONLY$
require 'freen' --$DEV-ONLY$
```

Die Kommentare `--$DEV-ONLY$` kennzeichen die Zeilen als nur für die Entwicklung relevant.

### Freen Einstellungen

Freen kann über das globale Objekt FREEN konfiguriert werden.

- fontsize: Schriftgröße für Freen Fenster.
- portStart: Port Mapping Offset bei Netzwerkkarten.

## Unterschiede zu FIN

Trotz größter Mühen des Entwicklers eine identische API zu realisieren, gibt es einige unvermeidbare Abweichungen.

Da die API in Lua geschrieben ist, weist sie einige Unterschiede auf.
Beispielsweise ist das Reflektion System nur sehr rudimentär umgesetzt.

Um eingie Funktionalitäten zu unterstützen, werden teilweise auf Funktionen des Betreibssystems zurückgegriffen, die sich anderes Verhalten.
Dies ist besonders bei der Dateisystem und Netzwerk Implementierung der Fall.

Teilweise sind Abweichungen auch gewollt so.
Statt wie im Spiel auf ein komplettes Komponenten Netzwerk zurückzugreifen, werden hier Komponenten dynamisch erzeugt.
Oft handelt es sich hierbei um Mocks, die anstelle von nil, halbwegs sinnvolle Default Werte zurück liefern sollen.
