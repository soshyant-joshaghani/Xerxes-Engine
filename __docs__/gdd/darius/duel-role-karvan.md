# Karvan: Cargo escort

Darius (2D) · **Duel x Role** · Coming soon (empty scene)

| | |
|---|---|
| Name | A karvan is a caravan, the merchant convoys of the Royal Road and the Silk Road. |
| Mode | Duel x Role ([taxonomy](../../game-modes.md)), attacker vs defender |
| Modifiers | Escort, Respawn (a different delay for each role), TimeLimit, asymmetric (attackers and defenders may differ in number) |
| Camera | Top-down, scrolling along the road, following the wagon |
| Players | 2 teams, 1v1 up to 4v4 (+ spectators) |
| Physics | none from Rapier: character mover and casts; the wagon follows the road's path |
| Screens | Local: one shared camera on the wagon, splitting when the team spreads. Online: one camera per player ([Screens](#screens)) |
| Scene | `games/__templates__/darius/assets/scenes/games/duel/karvan.scene.rs` |
| Build rung | R5 Crowds, paths and objectives ([ladder](../README.md#the-build-ladder)) |
| Twin | [Atashkadeh](../cyrus/duel-role-atashkadeh.md) (Cyrus (3D), same mode) |

## Pitch

A merchant wagon sits at the start of a long road with a gate at the far end. One team are the **escorts** and must bring it through. The other team are the **ambushers** and must stop it. The two teams do different jobs with different tools, then swap and do it again; the team that took the wagon further, or faster, wins.

## Why it is this mode

Two teams, two different jobs and different win terms (deliver or stop), roles swapped each round to keep it fair: Duel x Role (attacker vs defender, cargo).

## Rules (the `Sim`)

- **State:** the wagon (position along the road, HP, speed), checkpoints, the escorts and ambushers (position, hp, role kit), barricades and traps placed, the round clock, the round number.
- **The wagon** moves only while **an escort stands near it** (faster with up to three); an ambusher near it, with no escort there, **holds** it; both near it, it is **contested** and stops. A passed checkpoint extends the clock and moves the escorts' respawn point forward.
- **Inputs (each player):** the pad. Fire (shoot, planted), Interact / Switch, Reload / Dash, as in [Ahriman's Horde](solo-survival-ahrimans-horde.md). Guns are found on the road.
- **Role kits (Interact does the role's job):**
  - **Escorts:** repair the wagon, raise a shield panel on it, call a mule (a second shield).
  - **Ambushers:** raise a barricade at a marked site, lay caltrops, cut a rope bridge. Each has a few charges, refilled at their supply spots.
- **Respawn:** escorts come back at the last checkpoint after a short delay; ambushers after a longer one, so a good ambush matters.
- **A round ends when:** the wagon reaches the gate (escorts win the round), the wagon is destroyed, or the clock runs out (ambushers win the round).
- **A match is:** two rounds with the roles swapped, compared by how far or how fast each team took the wagon. A tie is a short sudden-death road.
- **Result:** ranked: the winning team.

## Controls

| Keyboard / mouse | Touch |
|---|---|
| As Ahriman's Horde: WASD, mouse aim, hold left button Fire, E Interact / Switch, Shift or R Reload / Dash | Stick + the stack of three buttons |

## Screens

- **Local:** a shared camera that follows the wagon; if the team spreads further than a screen it splits into viewports, up to one per local player.
- **Online:** one camera per player.
- Nothing is hidden: both teams see the whole road.

## Offline, online, spectate

- **Offline:** you plus **bots** on both teams: escorts that stay near the wagon and repair it, ambushers that hold the nearest barricade site and shoot.
- **Online:** a queue, 1v1 to 4v4.
- **Spectate:** follow the wagon or either team; replay from the inputs.

## What it forces in the engine

- Escort / MovingObjective as real gameplay, with a contested movement rule
- Role kits and per-role Respawn delays; the asymmetric flag
- Round swap of roles and a two-round comparison result
- Guns from the Gun add-on on the road

## Slices

1. One road, a wagon, escorts and ambushers (two local players), the movement rule
2. Role kits, checkpoints, respawn delays
3. The swap and the two-round result; bots on both sides
4. Online 1v1 to 4v4; spectate

## Open questions

- Is the match decided by distance, by time, or "delivered, else distance"?
- Can ambushers also hurt the wagon directly, or only block and hold it?
