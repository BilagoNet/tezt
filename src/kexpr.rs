//! Boolean expression evaluation, pytest-style. Shared by `-k` (keyword
//! substring match) and `-m` (mark set membership).
//!
//! Grammar:
//!   expr  := or
//!   or    := and ("or" and)*
//!   and   := unary ("and" unary)*
//!   unary := "not" unary | "(" expr ")" | TERM
//!
//! The grammar is identical for both flags; only the meaning of a TERM differs.
//! For `-k` a TERM matches if it is a case-insensitive substring of the test
//! id; for `-m` a TERM matches if it is one of the item's marks. That single
//! difference is captured by the term predicate passed to [`KExpr::eval_with`].

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

    /// Evaluate the expression against a custom term predicate. `-k` uses
    /// substring matching on the test id; `-m` uses exact mark-set membership.
    ///
    /// Keeping the boolean structure (and/or/not/parens) in one evaluator and
    /// parameterizing only the leaf test means both flags share identical
    /// short-circuit and precedence semantics — there is exactly one place
    /// where `not`/`and`/`or` are interpreted.
    pub fn eval_with<F: Fn(&str) -> bool>(&self, pred: &F) -> bool {
        eval_pred(&self.root, pred)
    }

    /// Does the given test id match the expression? (`-k` semantics:
    /// case-insensitive substring match of each term against the id.)
    pub fn matches(&self, test_id: &str) -> bool {
        let hay = test_id.to_lowercase();
        self.eval_with(&|term: &str| hay.contains(&term.to_lowercase()))
    }
}

/// Walk the expression tree, resolving each leaf `Term` through `pred`. The
/// boolean operators short-circuit exactly as Python's do, matching pytest.
fn eval_pred<F: Fn(&str) -> bool>(node: &Node, pred: &F) -> bool {
    match node {
        Node::And(a, b) => eval_pred(a, pred) && eval_pred(b, pred),
        Node::Or(a, b) => eval_pred(a, pred) || eval_pred(b, pred),
        Node::Not(a) => !eval_pred(a, pred),
        Node::Term(t) => pred(t),
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

    #[test]
    fn eval_with_set_membership() {
        // The `-m` use: terms are mark names, matched by exact set membership
        // rather than substring. Same grammar, different leaf predicate.
        use std::collections::HashSet;
        let k = KExpr::compile("slow and not net").unwrap();
        let has = |set: &HashSet<&str>| k.eval_with(&|term: &str| set.contains(term));

        let only_slow: HashSet<&str> = ["slow"].into_iter().collect();
        assert!(has(&only_slow), "slow present, net absent => true");

        let slow_and_net: HashSet<&str> = ["slow", "net"].into_iter().collect();
        assert!(!has(&slow_and_net), "net present => `not net` is false");

        // Membership is exact, not substring: "slowish" must not satisfy `slow`.
        let slowish: HashSet<&str> = ["slowish"].into_iter().collect();
        assert!(!has(&slowish), "exact membership, not substring");

        // `not slow` against an empty set is true (no marks at all).
        let neg = KExpr::compile("not slow").unwrap();
        let empty: HashSet<&str> = HashSet::new();
        assert!(neg.eval_with(&|term: &str| empty.contains(term)));
    }
}
