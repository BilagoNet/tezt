//! `-k` keyword expression evaluation, pytest-style.
//!
//! Grammar:
//!   expr  := or
//!   or    := and ("or" and)*
//!   and   := unary ("and" unary)*
//!   unary := "not" unary | "(" expr ")" | TERM
//!
//! A TERM matches if it is a case-insensitive substring of the test id.

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    And,
    Or,
    Not,
    LParen,
    RParen,
    Term(String),
}

fn tokenize(input: &str) -> Result<Vec<Tok>, String> {
    let mut toks = Vec::new();
    let mut cur = String::new();
    let flush = |cur: &mut String, toks: &mut Vec<Tok>| {
        if cur.is_empty() {
            return;
        }
        let t = match cur.as_str() {
            "and" => Tok::And,
            "or" => Tok::Or,
            "not" => Tok::Not,
            other => Tok::Term(other.to_string()),
        };
        toks.push(t);
        cur.clear();
    };
    for ch in input.chars() {
        match ch {
            '(' => {
                flush(&mut cur, &mut toks);
                toks.push(Tok::LParen);
            }
            ')' => {
                flush(&mut cur, &mut toks);
                toks.push(Tok::RParen);
            }
            c if c.is_whitespace() => flush(&mut cur, &mut toks),
            c => cur.push(c),
        }
    }
    flush(&mut cur, &mut toks);
    if toks.is_empty() {
        return Err("empty -k expression".to_string());
    }
    Ok(toks)
}

#[derive(Debug)]
enum Node {
    And(Box<Node>, Box<Node>),
    Or(Box<Node>, Box<Node>),
    Not(Box<Node>),
    Term(String),
}

struct ParserState {
    toks: Vec<Tok>,
    pos: usize,
}

impl ParserState {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos)
    }
    fn next(&mut self) -> Option<Tok> {
        let t = self.toks.get(self.pos).cloned();
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    fn parse_or(&mut self) -> Result<Node, String> {
        let mut left = self.parse_and()?;
        while matches!(self.peek(), Some(Tok::Or)) {
            self.next();
            let right = self.parse_and()?;
            left = Node::Or(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Node, String> {
        let mut left = self.parse_unary()?;
        while matches!(self.peek(), Some(Tok::And)) {
            self.next();
            let right = self.parse_unary()?;
            left = Node::And(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Node, String> {
        match self.next() {
            Some(Tok::Not) => Ok(Node::Not(Box::new(self.parse_unary()?))),
            Some(Tok::LParen) => {
                let inner = self.parse_or()?;
                match self.next() {
                    Some(Tok::RParen) => Ok(inner),
                    _ => Err("expected ')' in -k expression".to_string()),
                }
            }
            Some(Tok::Term(t)) => Ok(Node::Term(t)),
            other => Err(format!("unexpected token in -k expression: {other:?}")),
        }
    }
}

/// A compiled `-k` expression.
pub struct KExpr {
    root: Node,
}

impl KExpr {
    pub fn compile(input: &str) -> Result<Self, String> {
        let toks = tokenize(input)?;
        let mut p = ParserState { toks, pos: 0 };
        let root = p.parse_or()?;
        if p.pos != p.toks.len() {
            return Err("trailing tokens in -k expression".to_string());
        }
        Ok(Self { root })
    }

    /// Does the given test id match the expression?
    pub fn matches(&self, test_id: &str) -> bool {
        let hay = test_id.to_lowercase();
        eval(&self.root, &hay)
    }
}

fn eval(node: &Node, hay: &str) -> bool {
    match node {
        Node::And(a, b) => eval(a, hay) && eval(b, hay),
        Node::Or(a, b) => eval(a, hay) || eval(b, hay),
        Node::Not(a) => !eval(a, hay),
        Node::Term(t) => hay.contains(&t.to_lowercase()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_substring() {
        let k = KExpr::compile("alpha").unwrap();
        assert!(k.matches("testdata/kfilter/test_names.py::test_alpha"));
        assert!(!k.matches("testdata/kfilter/test_names.py::test_beta"));
    }

    #[test]
    fn case_insensitive() {
        let k = KExpr::compile("ALPHA").unwrap();
        assert!(k.matches("test_alpha"));
    }

    #[test]
    fn and_or_not() {
        let k = KExpr::compile("alpha or beta").unwrap();
        assert!(k.matches("x::test_alpha"));
        assert!(k.matches("x::test_beta"));
        assert!(!k.matches("x::test_delta"));

        let k = KExpr::compile("test and not beta").unwrap();
        assert!(k.matches("x::test_alpha"));
        assert!(!k.matches("x::test_beta"));
    }

    #[test]
    fn parens() {
        let k = KExpr::compile("not (alpha or beta)").unwrap();
        assert!(!k.matches("test_alpha"));
        assert!(k.matches("test_delta"));
    }

    #[test]
    fn bad_exprs() {
        assert!(KExpr::compile("").is_err());
        assert!(KExpr::compile("(alpha").is_err());
        assert!(KExpr::compile("alpha or").is_err());
    }
}
