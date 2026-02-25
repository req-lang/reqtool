use reqtool::{
    renderer::{Render, linter::Renderer},
    *,
};

pub const PATH_ROOT: &str = "tests/data/valid";

fn assert_isomorphic_linter(filename: &str) {
    let path = std::path::PathBuf::from(PATH_ROOT).join(filename);
    let input = std::fs::read_to_string(path).unwrap();
    let mut parser = syntax::NodeParser::default();
    let expected = parser.parse(&input).unwrap();

    let output = Renderer::new().render(&expected);
    let mut parser = syntax::NodeParser::default();
    let actual = parser.parse(&output).unwrap();

    if actual != expected {
        eprintln!("*** INPUT ***\n{}\n\n", input);
        eprintln!("*** OUTPUT ***\n{}\n", output);
    }

    assert!(actual == expected);
}

mod linter {
    use crate::assert_isomorphic_linter;

    #[test]
    fn simple() {
        assert_isomorphic_linter("simple.req");
    }

    #[test]
    fn formal() {
        assert_isomorphic_linter("formal.req");
    }

    #[test]
    fn arithmetics() {
        assert_isomorphic_linter("arithmetics.req");
    }

    #[test]
    fn logical() {
        assert_isomorphic_linter("logical.req");
    }

    #[test]
    fn temporal() {
        assert_isomorphic_linter("temporal.req");
    }

    #[test]
    fn set() {
        assert_isomorphic_linter("set.req");
    }

    #[test]
    fn forall() {
        assert_isomorphic_linter("forall.req");
    }

    #[test]
    fn exists() {
        assert_isomorphic_linter("exists.req");
    }

    #[test]
    fn select() {
        assert_isomorphic_linter("select.req");
    }

    #[test]
    fn when() {
        assert_isomorphic_linter("when.req");
    }

    #[test]
    fn branch() {
        assert_isomorphic_linter("branch.req");
    }

    #[test]
    fn aggregator() {
        assert_isomorphic_linter("aggregator.req");
    }

    #[test]
    fn traceability() {
        assert_isomorphic_linter("traceability.req");
    }
}
