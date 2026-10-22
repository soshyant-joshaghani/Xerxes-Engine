# Ahriman's Horde: Infinite waves

Darius (2D) · **Solo x Survival** · Slice 1 playable (the old courtyard, armed); waves are the next slice

| | |
|---|---|
| Name | Ahriman is the lord of the lie in Zoroastrian belief; his drujs come out of the dark. Here they come in waves, and every daric you take from them is a point. |
| Mode | Solo x Survival ([taxonomy](../../game-modes.md)), endurance: infinite rounds |
| Modifiers | Permadeath (one run, one life) |
| Camera | Top-down, one courtyard on screen |
| Players | 1 (+ spectators) |
| Physics | Rapier (character mover, walls, sensors); bullets are cheap kinematic casts, not rigid bodies |
| Scene | `games/__templates__/darius/assets/scenes/games/solo/ahrimans_horde.scene.rs` |
| Build rung | R0 Catalogue for the arena and the pad; the crowds' paths come with R5 ([ladder](../README.md#the-build-ladder)) |
| Twin | [Atash](../cyrus/solo-survival-atash.md) (Cyrus (3D), same mode) |

## Pitch

A walled courtyard in a Persian city, its gates open, and the horde comes through them. Round after round, more and faster drujs; between rounds you repair the barricades and spend your darics. Every kill is points. A mystery chest gives a random gun. The run is endless, and it ends when your health does. The number on the board is the round you reached.

## Reference: Call of Duty: Zombies

Zombies is the survival mode people replay: a round counter that never stops, points for every kill, barricades to rebuild between rounds, mystery boxes, power-ups that drop from the dead, and a board of best rounds. Ahriman's Horde takes all of that, with the courtyard, the Persian drujs and the darics as the points. It does **not** take the co-op revive or the buy-doors map progression: one player, one courtyard, one life.

## Why it is this mode

Nothing here is a score you push up for its own sake. The rounds push back harder every time and the result is how long you outlast them: the endurance run, ranked by round reached. Points are the tiebreak, not the goal.

## The pad

Left: a stick. Right: a stack of three buttons. (Unchanged from the first slice.)

| Button | Condition | Effect |
|---|---|---|
| **Fire** | held | You **stop**; the stick is the aim; the weapon fires along it |
| **Interact / Switch** | near an interactable | **Interact** (take a gun, repair a barricade, open a chest, buy) |
| | far from any | **Switch weapon** (next gun in the inventory) |
| **Reload / Dash** | weapon fully loaded | **Dash** |
| | weapon not full | **Reload** |

- With Fire not held the stick moves you at full speed and you cannot shoot. With Fire held you are planted and the stick only aims.
- **Reload is the dash cooldown.** Any shot makes the magazine not full, so the next press of the bottom button reloads; once full it is a dash again.
- The button's icon follows the state (hand or gun; reload ring or dash arrow).
- Walking onto a gun never picks it up; you press Interact. With several interactables near, the one closest to the stick direction wins and is outlined.

## Rules (the `Sim`)

- **State:** Player (position, aim, planted, dash timer, health, time since last hit), inventory, the courtyard (walls, cover, barricades at the gates, enemies, interactables, projectiles), the run (seed, round, points, time).
- **Inputs:** Stick vector; Fire held or not; Interact / Switch pressed; Reload / Dash pressed. Four values per tick: a replay is a short line of text.
- **Dash:** requires a **full magazine**. A short fixed-distance move with **i-frames for its active frames** (brief), no damage dealt. You are hit normally outside those frames, including while reloading or planted.
- **Health:** a hit takes one of your three hearts. After 6 s with no hit, health refills to full (the Zombies regeneration). Hearts also drop from kills, rarely, and do not go past full.
- **Rounds:** round *r* spawns a larger count of drujs than round *r - 1*, from the gates, faster and tougher as the rounds go up. A round starts after a short breather of 3 s. It ends when its last druj falls; the next round starts after the next breather. There is no round cap.
- **Barricades:** the gates are barricaded. Drujs tear a barricade down before they can walk through it; Interact repairs one for darics. A flipped table is cover, as before.
- **Points:** darics are points. Each kill gives points by kind (rushers small, archers more, heavies most). Points are also spent: repairs, mystery chests, shrines. The score is the **points earned**, but the board ranks rounds first.
- **Assassin Mode (the power-up):** a crate dropped by a kill, for a short fixed time (a visible countdown and a tint): unlimited ammo, the gun is always loaded (the dash is always ready) and points are doubled. You still take damage.
- **A run ends when:** your health reaches 0 (Permadeath).
- **Result:** Finished, ranked by round reached, then points, then time survived.

