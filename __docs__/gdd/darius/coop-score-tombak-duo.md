# Tombak Duo: Echo drumming

Darius (2D) · **Coop x Score** · Coming soon (empty scene)

| | |
|---|---|
| Name | The tombak is the Persian goblet drum, often played in pairs. |
| Mode | Coop x Score ([taxonomy](../../game-modes.md)), shared high score |
| Modifiers | none |
| Camera | Two note lanes (UI), side by side |
| Players | 2 (+ spectators) |
| Physics | none |
| Screens | see [Screens](#screens) |
| Scene | `games/__templates__/darius/assets/scenes/games/coop/tombak_duo.scene.rs` |
| Build rung | R6 Audio clock ([ladder](../README.md#the-build-ladder)) |
| Twin | [Nowruz Feast](../cyrus/coop-score-nowruz-feast.md) (Cyrus (3D), same mode) |

## Pitch

Two drummers, one song, and **your partner's playing is your chart**. A phrase is four beats. In the *call*, one drummer plays the notes shown on their lane. In the *answer*, the other drummer's lane shows the **ghosts of what their partner actually played**, the timing and the mistakes included, and they must play it back. Then the roles swap. Every phrase passes through both of you.

## Why it is this mode

One shared score, and the bonus only exists when both play: the combo grows only when an answer matches its call. That is Coop x Score (a shared score with a synergy bonus), not two solos in parallel.

## Rules (the `Sim`)

- **State:** the song clock (audio time), the phrase, each lane's notes and ghosts, the shared score, the combo, a shared groove meter.
- **Inputs:** a hit on one of two pads per player (low *doum*, high *tak*).
- **Judgement:** each hit is judged against the song clock (perfect, good, miss), never the frame.
- **Combo:** grows when an answer matches its call within a window (notes and timing), and resets when it does not. A call is judged on its own too.
- **Groove:** a mistake by either player drops it; it recovers on clean phrases. At zero the song ends early.
- **A round ends when:** the song ends or the groove reaches zero.
- **Result:** Finished, with the shared score and a grade, ranked per song.
- Songs use Persian rhythm patterns (6/8, 5/8, 7/8); harder songs shorten the call and add syncopation.

## Controls

| Keyboard / mouse | Touch |
|---|---|
| Player 1: F / G, player 2: J / K | Each player: two pads on their half, or one phone each |

## Screens

- **Local:** a split screen, the two lanes side by side; nothing is hidden, so both see everything.
- **Online:** one screen per player: their own lane, and in the answer phase the ghosts of their partner's actual hits.

## Offline, online, spectate

- **Offline:** you plus a **bot partner** with good timing that sometimes makes the mistakes you then have to echo (the point of the mode), or two people on one device.
- **Online:** party code, 2 players; input-delay compensation and a calibration screen.
- **Spectate:** both lanes; the replay plays the hits in sync.

## What it forces in the engine

- An audio clock the rules read (hits judged against the song, not the frame)
- Timing windows and latency calibration
- Recording one player's actual input stream during a phrase and showing it to the other (a ghost within a match)
- A mode where online latency is the hard part

## Slices

1. One song, one player, judged hits
2. The second lane, call and answer, the echo combo
3. Groove meter, bot partner, calibration screen
4. Online (with input delay compensation); spectate

## Open questions

- Music source: authored stems, or note charts over one loop?
- Is a missed call note shown to the partner as a ghost gap, or skipped?
