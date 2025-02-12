pub mod free;
pub mod linear;

#[derive(Clone)]
pub enum Strategy {
    Linear,
    Free,
}
