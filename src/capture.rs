use bevy::prelude::*;

#[derive(Component)]
#[relationship(relationship_target = CameraCaptures)]
pub struct CameraSource(pub Entity);

#[derive(Component)]
#[require(Camera)]
#[relationship_target(relationship = CameraSource, linked_spawn)]
pub struct CameraCaptures(Vec<Entity>);

#[derive(Component)]
pub struct CaptureConfig {
    width: u32, height: u32, fps: u32,
}

// TODO: this is dumb, just reuse the screen's. dont offer multiple video outputs.

impl CaptureConfig {
    pub fn res_1080p30() -> Self {
        Self { width: 1080, height: 720, fps: 30}
    }
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self::res_1080p30()
    }
}

pub struct CameraCapturePlugin;

impl CameraCapturePlugin {
    fn setup_capture_targets(
        mut commands: Commands,
        // mut cams: Query<(&CameraCapture, &mut Camera)>,
    ) {
        
    }
}

impl Plugin for CameraCapturePlugin {
    fn build(&self, app: &mut App) {
        todo!()
    }
}
