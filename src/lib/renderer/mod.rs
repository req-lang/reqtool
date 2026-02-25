pub mod debug;
pub mod formatter;

pub trait Render<N> {
    fn render(self, node: &N) -> String;
}
