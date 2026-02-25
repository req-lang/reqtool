pub mod debug;
pub mod linter;

pub trait Render<N> {
    fn render(self, node: &N) -> String;
}
