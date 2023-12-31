use bevy::{prelude::*, ui::FocusPolicy};
use bevy_ui_navigation::prelude::FocusState;

use crate::ui::intermediary_node_bundles::IntermediaryNodeBundleHandler;

use super::super::colors::{self, *};

pub fn c_root(b: &mut NodeBundle) {
    b.style.width = Val::Percent(100.);
    b.style.height = Val::Percent(100.);
    b.style.display = Display::Flex;
    b.style.justify_content = JustifyContent::Center;
    b.style.align_items = AlignItems::Center;
    b.style.position_type = PositionType::Absolute;
    b.style.left = Val::Px(0.);
    b.style.top = Val::Px(0.);
}

pub fn overlay(b: &mut NodeBundle) {
    b.z_index = ZIndex::Global(20);
    b.background_color.0 = OVERLAY_COLOR;
}

pub fn primary_box(b: &mut NodeBundle) {
    b.style.margin = UiRect::all(Val::Px(10.));
    b.style.padding = UiRect::all(Val::Px(30.));
    b.background_color.0 = PRIMARY_BACKGROUND_COLOR;
    b.border_color.0 = BORDER_COLOR;
    b.style.border = UiRect::all(Val::Px(2.));
    b.style.display = Display::Grid;

    b.style.grid_template_columns = vec![GridTrack::auto(), GridTrack::auto(), GridTrack::auto()];
    b.style.grid_template_rows = vec![
        GridTrack::percent(50.),
        GridTrack::fr(1.),
        GridTrack::fr(1.),
    ];
    b.style.grid_auto_flow = GridAutoFlow::Column;

    b.style.align_items = AlignItems::Center;
    b.style.justify_content = JustifyContent::Center;

    b.style.row_gap = Val::Px(20.);
}

pub fn primary_box_main(b: &mut dyn IntermediaryNodeBundleHandler) {
    b.style().grid_row = GridPlacement::start(1);
    b.style().grid_column = GridPlacement::start(1).set_span(3);
    b.style().padding = UiRect::all(Val::Px(40.));
}

pub fn primary_box_item(b: &mut dyn IntermediaryNodeBundleHandler) {
    b.style().grid_column = GridPlacement::start(2).set_span(1);
    b.style().padding = UiRect::all(Val::Px(20.));
}

pub fn c_button(b: &mut dyn IntermediaryNodeBundleHandler) {
    b.style().padding = UiRect::all(Val::Px(15.));
    b.style().border = UiRect::all(Val::Px(2.));
    b.style().margin = UiRect::all(Val::Px(10.));
    b.style().justify_content = JustifyContent::Center;
    b.style().align_items = AlignItems::Center;
    b.background_color().0 = colors::PRIMARY_COLOR_BUTTON;
}

pub fn c_button_prioritized(b: &mut dyn IntermediaryNodeBundleHandler) {
    b.background_color().0 = colors::PRIMARY_COLOR_PRIORITIZED;
}

pub fn c_button_focused(b: &mut dyn IntermediaryNodeBundleHandler) {
    b.background_color().0 = colors::PRIMARY_COLOR_PRIORITIZED;
}

pub fn c_button_active(b: &mut dyn IntermediaryNodeBundleHandler) {
    b.background_color().0 = colors::PRIMARY_COLOR_ACTIVE;
}

pub fn c_button_blocked(b: &mut dyn IntermediaryNodeBundleHandler) {
    b.background_color().0 = colors::PRIMARY_COLOR_BLOCKED;
}

pub fn apply_button_state(state: FocusState) -> NodeBundle {
    let mut bundle = NodeBundle::default();
    c_button(&mut bundle);
    primary_box_item(&mut bundle);
    match state {
        FocusState::Prioritized => c_button_prioritized(&mut bundle),
        FocusState::Focused => c_button_focused(&mut bundle),
        FocusState::Active => c_button_active(&mut bundle),
        FocusState::Blocked => c_button_blocked(&mut bundle),
        FocusState::Inert => {}
    };
    bundle
}

pub fn button_text(assets: &AssetServer, t: &mut TextStyle) {
    t.font_size = 10.;
    t.color = colors::BORDER_COLOR;
    t.font = assets.load("fonts/Roman Antique.ttf");
}

pub fn main_text(assets: &AssetServer, t: &mut TextStyle) {
    t.font_size = 40.;
    t.color = PRIMARY_COLOR;
    t.font = assets.load("fonts/Roman Antique Italic.ttf");
}

pub fn standard_text(assets: &AssetServer, t: &mut TextStyle) {
    t.font_size = 10.;
    t.color = PRIMARY_COLOR;
    t.font = assets.load("fonts/Roman Antique.ttf");
}

pub fn span(b: &mut dyn IntermediaryNodeBundleHandler) {
    b.style().display = Display::Flex;
    b.style().flex_direction = FlexDirection::Row;
    b.style().justify_content = JustifyContent::FlexStart;
    b.style().align_items = AlignItems::Center;
}

