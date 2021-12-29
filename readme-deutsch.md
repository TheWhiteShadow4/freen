Dieser Bibliothek erlaubt dir, Fenster Inhalte für die Satisfactory Mod Ficsit Network (FIN), außerhalb des Spiels in lua zu entwickeln und teilweise zu testen.

Um die Programme lokal auszuführen wird ein lua Interpreter mit FFI benötigt. z.B luaJIT

## Freen dll Bauen
Die Bibliothek kann über das Rust Werkzeug Cargo gebaut werden.
In der Regel handelt es sich bei Lua Interpretern um 32bit Versionen, was für den Bau von Freen berücksichtigt werden muss.

Beispiel mit rustup Toolchain:
```
cargo build --target=i686-pc-windows-msvc;
```

## Freen Einbinden
Für die integration werden zwei Includes benötigt:

```
require 'ficsit-api' --$DEV-ONLY$
require 'freen' --$DEV-ONLY$
```

Das Modul ficsit-api stellt einige Funktionen aus FIN zur verfügung.
Die meisten sind lediglich Mocks, die nichts tun und bisher nur zur Kompatiblität dienen.

Das Modul freen erweitert die API, indem es einige Grafik relevante Funktionen implementiert z.B. die GPU.
Diese greifen auf die native dll für die Anzeige zu.

Die Kommentare `--$DEV-ONLY$` kennzeichen die Zeilen als nur für die Entwicklung relevant.

## BuildSkript build.js
Das Repository verfügt über ein Node.js Buildskript für FIN Lua Programme.
Dieses integriert Module zu einer fertigen Datei, die entweder per Copy-Paste oder über die Dateisystem-API von FIN hochgeladen werden kann.

Um damit Module von außerhalb des Projekts zu laden, kann der Pfad im Attribut libs in der package.json angegeben oder über die Umgebungsvariable LUA_PATH gesetzte werden.