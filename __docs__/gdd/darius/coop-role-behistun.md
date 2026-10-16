# Behistun: Read the cliff

Darius (2D) · **Coop x Role** · Coming soon (empty scene)

| | |
|---|---|
| Name | The Behistun inscription, carved for Darius I in three languages, was the key to reading cuneiform. |
| Mode | Coop x Role ([taxonomy](../../game-modes.md)), functional split |
| Modifiers | FogOfWar (each sees half), TimeLimit (the rope) |
| Camera | UI only: a tablet for one, a codex for the other |
| Players | 2 (+ spectators) |
| Physics | none |
| Screens | see [Screens](#screens) |
| Scene | `games/__templates__/darius/assets/scenes/games/coop/behistun.scene.rs` |
| Build rung | R4 Hidden information ([ladder](../README.md#the-build-ladder)) |
| Twin | [Pasargadae Depths](../cyrus/coop-role-pasargadae-depths.md) (Cyrus (3D), same mode) |

## Pitch

The inscription is carved high on a cliff. The **Reader** hangs on a rope in front of it and sees a tablet of wedge signs, dials and seals, but no key. The **Scholar** stands below with the codex that says what every sign means and what to do, but cannot see the tablet. The Reader describes what they see; the Scholar finds it and answers. Open the tablet before the rope frays.

## Why it is this mode

Different jobs, different information, and neither can win alone: Coop x Role (functional split). It needs no shooting and no reflexes, only talk.

## Rules (the `Sim`)

- **State:** the tablet (modules, each with signs and a state), the codex (rules, from the seed), mistakes left (the rope's frays), the clock.
- **Modules (three kinds, mixed by the seed):**
  - **Signs:** the tablet shows a sign made of generated wedges; the codex lists signs by wedge count and direction, not by picture. The Reader must **describe** it ("three wedges pointing right and a hook"); the Scholar names which one it is and what to press.
  - **Seals:** press the seals in an order the codex gives, which depends on their colours and how many carry a sign.
  - **Dials:** set each dial to the value the codex tables give for the sign beside it.
- **Inputs:** Reader: turn a dial, press a seal. Scholar: page through the codex (private).
- **A round ends when:** every module is open (win), or the rope is cut (three mistakes) or the clock runs out (loss).
- **Result:** Won with a tier by time and frays left, or Lost.
- **Seeded:** the tablet and the codex come from the seed, so a replay and a daily board are possible.

## Controls

| Keyboard / mouse | Touch |
|---|---|
| Mouse | Tap and drag |

## Screens

- **Online:** one screen per player, each showing only their own half. This is the natural mode.
- **Local:** a literal split screen is **not offered**, because each half would show the other player's secret. Instead the shared screen shows the tablet and the Scholar's codex opens on **a second device** that joins by a short code on the local network (the party-game model), or the pair passes one device behind a privacy screen.

## Offline, online, spectate

- **Offline:** you plus a **bot** in the other role. A Scholar bot answers your description by matching it; a Reader bot describes the tablet in set phrases that you look up in the codex. Or hot-seat with a privacy screen.
- **Online:** party code, 2; text or quick-phrase chat. Voice is not required.
- **Spectate:** live, the tablet only (public) with a delay; the codex is shown after the match. Replay: both.

## What it forces in the engine

- A private view per participant (the rules give each only what it may see)
- Spectator rules for hidden information
- Seeded procedural content (wedge signs, rule tables)
- Chat or quick phrases as part of a match

## Slices

1. One module (signs), hot-seat with a privacy screen
2. Three module types, seeded codex, the rope fray
3. Bot in either role
4. Online with redacted spectating (M3)

## Open questions

- Is voice in scope, or text and quick phrases only? (Assumed the latter.)
