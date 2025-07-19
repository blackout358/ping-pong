# Ping Pong

## Motivation

I was curious how client-server multiplayer games worked and I figured I would create my own. There are some unfinished parts like handling user disconnections, and the game does not have a score cap.

## How-To

When you run the server, it binds and listens to a tcp port.

```
cargo run --release
```

To connect to the server, the client define the servers address at the top of the `main.rs` file. After which you can run the client. You are able to specify a username when running

```
cargo run --release -- {Username}
```

## Not implemented

- Checking if player is still connected when creating game
- Cleanup when player leaves during the game
- Leaderboard of some such to make the usernames meaningful
- A score cap for the games, currently the game never ends
