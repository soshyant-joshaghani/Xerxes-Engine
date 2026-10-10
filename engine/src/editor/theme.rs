//! The editor's look (a dark theme) and its frame: the window and its safe area, from which
//! the Dioxus areas and the Bevy Scene view are both placed (see `layout`), on every platform.

use bevy::math::Vec2;

/// Toolbar height in logical pixels.
pub fn toolbar() -> f32 {
    if crate::editor::layout::touch() {
        52.0
    } else {
        38.0
    }
}
/// The app menu buttons (File, Edit, Window, Help) and the layout tabs that follow them in the
/// top bar have fixed widths, so a menu or a drag can be placed against one without measuring.
pub const MENU_WIDTH: f32 = 56.0;
pub const MENU_GAP: f32 = 2.0;
pub const TAB_WIDTH: f32 = 112.0;
pub const TAB_GAP: f32 = 2.0;
/// An area header's editor-type button and its menu buttons (View, Select...), fixed for the
/// same reason.
pub const ETYPE_WIDTH: f32 = 124.0;
pub const HMENU_WIDTH: f32 = 54.0;
pub const HMENU_GAP: f32 = 2.0;
/// The top bar's padding and the gap between its groups (see the `.bar` style).
const BAR_PAD: f32 = 8.0;
const BAR_GAP: f32 = 6.0;

/// Where the first layout tab starts in the top bar (after the four app menus).
pub fn tabs_left() -> f32 {
    BAR_PAD + 4.0 * MENU_WIDTH + 3.0 * MENU_GAP + BAR_GAP
}

/// Parts of the window the system draws over (Android's navigation bar, a display cutout),
/// in logical pixels. The editor lays itself out inside them.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Insets {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

/// The editor's frame for a window: its size and the safe area. Computed by the stage and sent
/// to the UI. Both then solve the same workspace inside [`Layout::content`] (see
/// `editor::layout`), so the areas and the Scene view's cameras always agree.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layout {
    pub insets: Insets,
    /// The window, in logical pixels.
    pub window: Vec2,
    /// Touch sizes are on (see `layout::touch`).
    pub touch: bool,
}

impl Default for Layout {
    fn default() -> Self {
        Self::for_window(Vec2::new(1920.0, 1080.0), Insets::default())
    }
}

impl Layout {
    pub fn for_window(window: Vec2, insets: Insets) -> Self {
        Self {
            insets,
            window,
            touch: crate::editor::layout::touch(),
        }
    }

    /// Where the areas go: inside the insets, under the toolbar, in window coordinates. The
    /// UI lays the areas out relative to its top-left corner; the stage adds `x` and `y`.
    pub fn content(&self) -> crate::editor::layout::Rect {
        let i = self.insets;
        crate::editor::layout::Rect::new(
            i.left,
            i.top + toolbar(),
            (self.window.x - i.left - i.right).max(1.0),
            (self.window.y - i.top - i.bottom - toolbar()).max(1.0),
        )
    }

    /// Where the editor's root sits (CSS), inside the insets.
    pub fn root_style(&self) -> String {
        let i = self.insets;
        format!(
            "left: {}px; top: {}px; right: {}px; bottom: {}px;",
            i.left, i.top, i.right, i.bottom
        )
    }
}

