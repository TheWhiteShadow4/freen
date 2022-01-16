# Freen

This package allows you to develop and test Lua programs for the Satisfactory Mod Ficsit Network (FIN), partly outside of the game.

It currently contains the following components:
- Ficsit-api.lua
Contains mocks and utilities for compatibility with the FIN API.
- Freen
Extension with native support for the GPU and network API.

## Features

- `findClass ()`
- computer
- component
- event
- filesystem
- GPU and buffer (with freen)
- network (with freen)

The global function `findClass ()` already contains some implemented classes.
Others can be created for testing with `defineClass () '.

```lua
MyClass = defineClass ({
aliase = {"MyClass"},
displayName = "My Class"
}, function (p)
p.value = 5
end)
...
id = component.findComponent (findClass ("MyClass")) [1]
comp = component.proxy (id)
print (comp.value) -> 5
```
Components can also be added to the network via `addNetworkComponent ()` and then found via their ID.
Dynamic components are given a random ID which can be different from start to start.

## Requirements

- Lua interpreter with FFI required. e.g. luaJIT
- Node.js to create the bundle files.
- Rust compiler to compile Freen.

## Build Freen dll

For some releases a fully compiled Windows 32bit dll is included.
However, this must be built on different systems.

The library can be built using the Rust tool Cargo.
As a rule, Lua interpreters are 32-bit versions, which must be taken into account for the construction of Freen.

Example with rustup toolchain:
```
cargo build --target = i686-pc-windows-msvc;
```

The finished dll must be linked accordingly in the freen.lua.

## BuildScript build.js
The repository has a Node.js build script for FIN Lua programs.
This integrates modules into a finished file that can be uploaded either by copy-paste or via the file system API from FIN.

In order to load modules from outside the project, the path can be specified in the libs attribute in the package.json or set using the LUA_PATH environment variable.

### Integrate Freen
Two includes are required for the integration:

```lua
require 'ficsit-api' - $ DEV-ONLY $
require 'freen' - $ DEV-ONLY $
```

The comments `- $ DEV-ONLY $` mark the lines as relevant for development only.

### Freen settings

Freen can be configured via the global object FREEN.

- fontsize: Font size for Freen window.
- portStart: Port mapping offset for network cards.

## Differences to FIN

Despite the great efforts of the developer to implement an identical API, there are some unavoidable deviations.

Because the API is written in Lua, it has a few differences.
For example, the reflection system is only implemented in a very rudimentary way.

In order to support some functionalities, functions of the operating system are sometimes used that behave differently.
This is particularly the case with the file system and network implementation.

In some cases, deviations are intentional.
Instead of using a complete network of components as in the game, components are generated dynamically here.
Often these are mocks that are supposed to return halfway meaningful default values ​​instead of nil.