pub fn soul_bar_root(b: &mut NodeBundle) {
    b.style.width = Val::Percent(100.);
    b.style.height = Val::Percent(100.);
    b.style.display = Display::Flex;
    b.style.flex_direction = FlexDirection::Column;
    b.style.justify_content = JustifyContent::FlexStart;
    b.style.align_items = AlignItems::FlexStart;
    b.style.position_type = PositionType::Absolute;
    b.style.left = Val::Px(0.);
    b.style.top = Val::Px(0.);
    b.focus_policy = FocusPolicy::Pass;
    b.style.padding = UiRect::all(Val::Px(10.));
}

pub fn soul_bar_container(b: &mut NodeBundle) {
    b.style.width = Val::Vw(10.);
    b.style.height = Val::Px(20.);
    b.style.display = Display::Flex;
    b.style.flex_direction = FlexDirection::Row;
    b.style.justify_content = JustifyContent::FlexStart;
    b.style.align_items = AlignItems::Stretch;
    b.background_color.0 = colors::BORDER_COLOR;
    b.focus_policy = FocusPolicy::Pass;
}

pub fn soul_bar(b: &mut NodeBundle) {
    b.background_color.0 = colors::PRIMARY_COLOR;
    b.style.height = Val::Percent(100.);
    b.style.flex_grow = 0.;
    b.style.flex_shrink = 0.;
    b.focus_policy = FocusPolicy::Pass;
}

pub fn checkpoint_marker_root(b: &mut NodeBundle) {
    b.style.width = Val::Percent(100.);
    b.style.height = Val::Percent(100.);
    b.style.display = Display::Flex;
    b.style.flex_direction = FlexDirection::Column;
    b.style.justify_content = JustifyContent::FlexStart;
    b.style.align_items = AlignItems::FlexEnd;
    b.style.position_type = PositionType::Absolute;
    b.style.left = Val::Px(0.);
    b.style.top = Val::Px(0.);
    b.focus_policy = FocusPolicy::Pass;
    b.style.padding = UiRect::all(Val::Px(10.));
}
pub fn checkpoint_marker(b: &mut NodeBundle) {
    b.style.width = Val::Px(50.);
    b.style.height = Val::Px(50.);
    b.style.margin = UiRect::all(Val::Px(5.));
}

pub fn checkpoint_marker_background(assets: &AssetServer, b: &mut ImageBundle) {
    b.style.position_type = PositionType::Absolute;
    b.style.top = Val::Px(0.);
    b.style.left = Val::Px(0.);
    b.style.bottom = Val::Px(0.);
    b.style.right = Val::Px(0.);
    b.image = UiImage {
        texture: assets.load("textures/checkpoint_slot.png"),
        ..default()
    };
}

pub fn checkpoint_marker_empty(assets: &AssetServer, b: &mut ImageBundle) {
    b.style.position_type = PositionType::Absolute;
    b.style.top = Val::Px(0.);
    b.style.left = Val::Px(0.);
    b.style.bottom = Val::Px(0.);
    b.style.right = Val::Px(0.);
    b.image = UiImage {
        texture: assets.load("textures/checkpoint-empty.png"),
        ..default()
    };
}

pub fn checkpoint_marker_inner_background(assets: &AssetServer, b: &mut ImageBundle) {
    b.style.position_type = PositionType::Absolute;
    b.style.left = Val::Px(0.);
    b.style.bottom = Val::Px(0.);
    b.style.right = Val::Px(0.);
    b.style.width = Val::Px(50.);
    b.style.height = Val::Px(50.);
    b.image = UiImage {
        texture: assets.load("textures/checkpoint-full.png"),
        ..default()
    };
}

pub fn checkpoint_marker_content(b: &mut NodeBundle) {
    b.style.position_type = PositionType::Absolute;
    b.style.left = Val::Px(0.);
    b.style.bottom = Val::Px(0.);
    b.style.right = Val::Px(0.);
    b.style.flex_grow = 0.;
    b.style.width = Val::Percent(100.);
    b.style.height = Val::Percent(0.);
    b.style.display = Display::None;
    b.style.overflow = Overflow::clip();
}

pub fn disable_button(b: &mut dyn IntermediaryNodeBundleHandler) {
    b.style().display = Display::None;
}

pub fn in_game_text_box(b: &mut dyn IntermediaryNodeBundleHandler) {
    b.style().position_type = PositionType::Absolute;
    b.style().left = Val::Px(10.);
    b.style().bottom = Val::Px(10.);
    b.style().right = Val::Px(10.);
    b.style().padding = UiRect::all(Val::Px(20.));
    b.background_color().0 = colors::PRIMARY_BACKGROUND_COLOR;
    b.style().flex_direction = FlexDirection::Column;
    b.style().align_items = AlignItems::FlexStart;
}

pub fn devil_in_game_text_box(assets: &AssetServer, b: &mut ImageBundle) {
    b.style.position_type = PositionType::Absolute;
    b.z_index = ZIndex::Global(-1);
    b.style.top = Val::Px(-68.);
    b.style.left = Val::Percent(6.5);
    b.style.width = Val::Auto;
    b.style.height = Val::Px(70.);
    b.image = UiImage {
        texture: assets.load("textures/devil_in_game_talking.png"),
        ..default()
    };
}
