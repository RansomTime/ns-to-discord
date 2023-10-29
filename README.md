# ns-to-discord

Simple rust program that displays mmol/L values in discord's Rich presence.

Based off https://github.com/legoandmars/nightscout-discord-rich-presence - I have a fork here: https://github.com/RansomTime/nightscout-discord-rich-presence with a few dependency bumps, but I ended up getting into dependency hell with it and made this rust version instead.

You may be more interested in the above as it has a config file and so you don't need to compile it from source

## Build instructions

Currently you'll have to compile and build yourself. I'm lazy and haven't built in a config file.

1. Install rust https://www.rust-lang.org/tools/install
2. Clone repo 
3. Change ENDPOINT in main.rs to 
4. Change TOKEN in main.rs to Some("Your token") or None if your nightscout is visable to all.
5. Cargo run --release to run

Doesn't like talking to containerised versions of Discord on linux, but works fine with the .deb

In order to show your status on servers you have

Games you're playing seem to take precidence over this.

## Development

PRs are welcome, this is a pretty bare bones system.

To start developing see the build instructions above.

You're welcome to keep my secrets in the code, as my nightscout is public. At some point I might write a config file.