# Prefabs

A prefab (`*.prefab.rs`) is a reusable object tree — a module a scene drops in. The rule:

- **A prefab holds the invariant frame; the scene adds what varies.** An instance can replace a
  component of the prefab's root by type (`Object::prefab(p).with(LevelButton(3))`) and append
  children, but it cannot reach inside the prefab's own children. So anything that differs per
  use (a title, a marker on a label) is a child the scene adds, not something the prefab holds.
- **This folder is the game's global prefabs:** the screen frame (`header`, `content`, `footer`,
  `banner`), the HUD widgets and the main menu's cells. Any scene may use them.
- **`games/<family>/<game>/` holds one game's own prefabs,** mirroring `scenes/games/<family>/`.
- **A variant is a prefab made of a prefab:** `PrefabAsset { root: Object::prefab(base).with(..) }`
  is the base with changes of its own (`menu_tile_narrow` is `menu_tile`, narrower). Three layers
  can meet on one object — the base, the variant, the scene's instance — and the last one wins by
  component type, while children add up.
- **A game spawns these same prefabs while it plays** (`assets::spawn(&mut commands, ..)`), so what
  a thing looks like is said once: Siavash's goons, flyers and road chunks are made that way.
- Prefabs are plain data, like scenes (no helper functions), so the editor reads, shows and
  edits them, and can be entered from a scene.
