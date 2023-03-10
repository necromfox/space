use super::markers::SelectedBody;
use bevy::prelude::*;

pub struct SelectionRaycastSet;

#[derive(Component)]
pub struct SelectionTargetRedirect(pub Entity);

pub struct DeselectionEvent(pub Entity);
pub struct SelectionEvent(pub Entity);

pub mod systems {
    use bevy::prelude::*;
    use bevy_ecs_markers::params::MarkerMut;
    use bevy_mod_raycast::{RaycastMesh, RaycastMethod, RaycastSource};

    use crate::space::ext::EntityOpsExt;

    use super::{
        DeselectionEvent, SelectedBody, SelectionEvent, SelectionRaycastSet,
        SelectionTargetRedirect,
    };

    pub fn update_raycast_with_cursor(
        mut cursor: EventReader<CursorMoved>,
        mut query: Query<&mut RaycastSource<SelectionRaycastSet>>,
    ) {
        let cursor_position = match cursor.iter().last() {
            Some(cursor_moved) => cursor_moved.position,
            None => return,
        };

        for mut pick_source in &mut query {
            pick_source.cast_method = RaycastMethod::Screenspace(cursor_position);
        }
    }

    pub fn selection_raycast_update(
        camera: Query<&RaycastSource<SelectionRaycastSet>>,
        bodies: Query<
            (Entity, Option<&SelectionTargetRedirect>),
            With<RaycastMesh<SelectionRaycastSet>>,
        >,
        mut selected: MarkerMut<SelectedBody>,
        mut deselection_events: EventWriter<DeselectionEvent>,
        mut selection_events: EventWriter<SelectionEvent>,
    ) {
        use SelectedBody::*;

        let intersection = camera.single().intersections().first();

        for (entity, redirect) in &bodies {
            if intersection.map(|(e, ..)| *e == entity).unwrap_or(false) {
                if selected[Current] == entity {
                    return;
                }

                selection_events.send(SelectionEvent(entity));

                selected[Current] = entity;
                selected[CurrentRedirected] =
                    redirect.map_or(entity, |SelectionTargetRedirect(e)| *e);
                if selected[Previous].is_valid() {
                    deselection_events.send(DeselectionEvent(selected[Previous]));
                }
                selected[PreviousRedirected] = selected[CurrentRedirected];
                selected[Previous] = selected[Current];

                return;
            }
        }

        if selected[Current].is_valid() {
            deselection_events.send(DeselectionEvent(selected[Current]));
            selected[Current].invalidate();
            selected[Previous].invalidate();
            selected[CurrentRedirected].invalidate();
            selected[PreviousRedirected].invalidate();
        }
    }

    pub fn select_current_body(
        bodies: Query<&Handle<StandardMaterial>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        mut selection_events: EventReader<SelectionEvent>,
    ) {
        for SelectionEvent(entity) in selection_events.iter() {
            if let Ok(material_handle) = bodies.get(*entity) {
                materials.get_mut(material_handle).unwrap().base_color = Color::WHITE;
            }
        }
    }

    pub fn deselect_previous_body(
        bodies: Query<&Handle<StandardMaterial>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        mut deselection_events: EventReader<DeselectionEvent>,
    ) {
        for DeselectionEvent(entity) in deselection_events.iter() {
            if let Ok(material_handle) = bodies.get(*entity) {
                materials.get_mut(material_handle).unwrap().base_color = Color::GRAY;
            }
        }
    }
}
