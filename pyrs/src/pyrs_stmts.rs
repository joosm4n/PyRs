use crate::{pyrs_exprtypes::exprs::*, pyrs_tokentypes::EQUALOp};

pub enum SimpleStmt {
    Expr,
    Assignment,
    AugmentedAssignment,
    AnnotatedAssignement,
    Pass,
    Del,
    Return,
    Yield,
    Raise,
    Break,
    Continue,
    Import,
    Future,
    Global,
    NonLocal,
    Type,
}

pub type ExprStmt = StarredExpr;

pub struct AssignmentStmt(VecNE<(TargetList, EQUALOp)>, Or<StarredExpr, YieldExpr>);
impl AssignmentStmt {
    pub fn new(
        target_list_vec: VecNE<(TargetList, EQUALOp)>,
        or: Or<StarredExpr, YieldExpr>,
    ) -> Self {
        Self {
            0: target_list_vec,
            1: or,
        }
    }
}
