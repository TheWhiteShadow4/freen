This library allows you to develop and partially test window contents for the satisfactory Mod Ficsit Network (FIN) outside of the game in lua.

A lua interpreter with FFI is required to run the programs locally. e.g. luaJIT

## Build Freen
The library can be built using the Rust tool Cargo.
As a rule, Lua interpreters are 32-bit versions, which must be taken into account when building Freen.

Example with rustup toolchain:
``
Cargo-Build --target = i686-pc-windows-msvc;
``

## Include Freen
Two items are required for the integration:

``
'ficsit-api' required - $ DEV-ONLY $
require 'freen' - $ DEV-ONLY $
``

The module ficsit-api provides some functions from FIN.
Most are just mocks that do nothing and only exist for compatibility.

The freen module extends the API by implementing some graphics-relevant functions, e.g. the GPU.

The `- $ DEV-ONLY $` comments mark the lines as relevant to development only.

## BuildScript build.js
The repository has a build script for FIN Lua programs.
This integrated module turns into a finished file that can be uploaded either by copy-paste or via the FIN file system API.