/// The editor stylesheet. Only CSS that Blitz (the native renderer) supports: flexbox, no
/// grid, no `text-transform`, no scrollbars, no `:hover`-dependent layout.
pub fn style() -> String {
    let toolbar = toolbar();
    let header = crate::editor::layout::header();
    let mut css = format!(
        r#"
.ed {{ position: absolute; display: flex; flex-direction: column; color: #d2d2d2;
       font: 12px/1.4 system-ui, sans-serif; }}
.ed * {{ box-sizing: border-box; }}
.ed .bar {{ height: {toolbar}px; flex: none; display: flex; align-items: center; gap: 6px; padding: 0 8px;
            background: #3c3c3c; border-bottom: 1px solid #1e1e1e; overflow-x: auto; overflow-y: hidden; }}
.ed .bar > * {{ flex: none; white-space: nowrap; }}
.ed .actions button {{ white-space: nowrap; }}
/* The screen: areas are placed with absolute boxes from the layout solver (flex only inside them). */
.ed .shell {{ flex: 1; position: relative; min-height: 0; }}
.ed .area {{ position: absolute; display: flex; flex-direction: column; overflow: hidden; background: #383838;
             border: 1px solid #1e1e1e; }}
/* The Viewport's body is not painted: Bevy shows through (and gets the pointer). */
.ed .area.clear {{ background: transparent; }}
.ed .area-head {{ height: {header}px; flex: none; display: flex; align-items: center; gap: 4px; padding: 0 4px;
                  background: #303030; border-bottom: 1px solid #1e1e1e; overflow-x: auto; overflow-y: hidden; }}
.ed .area-head > * {{ flex: none; white-space: nowrap; }}
.ed .area-body {{ flex: 1; min-height: 0; display: flex; flex-direction: column; }}
.ed button.etype {{ display: flex; align-items: center; gap: 5px; width: {ETYPE_WIDTH}px; padding: 1px 6px; background: #3f3f3f; overflow: hidden; white-space: nowrap; }}
.ed button.etype span {{ white-space: nowrap; }}
.ed button.etype .grow {{ overflow: hidden; }}
.ed .hmenus {{ display: flex; gap: {HMENU_GAP}px; }}
.ed .hmenus button {{ width: {HMENU_WIDTH}px; padding: 3px 0; background: transparent; border-color: transparent; }}
.ed .hmenus button.on {{ background: #4a4a4a; border-color: #2a2a2a; }}
.ed .ico {{ display: inline-block; min-width: 20px; padding: 0 2px; text-align: center; font-size: 10px; font-weight: 700;
            color: #cfd8e6; background: #4a5568; border-radius: 3px; }}
.ed .caret {{ width: 0; height: 0; border-left: 4px solid transparent; border-right: 4px solid transparent;
              border-top: 5px solid #b4b4b4; }}
.ed .backdrop {{ position: absolute; left: 0; top: 0; right: 0; bottom: 0; }}
.ed .etype-menu {{ position: absolute; display: flex; flex-wrap: wrap; gap: 4px; padding: 8px; overflow: auto;
                   background: #1f1f1f; border: 1px solid #0e0e0e; border-radius: 4px; }}
.ed .etype-col {{ width: 168px; display: flex; flex-direction: column; }}
.ed .etype-cat {{ padding: 2px 8px 4px; color: #8a8a8a; border-bottom: 1px solid #333; margin-bottom: 4px; }}
.ed .etype-item {{ display: flex; align-items: center; gap: 7px; height: 26px; padding: 0 8px; border-radius: 3px; cursor: pointer; }}
.ed .etype-item.on {{ background: #2c5d87; color: #fff; }}
.ed .etype-item.soon {{ color: #8a8a8a; }}
.ed .etype-item.off {{ color: #5c5c5c; cursor: default; }}
.ed .etype-item .grow {{ overflow: hidden; }}
.ed .appmenus {{ display: flex; gap: {MENU_GAP}px; flex: none; }}
.ed .appmenus button {{ width: {MENU_WIDTH}px; padding: 3px 0; background: transparent; border-color: transparent; }}
.ed .appmenus button.on {{ background: #4a4a4a; border-color: #2a2a2a; }}
.ed .wtabs {{ display: flex; align-items: center; gap: {TAB_GAP}px; flex: none; }}
.ed .wtab {{ width: {TAB_WIDTH}px; height: 26px; display: flex; align-items: center; justify-content: center; padding: 0 8px;
             border-radius: 3px; color: #9a9a9a; cursor: pointer; overflow: hidden; white-space: nowrap; }}
.ed .wtab.on {{ background: #5a5a5a; color: #f0f0f0; }}
.ed input.wtab-input {{ flex: none; width: {TAB_WIDTH}px; height: 24px; }}
.ed .menu .row.off {{ color: #5c5c5c; cursor: default; }}
.ed .menu .rule {{ height: 1px; margin: 3px 0; background: #333; }}
.ed .palette {{ position: absolute; display: flex; flex-direction: column; padding: 6px 0; background: #1f1f1f;
                border: 1px solid #0e0e0e; border-radius: 4px; max-height: 70%; overflow: auto; }}
.ed .palette .row {{ padding: 0 12px; height: 24px; }}
.ed .palette .row.off {{ color: #5c5c5c; cursor: default; }}
.ed input.palette-input {{ flex: none; width: auto; height: 26px; margin: 0 8px 6px; font-size: 13px; }}
.ed .row.muted {{ color: #7a7a7a; }}
.ed textarea.code-edit {{ flex: 1; width: 100%; min-height: 0; resize: none; background: #1e1e1e; color: #d4d4d4; border: none;
                          padding: 6px 8px; font: 12px ui-monospace, monospace; white-space: pre; overflow: auto; box-sizing: border-box; }}
.ed .imgview {{ flex: 1; min-height: 0; overflow: auto; background: #262626; padding: 8px; }}
.ed .imgview img {{ display: block; }}
.ed iframe.preview-frame {{ flex: 1; width: 100%; min-height: 0; border: none; background: #000; }}
.ed .sheet {{ flex: 1; min-height: 0; overflow: auto; }}
.ed .srow {{ display: flex; align-items: center; height: 20px; min-width: 860px; cursor: pointer; white-space: nowrap; }}
.ed .srow.head {{ background: #303030; color: #b4b4b4; font-weight: 600; cursor: default; }}
.ed .srow.sel {{ background: #2c5d87; color: #fff; }}
.ed .srow .c {{ flex: none; overflow: hidden; padding: 0 6px; }}
.ed .srow .c-id {{ width: 44px; color: #8a8a8a; }}
.ed .srow .c-name {{ width: 170px; }}
.ed .srow .c-kind {{ width: 170px; color: #9aa7b8; }}
.ed .srow .c-num {{ width: 62px; text-align: right; }}
.ed .srow .c-count {{ width: 52px; text-align: right; }}
.ed input.sheet-filter {{ flex: none; width: 220px; }}
.ed .edge {{ position: absolute; }}
.ed .edge.ew {{ cursor: ew-resize; }}
.ed .edge.ns {{ cursor: ns-resize; }}
.ed .capture {{ position: absolute; left: 0; top: 0; right: 0; bottom: 0; }}
.ed .capture.pickmode {{ cursor: crosshair; }}
.ed .pick {{ position: absolute; border: 2px solid #4a90d9; background: rgba(74, 144, 217, 0.14); }}
.ed .pick.on {{ border-color: #e0a030; background: rgba(224, 160, 48, 0.16); }}
.ed .splitline {{ position: absolute; background: #e0a030; }}
.ed .hintbar {{ position: absolute; left: 12px; bottom: 12px; padding: 6px 12px; background: #1f1f1f; border: 1px solid #0e0e0e;
                border-radius: 4px; color: #e6e6e6; }}
.ed .edge-menu {{ position: absolute; min-width: 190px; padding: 4px 0; background: #1f1f1f; border: 1px solid #0e0e0e; border-radius: 4px; }}
.ed .edge-menu .title {{ padding: 2px 12px 4px; color: #8a8a8a; }}
.ed .edge-menu .rule {{ height: 1px; margin: 3px 0; background: #333; }}
.ed .edge-menu .item {{ display: flex; align-items: center; height: 24px; padding: 0 12px; cursor: pointer; }}
.ed .edge-menu .item.off {{ color: #5c5c5c; cursor: default; }}
.ed .body {{ flex: 1; overflow: auto; min-height: 0; padding: 4px 0; }}
.ed .row {{ display: flex; align-items: center; height: 20px; padding-right: 8px; cursor: pointer; white-space: nowrap; }}
.ed .row.sel {{ background: #2c5d87; color: #ffffff; }}
.ed .muted {{ color: #8a8a8a; }}
.ed .hint {{ color: #8a8a8a; padding: 6px 10px; }}
.ed .icon {{ width: 16px; flex: none; text-align: center; color: #9a9a9a; }}
.ed .prefab {{ color: #7fb4ff; }}
.ed button {{ cursor: pointer; border: 1px solid #2a2a2a; border-radius: 3px; padding: 3px 9px; background: #585858;
              color: #e6e6e6; font: inherit; }}
.ed button.on {{ background: #2c5d87; border-color: #3e77aa; }}
.ed button.play {{ background: #3a6f3a; }}
/* `[disabled]`, not `:disabled`: Blitz (native) matches the attribute but not the state. */
.ed button[disabled]:not([disabled="false"]) {{ opacity: 0.4; cursor: default; }}
.ed .sep {{ width: 1px; height: 22px; background: #2a2a2a; margin: 0 4px; }}
.ed .grow {{ flex: 1; }}
.ed .drop {{ position: relative; }}
.ed .menu {{ position: absolute; min-width: 240px; background: #2f2f2f; border: 1px solid #1a1a1a; padding: 3px 0; }}
.ed .menu .row {{ gap: 12px; }}
.ed .menu .row {{ padding-left: 10px; }}
.ed .section {{ border-bottom: 1px solid #2a2a2a; padding: 4px 0 6px; }}
.ed .section-head {{ display: flex; align-items: center; height: 22px; padding: 0 8px; background: #3e3e3e; font-weight: 600; }}
.ed .field {{ display: flex; align-items: center; min-height: 22px; padding: 0 8px 0 18px; gap: 6px; }}
.ed .field .label {{ width: 92px; flex: none; color: #b4b4b4; overflow: hidden; }}
.ed .field .value {{ flex: 1; display: flex; gap: 4px; min-width: 0; }}
.ed .num {{ flex: 1; min-width: 0; background: #2a2a2a; border: 1px solid #222; border-radius: 2px; padding: 1px 4px; color: #e6e6e6; }}
.ed input.num {{ font: inherit; height: 20px; width: 0; }}
.ed input.name-input {{ flex: 1; font-size: 13px; font-weight: 600; }}
.ed .actions {{ display: flex; gap: 4px; padding: 4px 6px; border-bottom: 1px solid #2a2a2a; }}
.ed .actions button {{ padding: 1px 7px; font-size: 11px; flex: none; white-space: nowrap; }}
.ed .axis {{ color: #8a8a8a; width: 10px; flex: none; }}
.ed .code {{ font: 11px ui-monospace, monospace; color: #b8c7d9; white-space: pre-wrap; word-break: break-all; }}
.ed .swatch {{ width: 34px; height: 14px; border: 1px solid #222; flex: none; }}
.ed .title {{ display: flex; align-items: center; gap: 8px; padding: 6px 8px; border-bottom: 1px solid #2a2a2a; }}
.ed .title .name {{ font-size: 13px; font-weight: 600; color: #f0f0f0; }}
.ed .split {{ flex: 1; display: flex; min-height: 0; }}
.ed .tree {{ width: 220px; flex: none; overflow: auto; border-right: 1px solid #2a2a2a; padding: 4px 0; }}
.ed .files {{ flex: 1; overflow: auto; padding: 4px 0; }}
.ed .row.asset {{ height: 30px; gap: 8px; }}
.ed .thumb {{ position: relative; width: 24px; height: 24px; flex: none; display: flex; align-items: center; justify-content: center;
              background: #4a4a4a; border: 1px solid #2a2a2a; border-radius: 3px; color: #d0d0d0; font-size: 10px; font-weight: 700; overflow: hidden; }}
.ed .thumb.t-image {{ background: #3b4a3b; }}
.ed .thumb.t-scene {{ background: #2c4a6a; }}
.ed .thumb.t-prefab {{ background: #3a4f7a; color: #7fb4ff; }}
.ed .thumb.t-project {{ background: #5a4a2a; }}
.ed .thumb.t-mesh {{ background: #5a3b5a; }}
.ed .thumb.t-audio {{ background: #3b5a5a; }}
.ed .thumb.t-video {{ background: #5a3b3b; }}
.ed .thumb.t-folder {{ background: transparent; border-color: transparent; color: #9a9a9a; }}
.ed .thumb-img {{ position: absolute; left: 0; top: 0; width: 24px; height: 24px; }}
.ed .console-line {{ font: 11px ui-monospace, monospace; padding: 1px 10px; white-space: pre-wrap; }}
.ed .console-line.err {{ color: #ff8a80; }}
.ed .manager {{ position: absolute; inset: 0; display: flex; flex-direction: column; gap: 16px; padding: 24px;
               overflow: auto; background: #2b2b2b; }}
.ed .manager-head {{ display: flex; align-items: center; gap: 10px; }}
.ed .manager-body {{ display: flex; flex-wrap: wrap; gap: 24px; align-items: flex-start; }}
.ed .manager-col {{ flex: 1 1 320px; min-width: 0; display: flex; flex-direction: column; gap: 8px; }}
.ed .manager-col.narrow {{ flex: 0 1 380px; }}
.ed .manager-title {{ font-size: 18px; font-weight: 600; color: #f0f0f0; }}
.ed .manager-section {{ font-size: 15px; font-weight: 600; color: #f0f0f0; }}
.ed .manager-list {{ display: flex; flex-direction: column; gap: 4px; }}
.ed .manager-row {{ display: flex; align-items: center; gap: 12px; padding: 8px 12px; background: #383838;
                    border: 1px solid #222; border-radius: 4px; cursor: pointer; }}
.ed .manager-row.sel {{ background: #2c5d87; border-color: #3e77aa; }}
.ed .manager-name {{ font-size: 13px; font-weight: 600; color: #f0f0f0; }}
.ed .manager-desc {{ font-size: 12px; color: #a8a8a8; margin-top: 2px; }}
.ed input.manager-input {{ flex: none; width: 100%; height: 26px; box-sizing: border-box; font-size: 13px; }}
.ed .manager-problem {{ color: #ff8a80; }}
.ed .manager-buttons {{ display: flex; gap: 6px; }}
.ed .manager-buttons button {{ padding: 6px 12px; }}
.ed .manager-log {{ background: #1e1e1e; border-radius: 3px; padding: 4px 0; max-height: 170px; overflow: auto; }}
.ed .manager-banner {{ display: flex; flex-direction: column; gap: 4px; padding: 10px 12px; background: #4a2f2f;
                       border: 1px solid #7a3b3b; border-radius: 4px; color: #e6e6e6; }}
.ed .new-scene {{ display: flex; align-items: center; gap: 6px; padding: 4px 8px; border-bottom: 1px solid #2a2a2a; }}
.ed .new-scene input.num {{ flex: none; width: 160px; }}
.ed button.dim {{ padding: 0 6px; font-size: 10px; margin-left: 3px; }}
.ed button.dim.on {{ background: #2c5d87; border-color: #3e77aa; }}
.ed .overlay {{ position: absolute; left: 8px; top: 6px; color: #c8c8c8; }}
"#
    );
    if crate::editor::layout::touch() {
        css.push_str(TOUCH_CSS);
    }
    css
}

/// What touch mode changes in the look: bigger buttons, rows and fields (a fingertip is far
/// less exact than a mouse). Widths that menus are placed by stay as they are.
const TOUCH_CSS: &str = r#"
.ed { font-size: 14px; }
.ed button { padding: 7px 12px; }
.ed .area-head button, .ed .bar button { padding: 6px 10px; }
.ed .hmenus button, .ed .appmenus button { padding: 8px 0; }
.ed .row { height: 38px; }
.ed .menu .row, .ed .palette .row { height: 40px; }
.ed .etype-item { height: 40px; }
.ed .edge-menu .item { height: 42px; }
.ed .wtab { height: 38px; }
.ed input.num { height: 34px; }
.ed input.wtab-input { height: 38px; }
.ed input.palette-input { height: 40px; font-size: 15px; }
.ed .row.asset { height: 46px; }
.ed .field { min-height: 36px; }
.ed .section-head { height: 34px; }
.ed .thumb { width: 30px; height: 30px; }
.ed .srow { height: 34px; }
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_content_sits_under_the_toolbar() {
        let content = Layout::for_window(Vec2::new(1920.0, 1080.0), Insets::default()).content();
        assert_eq!((content.x, content.y), (0.0, toolbar()));
        assert_eq!((content.w, content.h), (1920.0, 1080.0 - toolbar()));
    }

    #[test]
    fn the_layout_stays_inside_the_system_bars() {
        // Android's navigation bar on the right (48 px) and a cutout on the left (40 px).
        let insets = Insets {
            left: 40.0,
            right: 48.0,
            ..Default::default()
        };
        let layout = Layout::for_window(Vec2::new(1100.0, 519.0), insets);
        let content = layout.content();
        assert_eq!((content.x, content.w), (40.0, 1100.0 - 40.0 - 48.0));
        assert_eq!(
            layout.root_style(),
            "left: 40px; top: 0px; right: 48px; bottom: 0px;"
        );
    }
}
