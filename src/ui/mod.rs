use bevy::prelude::*;
use bevy_egui::egui;

pub mod confirmation;
pub mod widget;
pub mod widgets;

pub fn with_world_and_egui_context<T>(
    world: &mut World,
    f: impl FnOnce(&mut World, &mut egui::Context) -> T,
) -> T {
    use bevy::window::PrimaryWindow;
    use bevy_egui::EguiContext;

    let mut state = world.query_filtered::<Entity, (With<EguiContext>, With<PrimaryWindow>)>();
    let entity = state.single(world);
    let mut egui_context = world.entity_mut(entity).take::<EguiContext>().unwrap();

    let ctx = egui_context.get_mut();
    let res = f(world, ctx);
    world.entity_mut(entity).insert(egui_context);

    res
}