## Weapons and inventory

- You always have the **unlimited pistol**: lowest damage, the fallback, never runs dry.
- You may carry **any number** of ground guns. Each has a **magazine** and a **total** (bullets in reserve).
- **Picking up a gun you already carry adds its bullets** to that gun instead of a second copy.
- A gun with both magazine and total at zero is removed, so Switch never cycles through dead guns.
- Switch cycles in recency order, so a tap toggles your two main guns.
- Weapons are plain data: damage, rate, magazine size, reload time, spread, projectile. Magazine and reload are design levers on the dash, not only on damage.
- The **mystery chest** (bought with darics) gives a random gun from a table weighted to the round; the **mythic finds** from the Shahnameh sit at the top of that table.

| Weapon | Reserve | Feel |
|---|---|---|
| **Pistol** (starting) | unlimited | steady single shots, small magazine, quick reload |
| Bow | limited | slow, hard-hitting, pierces |
| Javelin thrower | limited | 3-shot burst |
| Naphtha pot | limited | arcing, leaves fire on the floor |
| Spear-thrower | limited | spread |
| Mythic finds (Shahnameh) | very limited | strong, from the top of the chest table only |

## The run and the day

- **The courtyard:** one hand-made arena with its gates, cover and barricades; the same arena for every round, so a round is learned by its place.
- **Drujs:** rushers (you must break contact to shoot), archers with telegraph lines, shield-bearers (hit from the side), turrets in cover, summoners, heavies. A hard rule: every attack is **telegraphed and slow enough to read**, because a planted player cannot dodge what they cannot see.
- **Interactables:** barricades, tables (flip for cover), the chest, the shop (spend darics), a shrine (trade a heart for a power), a rest fire.
- **Daily seed**, shared with Chapar: the same drujs in the same order in each round for everyone that day, so the board compares like with like.

## Controls

| Keyboard / mouse | Touch | Gamepad |
|---|---|---|
| WASD / **left-drag** stick = move; **right-drag** on stick or **Space** = fire | Left stick; right stack: Fire, Interact / Switch, Reload / Dash | Left stick; right trigger Fire, X Interact / Switch |
| **R** Reload / Dash, **F** Switch / Interact, Esc pause | D-Pad / Stick toggle in the **header** | B Reload / Dash |

## Offline, online, spectate

- **Offline:** alone; your best round kept on this device.
- **Online:** the **leaderboard**: a run's record (round, points, time) is submitted when the run ends. Two boards: all-time best round, and the daily-seed board.
- **Spectate:** follow a live run, or replay any board entry from its seed and inputs. The rules are a deterministic fixed-step `Sim`, so a replay is exact.

## What it forces in the engine

- A stack of three context buttons beside the virtual joystick, whose label and state change at runtime (the HUD's `TouchButton`)
- Aim-as-stick while Fire is held; keyboard, mouse and gamepad mappings
- Waves as data: a round is a spawn table from its number and the seed
- Barricades that take hits and are repaired for a currency
- Crowd paths (A*) for the rounds past the first few (R5)
- Cheap projectiles (kinematic casts) in a deterministic sim
- Enemy behaviours as data with telegraphs
- A weapon as data, and an inventory that merges duplicates (the Gun, extracted into an add-on at its second use, Atash)
- Submitting a record to a leaderboard at the end of a run (M5)

## Slices

1. **Done (2026-10-15, as Daric Dash): the courtyard, armed.** Arena, the Fire / Switch / Reload-Dash pad, the unlimited pistol, one rusher, a gun and health pickups, kill → portal → warp. The warp goes when the rounds arrive.
2. **Rounds.** The round counter, a spawn table that grows, the breather, points per kill, health with regeneration, three drujs, the barricades at the gates.
3. **The shop and the chest.** Points spent on repairs and mystery chests; the inventory merges duplicates; Assassin Mode from the crates.
4. **The run and the day.** The daily seed, the results screen (round, points, time), replay from inputs.
5. **Online.** The leaderboard (all-time and daily); spectate (M1).

## Open questions

- Health: regeneration to full after 6 s (as drawn), or a few hearts that come back one at a time?
- Points per kind, the price of a repair and a chest: first guesses, to play-test.
- Should a boss come every fifth round, as the Zombies dogs and bosses do, or should the rounds stay even?
- The warp: gone from this game, or kept as the rest between floors of a longer run?
- Does the daily seed fix the drujs of every round, or only the first few rounds?
