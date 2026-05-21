use crate::{pyrs_parser2::DynError, pyrs_tokentypes::*};

// #[derive(Debug, Clone, PartialEq)]
// pub struct Expr {
//     data: String,
//     kind: ExprKin
mod expressions {

    use crate::pyrs_tokentypes::*;

    pub enum VecOr<T1, T2> {
        A(Vec<T1>),
        B(Vec<T2>),
    }
    pub enum Or<T1, T2> {
        A(Box<T1>),
        B(Box<T2>),
    }
    pub enum VecOr3<T1, T2, T3> {
        A(Vec<T1>),
        B(Vec<T2>),
        C(Vec<T3>),
    }
    pub enum Or3<T1, T2, T3> {
        A(Box<T1>),
        B(Box<T2>),
        C(Box<T3>),
    }
    type VecNE<T> = Vec<T>;
    type CommaThen<T> = (COMMAOp, T);

    pub enum Atom {
        True(TrueKW),
        False(FalseKW),
        None(NoneKW),
        Elipsis(ELLIPSISOp),
        Identifier(Identifier),
        Literal(Literal),
        Enclosure(Enclosure),
    }
    pub enum Enclosure {
        ParenthForm(ParenthForm),
        ListDisplay(ListDisplay),
        DictDisplay(DictDisplay),
        SetDisplay(SetDisplay),
        GeneratorExpr(GeneratorExpr),
        YieldAtom(YieldAtom),
    }

    pub enum Literal {
        Strings(Strings),
        Number(Number),
    }

    pub struct Identifier {
        name: String,
    }

    pub struct StringNorm {}
    pub struct FString {}
    pub struct TString {}

    pub enum Strings {
        Strs(VecOr<StringNorm, FString>),
        TString(Vec<TString>),
    }

    pub struct ParenthForm(LPAROp, Option<StarredExpr>, RPAROp);

    pub struct Comprehension(AssignmentExpr, CompFor);
    pub struct CompFor(
        Option<AsyncKW>,
        ForKW,
        TargetList,
        InKW,
        OrTest,
        Option<CompIter>,
    );
    pub enum CompIter {
        CompFor(Box<CompFor>),
        CompIf(Box<CompIf>),
    }
    pub struct CompIf(IfKW, OrTest, Option<CompIter>);

    pub struct ListDisplay(LSQBOp, Option<Or<FlexibleExprList, Comprehension>>, RSQBOp);

    pub struct SetDisplay(
        LBRACEOp,
        Option<Or<FlexibleExprList, Comprehension>>,
        RBRACEOp,
    );

    pub struct DictDisplay(LBRACEOp, Or<DictItem, DictComprehension>, RBRACEOp);
    pub struct DictItemList(DictItem, Vec<(COMMAOp, DictItem)>, Option<COMMAOp>);
    pub struct DictItem(Or<(Expr, COLONOp, Expr), (DOUBLESTAROp, OrExpr)>);
    pub struct DictComprehension(Expr, COLONOp, Expr, CompFor);

    pub struct GeneratorExpr(LPAROp, Expr, CompFor, RPAROp);

    pub struct YieldAtom(LPAROp, YieldExpr, RPAROp);
    pub struct YieldFrom(YieldKW, FromKW, Expr);
    pub struct YieldExpr(Or<(YieldKW, YieldList), YieldFrom>);

    pub enum Primary {
        Atom(Box<Atom>),
        AttributeRef(Box<AttributeRef>),
        Subscription(Box<Subscription>),
        Call(Box<Call>),
    }

    pub struct AttributeRef(Primary, DOTOp, Expr);

    pub struct Subscription(Primary, LSQBOp, Subscript, RSQBOp);
    pub struct Subscript(Or<SingleSubscript, TupleSubscript>);
    pub struct SingleSubscript(Or<ProperSlice, AssignmentExpr>);
    pub struct ProperSlice(
        Option<Expr>,
        COLONOp,
        Option<Expr>,
        Option<(COLONOp, Option<Expr>)>,
    );
    pub struct TupleSubscript(
        COMMAOp,
        Or<VecNE<SingleSubscript>, Vec<StarredExpr>>,
        Option<COMMAOp>,
    );

    pub struct Call(
        Primary,
        LBRACEOp,
        Option<Or<(ArgumentList, Option<COMMAOp>), Comprehension>>,
        RBRACEOp,
    );

    pub struct ArgListA(
        PositionalArguments,
        Option<(COMMAOp, StarredAndKeywords)>,
        Option<(COMMAOp, KeywordsArguments)>,
    );
    pub struct ArgListB(StarredAndKeywords, Option<(COMMAOp, KeywordsArguments)>);
    pub struct ArgumentList(Or3<ArgListA, ArgListB, KeywordsArguments>);

