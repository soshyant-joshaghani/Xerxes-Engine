# Bound Bros: Two prisoners, one chain

Darius (2D) · **Coop x Objective** · Coming soon (empty scene)

| | |
|---|---|
| Name | Two prisoners shackled together, escaping the dungeons under Persepolis. |
| Mode | Coop x Objective ([taxonomy](../../game-modes.md)), shared clear |
| Modifiers | Permadeath, and **linked fate**: if one dies, both die |
| Camera | Top-down, one room on screen |
| Players | 2 (+ spectators) |
| Physics | none from Rapier: the chain is a deterministic rope the rules own (verlet links); characters are kinematic |
| Screens | Shared (the chain keeps them on one screen), local split screen, or one screen per player online ([Screens](#screens)) |
| Scene | `games/__templates__/darius/assets/scenes/games/coop/bound_bros.scene.rs` |
| Build rung | R2 Participants and bots; it also introduces split-screen viewports ([ladder](../README.md#the-build-ladder)) |
| Twin | [Croesus' Gold](../cyrus/coop-objective-croesus-gold.md) (Cyrus (3D), same mode) |

## Pitch

Two prisoners, one chain. The chain is also the only weapon you have: swing it through the guards and they fall. One of you plants and holds, the other circles, and the chain becomes a flail. Dash outward and it cracks like a whip. But anything that touches either of you costs a heart, and if one of you dies, so does the other. Fight your way through the rooms to the stairs.

## Why it is this mode

A defined goal, pass or fail, tiered instead of ranked: Coop x Objective (shared clear). It is a fixed course of rooms with an exit, not an endless wave, and the chain makes "together" a physical fact.

## Rules (the `Sim`)

- **State:** Two players (position, velocity, hearts, dash timer), the chain (a fixed number of links, a maximum length, a stiffness), enemies, the room's goal state, keys, doors, the exit; the run's time and hearts.
- **Hearts:** each player has **3**. **An enemy that touches a player costs that player one heart**, then a second of i-frames. Heart pickups heal whoever takes them. **If either player's hearts reach 0, both die and the run ends.** The team is only as strong as its weaker player.
- **The chain is the weapon:** any enemy a link sweeps through takes damage that scales with the link's speed there; below a minimum speed it only pushes. The chain never hurts a player.
- **Inputs (each player):** stick; **Fire held = brace** (you plant and become an anchor; the chain cannot drag you); **Interact** (grab or drop a key or crate; hook the chain to a post, which turns it into a tripwire); **Utility = dash** (a short burst with i-frames, on a cooldown). Dashing away while the partner is braced cracks the chain like a whip.
- **Enemies:** crawlers (swarm), chargers (telegraphed run), shield guards (only the chain from behind hurts them), spiked ones (never touch; chain only), nests that spawn crawlers until the chain smashes them.
- **A room opens when:** its goal is met: clear the room, smash every nest, or bring the key to the door (drag it on the chain).
- **A run ends when:** both players stand on the exit stairs of the last room (win), or either player is out (loss), or an optional time limit passes.
- **Result:** Won with a tier (gold, silver, bronze by hearts left and time), or Lost.
- **The chain is shorter than half a screen**, so one camera always holds both players.

## Controls

| Keyboard / mouse | Touch |
|---|---|
| P1 WASD, F brace, E interact, Shift dash; P2 arrows, K brace, L interact, Enter dash | Stick + the stack of three buttons (Fire = brace, Context = interact, Utility = dash); one device each online |

## Screens

- **Online:** one screen per player with its camera on that player; the chain and the partner are always in view.
- **Local:** one shared camera on the midpoint while the chain holds them together (which is always, by design), or **split into two viewports** if the players choose, each centred on its own player.
- Nothing is hidden.

## Offline, online, spectate

- **Offline:** you plus a **bot bro** that follows, braces when you dash, and circles when you are boxed in; or two on one keyboard.
- **Online:** party code, 2.
- **Spectate:** the room from a shared camera; replay from seed and inputs.

## What it forces in the engine

- **A deterministic chain** in `xerxes_sim`, with swept-segment damage and collision with pillars
- **Linked fate**: a match outcome where one side's member going down ends the side
- Two Local players on one device, and **several viewports** in one scene
- A trigger graph (nests, keys, doors) as data
- Enemies and hearts shared with Ahriman's Horde

## Slices

1. Two players, a chain, one room of crawlers, chain damage, hearts and linked death
2. Brace and dash whip; chargers and shield guards; the exit
3. Nests, the key, several rooms, tiers
4. Bot bro; local split screen; online (the chain is a shared object, so it is server-authoritative)

## Open questions

- Are hearts best per player (as written, so the weaker player is the limit) or a shared pool?
- Does the chain wrap pillars and shorten, or only slide?
- How much latency can a shared whipping chain stand online (prediction, or a slower chain)?
