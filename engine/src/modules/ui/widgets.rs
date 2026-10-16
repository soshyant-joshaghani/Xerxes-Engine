//! The widgets Bevy's `Button`, `Text` and `ImageNode` do not cover: [`Toggle`], [`Slider`] and
//! [`Dropdown`]. Each is plain data (author it in a scene), builds its own children when it
//! appears and writes a [`WidgetChanged`] message when the user changes it. Style them like any
//! node: a later `Node` or `BackgroundColor` on the object replaces the default.

use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

const FIELD: Color = Color::srgb(0.165, 0.184, 0.216);
const ACCENT: Color = Color::srgb(0.95, 0.45, 0.12);
const TEXT: Color = Color::srgb(0.91, 0.918, 0.929);

/// A widget's value changed because the user used it.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct WidgetChanged(pub Entity);

// ---- Toggle ---------------------------------------------------------------------------

/// A checkbox.
#[derive(Component, Debug, Clone, PartialEq)]
#[require(Button, Node = toggle_node(), BackgroundColor = FIELD)]
pub struct Toggle {
    pub on: bool,
}

fn toggle_node() -> Node {
    Node {
        width: px(24),
        height: px(24),
        border_radius: BorderRadius::all(px(5)),
        ..default()
    }
}

impl Toggle {
    pub fn new(on: bool) -> Self {
        Self { on }
    }
}

pub(super) fn build_toggle(mut toggles: Query<(&Toggle, &mut BackgroundColor), Added<Toggle>>) {
    for (toggle, mut background) in &mut toggles {
        background.0 = if toggle.on { ACCENT } else { FIELD };
    }
}

pub(super) fn press_toggle(
    mut toggles: Query<(Entity, &Interaction, &mut Toggle), Changed<Interaction>>,
    mut changed: MessageWriter<WidgetChanged>,
) {
    for (entity, interaction, mut toggle) in &mut toggles {
        if *interaction == Interaction::Pressed {
            toggle.on = !toggle.on;
            changed.write(WidgetChanged(entity));
        }
    }
}

pub(super) fn refresh_toggle(mut toggles: Query<(&Toggle, &mut BackgroundColor), Changed<Toggle>>) {
    for (toggle, mut background) in &mut toggles {
        background.0 = if toggle.on { ACCENT } else { FIELD };
    }
}

// ---- Slider ---------------------------------------------------------------------------

/// A horizontal slider from `min` to `max`.
#[derive(Component, Debug, Clone, PartialEq)]
#[require(Node = slider_node(), Interaction, RelativeCursorPosition, BackgroundColor = FIELD)]
pub struct Slider {
    pub min: f32,
    pub max: f32,
    pub value: f32,
}

fn slider_node() -> Node {
    Node {
        width: px(200),
        height: px(14),
        border_radius: BorderRadius::all(px(7)),
        overflow: Overflow::clip(),
        ..default()
    }
}

impl Slider {
    pub fn new(min: f32, max: f32, value: f32) -> Self {
        Self { min, max, value }
    }