    pub struct PositionalArguments(PositionalItem, Vec<(COMMAOp, PositionalItem)>);
    pub struct PositionalItem(Or<AssignmentExpr, (COMMAOp, PositionalItem)>);

    pub struct StarredAndKeywords(
        Or<(STAROp, Expr), KeywordItem>,
        VecOr<(COMMAOp, KeywordItem), (COMMAOp, DOUBLESTAROp, Expr)>,
    );

    pub struct KeywordsArguments(VecOr<(COMMAOp, KeywordItem), (COMMAOp, DOUBLESTAROp, Expr)>);
    pub struct KeywordItem(Identifier, EQUALOp, Expr);

    pub struct AwaitExpr(AwaitKW, Primary);

    pub struct Power(Or<AwaitExpr, Primary>, Option<(DOUBLESTAROp, UExpr)>);

    // Unary and Binary Exprs v
    pub enum UExpr {
        A((), Box<Power>),
        B(MINUSOp, Box<UExpr>),
        C(PLUSOp, Box<UExpr>),
        D(TILDEOp, Box<UExpr>),
    }

    pub enum MExpr {
        A(Box<UExpr>),
        B(Box<MExpr>, STAROp, Box<MExpr>),
        C(Box<MExpr>, ATOp, Box<MExpr>),
        D(Box<MExpr>, DOUBLESLASHOp, Box<UExpr>),
        E(Box<MExpr>, SLASHOp, Box<UExpr>),
        F(Box<MExpr>, PERCENTOp, Box<UExpr>),
    }
    pub enum AExpr {
        A(Box<MExpr>),
        B(Box<AExpr>, PLUSOp, Box<MExpr>),
        C(Box<AExpr>, MINUSOp, Box<MExpr>),
    }

    pub struct ShiftExpr(Or<AExpr, (ShiftExpr, Or<LEFTSHIFTOp, RIGHTSHIFTOp>, AExpr)>);
    pub struct AndExpr(Or<ShiftExpr, (AndExpr, AMPEROp, ShiftExpr)>);
    pub struct XorExpr(Or<AndExpr, (XorExpr, CIRCUMFLEXOp, AndExpr)>);
    pub struct OrExpr(Or<XorExpr, (OrExpr, VBAROp, XorExpr)>);
    // Unary and Binary Exprs ^

    // Comparison v
    pub struct Comparison(OrExpr, Vec<(CompOperator, OrExpr)>);
    pub enum CompOperator {
        LessThan(LESSOp),
        GreaterThan(GREATEROp),
        Equals(EQEQUALOp),
        LessEq(LESSEQUALOp),
        GreaterEq(GREATEREQUALOp),
        NotEq(NOTEQUALOp),
        Is(IsKW, Option<NotKW>),
        In(Option<NotKW>, InKW),
    }

    pub struct OrTest(Or<AndTest, (OrTest, OrKW, AndTest)>);
    pub struct AndTest(Or<NotTest, (AndTest, AndKW, NotTest)>);
    pub struct NotTest(Or<Comparison, (NotKW, NotTest)>);
    // Comparison ^

    // Exprs v
    pub struct AssignmentExpr(Option<(Identifier, COLONEQUALOp)>, Expr);
    pub struct ConditionalExpr(OrTest, Option<(IfKW, OrTest, ElseKW, Expr)>);
    pub struct Expr(Or<ConditionalExpr, LambdaExpr>);

    pub struct LambdaExpr(LambdaKW, Option<ParameterList>, COLONOp, Expr);

    pub struct StarredExpr(Or<(STAROp, OrExpr), Expr>);
    pub struct FlexibleExpr(Or<AssignmentExpr, StarredExpr>);
    pub struct FlexibleExprList(FlexibleExpr, Vec<(COMMAOp, FlexibleExpr)>, Option<COMMAOp>);
    pub struct StarredExprList(StarredExpr, Vec<(COMMAOp, StarredExpr)>, Option<COMMAOp>);
    pub struct ExprList(Expr, Vec<(COMMAOp, Expr)>, Option<COMMAOp>);
    pub struct YieldList(Or<ExprList, (StarredExpr, COMMAOp, Option<StarredExprList>)>);

    // TODO: BELOW HERE ------------

    pub struct ParameterList {}

    pub enum Target {
        Identifier(Identifier),
        List(Option<TargetList>),
        AttributeRef(AttributeRef),
        Subscription(Subscription),
        Target(Box<Target>),
    }
    pub struct TargetList(Vec<Target>);
}

use expressions::*;

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
    exprs: Vec<Expr>,
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
                    if iter.peek().is_some_and(|t| {
                        t.kind == TokenKind::Op(Op::new('=').unwrap_or(Op::ATEQUAL_))
                    }) {
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
