# UI: the Canvas

Xerxes UI works the way Unity's does: a **Canvas** holds the UI elements, and the Canvas is either drawn over the screen, drawn by a camera (it sticks to it), or placed in the 3D world. UI is authored as objects in a scene (`.scene.rs`), so the editor shows it in the Scene view and a game runs the same objects. The runtime is bevy_ui; Dioxus is the editor's own UI and never part of a game ([engine.md](engine.md#games-bevy-only)).

Code: `engine/src/modules/ui/` (`canvas.rs`, `widgets.rs`), added by `launch_project` and by the editor's stage as `UiPlugin`. Editor side: `engine/src/editor/bridge/ui_view.rs` (reads the scene's UI) and `engine/src/editor/stage/ui.rs` (draws it).

## Canvas

```rust
Object::new("HUD")
    .with(Canvas::overlay())                 // Canvas::camera(), Canvas::world(w, h, pixels_per_unit)
    .child(Object::new("Title").with(Text::new("Darius")).with(TextFont::from_font_size(14.0)))
```

| Xerxes | Unity | What it does |
|--------|-------|--------------|
| `Canvas::overlay()` (`ScreenSpaceOverlay`) | Screen Space - Overlay | Drawn over the window by the default UI camera, sorted by `sort_order` |
| `Canvas::camera()` (`ScreenSpaceCamera`) | Screen Space - Camera | Laid out and drawn by the scene's camera (`CanvasCamera` marks which, else the highest camera that draws to the window), so it sticks to that camera and its viewport |
| `Canvas::world(w, h, ppu)` (`WorldSpace`) | World Space | A UI camera renders the canvas into an image of `w` x `h` pixels, shown on a quad at the canvas object's `Transform`, `ppu` pixels to one world unit |

- `Canvas` requires a `Node` (full size by default). Children are ordinary objects with bevy_ui components: `Node`, `Text`, `TextFont`, `TextColor`, `BackgroundColor`, `Button`, `ImageNode`, placed with `RectTransform` (below).
- A root `Node` with no `Canvas` still works: it is a screen-space overlay (older games, the engine's own flow screens).
- **A world-space canvas with no UI under it is a layer for world objects.** Sprites, meshes and bodies under it are placed relative to it and nothing is rendered for it. Darius's `Game` canvas is that: floor, walls, player and coins live under it, and the editor draws its rectangle. A world-space canvas with UI under it renders that UI onto a quad, as above.
- **The reference resolution is a project setting.** The size the UI is designed for is the project's window size (Project Settings, Window; `UiSettings`), not a value on each canvas. `.reference(w, h)` overrides it for one canvas. The editor draws a screen-space canvas that big. (Scaling modes, like Unity's CanvasScaler, are not built yet.)

## Anchoring: RectTransform

`RectTransform` places a UI element against its parent's rectangle, as Unity's does. Y is up; anchors are fractions of the parent (`(0, 0)` bottom-left).

```rust
// 16 px in from the top-left corner, sized by its content (size 0 = content).
.with(RectTransform::anchored(Anchor::TopLeft, Vec2::new(16.0, -16.0), Vec2::new(0.0, 0.0)))
// Fill the parent, inset by left, right, top, bottom.
.with(RectTransform::stretch(0.0, 0.0, 0.0, 0.0))
// Or every field: anchor_min, anchor_max, pivot, position, size.
```

| Field | Unity | Meaning |
|-------|-------|---------|
| `anchor_min`, `anchor_max` | anchorMin / anchorMax | Equal on an axis: the element has a size there. Different: it stretches between them |
| `pivot` | pivot | The point of the element that `position` places |
| `position` | anchoredPosition | The pivot's offset from the anchor, in pixels |
| `size` | sizeDelta | The size on a fixed axis (0 = the content's), or the growth beyond the anchors on a stretched one |

`Anchor` has the nine presets (`TopLeft` ... `BottomRight`). The component owns the element's `position_type`, `left/right/top/bottom`, margins, `width`, `height` and (only when it must) the `UiTransform` translation; the `Node`'s flex fields (`flex_direction`, `padding`, `row_gap`...) stay its own, so a panel can be anchored and still lay out its children as a column.

The pivot is folded into the layout box itself — a margin, or anchoring the far edge — whenever the element's size on that axis is known (a fixed size, or a left/top/right/bottom pivot on a content-sized axis). A `UiTransform` translation is used only for the remaining case, a content-sized axis with a fractional pivot (e.g. a centred, auto-sized element). This keeps any anchor **clickable where it is drawn**: bevy_ui picking hit-tests an element's layout box, so a translation would move the render away from the hit-box — a right/bottom-anchored button that draws in the corner but is clickable off-screen. So prefer a fixed `size` for anchored buttons.

## Widgets

Plain data components that build their own children and write a `WidgetChanged(entity)` message when the user changes them. Restyle them with a `Node` / `BackgroundColor` on the same object.

| Widget | Unity | Data |
|--------|-------|------|
| `Toggle::new(on)` | Toggle | `on` |
| `Slider::new(min, max, value)` | Slider | `value` (drag) |
| `Dropdown::new(["Low", "High"]).selected(1)` | Dropdown | `options`, `selected`, `open` |

Text is `Text`; a button is `Button` plus a child `Text` (a click is `Interaction::Pressed`, as in any Bevy game).

## Mobile inputs: virtual joystick and touch pad

Official touch controls for 2D (and any) games, as elements you put on a canvas. The element's rectangle is the zone, the big box; place it with `RectTransform` like any element.

```rust
// Size a touch zone by proportion (45% x 60% from the lower-left), not in pixels: a phone
// is about 900 x 400 dp, so a 440 px box would cover half of it.
Object::new("Move Stick")
    .with(VirtualJoystick::new(52.0).dead_zone(0.12))
    .with(RectTransform {
        anchor_min: Vec2::new(0.0, 0.0),
        anchor_max: Vec2::new(0.45, 0.6),
        pivot: Vec2::new(0.5, 0.5),
        position: Vec2::new(0.0, 0.0),
        size: Vec2::new(0.0, 0.0),
    })
Object::new("Look Pad")
    .with(TouchPad::new())
    .with(RectTransform {
        anchor_min: Vec2::new(0.55, 0.0),
        anchor_max: Vec2::new(1.0, 0.6),
        pivot: Vec2::new(0.5, 0.5),
        position: Vec2::new(0.0, 0.0),
        size: Vec2::new(0.0, 0.0),
    })
```

| Element | Behaviour | Read it as |
|---------|-----------|------------|
| `VirtualJoystick::new(radius)` | **Floating**: it starts where the first touch lands in the zone, the knob follows the finger up to `radius`, and the output is analog. `.dead_zone(f)` is the fraction of the radius that reads as zero; `.with_mouse()` lets the left mouse button drive it on a desktop; `.source("move")` names it so an action binds it (`Binding::TouchStick("move")`) | `value: Vec2` (-1 to 1, y up), `active` |
| `TouchPad::new()` | What a finger does inside the zone; `.source("look")` names it for `Binding::TouchLook` | `delta` (this frame, y up), `direction` (unit vector from where it landed, once it moved 8 px), `offset`, `held`, `just_pressed`, `just_released` |

- A finger that lands in a zone belongs to that element until it lifts, so one thumb moves and the other aims (several fingers, several elements). Each element is driven by one finger.
- The ring (a hint in the zone's lower-left corner at rest, then where the thumb landed), the knob and a hairline frame around the zone are drawn only once a touch has been seen (`TouchSeen`), so a desktop player sees nothing. They are deliberately faint: UI blends in linear space, so even a 1% white fill lifts a dark background by about ten levels. The editor sets `TouchSeen` so the zones show in the Scene view.
- Games read the components and combine them with keys: Darius's `steer_player` adds the stick's `value` to the keyboard direction and clamps it to length 1, so a small push walks slowly (`games/__templates__/darius/src/top_down/world.rs`). The zone owns its `BackgroundColor`.
- The Add menu has Virtual Joystick and Touch Pad (UI group).
- Tested headless (`modules/ui/touch.rs`): the stick floats to the first touch, is analog and clamped, ignores touches outside its zone, honours the dead zone; the pad reports deltas and direction; two fingers drive two elements.

## Authoring rules (so the editor can show it)

- **Scenes are plain data.** A scene is `SceneAsset { objects: vec![Object::new(..).with(..).child(..)] }` written out in full. A helper function that returns an `Object` (`label(..)`, `block(..)`) is not read by the editor: the scene does not load and the Console says `unknown object constructor`. Use `const` items for repeated colors and numbers (the editor reads a file's own `const NAME: T = ..;`).
- **UI elements have no Transform.** A UI element (a `Node` or `RectTransform` below a canvas) is placed by its layout, so the Properties panel shows its Node / RectTransform section and no Position / Rotation / Scale (an element that does carry an authored `Transform` still lists it). A canvas root keeps its Transform: it places the canvas in the scene.
- **What the editor reads:** `RectTransform::anchored(..)`, `RectTransform::stretch(..)` or a struct literal of it; `Node { .. }` fields written as `px(..)`, `percent(..)`, `vw/vh(..)`, `auto()`, `UiRect::all/axes/new/horizontal/vertical/top/..`, `BorderRadius::all(..)` and the layout enums (`FlexDirection`, `JustifyContent`, `AlignItems`, `AlignSelf`, `PositionType`, `Display`, `FlexWrap`); `Color::srgb/srgba/srgb_u8`, `Color::WHITE/BLACK/NONE`; `Text::new("..")`, `TextFont::from_font_size(..)`, `TextColor(..)`, `BackgroundColor(..)`, `BorderColor::all(..)`, `ZIndex(..)`, `Visibility::Hidden`; `Canvas::..`, `Toggle`, `Slider`, `Dropdown`. Anything else stays as source text in the file and is not drawn.
- **Logic finds elements by marker components** (`ScoreText`, `HudButton(Action::Restart)`), not by spawning them. The scene is the one source of what the level looks like; the game code fills in text and reacts to buttons.
- **Text a system fills in** (a score, a timer) should carry a placeholder in the scene (`Text::new("Coins 0 / 8")`) so the editor shows the layout.

## In the editor's Scene view

Every canvas is shown the way Unity shows it: a rectangle in the scene, centred on the canvas object's origin, `reference_resolution` big. Its UI is the real bevy_ui, laid out and drawn by the same code the game runs (a world-space canvas), so the Scene view is faithful including text and rounded corners.

- In a **2D scene** one pixel is one world unit (a 1280 x 720 canvas is 1280 x 720 units, at the origin). In a **3D scene** the canvas is shown at 100 pixels to the unit (12.8 x 7.2), so it does not cover the scene.
- Each canvas has a frame drawn around it (Gizmos menu, Canvas Frames). Clicking a UI element selects it (the smallest element under the pointer wins, so a button beats its panel); the selection is outlined; the Outliner and Properties work as for any object. `Visibility::Hidden` elements are not drawn.
- The **Rect tool (T)** moves and resizes the selected sprite or UI element by its corner and side handles; the drag is one Undo step and rewrites the scene file. What it writes depends on how the element is placed:
  - a `RectTransform` element: its `RectTransform` (position, size);
  - a **flexbox `Node`** (Darius's menu: a column of rows of tiles, no `RectTransform`): its parent keeps the layout, so a move is an offset (`left` / `top`, which `PositionType::Relative` keeps in the flow) and a resize is `width` / `height` in px (dragging the left or top edge also shifts the offset, so the opposite edge stays). A `PositionType::Absolute` node is pinned from its parent's corner: `left`, `top`, `width` and `height` are written and `right` / `bottom` go back to auto;
  - a sprite: its `SpriteDef` size and its position.
- **W / E / R on a UI element** use the same machinery, because a UI element has no Transform: **Move (W)** drags the element's body (the offset rules above), **Scale (R)** drags from the body away from or toward the element's centre and writes `UiTransform.scale`, **Rotate (E)** turns it around its centre and writes `UiTransform.rotation` (degrees, clockwise on screen). Both are one Undo step; the outline follows the rotated or scaled element. Sprites and 3D objects keep the arrow / ring / box gizmos and their Transform.
- The Scene view is not interactive: buttons, sliders and drop-downs do not react to the pointer there (use Play).

## Where things are in the Scene view

- **Screen frame:** in a 2D scene the project's screen (its window size, Project Settings) is drawn as a lighter area behind the scene, centred on the origin, one pixel to one unit. An overlay canvas lives in it: that is where the HUD and the joystick zone are placed. (Gizmos menu, Canvas Frames.)
- **Camera frame:** an orthographic game camera shows an outline of the part of the world it frames on that screen (`AutoMin`, `AutoMax`, `Fixed`, `FixedVertical`, `FixedHorizontal` or window size, times `scale`), so the level can be placed against what the player will see (Gizmos menu, Camera and Light Icons). Darius's camera shows 1031 x 580 of the world on a 1280 x 720 screen.
- **Canvas frames:** each canvas has an outline; a joystick or pad zone has its own outline.

## Properties panel editors

Like Unity's inspectors, for the components that are written as a call in the scene file (`ui/components.rs`):

| Component | Editor |
|-----------|--------|
| `Canvas` | Render Mode (Screen Space - Overlay / Screen Space - Camera / World Space), Sort Order, Render Camera (the active camera, or one of the scene's cameras by name), Reference Size (the project's, or custom), and for World Space its size and pixels per unit |
| `Node` (titled **Rect Transform**) | Rotation and Scale (the element's `UiTransform`), Position (in flow / absolute), Width, Height (auto / px / %), the offset (`left`, `top`; `right`, `bottom` when absolute), Children (row / column), Justify, Align, gaps, Padding, Margin, Border (L R T B px) and Corner Radius. It is the flexbox element's layout, written as the scene's own `Node { .. }` fields |
| `BackgroundColor`, `BorderColor`, `TextColor` | A swatch (click it for the **color picker**, a window that floats by the right edge of the editor so the Properties panel's height and scroll do not matter, and closes with its button or when another object is selected: a hue wheel with a saturation / value square inside, R G B A bars with 0 to 255 numbers, the old color beside the new one; drag on the wheel, the square or a bar), and a hex field (`#RRGGBB[AA]`). The picker is built from clickable cells, so it needs no pointer coordinates and behaves the same in the browser and in Blitz (`ui/colorpicker.rs`). A file constant (`BackgroundColor(PANEL)`) shows by name with *Edit*, which makes the color this element's own literal |
| `Text` | The string |
| `TextFont` | The size. The font itself is not a project asset yet (see below) |
| `RectTransform` (titled **Anchors**) | The nine anchor presets and Stretch, Position, Size, Anchor Min / Max, Pivot |
| `SpriteDef` | Sprite image (none: a plain color), Color (a tint when there is an image; a file constant shows by name and can be made a literal), Size |
| `GameCamera2D` / `GameCamera3D` | Type, Active, Projection: Perspective or Orthographic. Authored gameplay cameras (they pull in Bevy's `Camera2d` / `Camera3d`); a 2D camera set to perspective becomes a 3D game camera, a 3D camera set to orthographic keeps its type and gets an orthographic `Projection`. Orthographic framing uses `ScalingMode` from the engine prelude (`AutoMin`, `AutoMax`, `Fixed`, …) — not a `bevy::camera` import. Editor Scene-view cameras are `ViewportCamera2D` / `ViewportCamera3D` and are never saved in a scene |

## Colliders follow the sprite and the scale

`Body2d::fixed_fit()`, `dynamic_fit()` and `kinematic_fit()` make a box collider that fits the object's sprite (`SpriteDef` size), so resizing the sprite with the Rect tool resizes the collider. Any collider is also scaled by the object's world scale when the body is created, so a player scaled up in the editor is as big to the walls as it looks. Bodies work under a parent (a canvas or a group): their position is the world position, and they write back their local transform.

## Not built yet

| Item | Note |
|------|------|
| Font assets | `TextFont` uses the engine's default font. A project font needs a typed asset (`Name.font.rs` beside the `.ttf`, in the bundle like images) and a way for a scene to name it; until then the Properties panel only edits the size |
| Pointer interaction with world-space canvases in a game | Ray from the pointer to the quad, converted to UI coordinates and fed to bevy_ui's focus. The surface is drawn already |
| `CanvasScaler` (scale with screen size), layout groups, anchor presets picker in Properties | Today: bevy_ui's flexbox (`Node`) and `RectTransform` written in the `.rs` |
| Multi-touch gestures (pinch), haptics | Mobile inputs later slice. On-screen buttons are `TouchButton("id")` on a `Button` (bound as `Binding::TouchButton`), and gamepads are in the input manager ([input](input.md)) |
| `InputField`, `ScrollView`, `Scrollbar`, `Image` / `RawImage` widgets, `TextMeshPro`-style rich text | Stage 3.10 (Text and UI kit); `InputField` needs the soft keyboard on Android |
| Drop-down list drawn above its siblings in world space | Uses `GlobalZIndex`, which is per canvas camera |