    fn fraction(&self) -> f32 {
        if self.max > self.min {
            ((self.value - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

/// The filled part of a slider.
#[derive(Component)]
pub(super) struct SliderFill;

pub(super) fn build_slider(
    mut commands: Commands,
    sliders: Query<(Entity, &Slider), Added<Slider>>,
) {
    for (entity, slider) in &sliders {
        commands.spawn((
            SliderFill,
            Node {
                width: percent(slider.fraction() * 100.0),
                height: percent(100),
                ..default()
            },
            BackgroundColor(ACCENT),
            ChildOf(entity),
        ));
    }
}

pub(super) fn drag_slider(
    mut sliders: Query<(Entity, &Interaction, &RelativeCursorPosition, &mut Slider)>,
    mut changed: MessageWriter<WidgetChanged>,
) {
    for (entity, interaction, cursor, mut slider) in &mut sliders {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(at) = cursor.normalized else {
            continue;
        };
        let value = slider.min + (at.x + 0.5).clamp(0.0, 1.0) * (slider.max - slider.min);
        if value != slider.value {
            slider.value = value;
            changed.write(WidgetChanged(entity));
        }
    }
}

pub(super) fn refresh_slider(
    sliders: Query<(&Slider, &Children), Changed<Slider>>,
    mut fills: Query<&mut Node, With<SliderFill>>,
) {
    for (slider, children) in &sliders {
        for child in children {
            if let Ok(mut node) = fills.get_mut(*child) {
                node.width = percent(slider.fraction() * 100.0);
            }
        }
    }
}

// ---- Dropdown -------------------------------------------------------------------------

/// A drop-down list: the header shows the selected option, a press opens the list.
#[derive(Component, Debug, Clone, PartialEq)]
#[require(Button, Node = dropdown_node(), BackgroundColor = FIELD)]
pub struct Dropdown {
    pub options: Vec<String>,
    pub selected: usize,
    pub open: bool,
}

fn dropdown_node() -> Node {
    Node {
        width: px(200),
        height: px(32),
        padding: UiRect::axes(px(10), px(6)),
        align_items: AlignItems::Center,
        border_radius: BorderRadius::all(px(6)),
        ..default()
    }
}

impl Dropdown {
    pub fn new<S: Into<String>>(options: impl IntoIterator<Item = S>) -> Self {
        Self {
            options: options.into_iter().map(Into::into).collect(),
            selected: 0,
            open: false,
        }
    }

    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index;
        self
    }

    /// The chosen option's text.
    pub fn value(&self) -> &str {
        self.options.get(self.selected).map_or("", String::as_str)
    }
}

/// The entities a dropdown built for itself.
#[derive(Component)]
pub(super) struct DropdownParts {
    label: Entity,
    list: Entity,
}

/// One row of a dropdown's open list.
#[derive(Component)]
pub(super) struct DropdownOption {
    dropdown: Entity,
    index: usize,
}

pub(super) fn build_dropdown(
    mut commands: Commands,
    dropdowns: Query<(Entity, &Dropdown), Added<Dropdown>>,
) {
    for (entity, dropdown) in &dropdowns {
        let label = commands
            .spawn((
                Text::new(dropdown.value()),
                TextFont::from_font_size(14.0),
                TextColor(TEXT),
                ChildOf(entity),
            ))
            .id();
        let list = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: percent(100),
                    left: px(0),
                    min_width: percent(100),
                    flex_direction: FlexDirection::Column,
                    display: if dropdown.open {
                        Display::Flex
                    } else {
                        Display::None
                    },
                    border_radius: BorderRadius::all(px(6)),
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(FIELD),
                GlobalZIndex(100),
                ChildOf(entity),
            ))
            .id();
        for (index, option) in dropdown.options.iter().enumerate() {
            commands
                .spawn((
                    Button,
                    DropdownOption {
                        dropdown: entity,
                        index,
                    },
                    Node {
                        padding: UiRect::axes(px(10), px(6)),
                        ..default()
                    },
                    BackgroundColor(if index == dropdown.selected {
                        ACCENT
                    } else {
                        FIELD
                    }),
                    ChildOf(list),
                ))
                .with_child((
                    Text::new(option.clone()),
                    TextFont::from_font_size(14.0),
                    TextColor(TEXT),
                ));
        }
        commands
            .entity(entity)
            .insert(DropdownParts { label, list });
    }
}

pub(super) fn press_dropdown(
    mut dropdowns: Query<(Entity, Ref<Interaction>, &mut Dropdown)>,
    options: Query<(Ref<Interaction>, &DropdownOption)>,
    mut changed: MessageWriter<WidgetChanged>,
) {
    // A press on an option picks it and closes the list; on the header it opens or closes it.
    let picks: Vec<(Entity, usize)> = options
        .iter()
        .filter(|(i, _)| i.is_changed() && **i == Interaction::Pressed)
        .map(|(_, o)| (o.dropdown, o.index))
        .collect();
    for (entity, interaction, mut dropdown) in &mut dropdowns {
        if let Some((_, index)) = picks.iter().find(|(d, _)| *d == entity) {
            dropdown.selected = *index;
            dropdown.open = false;
            changed.write(WidgetChanged(entity));
        } else if interaction.is_changed() && *interaction == Interaction::Pressed {
            dropdown.open = !dropdown.open;
        }
    }
}

pub(super) fn refresh_dropdown(
    dropdowns: Query<(Entity, &Dropdown, &DropdownParts), Changed<Dropdown>>,
    mut texts: Query<&mut Text>,
    mut nodes: Query<&mut Node>,
    mut rows: Query<(&DropdownOption, &mut BackgroundColor)>,
) {
    for (entity, dropdown, parts) in &dropdowns {
        if let Ok(mut text) = texts.get_mut(parts.label) {
            text.0 = dropdown.value().to_string();
        }
        if let Ok(mut node) = nodes.get_mut(parts.list) {
            node.display = if dropdown.open {
                Display::Flex
            } else {
                Display::None
            };
        }
        for (row, mut background) in &mut rows {
            if row.dropdown == entity {
                background.0 = if row.index == dropdown.selected {
                    ACCENT
                } else {
                    FIELD
                };
            }
        }
    }
}
