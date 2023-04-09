use crate::MainCamera;
use bevy::ecs::query::WorldQuery;
use bevy::prelude::*;
use bevy::render::camera::NormalizedRenderTarget;
use bevy::ui::{FocusPolicy, RelativeCursorPosition, UiStack};
use bevy::window::PrimaryWindow;
use smallvec::SmallVec;

#[derive(Default)]
pub struct State {
    entities_to_reset: SmallVec<[Entity; 1]>,
}

#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct NodeQuery {
    entity: Entity,
    node: &'static Node,
    global_transform: &'static GlobalTransform,
    interaction: Option<&'static mut Interaction>,
    relative_cursor_position: Option<&'static mut RelativeCursorPosition>,
    focus_policy: Option<&'static FocusPolicy>,
    calculated_clip: Option<&'static CalculatedClip>,
    computed_visibility: Option<&'static ComputedVisibility>,
}

/// This is a copy of [`bevy::ui::ui_focus_system`] but treats the main camera as the camera to
/// detect mouse position from even though it has ui disabled.
#[allow(clippy::too_many_arguments)]
pub fn ui_focus_system(
    mut state: Local<State>,
    camera: Query<&Camera, With<MainCamera>>,
    windows: Query<&Window>,
    mouse_button_input: Res<Input<MouseButton>>,
    touches_input: Res<Touches>,
    ui_stack: Res<UiStack>,
    mut node_query: Query<NodeQuery>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
) {
    let primary_window = primary_window.iter().next();

    // reset entities that were both clicked and released in the last frame
    for entity in state.entities_to_reset.drain(..) {
        if let Ok(mut interaction) = node_query.get_component_mut::<Interaction>(entity) {
            *interaction = Interaction::None;
        }
    }

    let mouse_released =
        mouse_button_input.just_released(MouseButton::Left) || touches_input.any_just_released();
    if mouse_released {
        for node in node_query.iter_mut() {
            if let Some(mut interaction) = node.interaction {
                if *interaction == Interaction::Clicked {
                    *interaction = Interaction::None;
                }
            }
        }
    }

    let mouse_clicked =
        mouse_button_input.just_pressed(MouseButton::Left) || touches_input.any_just_pressed();

    let cursor_position = camera
        .iter()
        .filter_map(|camera| {
            if let Some(NormalizedRenderTarget::Window(window_id)) =
                camera.target.normalize(primary_window)
            {
                Some(window_id)
            } else {
                None
            }
        })
        .find_map(|window_ref| {
            windows.get(window_ref.entity()).ok().and_then(|window| {
                window.cursor_position().map(|mut cursor_pos| {
                    cursor_pos.y = window.height() - cursor_pos.y;
                    cursor_pos
                })
            })
        })
        .or_else(|| touches_input.first_pressed_position());

    // prepare an iterator that contains all the nodes that have the cursor in their rect,
    // from the top node to the bottom one. this will also reset the interaction to `None`
    // for all nodes encountered that are no longer hovered.
    let mut moused_over_nodes = ui_stack
        .uinodes
        .iter()
        // reverse the iterator to traverse the tree from closest nodes to furthest
        .rev()
        .filter_map(|entity| {
            if let Ok(node) = node_query.get_mut(*entity) {
                // Nodes that are not rendered should not be interactable
                if let Some(computed_visibility) = node.computed_visibility {
                    if !computed_visibility.is_visible() {
                        // Reset their interaction to None to avoid strange stuck state
                        if let Some(mut interaction) = node.interaction {
                            // We cannot simply set the interaction to None, as that will trigger change detection repeatedly
                            interaction.set_if_neq(Interaction::None);
                        }

                        return None;
                    }
                }

                let position = node.global_transform.translation();
                let ui_position = position.truncate();
                let extents = node.node.size() / 2.0;
                let mut min = ui_position - extents;
                if let Some(clip) = node.calculated_clip {
                    min = Vec2::max(min, clip.clip.min);
                }

                // The mouse position relative to the node
                // (0., 0.) is the top-left corner, (1., 1.) is the bottom-right corner
                let relative_cursor_position = cursor_position.map(|cursor_position| {
                    Vec2::new(
                        (cursor_position.x - min.x) / node.node.size().x,
                        (cursor_position.y - min.y) / node.node.size().y,
                    )
                });

                // If the current cursor position is within the bounds of the node, consider it for
                // clicking
                let relative_cursor_position_component = RelativeCursorPosition {
                    normalized: relative_cursor_position,
                };

                let contains_cursor = relative_cursor_position_component.mouse_over();

                // Save the relative cursor position to the correct component
                if let Some(mut node_relative_cursor_position_component) =
                    node.relative_cursor_position
                {
                    *node_relative_cursor_position_component = relative_cursor_position_component;
                }

                if contains_cursor {
                    Some(*entity)
                } else {
                    if let Some(mut interaction) = node.interaction {
                        if *interaction == Interaction::Hovered || (cursor_position.is_none()) {
                            interaction.set_if_neq(Interaction::None);
                        }
                    }
                    None
                }
            } else {
                None
            }
        })
        .collect::<Vec<Entity>>()
        .into_iter();

    // set Clicked or Hovered on top nodes. as soon as a node with a `Block` focus policy is detected,
    // the iteration will stop on it because it "captures" the interaction.
    let mut iter = node_query.iter_many_mut(moused_over_nodes.by_ref());
    while let Some(node) = iter.fetch_next() {
        if let Some(mut interaction) = node.interaction {
            if mouse_clicked {
                // only consider nodes with Interaction "clickable"
                if *interaction != Interaction::Clicked {
                    *interaction = Interaction::Clicked;
                    // if the mouse was simultaneously released, reset this Interaction in the next
                    // frame
                    if mouse_released {
                        state.entities_to_reset.push(node.entity);
                    }
                }
            } else if *interaction == Interaction::None {
                *interaction = Interaction::Hovered;
            }
        }

        match node.focus_policy.unwrap_or(&FocusPolicy::Block) {
            FocusPolicy::Block => {
                break;
            }
            FocusPolicy::Pass => { /* allow the next node to be hovered/clicked */ }
        }
    }
    // reset `Interaction` for the remaining lower nodes to `None`. those are the nodes that remain in
    // `moused_over_nodes` after the previous loop is exited.
    let mut iter = node_query.iter_many_mut(moused_over_nodes);
    while let Some(node) = iter.fetch_next() {
        if let Some(mut interaction) = node.interaction {
            // don't reset clicked nodes because they're handled separately
            if *interaction != Interaction::Clicked {
                interaction.set_if_neq(Interaction::None);
            }
        }
    }
}
