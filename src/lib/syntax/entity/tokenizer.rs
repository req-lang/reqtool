use std::str::CharIndices;

use Keyword::*;

use crate::syntax::{Position, Span};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Keyword {
    Package,
    Part,
    Requirement,
    Import,
    Refines,
    Specializes,
    Derives,
    Is,
    In,
    Let,
}

impl<'a> TryFrom<Lexeme<'a>> for Keyword {
    type Error = &'a str;

    fn try_from(value: Lexeme<'a>) -> Result<Self, Self::Error> {
        use Lexeme::*;

        if let Word(w) = value {
            match w {
                "package" => Ok(Package),
                "part" => Ok(Part),
                "requirement" => Ok(Requirement),
                "import" => Ok(Import),
                "refines" => Ok(Refines),
                "specializes" => Ok(Specializes),
                "derives" => Ok(Derives),
                "in" => Ok(In),
                "is" => Ok(Is),
                "let" => Ok(Let),
                w => Err(w),
            }
        } else {
            Err("")
        }
    }
}

impl Keyword {
    pub fn as_str(&self) -> &'static str {
        match self {
            Package => "package",
            Part => "part",
            Requirement => "requirement",
            Import => "import",
            Refines => "refines",
            Specializes => "specializes",
            Derives => "derives",
            In => "in",
            Is => "is",
            Let => "let",
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum TokenVariant<'a> {
    Identifier(&'a str),
    Expression(&'a str),
    Markup(&'a str),
    Unit(&'a str),
    Reserved(Keyword),
    Tag(&'a str, Option<&'a str>),
}

#[derive(Debug, Eq, PartialEq)]
pub struct Token<'a> {
    pub span: Span,
    pub variant: TokenVariant<'a>,
}

impl<'a> Token<'a> {
    pub fn new(span: Span, variant: TokenVariant<'a>) -> Self {
        Self { span, variant }
    }

    pub fn as_str(&self) -> &str {
        match &self.variant {
            TokenVariant::Identifier(id) => id,
            TokenVariant::Expression(expr) => expr,
            TokenVariant::Markup(informal) => informal,
            TokenVariant::Unit(unit) => unit,
            TokenVariant::Reserved(k) => k.as_str(),
            TokenVariant::Tag(id, _) => id,
        }
    }
}

pub const INFORMAL_DELIMITER: Lexeme = Lexeme::Word("@@");
pub const TAG_DELIMITER: Lexeme<'_> = Lexeme::Punctation('#');
pub const START_UNIT: Lexeme<'_> = Lexeme::Punctation('[');
pub const END_UNIT: Lexeme<'_> = Lexeme::Punctation(']');

#[inline]
pub fn is_reserved(c: char) -> bool {
    match c {
        '[' => true,
        ']' => true,
        '#' => true,
        _ => false,
    }
}

#[derive(Debug, Eq, PartialEq)]
enum Mode {
    Other,
    Expression(Span),
    Unit(Span),
    Informal(Span),
    TagId(Span),
    TagValue(Span, Span),
}

pub struct PositionIterator<'a> {
    chars: CharIndices<'a>,
    position: Position,
    input: &'a str,
    last: Option<(Position, char)>,
}

impl<'a> Iterator for PositionIterator<'a> {
    type Item = (Position, char);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(_) = self.last {
            return std::mem::take(&mut self.last);
        }

        let (idx, c) = self.chars.next()?;
        self.position.index = idx;

        let ret = (self.position, c);
        match c {
            '\r' => {
                if let Some((_, n)) = self.chars.next() {
                    if n != '\n' {
                        self.position.line += 1;
                        self.position.column = 1;
                        self.last = Some((self.position, c));
                    }
                }
                self.position.line += 1;
                self.position.column = 1;
            }

            '\n' => {
                self.position.line += 1;
                self.position.column = 1;
            }

            _ => {
                self.position.column += 1;
            }
        }

        return Some(ret);
    }
}

pub trait IntoPosition<'a> {
    fn positioned(self) -> PositionIterator<'a>;
}

impl<'a> IntoPosition<'a> for CharIndices<'a> {
    fn positioned(self) -> PositionIterator<'a> {
        PositionIterator {
            input: self.as_str(),
            chars: self,
            position: Position::start(),
            last: None,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Lexeme<'a> {
    Punctation(char),
    Word(&'a str),
}

pub struct LexemeIterator<'a> {
    positions: PositionIterator<'a>,
    input: &'a str,
    last: Option<(Span, Lexeme<'a>)>,
}

impl<'a> Iterator for LexemeIterator<'a> {
    type Item = (Span, Lexeme<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        use Lexeme::*;

        let mut has_word = false;

        if let Some(_) = self.last {
            return std::mem::take(&mut self.last);
        }

        while let Some((pos, c)) = self.positions.next() {
            if c.is_whitespace() {
                continue;
            }

            let start = pos;
            let mut position = (pos, c);
            loop {
                let (pos, c) = position;
                if is_reserved(c) {
                    let end = Position::next(pos);
                    let span = Span::new(pos, end);
                    let ret = (span, Punctation(c));
                    if !has_word {
                        return Some(ret);
                    }

                    self.last = Some(ret);
                    let span = Span::new(start, pos);
                    let range = span.range();
                    return Some((span, Word(&self.input[range])));
                }
                if c.is_whitespace() {
                    let span = Span::new(start, pos);
                    let range = span.range();
                    return Some((span, Word(&self.input[range])));
                }

                has_word = true;
                match self.positions.next() {
                    Some(p) => {
                        position = p;
                    }

                    None => {
                        let span = Span::new(start, Position::next(pos));
                        let range = span.range();
                        return Some((span, Word(&self.input[range])));
                    }
                }
            }
        }

        None
    }
}

pub trait IntoLexeme<'a> {
    fn lexemes(self) -> LexemeIterator<'a>;
}

impl<'a> IntoLexeme<'a> for PositionIterator<'a> {
    fn lexemes(self) -> LexemeIterator<'a> {
        LexemeIterator {
            input: self.input,
            positions: self,
            last: None,
        }
    }
}

