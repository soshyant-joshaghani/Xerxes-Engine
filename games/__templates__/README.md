# Template games

**Finished games**, committed with the engine, that people can start a project from or study. They are not the starting point of every project: the engine creates that itself (New Project in the editor, `game new`), an empty `starter` project with one scene (`engine/starter/`).

A game graduates here from `games/<name>` when it is finished (AGENTS.md). The exception is the **Darius games** (`darius`, `cyrus`): they are developed in place here, because they are the engine's proof and grow with it, and `games/` itself is gitignored. Run them with `template dev darius`.

## Read-only in the engine

Template games are not edited in place. The editor (and the backend) open them to look at and run, and refuse every change to them: writing, deleting, moving files, adding logic. To work with one, use it as the template of a new project (Project Manager: "Use as template", or `game new <name> --from <template>`); that makes your own copy under `games/` (or your projects folder), and you edit that. The template itself only changes when the engine's authors update it in this repo.

## Naming

Name a template for what it is, not that it is a template: `<game>-<dimension>`, kebab-case, starting with a letter (Cargo and the project rules forbid a leading digit, so `darius`, shown as "Darius").

| Name | What |
|------|------|
| `darius` | Darius, the 2D game with a mini-game for every game mode (in progress, developed in place) |
| `cyrus` | (later) Cyrus, its 3D sibling |

No `game-template`, `template-1`, `demo` or `test`. Folder names are also the template names in New Project and in `game new --from <name>`, so they must read well there. Names cannot clash with the engine's reserved ones (`engine`, `starter`).

Each template is a full game crate (its own `Cargo.toml`, `assets/`, `README.md` whose first sentence is its description). `test templates` builds and tests every one.
