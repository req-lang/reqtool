use std::str::CharIndices;

#[derive(Debug, PartialEq, Eq)]
pub enum Token<'a> {
    Text(&'a str),
    Reference(&'a str),
}

pub const REFERENCE_START: char = '{';
pub const REFERENCE_END: char = '}';

pub struct TokenIterator<'a> {
    start: usize,
    input: &'a str,
    chars: CharIndices<'a>,
    has_yield_last: bool,
}

pub trait IntoTokens<'a> {
    fn tokens(self) -> TokenIterator<'a>;
}

impl<'a> IntoTokens<'a> for CharIndices<'a> {
    fn tokens(self) -> TokenIterator<'a> {
        TokenIterator {
            start: 0,
            input: self.as_str(),
            chars: self,
            has_yield_last: false,
        }
    }
}

impl<'a> Iterator for TokenIterator<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        use Token::*;

        while let Some((idx, c)) = self.chars.next() {
            if c == REFERENCE_START {
                let ret = Text(&self.input[self.start..idx]);
                self.start = idx + 1;
                return Some(ret);
            }

            if c == REFERENCE_END {
                let ret = Reference(&self.input[self.start..idx]);
                self.start = idx + 1;
                return Some(ret);
            }
        }

        if !self.has_yield_last {
            self.has_yield_last = true;
            return Some(Text(&self.input[self.start..self.input.len()]));
        }

        return None;
    }
}

#[cfg(test)]
mod tests {
    use super::IntoTokens;
    use super::Token::*;

    #[test]
    fn tokenizes_simple_input() {
        let input = "some text {a reference} some text";
        let tokens: Vec<_> = input.char_indices().tokens().collect();
        assert_eq!(
            &tokens[..],
            [
                Text("some text "),
                Reference("a reference"),
                Text(" some text")
            ]
        );
    }

    #[test]
    fn tokenizes_reference_at_the_start() {
        let input = "{a reference} some text";
        let tokens: Vec<_> = input.char_indices().tokens().collect();
        assert_eq!(
            &tokens[..],
            [Text(""), Reference("a reference"), Text(" some text")]
        );
    }

    #[test]
    fn tokenizes_reference_at_the_end() {
        let input = "some text {a reference}";
        let tokens: Vec<_> = input.char_indices().tokens().collect();
        assert_eq!(
            &tokens[..],
            [Text("some text "), Reference("a reference"), Text("")]
        );
    }
}
