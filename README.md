# rts.rs - An oldschool RTS game prototype

> Note: Requires rust >=1.19.0 release due to use of `HashMap::retain`.

## Quick start

### Singleplayer

```
cd game
cargo run 
```

Use `WASD` to scroll the map and `LeftMouse` to "chuck" down trees or rocks.

### Multiplayer

__Host__

```
cd game
cargo run -- $PORT -m $MIN_PLAYERS_TO_START
```

__Client(s)__

```
cd game
cargo run -- $PORT --addr $HOST_ADDRESS
```

## Current Features

- Lockstepped, peer-to-peer network protocol with automatic host migration
- gfx-rs based tiled map rendering based on `.tsm` and `.tsx` files
- Support for terrain types and "reflow" (basically you can chuck down trees and the surrounding tiles adjust correctly)


## Next Steps

### Game 

- Build a sprite renderer
- Initial unit abstraction
    - Selection
    - Command Queue

- Start work on path finding
- Concept for fog of war
- Concept for encapuslating command inputs for network transmission
- Concept for interpolating local positions between ticks and rendererd frames

    - Keep in mind that multiple ticks might be executed at once in order to compensate for high network RTT

### Clockwork

- Fix rare, random local time out when migrating and starting a new host locally
- Implement state checksum and comparison to abort de-synced games


## License

Licensed under either of
 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.


### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be dual licensed as above, without any
additional terms or conditions.

