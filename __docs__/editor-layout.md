# Editor layout: Blender-style areas, workspaces and headers

Design for the engine app's shell (stage 2g in [PROGRESS.md](../__plans__/PROGRESS.md)). Research done 2026-10-10 from Renzora, Fyrox, Jackdaw, BerryCode, Bedina and our own editor code.

**Update, later on 2026-10-10:**
- **Joins** (`layout/join.rs`) replaced the sibling-only rule: any two neighbours that share a whole edge (lines within `SNAP` = 3 px count as one) join. The smallest subtree holding both is turned into rectangles, merged, and cut into a tree again; the rest of the tree keeps its shape. A result that straight cuts cannot make (a pinwheel) is refused. The edge menu finds the two areas on either side of where you clicked.
- **Saved layouts (2g.3):** `layout/layouts.rs` (`Layouts`: all workspaces + the active one, JSON, repaired on load), shipped layouts Layout, Assets, Scripting, Animation (`layout/presets.rs`), the tab bar `ui/workspaces.rs` (click a tab to switch, click the showing one or right click for Rename / Duplicate / Reset Layout / Delete, `+` for a copy or a shipped layout, Reset All Layouts), persisted by `editor/prefs.rs` (local storage on the web, `%APPDATA%\Xerxes\layouts.json` or the config folder natively, the app's storage on Android; `XERXES_CONFIG` overrides). Tabs have a fixed width so menus are placed without measuring.
- **Browser right click:** the host page (`engine/index.html`) cancels the browser's context menu everywhere except text fields; `prevent_default` in a Dioxus handler did not stop it.

- **Top bar (2g.3, finished):** `File Edit Window Help` (`ui/appmenu.rs`) then the layout tabs, then the project controls, all in the one top bar (the separate tab row is gone). File: Project Manager, Save. Edit: Undo/Redo (marked soon), Add Empty, Duplicate, Delete. Window: Rename, Duplicate, Reset Layout, Reset All Layouts. Help: About, Keyboard Shortcuts (printed to the Console). Their buttons and the tabs have fixed widths (`theme::MENU_WIDTH`, `TAB_WIDTH`, `tabs_left()`), which is how menus and the tab drag are placed without measuring; if the top bar's padding or gap changes, change `tabs_left()` with it. Dragging a tab reorders (`Layouts::move_tab`), a click without moving switches, and a click on the showing tab opens its menu. A rename ends on Enter, Escape or a click on an area.

- **Several Viewports and header menus (2g.4):**
  - **Cameras:** `stage/camera.rs` keeps one editor camera per Viewport area (`ViewportArea`, at most `MAX_VIEWPORTS` = 4), each with its viewport on its area. The camera under the pointer carries `ActiveCamera`; navigation, picking and the gizmos use it. While a mouse button is held, the Viewport the press started in stays active. Gizmo drawings (grid, selection) are shared by all cameras.
  - **Poses:** a camera's pose is remembered per layout name and area for the session (it survives switching editors in the area and switching layouts). Renaming a layout resets its cameras; the poses are not saved to disk yet.
  - **Header menus:** an area's header has the editor-type button, then that editor's menus (`headers::menus`). The Viewport has View (Frame Selected, Reset View, Top, Front, Right), Select (None), Add (Empty), Object (Duplicate, Delete); Wireframe, Orthographic, Select All, Add Mesh/Light/Camera are shown as soon. View commands go to that Viewport's camera through `EditorCommand::ViewAction`. Splitting a Viewport now gives another Viewport (an `Empty` area from the fifth).
  - **Not done:** per-viewport shading (wireframe, solid, material), orthographic toggle, and a per-viewport grid toggle (the grid is a shared gizmo).

- **Undo, keymap, palette (2g.5):**
  - **History:** `editor/history.rs` is a stack of steps, each holding the scene and the selection before and after (snapshots, not inverse operations: scenes are small). `ui/edit.rs` is the only place that changes the scene, and records every change. Edits with one merge key (`transform:{id}` while a gizmo drags, `field:{id}:{type}:{name}` and `rename:{id}` while typing) fold into one step until it is sealed: the drag ends (the stage stops reporting `dragged`), another object is selected, an undo or redo happens, or the scene is saved. A merged step keeps the label of its first edit. The scene is dirty exactly when it differs from the saved state, so undoing back to it is clean again. History is cleared when a scene opens. It does not cover layout changes.
  - **History editor:** lists the steps newest first; click one to go to the scene right after it, or "Original".
  - **Keymap:** `Action`s (`protocol::Action`) are bound to chords in a table (`keymap::DEFAULTS`); a `keymap` preference (JSON: action id to a list of chords) replaces the keys of the actions it names and leaves the rest, read once at start. The stage (`stage/shortcuts.rs`) matches keys (Ctrl also means Cmd) and publishes the action with the area under the pointer; the UI runs it. Keys are ignored while a UI text field has focus. W/E/R/X/F and the camera keys stay the Scene view's own and are not in the keymap yet; there is no UI to rebind keys yet.
  - **Commands:** `ui/commands.rs` is the one list of things the editor can be told to do (name, whether it can run now, its key, how to run it). The Edit menu, the Viewport header menus, the keys and the palette all use it. The palette (F3) filters it as you type, Enter runs the first available match.
  - **Web note:** a `onmounted` focus puts the cursor in the palette on the web (`autofocus` only works at page load); natively a click focuses the field.

- **Touch and phones (2g.6):**
  - **Touch mode:** `layout::touch()` is one flag for the whole editor (the UI and the Scene view must agree on sizes). It is on for Android, on the web when `(pointer: coarse)` matches, and elsewhere from the first finger that touches the screen (it then stays on). It makes `header()` 44 (26), `edge()` 22 (6), `theme::toolbar()` 52 (38) and appends `TOUCH_CSS` (taller rows, buttons and fields, 14 px text). The widths that menus are placed by do not change. `theme::Layout` carries the flag, so the UI re-lays out when it flips.
  - **Fingers:** touch already acted as the left mouse button. There is no right button and no long-press: tap an edge (a press that does not move) for its menu, tap the layout tab that is showing for its menu. Headers scroll sideways by touch drag when their buttons do not fit.
  - **Phone layouts:** `Phone` (landscape: Viewport with Outliner over Properties beside it) and `Phone Portrait` are in the `+` menu. On the first run (no saved layouts) in a window under 600 logical px high or 700 wide, the editor starts with one of them. Any area can be maximized (Max) or switched to another editor.
  - **QA hooks (native):** `XERXES_WINDOW=WxH` opens a window of that logical size and `XERXES_TOUCH=1` starts in touch mode, to check a phone-sized editor on a desktop (with `XERXES_OPEN` and `XERXES_SCREENSHOT`).
  - **Not verified:** a real Android device or the APK build (this machine has no NDK, adb, cargo-apk or Android Rust target). The browser pane's phone emulation was not usable (the pane pauses the page when it is hidden). Things to check on a device: tap and drag on edges and tabs, the soft keyboard in the palette and rename fields, the size of the targets (raise the touch numbers in `layout/geom.rs` and `theme.rs` if they feel small), and the top bar, which is wider than a phone (Save, Play and the project buttons scroll off to the right; File menu has Save).

- **Layouts per project:** each project keeps its own layouts under `layouts-<project name>` in the preferences (a file name natively, so only letters, digits, `-` and `_`). Opening a project loads them; a project with none starts from the old global `layouts` entry (a starting point; no longer written) or the shipped layouts, and the phone rule applies to it. Switching projects never saves one project's layouts under another's key (`layouts_for`).
- **More editors:**
  - **Text Editor** (`ui/texteditor.rs`): opened by clicking a source, logic, prefab or text file in the Asset Browser, or File > Edit Scene as Text. Save (button or Ctrl+S in the field), Revert; template games are read only; opening another file while this one has unsaved changes is refused. Saving the open scene's file reloads the scene (and clears its undo history). Ctrl+S is also kept from the browser's save dialog by the host page.
  - **Image Editor** (`ui/imageeditor.rs`): shows an image with Fit and 25% to 400% zoom; PNG files give their size (`browse::png_size`). It does not edit pixels.
  - **Project Settings** (`ui/projectsettings.rs`, with `project/settings_text.rs`): the settings file's title, window size and resizable, and backend URLs are fields; Apply replaces only those values in the text, so comments and the rest stay as written. Values with quotes or backslashes are refused. `WindowSettings::default()` and `BackendSettings::default()` show a Customize button that writes them out as structs. Levels, input actions and preloaded bundles are listed (change them in the Text Editor, one click away).
  - **Making room:** `show_editor` opens an editor that is not on screen in an Empty area, else an area showing another document (text, image, settings), else the Console, History or Properties. The area can be switched back with its editor-type button. The header button shows a short name (`EditorKind::short_title`) because the button has a fixed width.
  - **Async trap:** a task started with `spawn` ends when the component that started it goes away (a menu item closes at once). Editor-level work (open, save) uses `dioxus::core::spawn_forever`.

- **Game Preview** (`ui/gamepreview.rs`, `project/jobs.rs`): the toolbar's Play (Stop while it runs), Build and Publish, and the same buttons plus the platform (Web, Windows, Android) and a Log toggle in the editor. A job is `game dev|build|publish <project> <platform>` (`template ...` for a template game). On the web it is run by the backend on the host (`/api/v1/engine/jobs`, which uses `__ctrl__/.venv`'s Python; `XERXES_PYTHON` overrides); in the native editor it is a process of its own (`jobs::local`, needs the repo: `store::repo_root`), stopped when the editor closes. The editor follows a job once a second (`jobs::api::pause`: a page timer on the web, a sleeping thread natively) and shows the output. A running web job is shown in a frame at the address the pipeline prints (`jobs::preview_url`: only its own `[xerxes]` line counts, not the proxy addresses in the dev server's output); natively, an `Open in browser` button instead (Blitz has no frames). A finished job shows the `dist/...` path its `[xerxes]` line names (`jobs::output_path`). The id of the last job is kept in the preferences so a reloaded page follows it again (`gamepreview::resume`). The palette has Play/Stop, Build, Publish. One job at a time. QA hooks (native): `XERXES_PLAY=dev|build|publish` with `XERXES_PLATFORM` starts a job after `XERXES_OPEN` opens the project.

- **Spreadsheet** (`ui/spreadsheet.rs`, rows in `project/table.rs`): the open scene's objects as a table, parents before children (indented), a prefab instance taking its name, Transform and component count from its prefab; filter by name or kind; a click selects the object. Read only: values are changed in the Properties.
- **Preferences** (`ui/preferences.rs`, `editor/touch.rs`, `editor/keymap.rs`): Touch sizes Auto/On/Off (kept as the `touch` preference; Auto decides from the device and the first finger), and the keymap: each action with its keys as text, Set (checks every key, saves only what differs from the defaults, takes effect at once through `keymap::revision`), Reset, and a warning when two actions share a key.

**Built (2026-10-10): 2g.0, 2g.1, 2g.2.** Code: `engine/src/editor/layout/` (model, solver, preset), `engine/src/editor/ui/shell.rs` (areas, headers, menus, edge handling), `stage/camera.rs` (`fit_viewport` solves the same workspace), tests in `tests/engine/layout.rs`. Differences from the plan below, found while building:
- Blitz mouse events only have `client_coordinates` (element and page ones panic), so drags use client coordinates minus the screen corner (`Shell { x, y, .. }`).
- No long-press timer yet: a click on an edge that does not move opens the edge menu, which reaches it on touch without a right button. Right click also opens it.
- Join semantics: on an edge between A (left/top) and B, "Join Right/Down" lets A grow over B, "Join Left/Up" the reverse. Both need single areas on both sides.
- Close (x) removes an area and its sibling takes the space (works for any sibling, not only single areas); Max/Min toggles the maximized area.
- A second Viewport is refused in the menu (greyed) until stage 2g.4; splitting the Viewport gives an `Empty` area.
- Keyboard shortcuts (Ctrl+Space, Esc to cancel a pick) are not wired: a right click cancels a pick.

The goal: the **high concepts of Blender's UI** (any area can become any editor; area headers with their own menus; right-click an edge to split/join/swap; workspaces as saved layouts; an app menu bar), with **the contents of the panes** taken from what Renzora and Fyrox show an engine editor needs. Dioxus stays the UI, one source for web, Windows and Android ([engine.md](engine.md#the-editor-one-source-every-platform)). Web first; Android is verified at the end.

## Vocabulary (Blender's, used everywhere)

| Term | Meaning here |
|------|--------------|
| **Screen / Workspace** | A named layout: one tree of areas. The tabs at the top (Layout, Modeling, ...) switch workspaces. |
| **Area** | A rectangle that shows exactly one editor. Has a header and a body. No tabs inside an area (that is the difference from Renzora/Fyrox/Jackdaw docking). |
| **Editor** (Blender: "space") | What an area shows: Viewport, Outliner, Properties, Assets, Console... Chosen with the button at the header's left. |
| **Edge** | The border between two areas. Drag = resize. Right-click / long-press = Area Edge Options (split, join, swap). |
| **Header** | The area's own toolbar: editor-type button, then that editor's menus (View, Select, Add...), then its right-side controls. |

## What the research says

| Repo | Verdict | Take | Do not take |
|------|---------|------|-------------|
| **Renzora** (Bevy 0.19, early alpha, MIT/Apache) | Main reference for *content* and for data-model ideas | `DockTree` as a serde enum (`Leaf`/`Split{direction, ratio}`/`Empty`) saved as JSON with `#[serde(default)]` for forward-compatible files; workspaces as a ribbon of named trees with *Reset Layout / Reset Workspace* in a View menu; panels as string ids in a registry (`id, title, icon, category`, plus min size) feeding an "Add panel" picker grouped by category; panel list (hierarchy, inspector, assets, console, history, viewport 1-4, camera preview, timeline, animation, material/shader/blueprint graphs, network monitor, physics debug, navmesh, tilemap, 2D view); viewport toolbar (tools, snaps, shading ladder Wireframe/Solid/Material/Rendered, orientation gizmo, up to 4 viewports); command-based undo with per-document stacks (`Scene`, `MaterialGraph(path)`...), merge of consecutive edits to one field, `seal` on mouse-up; inspector as a registry of `FieldDef`s with get/set; Ctrl+P command palette; rebinding stores only changed keys | Its UI layer (own bevy_ui "ember"), tab-docking drag UX, dynamic-linked native plugins, Lua/Rust scripting (out of our plan), the global overlay bottom dock, Bevy 0.19 APIs (we are on 0.18). It also has a wasm-only editor build (`renzora_editor_app`): worth reading later for what a web editor over Bevy costs. |
| **Fyrox** (own retained UI, MIT, the most mature editor) | Reference for *robustness* | Layout descriptor tree (`Empty`/`Window(name)`/`MultiWindow{index,names}`/`SplitTiles{splitter, orientation, children:[2]}`) plus floating windows: windows are referenced by **name**, and a name that no longer exists restores as `Empty` instead of failing; `CommandTrait { name, execute, revert, finalize, is_significant }` with `CommandStack{commands, top, max_capacity}`, `CommandGroup`, selection commands that are not "significant" | Its widget library and message passing (tied to its own UI). Could not read `fyrox-ui/src/dock/tile.rs` (drag-docking flow) in this pass: read it before building tab-style docking, which we do not need. |
| **Jackdaw** (Bevy 0.19, MIT/Apache, early) | Closest *shape* to Blender | Separate crates per concern (`jackdaw_panels`, `_commands`, `_widgets`, `_snap`, `_select`...); `WindowRegistry` of descriptors (`id, name, icon, default_area, priority, build`); `WorkspaceDescriptor{id, name, icon, accent_color, tree}` + `WorkspaceRegistry{workspaces, active}` persisted with the project; two tabs of the same editor are told apart by a per-instance `TabId` (= our `AreaId`); `DockAreaStyle::Headless` ("the panel provides its own header") is exactly Blender's model; a CLI and an editor/runtime crate split. Its README says it follows the official Bevy editor Figma | Brush/CSG/terrain tools (not our scope yet), Esc/R/T tool keys. Docking is not documented in its README. |
| **BerryCode** (egui IDE for Bevy) | Little for layout | BRP-based inspector idea (talk to a *running* game), mobile toolchain detection, tabbed bottom panels | egui, VS Code layout |
| **Bedina** (DioxusLabs, 4 commits) | Confirms our host | Two examples: Dioxus-Native owns the window with Bevy rendering to a texture, or **Bevy owns the window with Dioxus-Native rendering to a texture** (our model). Both share one wgpu device | Nothing to copy now. Revisit if Blitz paint cost shows up in profiles: it is the official place that shares a GPU device between the two. |
| **dockviewers** (Dioxus crate) | Rejected | | Web-only (DOM), packed-grid tiles. We need Blitz too, and Blender's binary split tree. |

## Constraints found in our own code (these shape everything)

1. **The Scene view is a Bevy camera viewport under a transparent Dioxus layer.** Today `theme::Layout` computes the panel sizes in Rust and *both* sides use it: CSS for the panels, `camera::fit_viewport` for the camera. The Blender layout must keep that rule: **the area tree and its solver are pure Rust data**, used by the UI (to place areas) and by the stage (to place every viewport camera).
2. **Blitz cannot be asked where things are.** In `dioxus-native-dom` 0.7.10, `convert_mounted_data`, `convert_pointer_data`, `convert_drag_data` and `convert_scroll_data` are `unimplemented!()` (calling them panics). So: no `onmounted`/`get_client_rect`, no `onpointer*`, no `ondrag*`. Allowed: `onmousedown/move/up`, `onclick`, `onwheel`, `onkeydown`. Rects must be **computed**, never measured.
3. **Right click exists, `contextmenu` does not.** Blitz delivers the secondary button as a mouse event (`MouseEventButton::Secondary`). On web we also `prevent_default` on `oncontextmenu` so the browser's menu stays away.
4. **Touch is turned into mouse events** (`host/native.rs` `touch()`, `stage/touch.rs`). So touch has no right click: **long-press** (about 500 ms, under 8 px of movement) is synthesized into the secondary action by our own timer, in one place.
5. **No pointer capture**, so a drag that leaves the handle would lose events. While an edge drag, an area split or a drag-to-swap is active we mount a full-window transparent capture layer that owns `onmousemove`/`onmouseup`. Same code on web and native.
6. **CSS: flexbox only, no grid, no `:hover` layout, no scrollbars** ([theme.rs](../engine/src/editor/theme.rs)). The macro layout therefore uses **absolutely positioned boxes** (`left/top/width/height` in px from the solver); flex is used only inside an area.
7. **Menus paint last.** Blitz paints later elements over earlier ones regardless of `z-index`, so every popup (editor-type menu, edge menu, header menus) is rendered at the root after all areas, positioned from the solver rect of its anchor.
8. **Viewport areas have no panel under them.** The body of a Viewport area is not `data-interactive`; the pointer reaches Bevy. Headers, edges and popups are interactive.

## Data model (`engine/src/editor/layout/`, no Bevy, no Dioxus, fully unit-tested)

Same discipline as `bridge/`: it depends on neither framework, so the stage and the UI both use it and `cargo test` covers it.

```rust
pub struct AreaId(u32);                       // stable per area, like Jackdaw's TabId

pub enum Node {
    Area(Area),
    Split { axis: Axis, ratio: f32, a: Box<Node>, b: Box<Node> },   // ratio = a's share, 0..1
}

pub struct Area { pub id: AreaId, pub editor: EditorKind, pub state: AreaState }

pub struct Workspace { pub name: String, pub root: Node, pub maximized: Option<AreaId> }
pub struct Screen   { pub workspaces: Vec<Workspace>, pub active: usize }   // serde, JSON
```

- **`EditorKind`** is an enum for exhaustive `match` in the UI, with a stable string id for files (`"viewport"`, `"outliner"`, ...) and a descriptor table (title, icon, category, min size) like Renzora's/Jackdaw's registries. An unknown id in a saved file becomes an empty area (Fyrox's rule: never fail to restore).
- **`AreaState`** is per-area data: the Viewport's camera pose and shading, the Assets browser's folder (today global `editor.folder`), the Outliner's expanded rows. Project, scene, selection and dirty stay global in `Editor`.
- **Operations** (pure functions on `Workspace`, each returns an error instead of panicking): `split(area, axis, ratio)`, `join(area, toward)`, `swap(a, b)`, `set_editor(area, kind)`, `resize(edge, position)`, `maximize/restore`, `reset(to_preset)`.
- **Join rule.** A tree can join A with its sibling only when the sibling is a leaf (Blender requires the two areas to share a whole edge, which is the same condition in a tree). Joins across a sibling *subtree* are refused and the menu item is disabled. This is the one place the tree is less general than Blender's screen graph; it covers the layouts people actually make.
- **Solver**: `solve(&Node, Rect) -> Vec<(AreaId, Rect)>` and `edges(&Node, Rect) -> Vec<Edge>`, with minimum sizes (area at least ~140 x 90 logical px; the header is 26 px) and ratio clamping. `theme::Layout::for_window` becomes `Screen::solve(window, insets)`.
- **Persistence**: `serde_json`, already a dependency of the editor feature. Newer fields get `#[serde(default)]` (Renzora). Stored per user (prefs), not in the project: web in the browser's storage / backend, native in the config directory, through one adapter like `project::store`. Presets are code: *Reset Layout* returns to the preset.
- **Bridge**: the UI sends `EditorCommand::SetScreen(Arc<Screen>)`; the stage runs the same solver with the window size it already owns, so a window resize needs no round trip. One Bevy camera per Viewport area (limit 4, like Renzora), `Viewport` set to that area's rect; the area under the cursor is the one that receives navigation and picking.

## Interactions

1. **Editor-type button** (header, far left): opens the popup shown in the reference image: four columns (General, Animation, Scripting, Data) with icon, name and shortcut. On a narrow screen the columns stack and scroll.
2. **Edge options**: right-click (or long-press) on an edge, hit zone 6 px (16 px on touch) opens *Area Edge Options*: Vertical Split, Horizontal Split, Join Right/Left (Up/Down for horizontal edges), Swap Areas. Split and Join then use one **pick mode** (hover highlights the area, preview line, click confirms, Esc or right-click cancels), which works with a tap on touch. Blender's corner-drag split/join comes in a later pass.
3. **Resize**: drag an edge; both neighbours respect minimum sizes; double-click an edge is not used.
4. **Maximize area** (Ctrl+Space, header menu, and a visible button on phones) and fullscreen; the one-viewport phone layout is just a workspace.
5. **Header menus** are data (`Vec<MenuDef>` per editor) rendered by one shared dropdown component, so every editor gets its menus without custom popup code. Header position top/bottom is a per-area option, as in Blender.
6. **App bar** (top): File, Edit, Window, Help; workspace tabs (click, drag to reorder, double-click to rename, right-click duplicate/delete, **+** for a preset); project/scene picker; Play / Build / Publish (stage 2d) and fps on the right. *Window* holds the area/workspace commands.
7. **Keymap**: events go to the area under the cursor. We need to decide Blender keys (G/R/S, numpad views, Shift+A) versus today's Unity/Unreal ones (W/E/R, Unreal camera). Renzora supports both at once; the keymap should be data from day one, with only changed bindings stored.

## Editors (area contents)

Start with the five we already have, hosted as editors, then add rows as phases need them. Anything not built yet appears in the menu as a disabled or placeholder entry so the menu looks and behaves like Blender's from the start.

| Category | Editor | Source of the content | When |
|----------|--------|----------------------|------|
| General | **Viewport** (Scene View; 2D flat or 3D by scene dimension) | existing stage + Renzora's toolbar, shading ladder, snaps, orientation gizmo | 2g (1 viewport), then up to 4 |
| General | Game Preview (Play) | stage 2d | after 2d |
| General | Image Editor | later | pending |
| Data | **Outliner** (today's Hierarchy) | existing | 2g |
| Data | **Properties** (today's Inspector; later tabs like Blender: Object, Scene, Project) | existing + Renzora's field registry | 2g |
| Data | **Asset Browser** (today's Project panel) | existing | 2g |
| Scripting | **Console / Info** | existing | 2g |
| Data | Project Settings (game modes, input actions) | planned in 2c | pending |
| Data | History (undo list) | Renzora `renzora_history`, Fyrox command stack | with the undo stage |
| Animation | Timeline, Dope Sheet, Graph Editor | Renzora animation panels | when Darius needs animation |
| Data | **Warp, Quests, Dialogue** | our own tools, stages 3.7 to 3.9 | Phase 3 |
| Data | **Network Monitor** | Renzora's network panels, our M track | M-stages |
| General | Node Editor (shader/logic) | Phase 10 | later |
| Scripting | Text Editor (Rust files) | Renzora/Berry | later |
| Data | Tilemap / sprite animation (2D) | Renzora `tilemap`, `sprite_animation` | useful for Darius |

## Undo (comes right after the shell)

Blender expects Ctrl+Z everywhere, and several Properties/Outliner areas can show the same scene, so edits must not live in area code. Adopt Renzora's shape on Fyrox's trait: a `Command { name, execute, revert, merge(&next) }`, a stack per document context (`Scene`, later a graph file), consecutive edits to one field merged while dragging, sealed on mouse-up. `edit.rs` becomes the first user. Selection changes are not undo steps.

## Stages (2g, in this order, each a thin working slice, `test all` green)

| Stage | What | Done when |
|-------|------|-----------|
| 2g.0 Layout model | `editor/layout/`: tree, operations, solver, JSON round trip, presets, tests (resize clamps, split/join/swap, unknown editor restores empty, solver covers the window exactly with no gaps or overlaps) | tests pass; nothing visible changes |
| 2g.1 Shell | Areas placed from the solver (absolute boxes), headers with the editor-type button and popup; Viewport, Outliner, Properties, Assets, Console as editors; stage places its camera from the solver; default workspace equals today's layout | the editor looks and works as now, and any area can be switched to any editor, on web and Windows |
| 2g.2 Edges | Drag-resize, edge context menu, pick mode for split/join, swap, maximize; long-press on touch; capture layer | the reference image's menu works with mouse, and with touch on web (device emulation) |
| 2g.3 Workspaces and app bar | Workspace tabs, presets (Layout, Assets, Scripting, Debug), persistence, File/Edit/Window/Help menus, Reset Layout | layout survives a reload |
| 2g.4 Header menus and viewports | Per-editor header menus as data; viewport header (View, Select, Add, Object; shading; snaps; Frame); 2 to 4 viewports with their own cameras; area-local state | two viewports show the same scene from different cameras |
| 2g.5 Undo, keymap, palette | Command stack, History editor, data-driven keymap, command palette | Ctrl+Z works across all areas |
| 2g.6 Touch and Android pass | Phone workspace, touch hit sizes, long-press tuning on a device | verified on the Android build, last |

Risks to check early: (a) Blitz paint cost with many areas (measure at 2g.1); (b) one frame of lag between UI tree and stage while dragging an edge is avoided by both sides solving the same tree, so keep the solver deterministic and f32-exact; (c) `data-interactive` hit-testing when a popup overlaps a viewport body; (d) several cameras on WebGL2 (Compat profile, stage 2e) may cost more than WebGPU.
