use reqtool::{syntax::entity::Entity, *};

pub const PATH_ROOT: &str = "tests/data/valid";

fn assert_valid_parsing_and_analysis(filename: &str) -> Entity {
    let path = std::path::PathBuf::from(PATH_ROOT).join(filename);
    let input = std::fs::read_to_string(path).unwrap();
    let mut parser = syntax::NodeParser::default();
    let result = parser.parse(&input);
    assert!(result.is_ok());

    let root = result.unwrap();
    let analysis = Analysis::from(&root);

    for diagnostic in &analysis.diagnostics {
        eprintln!("{}", diagnostic);
    }
    assert!(analysis.diagnostics.is_empty());

    root
}

mod valid {
    use crate::assert_valid_parsing_and_analysis;

    #[test]
    fn simple() {
        assert_valid_parsing_and_analysis("simple.req");
    }

    #[test]
    fn formal() {
        assert_valid_parsing_and_analysis("formal.req");
    }

    #[test]
    fn arithmetics() {
        assert_valid_parsing_and_analysis("arithmetics.req");
    }

    #[test]
    fn logical() {
        assert_valid_parsing_and_analysis("logical.req");
    }

    #[test]
    fn temporal() {
        assert_valid_parsing_and_analysis("temporal.req");
    }

    #[test]
    fn set() {
        assert_valid_parsing_and_analysis("set.req");
    }

    #[test]
    fn forall() {
        assert_valid_parsing_and_analysis("forall.req");
    }

    #[test]
    fn exists() {
        assert_valid_parsing_and_analysis("exists.req");
    }

    #[test]
    fn select() {
        assert_valid_parsing_and_analysis("select.req");
    }

    #[test]
    fn when() {
        assert_valid_parsing_and_analysis("when.req");
    }

    #[test]
    fn branch() {
        assert_valid_parsing_and_analysis("branch.req");
    }

    #[test]
    fn aggregator() {
        assert_valid_parsing_and_analysis("aggregator.req");
    }

    #[test]
    fn traceability() {
        assert_valid_parsing_and_analysis("traceability.req");
    }
}
