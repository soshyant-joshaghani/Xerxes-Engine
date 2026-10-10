# Input: pointers, touch and pointer lock

How input reaches a game, what is solid today, and the pointer-lock design we want later. Status of the second half: **documented, not scheduled** (it is too early; nothing here is built).

## Today

- **Named actions.** Gameplay reads `actions.pressed("jump")` and `actions.axis("left", "right")`, never raw keys or buttons. The bindings (keys, mouse buttons, gamepad axes) live in the project settings, so remapping is one place on every platform.
- **Mouse on Android.** Stock winit 0.30 reports an Android mouse as touches (no right or middle button, no wheel). `engine/winit/` carries one patch (see `engine/winit/XERXES.md`) so a mouse is a mouse there: cursor position, left, right and middle buttons, and the wheel, which reach Bevy as normal mouse events. Every new game carries the same `[patch.crates-io]` line, so a game can support a mouse on a phone. The editor uses it for its camera (right-drag look, middle-drag pan, wheel dolly).
- **Touch** reaches Bevy as touch events, as usual.

## What a game must still handle

The patch makes the mouse *arrive*; two things are the game's job until the engine does them:

1. **Mouse and touch are separate paths.** A mouse press must not also count as a tap, and a touch must not move a "mouse cursor". Read each from its own Bevy events, or map both onto one named action on purpose.
2. **The cursor is always visible and free.** There is no lock yet, so a look-around that needs the mouse to keep moving past the screen edge (an FPS camera) is not possible. Right-drag look, as in the editor, works because the cursor stays visible.

## Pointer lock (later)

**Goal:** one clean API that every platform honours, with menus and the OS always winning.

```text
PointerLock { Free, Locked }          a resource the game sets (Locked while looking around)
```

- **While locked:** the cursor is hidden and held; the game reads **relative motion** (`AccumulatedMouseMotion`), not an absolute position.
- **It unlocks by itself, always, when:** a menu, the pause screen or any bevy_ui panel that needs the pointer opens; the window loses focus or the app goes to the background; the player presses the platform's escape (Esc on the web and desktop, Back on Android); a text field takes focus; the game changes scene (Warp) unless the new scene asks to lock again.
- **It relocks** only when the game asks, and on the web only from a user gesture (a click), because browsers require one.
- **Spectating, menus and the editor** never lock.

How each platform does it (to verify when it is built):

| Platform | Mechanism | Note |
|---|---|---|
| Windows, macOS, Linux | winit's cursor grab through Bevy's window cursor options (`Locked` or `Confined`, per what the OS supports) plus a hidden cursor | winit's grab modes differ per OS; the engine picks the best one available |
| Web | the browser Pointer Lock API (`requestPointerLock`) | needs a user gesture; the browser unlocks on Esc |
| Android | Android pointer capture (`requestPointerCapture` on the activity's view, API 26 and up, which is our minimum SDK); captured motion arrives as relative mouse events | winit does not do this today: it would be a second Xerxes patch in `engine/winit/`, next to the mouse one |
| Phone, touch only | no pointer to lock: the same "look" action is driven by a touch stick or a drag area | a game that supports both gives each its own source for the same named action |

## Design notes for when it is built

- **Named actions stay the contract.** A `look` action has a mouse-delta source, a gamepad stick source and a touch source; gameplay reads the action, so it works whichever device is active.
- **One owner for the lock.** The UI layer and the game both ask the engine to lock or unlock; the engine decides (the UI always wins), so a forgotten unlock can never trap the player.
- **Active-device tracking.** The UI kit already needs to know whether the last input was keyboard, gamepad, mouse or touch (prompts that follow the device); the lock logic uses the same signal.
- **Tests.** The state machine (lock requests, the unlock triggers above) is plain Rust and unit-tested without a window; the platform calls are checked by hand on a Windows build, a browser and the phone.

## Where it fits in the plan

It is not in any stage yet. It becomes relevant with Cyrus (Phase 4) and the FPS (Phase 5), and with the UI kit (stage 3.10) for the unlock-on-menu rule. When it is scheduled it gets its own stage in [PROGRESS.md](../__plans__/PROGRESS.md); until then it is listed there under "not scheduled".
