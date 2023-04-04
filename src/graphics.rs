use bevy::prelude::*;
use bevy_vfx_bag::{post_processing::lut::Lut, BevyVfxBagPlugin};

pub fn graphics_plugin(app: &mut App) {
    app.add_plugin(BevyVfxBagPlugin::default());
    // app.add_system(test_LUTs);
}

// Cycle through some preset LUTs.
fn test_LUTs(
    mut choice: Local<usize>,
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<Entity, With<Camera>>,
) {
    let choice_now = if keyboard_input.just_pressed(KeyCode::Left) {
        choice.saturating_sub(1)
    } else if keyboard_input.just_pressed(KeyCode::Right) {
        (*choice + 1).min(3)
    } else {
        *choice
    };
    if *choice != choice_now {
        let entity = query.single_mut();
        *choice = choice_now;
        match *choice {
            0 => {
                commands.get_or_spawn(entity).insert(Lut::neo());
                info!("Neo");
            }
            1 => {
                commands.get_or_spawn(entity).insert(Lut::arctic());
                info!("Arctic");
            }
            2 => {
                commands.get_or_spawn(entity).insert(Lut::slate());
                info!("Slate");
            }
            3 => {
                commands.get_or_spawn(entity).remove::<Lut>();
                info!("Disabled (default Bevy colors)");
            }
            _ => unreachable!(),
        }
    }
}
