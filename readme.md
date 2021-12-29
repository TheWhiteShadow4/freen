This library allows you to develop and partially test window content for the Satisfactory Mod Ficsit Network (FIN) outside of the game in lua.

A lua interpreter with FFI is required to run the programs locally. e.g. luaJIT

## Build Freen dll
The library can be built using the Rust tool Cargo.
As a rule, Lua interpreters are 32-bit versions, which must be taken into account for the construction of Freen.

Example with rustup toolchain:
``
cargo build --target = i686-pc-windows-msvc;
``

## Include Freen
Two includes are required for the integration:

``
require 'ficsit-api' - $ DEV-ONLY $
require 'freen' - $ DEV-ONLY $
``

The module ficsit-api provides some functions from FIN.
Most of them are just mocks that do nothing and so far only serve for compatibility.

The freen module extends the API by implementing some graphics-relevant functions, e.g. the GPU.
These access the native dll for display.

The comments `- $ DEV-ONLY $` mark the lines as relevant for development only.

## BuildScript build.js
The repository has a Node.js build script for FIN Lua programs.
This integrates modules into a finished file that can be uploaded either by copy-paste or via the file system API from FIN.

In order to load modules from outside the project, the path can be specified in the libs attribute in the package.json or set using the LUA_PATH environment variable.