pub struct TokenIterator<'a> {
    input: &'a str,
    lexemes: LexemeIterator<'a>,
    mode: Mode,
    last: Option<Token<'a>>,
}

impl<'a> Iterator for TokenIterator<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        use Mode::*;

        if let Some(_) = self.last {
            return std::mem::take(&mut self.last);
        }

        while let Some((span, l)) = self.lexemes.next() {
            match self.mode {
                Other => {
                    if l == INFORMAL_DELIMITER {
                        if let Some((start, l)) = self.lexemes.next() {
                            if l != INFORMAL_DELIMITER {
                                self.mode = Informal(start);
                                continue;
                            }

                            let mut span = start;
                            span.end = span.start;
                            let variant = TokenVariant::Markup("");
                            return Some(Token::new(span, variant));
                        }

                        continue;
                    }

                    if l == TAG_DELIMITER {
                        if let Some((span, l)) = self.lexemes.next() {
                            if l != TAG_DELIMITER {
                                self.mode = TagId(span);
                            }
                            continue;
                        }
                    }

                    match Keyword::try_from(l) {
                        Err(w) => {
                            return Some(Token::new(span, TokenVariant::Identifier(w)));
                        }

                        Ok(k) => {
                            match k {
                                Is | In => {
                                    if let Some((span, l)) = self.lexemes.next() {
                                        if l == INFORMAL_DELIMITER {
                                            if let Some((start, l)) = self.lexemes.next() {
                                                if l == INFORMAL_DELIMITER {
                                                    let mut span = start;
                                                    span.end = span.start;
                                                    let variant = TokenVariant::Markup("");
                                                    self.last = Some(Token::new(span, variant));

                                                    let variant = TokenVariant::Reserved(k);
                                                    return Some(Token::new(span, variant));
                                                }

                                                self.mode = Informal(start);
                                            }
                                        } else {
                                            self.mode = Expression(span);
                                        }
                                    }
                                }

                                _ => {}
                            }

                            return Some(Token::new(span, TokenVariant::Reserved(k)));
                        }
                    }
                }

                Expression(mut current) => {
                    if l == START_UNIT {
                        let variant = TokenVariant::Expression(&self.input[current.range()]);
                        let ret = Token::new(current, variant);
                        if let Some((span, _)) = self.lexemes.next() {
                            self.mode = Unit(span);
                        }
                        return Some(ret);
                    }

                    if l == INFORMAL_DELIMITER {
                        let variant = TokenVariant::Expression(&self.input[current.range()]);
                        let ret = Token::new(current, variant);
                        if let Some((start, l)) = self.lexemes.next() {
                            if l != INFORMAL_DELIMITER {
                                self.mode = Informal(start);
                                return Some(ret);
                            }

                            let mut span = start;
                            span.end = span.start;
                            let variant = TokenVariant::Markup("");
                            self.last = Some(Token::new(span, variant));
                            self.mode = Other;
                            return Some(ret);
                        }
                    }

                    if l == TAG_DELIMITER {
                        let variant = TokenVariant::Expression(&self.input[current.range()]);
                        let ret = Token::new(current, variant);

                        if let Some((span, l)) = self.lexemes.next() {
                            if l != TAG_DELIMITER {
                                self.mode = TagId(span);
                                return Some(ret);
                            }
                        }
                        self.mode = Other;
                        return Some(ret);
                    }

                    if let Ok(first) = Keyword::try_from(l) {
                        match first {
                            Package | Part | Requirement | Let => {
                                let variant =
                                    TokenVariant::Expression(&self.input[current.range()]);
                                let ret = Token::new(current, variant);
                                self.last = Some(Token::new(span, TokenVariant::Reserved(first)));
                                self.mode = Other;
                                return Some(ret);
                            }
                            _ => {}
                        }
                    }

                    current.end = span.end;
                    self.mode = Expression(current);
                }

                Unit(mut current) => {
                    if l == END_UNIT {
                        let variant = TokenVariant::Unit(&self.input[current.range()]);
                        let ret = Token::new(current, variant);
                        self.mode = Other;
                        return Some(ret);
                    }

                    current.end = span.end;
                    self.mode = Unit(current);
                }

                Informal(mut current) => {
                    if l == INFORMAL_DELIMITER {
                        let variant = TokenVariant::Markup(&self.input[current.range()]);
                        let ret = Token::new(current, variant);
                        self.mode = Other;
                        return Some(ret);
                    }

                    current.end = span.end;
                    self.mode = Informal(current);
                }

                TagId(current) => {
                    if l == TAG_DELIMITER {
                        let variant = TokenVariant::Tag(&self.input[current.range()], None);
                        let ret = Token::new(current, variant);
                        self.mode = Other;
                        return Some(ret);
                    }

                    self.mode = TagValue(current, span);
                }

                TagValue(id, mut value) => {
                    if l == TAG_DELIMITER {
                        let variant = TokenVariant::Tag(
                            &self.input[id.range()],
                            Some(&self.input[value.range()]),
                        );
                        let ret = Token::new(id.merge(value), variant);
                        self.mode = Other;
                        return Some(ret);
                    }

                    value.end = span.end;
                    self.mode = TagValue(id, value);
                }
            }
        }
        None
    }
}

