# multisnake

**A multiplayer implementation of the classic Snake game, written in Rust.**

The game uses a client-server architecture to let multiple players compete on a shared grid.

## Prerequisites

To run this project, you need **Rust** and **Cargo** installed.

## Running

### 1\. Start the server

Open a terminal and run the server first:

```bash
cargo run -p multisnake_server
```
you can also pass address and tickrate:
```bash
cargo run -p multisnake_server 127.0.0.1:4040 100
```


### 2\. Start a client

Open a **new** terminal window (do not close the server) and run a client:

```bash
cargo run -p multisnake_client
```
you can also pass server address:
```bash
cargo run -p multisnake_client 127.0.0.1:4040
```

## Notes
- Around first 5 second after spawning the snake is a ghost and is yellow during that. This means it can't eat food and doesn't collide with other snakes.


## DONE
- [x] **Server authoritative snake-to-snake collisions**
- [x] **Server authoritative food eating and growth**: Snakes consume food items, triggering immediate growth.
- [x] **Random food spawning**: Food items are spawned randomly across the grid when previous food has been eaten.
- [x] **Server-authoritative snake movement**: Implement game state updates related to snake movement to be fully server-authoritative, to prevent client cheating and ensure the game simulation runs independently of individual client network delays or messaging frequency.
- [x] **Server authoritative snake-to-wall collision**s.
- [x] **Lobby TUI:** Add a Terminal User Interface for joining rooms/lobbies.
- [x] **Spawn overlap:** Fix the issue where spawning a new snake on top of an existing one invalidates the game state.
- [x] **Cosmetics:** Support for different snake colors and smoother animations.
## TODO
- [ ] **Error handling:** Replace `unwrap()` calls with proper error propagation for resistance to incorrect client inputs.
- [ ] **Serialization redundancy:** Refactor message sending to use dedicated serialization/deserialization functions to reduce code redundancy.
- [ ] **Concurrency:** Optimize mutex usage to prevent thread blocking and improve performance.
- [ ] **Remove unused dependencies**.

## Tools used include
- **Tokio** for concurrency.
- **WebSockets**  for real-time communication.
- **macroquad** for simple 2D graphics rendering.
- **ratatui/crossterm** for TUI.

