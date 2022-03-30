pub mod instance;
pub mod instance_provider;
pub mod instance_type;

pub mod resources;
pub mod state;
pub mod template;

pub type EResult<T> = anyhow::Result<T>;
