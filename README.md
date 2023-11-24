# Overview

This is a very simple client-server game just to get myself familiar with a few technologies. The game is multi-player
where one player controls a hoop, and one or more others can shoot balls. The objective of the ball shooters is to
get their balls through the hoop, and the hoop controller is to nope them by moving the hoop out of the way.

# Server

The server is a simple tokio server listening on a TCP port. I'll add a very simple pass phrase (hashed) auth where
every client that knows the pass phrase for the server (passed on the command-line) can connect. The state of the game
is kept in-memory. A client can either request a new game or connect to an existing game.

# Client

The client is a bevy 2D game. It gets its role (hoop or ball) from the server, then the player controls that and passes
messages to the server with player inputs (moved the hoop, shot the ball) and gets messages with updates to the state.

# Proto

The protocol is a simple CBOR protocol (the easiest binary protocol I found).
