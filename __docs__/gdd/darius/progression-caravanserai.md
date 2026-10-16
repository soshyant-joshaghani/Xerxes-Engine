# Caravanserai: Silk Road trader

Darius (2D) · **Progression** · Coming soon (empty scene)

| | |
|---|---|
| Name | A caravanserai was a roadside inn where caravans rested and traded. |
| Mode | Progression ([taxonomy](../../game-modes.md)) |
| Modifiers | none |
| Camera | Map of the Royal Road + a trading screen |
| Players | 1 (+ spectators) |
| Physics | none |
| Scene | `games/__templates__/darius/assets/scenes/games/outside/caravanserai.scene.rs` |
| Build rung | R7 Tools (Warp, save, quests, dialogue) ([ladder](../README.md#the-build-ladder)) |
| Twin | [Haft Khan](../cyrus/progression-haft-khan.md) (Cyrus (3D), same mode) |

## Pitch

Lead a caravan from caravanserai to caravanserai. Buy silk where it is cheap, sell it where it is dear, hire guards, buy more camels. A **random-event deck** (ambush, storm, a rival caravan) resolves with one-tap choices. **Darics from [Ahriman's Horde](solo-survival-ahrimans-horde.md) carry over** through the save tool, so a run feeds the caravan, and later the caravan can fund Ahriman's Horde unlocks. There is no end, only a bigger caravan.

## Why it is this mode

Progressed, not won: loot, upgrades, an economy. Progression.

## Rules (the `Sim`)

- **State:** Caravan (camels, goods, guards), purse (darics, including those brought from other games), prices per city, unlocks, the event deck.
- **Inputs:** Travel to a city, buy, sell, hire, upgrade, resolve an event (one tap).
- **A round ends when:** Never.
- **Result:** None (persistent progress).

## Controls

| Keyboard / mouse | Touch |
|---|---|
| Mouse | Tap |

## Offline, online, spectate

- **Offline:** You; saved on the device.
- **Online:** Cloud save per player.
- **Spectate:** Watch a trade run.

## What it forces in the engine

- Save and load (stage 3.6), including darics shared with Ahriman's Horde
- Inventory and an economy
- Unlock state
- A one-tap event deck

## Slices

1. Three cities, buy and sell, save
2. Upgrades and the event deck on the road
3. Darics from Ahriman's Horde; cloud save

## Open questions

- Events as one-tap cards only, or small mini-scenes for ambushes later?
