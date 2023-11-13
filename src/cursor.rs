use bevy::prelude::*;

use std::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct WorldCursorSet;

#[derive(Default)]
pub struct WorldCursorPlugin<T: Component> {
    phantom: PhantomData<T>,
}

impl<T: Component> Plugin for WorldCursorPlugin<T> {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldCursor>()
            .register_type::<WorldCursor>()
            .add_systems(Update, update_cursor_pos::<T>.in_set(WorldCursorSet));
    }
}

#[derive(Default, Resource, Deref, DerefMut, Reflect)]
#[reflect(Resource)]
pub struct WorldCursor(pub Vec2);

fn update_cursor_pos<T: Component>(
    camera_query: Query<(&Camera, &GlobalTransform), With<T>>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut cursor_pos: ResMut<WorldCursor>,
) {
    let Ok((camera, transform)) = camera_query.get_single() else {
        return;
    };

    for moved_event in cursor_moved_events.read() {
        let Some(new) = camera
            .viewport_to_world(transform, moved_event.position)
            .map(|ray| ray.origin.truncate())
        else {
            return;
        };
        cursor_pos.0 = new;
    }
}

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_cursor)
            .add_systems(Update, move_cursor);
    }
}

#[derive(Component)]
struct CustomCursor;

fn setup_cursor(
    mut windows: Query<&mut Window>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let mut window: Mut<Window> = windows.single_mut();
    window.cursor.visible = true;
    let cursor_spawn: Vec3 = Vec3::ZERO;

    commands.spawn((
        ImageBundle {
            image: asset_server.load("cursor.png").into(),
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Auto,
                right: Val::Auto,
                bottom: Val::Auto,
                top: Val::Auto,
                ..default()
            },
            z_index: ZIndex::Global(15),
            transform: Transform::from_translation(cursor_spawn),
            ..default()
        },
        CustomCursor,
    ));
}

fn move_cursor(window: Query<&Window>, mut cursor: Query<&mut Style, With<CustomCursor>>) {
    let window: &Window = window.single();
    if let Some(position) = window.cursor_position() {
        let mut img_style = cursor.single_mut();
        img_style.left = Val::Px(position.x - 8.);
        img_style.top = Val::Px(position.y - 8.);
    }
}