pub trait IntoTokens<'a> {
    fn tokens(self) -> TokenIterator<'a>;
}

impl<'a> IntoTokens<'a> for LexemeIterator<'a> {
    fn tokens(self) -> TokenIterator<'a> {
        TokenIterator {
            input: self.input,
            lexemes: self,
            mode: Mode::Other,
            last: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::syntax::{
        Position, Span,
        entity::tokenizer::{IntoLexeme, IntoPosition, IntoTokens, Keyword::*, TokenVariant},
    };

    #[test]
    fn produces_correct_spans() {
        let input = "package x package\nis testing something part";
        let tokens = input.char_indices().positioned().lexemes().tokens();
        assert_eq!(
            tokens.into_iter().map(|x| x.span).collect::<Vec<_>>(),
            [
                Span::new(Position::new(1, 1, 0), Position::new(1, 8, 7)),
                Span::new(Position::new(1, 9, 8), Position::new(1, 10, 9)),
                Span::new(Position::new(1, 11, 10), Position::new(1, 18, 17)),
                Span::new(Position::new(2, 1, 18), Position::new(2, 3, 20)),
                Span::new(Position::new(2, 4, 21), Position::new(2, 21, 38)),
                Span::new(Position::new(2, 22, 39), Position::new(2, 26, 43)),
            ]
        );
    }

    #[test]
    fn ignore_keyword_within_comments() {
        let input = "@@ hello requirement is word @@ requirement is a requirement";
        let tokens = input.char_indices().positioned().lexemes().tokens();
        assert_eq!(
            tokens.into_iter().map(|x| x.variant).collect::<Vec<_>>(),
            [
                TokenVariant::Markup("hello requirement is word"),
                TokenVariant::Reserved(Requirement),
                TokenVariant::Reserved(Is),
                TokenVariant::Expression("a"),
                TokenVariant::Reserved(Requirement)
            ]
        );
    }

    #[test]
    fn ignore_at_symbols_within_requirements() {
        let input = "@@ hello @ word @@ requirement is @@ @ @@ requirement";
        let tokens = input.char_indices().positioned().lexemes().tokens();
        assert_eq!(
            tokens.into_iter().map(|x| x.variant).collect::<Vec<_>>(),
            [
                TokenVariant::Markup("hello @ word"),
                TokenVariant::Reserved(Requirement),
                TokenVariant::Reserved(Is),
                TokenVariant::Markup("@"),
                TokenVariant::Reserved(Requirement)
            ]
        );
    }

    #[test]
    fn tokenizes_simple_package() {
        let input = "package x package";
        let tokens = input.char_indices().positioned().lexemes().tokens();
        assert_eq!(
            tokens.into_iter().map(|x| x.variant).collect::<Vec<_>>(),
            [
                TokenVariant::Reserved(Package),
                TokenVariant::Identifier("x"),
                TokenVariant::Reserved(Package)
            ]
        );
    }

    #[test]
    fn tokenizes_comment() {
        let input = "@@ something commenting @@ package x package";
        let tokens = input.char_indices().positioned().lexemes().tokens();
        assert_eq!(
            tokens.into_iter().map(|x| x.variant).collect::<Vec<_>>(),
            [
                TokenVariant::Markup("something commenting"),
                TokenVariant::Reserved(Package),
                TokenVariant::Identifier("x"),
                TokenVariant::Reserved(Package)
            ]
        );
    }

    #[test]
    fn tokenizes_informal_requirement() {
        let input = "requirement x is @@ word @@ requirement";
        let tokens = input.char_indices().positioned().lexemes().tokens();
        assert_eq!(
            tokens.into_iter().map(|x| x.variant).collect::<Vec<_>>(),
            [
                TokenVariant::Reserved(Requirement),
                TokenVariant::Identifier("x"),
                TokenVariant::Reserved(Is),
                TokenVariant::Markup("word"),
                TokenVariant::Reserved(Requirement)
            ]
        );
    }

    #[test]
    fn tokenizes_formal_requirement() {
        let input = "requirement x is y = z requirement";
        let tokens = input.char_indices().positioned().lexemes().tokens();
        assert_eq!(
            tokens.into_iter().map(|x| x.variant).collect::<Vec<_>>(),
            [
                TokenVariant::Reserved(Requirement),
                TokenVariant::Identifier("x"),
                TokenVariant::Reserved(Is),
                TokenVariant::Expression("y = z"),
                TokenVariant::Reserved(Requirement)
            ]
        );
    }

    #[test]
    fn tokenizes_tags() {
        let input = "# something commenting # package x package";
        let tokens = input.char_indices().positioned().lexemes().tokens();
        assert_eq!(
            tokens.into_iter().map(|x| x.variant).collect::<Vec<_>>(),
            [
                TokenVariant::Tag("something", Some("commenting")),
                TokenVariant::Reserved(Package),
                TokenVariant::Identifier("x"),
                TokenVariant::Reserved(Package)
            ]
        );
    }

    #[test]
    fn tokenizes_expression() {
        let input = "part x let y in real part";
        let tokens = input.char_indices().positioned().lexemes().tokens();
        assert_eq!(
            tokens.into_iter().map(|x| x.variant).collect::<Vec<_>>(),
            [
                TokenVariant::Reserved(Part),
                TokenVariant::Identifier("x"),
                TokenVariant::Reserved(Let),
                TokenVariant::Identifier("y"),
                TokenVariant::Reserved(In),
                TokenVariant::Expression("real"),
                TokenVariant::Reserved(Part)
            ]
        );
    }

    #[test]
    fn tokenizes_expression_with_unit() {
        let input = "part x let y is real [m] part";
        let tokens = input.char_indices().positioned().lexemes().tokens();
        assert_eq!(
            tokens.into_iter().map(|x| x.variant).collect::<Vec<_>>(),
            [
                TokenVariant::Reserved(Part),
                TokenVariant::Identifier("x"),
                TokenVariant::Reserved(Let),
                TokenVariant::Identifier("y"),
                TokenVariant::Reserved(Is),
                TokenVariant::Expression("real"),
                TokenVariant::Unit("m"),
                TokenVariant::Reserved(Part)
            ]
        );
    }

    #[test]
    fn tokenizes_expressions() {
        let input = "part x let y is real let z is real part";
        let tokens = input.char_indices().positioned().lexemes().tokens();
        assert_eq!(
            tokens.into_iter().map(|x| x.variant).collect::<Vec<_>>(),
            [
                TokenVariant::Reserved(Part),
                TokenVariant::Identifier("x"),
                TokenVariant::Reserved(Let),
                TokenVariant::Identifier("y"),
                TokenVariant::Reserved(Is),
                TokenVariant::Expression("real"),
                TokenVariant::Reserved(Let),
                TokenVariant::Identifier("z"),
                TokenVariant::Reserved(Is),
                TokenVariant::Expression("real"),
                TokenVariant::Reserved(Part)
            ]
        );
    }

    #[test]
    fn ends_expressions_with_markup() {
        let input = "is x @@ @@ part";
        let tokens = input.char_indices().positioned().lexemes().tokens();
        assert_eq!(
            tokens.into_iter().map(|x| x.variant).collect::<Vec<_>>(),
            [
                TokenVariant::Reserved(Is),
                TokenVariant::Expression("x"),
                TokenVariant::Markup(""),
                TokenVariant::Reserved(Part)
            ]
        );
    }

    #[test]
    fn ends_expressions_with_tag() {
        let input = "is x # # part";
        let tokens = input.char_indices().positioned().lexemes().tokens();
        assert_eq!(
            tokens.into_iter().map(|x| x.variant).collect::<Vec<_>>(),
            [
                TokenVariant::Reserved(Is),
                TokenVariant::Expression("x"),
                TokenVariant::Reserved(Part)
            ]
        );
    }

    #[test]
    fn tokenizes_expressions_with_unit() {
        let input = "part x let y is real [m] let z is real [m] part";
        let tokens = input.char_indices().positioned().lexemes().tokens();
        assert_eq!(
            tokens.into_iter().map(|x| x.variant).collect::<Vec<_>>(),
            [
                TokenVariant::Reserved(Part),
                TokenVariant::Identifier("x"),
                TokenVariant::Reserved(Let),
                TokenVariant::Identifier("y"),
                TokenVariant::Reserved(Is),
                TokenVariant::Expression("real"),
                TokenVariant::Unit("m"),
                TokenVariant::Reserved(Let),
                TokenVariant::Identifier("z"),
                TokenVariant::Reserved(Is),
                TokenVariant::Expression("real"),
                TokenVariant::Unit("m"),
                TokenVariant::Reserved(Part)
            ]
        );
    }

    #[test]
    fn produces_empty_markup() {
        let input = "requirement x is @@ \n @@ requirement";
        let tokens = input.char_indices().positioned().lexemes().tokens();

        assert_eq!(
            tokens.into_iter().map(|x| x.variant).collect::<Vec<_>>(),
            [
                TokenVariant::Reserved(Requirement),
                TokenVariant::Identifier("x"),
                TokenVariant::Reserved(Is),
                TokenVariant::Markup(""),
                TokenVariant::Reserved(Requirement)
            ]
        );
    }

    #[test]
    fn produces_simple_traceability() {
        let input = "requirement x derives y is a requirement";
        let tokens = input.char_indices().positioned().lexemes().tokens();

        assert_eq!(
            tokens.into_iter().map(|x| x.variant).collect::<Vec<_>>(),
            [
                TokenVariant::Reserved(Requirement),
                TokenVariant::Identifier("x"),
                TokenVariant::Reserved(Derives),
                TokenVariant::Identifier("y"),
                TokenVariant::Reserved(Is),
                TokenVariant::Expression("a"),
                TokenVariant::Reserved(Requirement)
            ]
        );
    }

    #[test]
    fn produces_multiple_traceability() {
        let input = "requirement x refines y specializes z is a requirement";
        let tokens = input.char_indices().positioned().lexemes().tokens();

        assert_eq!(
            tokens.into_iter().map(|x| x.variant).collect::<Vec<_>>(),
            [
                TokenVariant::Reserved(Requirement),
                TokenVariant::Identifier("x"),
                TokenVariant::Reserved(Refines),
                TokenVariant::Identifier("y"),
                TokenVariant::Reserved(Specializes),
                TokenVariant::Identifier("z"),
                TokenVariant::Reserved(Is),
                TokenVariant::Expression("a"),
                TokenVariant::Reserved(Requirement)
            ]
        );
    }

    #[test]
    fn ignores_empty_tags() {
        let input = "# # package x package";
        let tokens = input.char_indices().positioned().lexemes().tokens();

        assert_eq!(
            tokens.into_iter().map(|x| x.variant).collect::<Vec<_>>(),
            [
                TokenVariant::Reserved(Package),
                TokenVariant::Identifier("x"),
                TokenVariant::Reserved(Package)
            ]
        );
    }

    #[test]
    fn tokenizes_complex_input() {
        let input = "package p part x let y is real let z is boolean part requirement u refines v is y = z requirement package";
        let tokens = input.char_indices().positioned().lexemes().tokens();
        assert_eq!(
            tokens.into_iter().map(|x| x.variant).collect::<Vec<_>>(),
            [
                TokenVariant::Reserved(Package),
                TokenVariant::Identifier("p"),
                TokenVariant::Reserved(Part),
                TokenVariant::Identifier("x"),
                TokenVariant::Reserved(Let),
                TokenVariant::Identifier("y"),
                TokenVariant::Reserved(Is),
                TokenVariant::Expression("real"),
                TokenVariant::Reserved(Let),
                TokenVariant::Identifier("z"),
                TokenVariant::Reserved(Is),
                TokenVariant::Expression("boolean"),
                TokenVariant::Reserved(Part),
                TokenVariant::Reserved(Requirement),
                TokenVariant::Identifier("u"),
                TokenVariant::Reserved(Refines),
                TokenVariant::Identifier("v"),
                TokenVariant::Reserved(Is),
                TokenVariant::Expression("y = z"),
                TokenVariant::Reserved(Requirement),
                TokenVariant::Reserved(Package)
            ]
        );
    }
}
