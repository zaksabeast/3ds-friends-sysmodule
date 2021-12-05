# 3DS Friends Sysmodule Replacement

This is an open source implementation of the 3ds friends system module written in Rust. The friends sysmodule is responsible for friends list CRUD operations, online play for the friends list, and helping facilitate the authentication of online play and server location for games.

This project is in its early stages and needs refactoring from new lessons learned.

The goals of this project are to:

- Learn more about no_std rust
- Learn more about operating systems
- Provide benefits for developers and users when the official servers go down (see the below section for more details)

Since this project is meant to modify the functionality of the official sysmodule and is written in a different language than the original, implementing a 1:1 version of the sysmodule is nonsensical, but keeping the same public API is necessary for compatibility with games.

## What benefits does this provide?

This sysmodule is currently a base implementation that implements most of the sysmodule's functionality or stubs the functionality with a valid response. For example, usres can view their existing friends and go online with games. The intention is for this base to be built upon in the future.

Here are examples of things that can be done in the future with this base, but are not necessarily goals of the project:

1. Replace nex with something standard for the friends server
   - Something with existing cross-language support
   - Something easier to implement/use
   - Something cheaper to host with cloud services than a 24/hour server
2. Use an updated version of TLS
   - This makes the console safer
   - Some cloud providers have cheaper services for newer TLS versions
3. Remove the reuse of a single online password between games
4. New extensions for homebrew and game patches
5. Provide a way for people to explore rust on the 3ds
6. Add fake friends to help unlock in-game bonuses (e.g. XY Friend Safari)
7. Discord integration
8. Add "profiles" for multiple friends lists and accounts

Items 1-3 can help developers of alternate game and friend servers once the official servers go down.

NEX is a custom websocket-like protocol built upon UDP. Replacing the friend nex setup with something more standard can make friend server development easier, cheaper to host, and able to use battle-tested technologies for a more stable result. An updated version of TLS can not only reduce costs, but make the alternate servers safer than the official servers.

Having each game use different passwords for online play will allow developers to independently implement game servers and not have access to user's accounts on other alternate servers.

Items 4-5 can help developers learn and do things with homebrew that are not currently possible. Since a reimplemented sysmodule means we're in 100% control of the server and client implementations, we can add new client and server side functionality for homebrew and game patches.

Items 6-8 are more user facing ideas that could be enjoyable.

## What is not implemented?

Since part of the project's goal is to provide a way to make friend servers without nex and the 3ds servers will probably go down in the near future, none of the online functionality has been reimplemented. This includes signalling events related to friends by extension.

All friend related CRUD operations have not been implemented to avoid creating data conflicts with official servers.

This sysmodule can handle notifications from other sysmodules, but does not currently send notifications.

## Building

Prerequisties:

- Rust toolchain
- clippy
- make
- devkitARM
- libctru

Build commands:

- `make debug` builds the debug build
- `make release` builds the release build
- `make test` builds and runs tests
- `make lint` runs the linter
- `make` builds debug, release, and tests

Debug build notes:

- Logs are saved to `/frd-rs.txt` in debug builds
- The console does not sleep in debug builds due to a conflict with logging

## Installing

Copy `0004013000003202` to `luma/titles/0004013000003202`.

## Arm vs Thumb

While the sysmodule should probably be a thumb build, there are a few reasons the build is only arm:

- Rust cannot currently have a split thumb and arm build
- During development, there have been a few issues with rust dependencies and thumb builds

With the hurdles that have popped up, on and off, it's currently a pure arm build, but that may change in the future.

## Credits

Thanks to these projects, teams, and individuals for being great resources:

- [Luma3DS](https://github.com/LumaTeam/Luma3DS) and [Wumiibo](https://github.com/hax0kartik/wumiibo) for being great references (build processes, double check my reverse engineering, etc.)
- [libctru](https://github.com/devkitPro/libctru/) for being a great reference and providing an easy way to make open source hombrew
- [3dbrew](https://www.3dbrew.org/) for documentation about how different parts of the 3ds works
- [The rust3ds team](https://github.com/rust3ds) for the 3ds.json, initial ctru_sys, and code references to help get rust working on the 3ds
- [devkitPro](https://github.com/devkitPro/) for their toolchain
- Kinnay for their [online play documentation](https://github.com/kinnay/NintendoClients/wiki)
- All 3ds researchers
