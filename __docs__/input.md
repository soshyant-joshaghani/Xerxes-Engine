# Input: actions, devices and pointer lock

How input reaches a game, what is solid today, and the pointer-lock design we want later. The input manager (actions and bindings, below) is **built**; pointer lock is **documented, not scheduled**.

## The input manager (`modules::input`)

Gameplay never reads a key, a button or a stick. It reads **named actions**, and each game says in its `<name>.project.rs` how every physical device drives them. The action names are the contract; the bindings are per game, so Darius and Cyrus map the same engine to their own controls.

```rust
input: InputMap::new()
    // A 2D value from every device at once.
    .vector2("move", [
        Binding::wasd(), Binding::arrows(),        // keyboard
        Binding::PadStick(PadStick::Left),         // gamepad analog stick
        Binding::PadDpad,                          // gamepad D-pad
        Binding::TouchStick("move"),               // on-screen stick: VirtualJoystick::new(..).source("move")
    ])
    // Mouse movement, the right stick and a touch look pad, as one look value.
    .vector2("look", [Binding::mouse_look(0.12), Binding::PadStick(PadStick::Right),
                      Binding::TouchLook { id: "look", scale: 0.2 }])
    .button("jump", [Binding::Key(KeyCode::Space), Binding::Pad(GamepadButton::South),
                     Binding::TouchButton("jump")])     // a TouchButton("jump") on a Button
```

```rust
fn steer(actions: Res<Actions>) {
    let wish = actions.vector("move");            // y up, clamped to a unit
    if actions.just_pressed("jump") { /* ... */ }
    let zoom = actions.value("zoom");             // an axis action
}
```

| Kind | Builder | Gives |
|---|---|---|
| Button | `.button(name, ..)` (also `.action`) | `pressed`, `just_pressed`, `just_released`; `value` is 0 to 1 (a trigger is analog) |
| Axis | `.axis(name, ..)` | `value(name)`, -1 to 1 |
| Vector2 | `.vector2(name, ..)` | `vector(name)`, y up |

| Device | Bindings |
|---|---|
| Keyboard | `Key`, `KeyAxis { negative, positive }`, `Keys { up, down, left, right }` (`Binding::wasd()`, `Binding::arrows()`) |
| Mouse | `Mouse(button)`, `MouseMotion { scale }`, `MouseWheel { scale }` |
| Gamepad | `Pad(button)` (triggers are analog), `PadAxis`, `PadStick(Left or Right)`, `PadDpad` |
| Touch | `TouchStick(id)`, `TouchLook { id, scale }`, `TouchButton(id)`, `TouchDpad { up, down, left, right }`; the id is the on-screen control's `source` (`VirtualJoystick`, `TouchPad`) or a `TouchButton("id")` component |

- **Several devices at once.** Every binding of an action is summed. Normal sources (keys, sticks, buttons) are clamped to a unit together, so WASD plus a stick does not run faster; motion sources (mouse delta, wheel, touch look) are deltas and are added after, unclamped.
- **Dead zone.** Gamepad sticks have a radial dead zone of `PAD_DEAD_ZONE` (0.15), rescaled so the edge reads 0. The on-screen stick has its own (`.dead_zone(f)`).
- **Edges.** `just_pressed` and `just_released` come from the buttons' own frame edges (a tap shorter than a frame is still seen) and from the action's value crossing `PRESS_THRESHOLD` (0.5), so analog and on-screen sources have edges too.
- **Active device.** `actions.device()` (and the `ActiveDevice` resource) is the device used last (keyboard, mouse, gamepad or touch), for prompts that follow the player's device.
- **`ActionDef::on(device)`** lists the bindings of one device, and `Binding::label()` gives short text (`W A S D`, `Pad South`), for a settings screen.
- **Where it lives.** `InputMap` and `Binding` are in `engine/src/modules/input/`, installed by `launch_project` (`ActionsPlugin`) and re-exported from the prelude. The rules (`Actions::update(&RawInput)`) are unit-tested without a window; `tests/engine/assets.rs` runs the real plugin.
- **The editor** lists each action, its kind and its bindings in Project Settings (read only for now; the file is edited as text).

### Darius and Cyrus

Both have their own `input` map in `assets/<name>.project.rs`. Darius: `move` (keyboard, left stick, D-pad, the footer's on-screen stick and D-pad), `pause`, `respawn`, `exit_to_menu`, and Qanat's `cell_*`, `turn`, `undo`, `clear`, `open`; its world and HUD read only these. Cyrus: `move`, `look` (mouse, right stick, touch look pad), `jump`, `sprint`, `interact`, `fire`, `pause`, `respawn`, `exit_to_menu`. Cyrus has no gameplay yet, so its map is the contract its first 3D controller will read.

### On phones

- **Mouse** reaches Bevy as mouse events through the patched winit (`engine/winit/`): cursor, left, right and middle buttons and the wheel.
- **Keyboard** (a Bluetooth or USB one) arrives as key events on Android and works as is.
- **Gamepad.** Bevy's gamepad backend (gilrs) has no Android support, so `modules::input::android` does it: the controller's buttons are key events with no key name (`Unidentified(Android(96))` is A) and are named there; its sticks, triggers and hat are motion events that stock winit reports as a phantom touch, so the vendored winit keeps them and hands them out (`winit::platform::android::gamepad_axes()`). The result fills the same `RawInput` as desktop gamepads, so the same bindings work. The D-pad also arrives as the arrow keys. **The mapping is unit-tested; it has not been run on a device yet** (no Android NDK on the dev machine, see the log).
- **Touch** is the on-screen controls, which feed actions by id.
- On the **web**, gamepads come through the browser's Gamepad API (gilrs on wasm); mouse, keyboard and touch as usual.

## What a game must still handle

The patch makes the mouse *arrive*; two things are the game's job until pointer lock is built:

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

- **Named actions stay the contract.** The `look` action already has a mouse-delta source, a gamepad stick source and a touch source (see above); pointer lock only changes how the mouse delta is produced.
- **One owner for the lock.** The UI layer and the game both ask the engine to lock or unlock; the engine decides (the UI always wins), so a forgotten unlock can never trap the player.
- **Active-device tracking.** `ActiveDevice` (built with the input manager) is the signal the UI kit's prompts and the lock logic share.
- **Tests.** The state machine (lock requests, the unlock triggers above) is plain Rust and unit-tested without a window; the platform calls are checked by hand on a Windows build, a browser and the phone.

## Where it fits in the plan

It is not in any stage yet. It becomes relevant with Cyrus (Phase 4) and the FPS (Phase 5), and with the UI kit (stage 3.10) for the unlock-on-menu rule. When it is scheduled it gets its own stage in [PROGRESS.md](../__plans__/PROGRESS.md); until then it is listed there under "not scheduled".
