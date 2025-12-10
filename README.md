# multisnake

> **A multiplayer implementation of the classic Snake game, written in Rust.**

The game uses a client-server architecture to let multiple players compete on a shared grid.

## Prerequisites

To run this project, you need **Rust** and **Cargo** installed.

## Running

### 1\. Start the server

Open a terminal and run the server first:

```bash
cargo run -p multisnake_server
```

### 2\. Start a client

Open a **new** terminal window (do not close the server) and run a client:

```bash
cargo run -p multisnake_client
```

To add more players, simply open additional terminal windows and run the client command again.

## ⚠️ Important: spawn issue

**Please read before adding a second player:**

Currently, all snakes spawn in the **middle row**.

> **You must move your snake away from the middle row before another client joins in.

If a new client joins while an existing snake is still occupying the spawn point (the middle row), the game state may become invalid or crash. This is one of the points in the TODO section.


## DONE
  - [x] **Server authoritative snake-to-snake collisions**
  - [x] **Server authoritative food eating and growth**: Snakes consume food items, triggering immediate growth.
  - [x] **Random food spawning**: Food items are spawned randomly across the grid when previous food has been eaten.

## TODO
  - [ ] **Server-authoritative snake movement**: Implement game state updates related to snake movement to be fully server-authoritative, to prevent client cheating and ensure the game simulation runs independently of individual client network delays or messaging frequency.
  - [ ] **Server authoritative snake-to-wall collision**s.
  - [ ] **Lobby TUI:** Add a Terminal User Interface for joining rooms/lobbies.
  - [ ] **Spawn overlap:** Fix the issue where spawning a new snake on top of an existing one invalidates the game state.
  - [ ] **Error handling:** Replace `unwrap()` calls with proper error propagation for resistance to incorrect client inputs.
  - [ ] **Serialization redundancy:** Refactor message sending to use dedicated serialization/deserialization functions to reduce code redundancy.
  - [ ] **Concurrency:** Optimize mutex usage to prevent thread blocking and improve performance.
  - [ ] **Cosmetics:** Support for different snake colors and smoother animations.
  - [ ] **Remove unused dependencies**.

## Tools used include
- **Tokio** for concurrency.
- **WebSockets**  for real-time communication.
- **macroquad** for simple 2D graphics rendering.

## What will be used
- **ratatui/crossterm** for TUI.
