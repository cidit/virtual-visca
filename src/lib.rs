use bevy::app::{PluginGroup, PluginGroupBuilder};

// pub mod capture;
// pub mod visca_socket;
pub mod visca_camera;
pub mod visca_impl;
pub mod visca_controls;
pub mod cli;


pub struct ViscaPluginGroup;

impl PluginGroup for ViscaPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder ::start::<Self>().add(visca_camera::plugin)
    }
}

