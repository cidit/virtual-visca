use bevy::ecs::message::Message;
use grafton_visca;

// TODO: move to separate file to imrpove separation of concerns
#[derive(Message, Clone, Debug)]
pub enum Command {
    PanTilt(grafton_visca::command::PanTilt),
    Zoom(grafton_visca::command::Zoom),
}

impl From<grafton_visca::command::PanTilt> for Command {
    fn from(value: grafton_visca::command::PanTilt) -> Self {
        Self::PanTilt(value)
    }
}
