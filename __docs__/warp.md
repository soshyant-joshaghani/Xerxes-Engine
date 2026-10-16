# Warp: the level manager

Most engines give you an array of scenes and a `load_scene(i)` call. Everything a real game needs around that (where the player arrives, what they carry, whether the door is open yet, what changed since last time, what the other players see) is left to each developer, and rewritten in every game. **Warp** is Xerxes' answer: a level manager that owns the whole journey between scenes.

Status: design, for Phase 3 stage 3.5 ([PROGRESS.md](../__plans__/PROGRESS.md)). Warp replaces the plain `levels: vec![LevelDef { ... }]` list in the project settings; the flow (`Preload → Splash → MainMenu → Level`) stays and Warp takes over what "Level" means.

## The model

```text
Warp map (one per project, an asset: <name>.warp.rs)
  Place     a scene the game can be in         ("Village", "Cave", "Arena: Duel x Objective")
    Port    a named entry or exit of a place   (entry "from_gate", exit "to_cave", exit "fell_in_hole")
  Link      exit -> entry, with rules          (Village.to_cave -> Cave.from_village)
    Lock      a Condition that must hold       (quest "find_the_key" is Success)
    Params    values passed to the next place  (variant: "after_flood", spawn_hp: 3)
    Events    named events fired on the way    (OnLeave, OnEnter, OnLinkTaken, OnLockedAttempt)
```

- **Place** = a `*.scene.rs` plus the `GameMode` of that level ([game-modes.md](game-modes.md)). One scene can be several places (the same room as a day place and a night place), because the place is the scene plus its mode plus its default params.
- **Ports** are the "inputs and outputs" of a level. An exit is something the scene can trigger by name (a door, a pit, a timer, a won match); an entry is where the player appears (a marker object in the scene, with facing and spawn rules). Scenes only ever name ports, never other scenes, so a scene can be reused and the map rewired in the editor.
- **Links** connect an exit to an entry. An exit may have several links; the first whose lock is open wins, in listed order (so "if the flood happened go to the flooded cave, else the dry cave" is two links on one exit). An unmatched exit is a map error caught when the map is validated.
- **Lock** uses the shared Condition data ([dialogue-quest.md](dialogue-quest.md#conditions-and-actions)): quest state, a variable, "has visited place X", "level X completed", item counts, or any mix. The same conditions drive dialogue options and quest steps, so a door, a line of dialogue and a quest objective all read one truth.
- **Params** are typed values attached to a link and passed to the entering scene as a `WarpArrival` resource: `from_place`, `from_port`, `visit_count`, and the link's own values plus per-visit values. The scene reads them in its setup (spawn extra enemies, change the music, skip the intro). **Revisiting** is first-class: the arrival knows how many times the place was visited and what quests changed since, so the cave after the flood is the same scene with different params, not a copy of the scene.
- **Events** are named messages on a bus ([the message system in the reference package](dialogue-quest.md#reference-map) is the model): Warp fires `warp:leave:<place>`, `warp:enter:<place>`, `warp:link:<id>`, `warp:locked:<id>`. Quests and dialogue can listen (finish a quest step on entering a place), and so can logic modules, with no code in Warp knowing about them.

## What a transition does

1. A scene (or a logic module, a quest reward, a dialogue action) asks to **take exit** `to_cave`, optionally with extra params.
2. Warp picks the link: first link with an open lock. If none is open it fires `warp:locked` with the reason (so the game can say "you need the key") and stays.
3. Leave: fires leave events, runs the scene's leave transition (fade, sequence), saves the place's persistent state (Save, stage 3.6) and the visit record.
4. Enter: spawns the next scene at the entry, builds `WarpArrival` from the link, fires enter events, runs the enter transition.
5. The session's current place and visit counts are saved with the game, so loading resumes exactly where you were, with the same revisit history.

## Online play

Warp must work when the game is a room ([multiplayer.md](multiplayer.md)):

- The **server owns the Warp state of a room**: current place, the party, the visit records. Clients ask to take an exit; the server checks the lock and answers with the new place and arrival. A client cannot open a locked door by editing memory.
- A link has a **scope**: `Everyone` (the whole room moves, as in a mode scene that ends and goes to the results place), `Party` (a group moves, the rest stay: co-op splitting up), or `Player` (one player moves between rooms/places, as in an open world).
- A moved player can **change room**: the exit can carry a room option (create/join by id or filter, the Colyseus matchmaker), so an arena entrance in the hub warps the player into a Duel room. Offline, the same link starts the local match with bots.
- **Spectators** receive place changes like any other state, read-only, and follow the party they watch.

## Editor

The editor edits the map as data first: a Warp panel (Places, Ports, Links, Locks, Params) with validation (unreachable places, exits with no link, ports named but not in the scene, links whose lock names a quest that does not exist). A node-graph view of the same data is Phase 10 ("node graph"); the data model does not change for it.

## Build order inside stage 3.5

1. Data model and validation (pure Rust, no Bevy): `Place`, `Port`, `Link`, `Lock`, `Params`, `Event`, plus tests (the revisit-after-quest case, first-open-link-wins, unreachable place).
2. Runtime in `modules/warp`: take an exit, transitions, `WarpArrival`, events, persisted visits.
3. Scene side: entry markers (a component), exit triggers (a component and a logic module), the arrival resource.
4. Darius map: the main menu scene is the first place, then one place per mini-game ([darius.md](darius.md)), gated and parameterised through Warp, so Warp is exercised by 21 scenes.
5. Quest integration (needs stage 3.8) and online scopes (needs track M1).

## Things Warp deliberately does not do

- It does not stream scenes or manage open-world chunks; a place is a whole scene.
- It does not own the mode rules: it only starts a place whose scene brings its `GameMode`.
- It does not know what a quest is: it evaluates the shared Conditions and listens to the shared event bus.
