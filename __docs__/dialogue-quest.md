# Dialogue, quests and the tools around them

Games of every genre need conversations, quests, saving, text in several languages and the UI to show them. No engine ships these; each team builds or buys them. Xerxes builds them as engine modules, designed from a real reference.

**The reference** is a Unity package kept locally in `__temp__/Dialogue System - Reference/` (gitignored: it is a commercial asset, never to be committed, shared or copied from). It is Pixel Crushers' *Dialogue System for Unity* plus the *Common* framework it sits on (message system, save system, text tables, UI helpers). We study its **design**: what it covers, how the parts split, where it is awkward. We write our own implementation in Rust and our own names where they matter. We do not port its code, text, art or file formats. If a part is ever taken more directly than that, its licence (and the Pixel Crushers copyright on it) must be checked first.

What follows comes from reading the package's folder layout, file and type names and a few sources (`QuestState`, `QuestLog`, the Sequencer command list, the triggers, the save system). Each row says to verify against the source when its stage starts.

## Reference map

| Reference part (where) | What it does | Xerxes equivalent | Stage |
|---|---|---|---|
| **Dialogue database** (`MVC/Model/Data`: `DialogueDatabase`, `Conversation`, `DialogueEntry`, `Actor`, `Item`, `Location`, `Variable`, `Link`, `Field`) | One database holds actors, conversations (a graph of entries joined by links), quests (called "items"), variables, locations. Every record has custom fields. | Typed assets, one per concept: `<name>.dialogue.rs` (a conversation), `<name>.quest.rs`, `<name>.actor.rs`, plus project-wide variables. They join the `Name.Type.rs` family (`xerxes_build::types`), so the browser shows them once with a preview and the editor edits them as data. Custom fields become typed Rust fields, not string maps. | 3.8, 3.9 |
| **Lua conditions and scripts** on entries (`Scripts/Lua`) | Each line and link carries a condition string and a script string run when it plays. | **No scripting.** A small typed data language instead: `Condition` and `Action` values (see below), evaluated by the engine, edited in forms. A scripting language is not planned for now. | 3.5 |
| **Variables and persistent data** (`Variable`, `DialogueLua`, `PersistentDataManager`) | Global variables read by conditions, written by actions, saved with the game. | `Variables` resource (typed: bool, int, float, text), part of the save. | 3.5, 3.6 |
| **Quest log** (`Scripts/Quests`: `QuestLog`, `QuestState`, `QuestGroupRecord`, `QuestStateListener`, `QuestStateIndicator`, `IncrementOnDestroy`) | Quest states Unassigned, Active, Success, Failure, Abandoned (a bit-flag set); entries (sub-tasks) with their own states; counters; groups; a listener that reacts to state changes; an indicator that shows "!" or "?" over a quest giver; a counter that rises when an object is destroyed. | `modules/quest`: the same five states; quest entries with state and counter; groups; `QuestChanged` messages on the event bus; an indicator component; counters driven by events (an enemy dies, an item is picked up, a place is entered). | 3.8 |
| **Quest UI** (`UI/Standard/Quest`: log window, title buttons, tracker, track template, alternate descriptions) | The quest log, a HUD tracker, descriptions that change with the state. | Quest log and tracker in the UI kit; a description is a list of (condition, text), so it changes with the state. | 3.10 |
| **Conversation runtime** (`MVC`: `ConversationController`, `ConversationModel`, `Subtitle`, `Response`, `ConversationState`) | Walks the entry graph: picks the entries whose conditions hold, shows subtitles, builds the response menu, runs the scripts, supports timed responses, and can run several conversations. | `modules/dialogue`: a conversation runner over the typed graph, as a plain state machine (no Bevy) so it can be tested and also run on a server for online scenes. | 3.9 |
| **Barks** (`BarkController`, `StandardBarkUI`, `BarkOnIdle`) | One-line speech that is not a conversation: an NPC shouting when you pass. | Barks: a short list of (condition, line) on an actor, shown by the same UI. | 3.9 |
| **Sequencer** (`MVC/Sequencer`: ~20 commands such as Camera, Delay, Fade, LoadLevel, LookAt, MoveTo, Animation, AudioWait, Voice, WaitForMessage, Zoom2D, QTE, TextInput) | A tiny command language attached to a line (cutscenes, camera cuts, waits) with `entrytag` and message waits. | `modules/sequence`: a typed list of steps (`Camera`, `Wait`, `Fade`, `MoveTo`, `Say`, `Play`, `WaitForEvent`, `Warp`, ...) run on a timeline, used by dialogue, quests, Warp transitions and tutorials. Edited as a list first, as a timeline in the editor later. | 3.9 |
| **Triggers and interaction** (`Scripts/Triggers`: `DialogueSystemTrigger`, `Selector`, `ProximitySelector`, `Usable`, `RangeTrigger`, `Condition`) | "On use / on trigger enter / on start / on destroy do X": start a conversation, set a quest, play a bark; a player selector that picks the usable thing in front of you. | Logic-library pieces (`interaction/`): `Interactable`, a proximity selector, and event-to-action triggers. They emit the same `Action`s. | 3.9 |
| **Dialogue UI** (`UI/Abstract`, `UI/Standard`: subtitle panel, response menu, alert, timer, QTE, input field, bark UI) | An abstract UI layer so any UI can be plugged in, and a standard UI built on it. | A bevy_ui implementation in the UI kit behind a small trait, so a game can replace it. | 3.10 |
| **Message system** (`Common/Message System`: `MessageSystem`, `MessageArgs`, `DataSynchronizer`) | A global publish/subscribe bus by message name and parameter. | The event bus used by Warp, quests and dialogue (named events with typed arguments), on top of Bevy messages. | 3.5 |
| **Save system** (`Common/Save System`: savers, storers, serializers, spawned objects, scene transitions; `Dialogue System/Save System`: persistent position/active/destructible, `LevelManager`) | Components that save themselves (position, active, enabled, animator, destructible); pluggable storage (disk, player prefs); JSON or binary serializers; respawning of spawned objects; leave/enter scene hooks. | `modules/save`: a `Saved` component family, pluggable storage per platform (disk, browser storage, Android app storage, the backend for cloud saves), a serializer, scene hooks that Warp calls. Saves include variables, quests, conversations, visits, and per-place state. | 3.6 |
| **Text tables and localization** (`Common/Text`: `TextTable`, `StringField`, `GlobalTextTable`, `UILocalizationManager`) | Text by key and language, a field that is either literal or a key, UI that re-localizes when the language changes. | `modules/text`: text tables as typed assets (`<name>.text.rs`), a language setting, a `Text` field that is a literal or a key. Dialogue and quests use it from the start. | 3.10 |
| **UI helpers** (`Common/UI`: `UIPanel`, `UIButtonKeyTrigger`, `InputDeviceManager`, `KeepRectTransformOnscreen`, `UITextField`) | Animated show/hide panels; key shortcuts on buttons; detecting keyboard vs gamepad vs touch to show the right prompts. | UI kit: panels with open/close animation, input prompts that follow the active device (keyboard, gamepad, touch), on-screen clamping. | 3.10 |
| **Misc and events** (`Common/Misc`, `UnityEvents`: `Pool`, `GameTime`, `SceneNotifier`, `TagMask`, `TimedEvent`, `CollisionEvent`, `TriggerEvent`) | Object pooling, pausable game time, scene notifications, "on collision/trigger/timer fire these events". | Pooling and pausable time are engine pieces as the games need them; collision/trigger/timer events are Rapier sensor events mapped to the event bus. | 3.1, 3.5 |
| **Editors** (the package's database editor, conversation node editor, quest editor, templates) | Where designers author all of this. | Forms first (the Inspector pattern we already have), a conversation/quest graph later (Phase 10, the node graph). The data model does not change between the two. | 3.8, 3.9, 10 |
| **Importers** (Articy, Chat Mapper) and the many third-party support packages | Authoring-tool imports; adapters for other Unity assets (inventory, cameras, localization). | Not ported. An importer for a text format (Ink, Yarn, Twine) is a later option. The adapters show where such a system needs seams: inventory, localization, save, camera, audio. | later |

## Conditions and actions

The reference uses Lua strings. Xerxes starts with **typed data**, so the editor can offer forms and the compiler checks the names:

```rust
// Condition: reads the world, never changes it.
Condition::All(vec![
    Condition::Quest("find_the_key", QuestState::Success),
    Condition::Var("reputation", Cmp::AtLeast, 3),
    Condition::Not(Box::new(Condition::Visited("cave"))),
])

// Action: changes the world, run when a line plays, a quest step completes, a link is taken.
Action::StartQuest("find_the_key")
Action::SetVar("met_elder", true)
Action::Warp { exit: "to_cave", params: params!{ variant: "after_flood" } }
Action::Say("elder", "greeting")
```

- One `Condition` / `Action` set serves **dialogue** (which responses appear, what a line does), **quests** (when a step counts, what completing gives), **Warp** (which link is open, what an arrival carries) and **triggers**.
- They are serializable, so they live inside the typed assets, are saved, and can be sent to a server. They are also pure data: evaluation is a function of the world state, so it works the same offline and in a room.
- If a scripting language ever comes (it is out of the plan for now), it would be another `Condition`/`Action` source rather than a replacement.

## What this adds to the plan

The stages that build this are 3.5 (shared data), 3.6 (save), 3.7 (Warp), 3.8 (quests), 3.9 (dialogue and sequencer), 3.10 (text and UI kit); see [PROGRESS.md](../__plans__/PROGRESS.md). Each is exercised by darius's narrative, progression and composite scenes ([darius.md](darius.md)), and each stage is only `done` when a scene of the game uses it.
