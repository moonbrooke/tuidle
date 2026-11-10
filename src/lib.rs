pub mod game;
pub mod model;
pub mod styles;
pub mod view;
pub mod words;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests;
