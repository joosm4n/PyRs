use crate::{pyrs_parser2::DynError, pyrs_tokentypes::*};

pub struct Expr {
    kind: ExprKind,
}

pub enum ExprKind {
    Assign,
    Keyword,
    Number,
}

#[derive(Debug, Clone)]
pub struct ExprError {
    pub msg: String,
    pub token: TokenOwned,
}
impl ExprError {
    pub fn new_dyn(msg: String, token: &impl TokenData) -> DynError {
        Box::new(ExprError {
            msg,
            token: token.to_owned_token(),
        })
    }
}
impl std::fmt::Display for ExprError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EvalError: {} for {:?}", self.msg, self.token)
    }
}
impl std::error::Error for ExprError {}

pub struct ExprBuilder {
    exprs: Vec<Box<Expr>>,
    block: Option<Expr>,
}

impl ExprBuilder {
    pub fn tokens_to_exprs<'a>(tokens: &[Token<'a>]) -> Result<Vec<Expr>, DynError> {
        let mut builder = ExprBuilder {
            exprs: vec![],
            block: None,
        };
        let mut iter = tokens.iter().peekable();

        while let Some(tk) = iter.next() {
            match tk.kind {
                TokenKind::Name => {
                    if iter
                        .peek()
                        .is_some_and(|t| t.kind == TokenKind::Op(Op::EQUAL))
                    {
                        return Ok(vec![]); // TODO: Tired and will do this later
                    }
                }
                _ => {
                    return Err(ExprError::new_dyn(
                        "Not Implemented/Unknown Token to Expr".into(),
                        tk,
                    ))
                }
            }
        }

        Ok(vec![])
    }
}
