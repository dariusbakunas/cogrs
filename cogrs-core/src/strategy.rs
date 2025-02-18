pub mod free;
pub mod linear;

#[derive(Clone, Copy)]
pub enum Strategy {
    Linear,
    Free,
}
