use crate::{pyrs_parser2::DynError, pyrs_tokentypes::*};

// #[derive(Debug, Clone, PartialEq)]
// pub struct Expr {
//     data: String,
//     kind: ExprKin

enum VecOr<T1, T2> {
    A(Vec<T1>),
    B(Vec<T2>),
}
enum OrTypes<T1, T2> {
    A(T1),
    B(T2),
}
type Or<T1, T2> = Box<OrTypes<T1, T2>>;

enum SimpleStmt {
    ExprStmt(ExprStmt),
}

struct Identifier {
    name: String,
}

struct StringNorm {}
struct FString {}
struct TString {}

enum VecStrOrFStr {
    String(Vec<StringNorm>),
    FString(Vec<FString>),
}

enum Strings {
    Strs(VecStrOrFStr),
    TString(Vec<TString>),
}

struct Number {
    // TODO:
}

enum Literal {
    Strings(Strings),
    Number(Number),
}

enum CompOperator {
    LessThan(LESSOp),
    GreaterThan(GREATEROp),
    Equals(EQEQUALOp),
    LessEq(LESSEQUALOp),
    GreaterEq(GREATEREQUALOp),
    NotEq(NOTEQUALOp),
    Is(bool),
    In(bool),
}

enum Expr {
    ConditionalExpr(ConditionalExpr),
    LambdaExpr(LambdaExpr),
}
struct ExprList(Vec<Expr>);

struct AttributeRef(Primary, Expr);

struct ConditionalExpr {
    // TODO:
}
struct LambdaExpr {
    // TODO:
}

struct AwaitExpr {}
struct Power(Or<AwaitExpr, Primary>, Option<(DOUBLESTAROp, UExpr)>);
struct UExpr(
    Or<Power, MINUSOp>,
    Or<UExpr, PLUSOp>,
    Or<UExpr, TILDEOp>,
    Box<UExpr>,
);
struct MExpr(
    Or<UExpr, MExpr>,
    STAROp,
    Or<UExpr, MExpr>,
    ATOp,
    Or<MExpr, MExpr>,
    DOUBLESLASHOp,
    Or<UExpr, MExpr>,
    SLASHOp,
    Or<UExpr, MExpr>,
    PERCENTOp,
    UExpr,
);
struct AExpr(Or<MExpr, AExpr>, PLUSOp, Or<MExpr, AExpr>, MINUSOp, MExpr);

struct ShiftExpr(Or<AExpr, ShiftExpr>, Or<LEFTSHIFTOp, RIGHTSHIFTOp>, AExpr);
struct AndExpr(Or<ShiftExpr, AndExpr>, AMPEROp, ShiftExpr);
struct XorExpr(Or<AndExpr, XorExpr>, CIRCUMFLEXOp, AndExpr);
struct OrExpr(Or<XorExpr, OrExpr>, VBAROp, XorExpr);
enum StarredExpr {
    OrExpr(OrExpr),
    Expr(Expr),
}

struct StarredExprList(Vec<StarredExpr>);
struct AssignmentExpr(Identifier, Expr);
struct ProperSlice(Option<Expr>, Option<Expr>, Option<Expr>);
struct Subscription(Primary, Subscript);
struct Comparison(OrExpr, Vec<(CompOperator, OrExpr)>);

enum TupleSubscript {
    SingleSubscript(Vec<SingleSubscript>),
    StarredExpr(Vec<StarredExpr>),
}

enum SingleSubscript {
    ProperSlice(ProperSlice),
    AssignmentExpr(AssignmentExpr),
}

enum Subscript {
    Single(SingleSubscript),
    Tuple(TupleSubscript),
}

enum Target {
    Identifier(Identifier),
    TargetList(Option<TargetList>),
    AttributeRef(AttributeRef),
    Subscription(Subscription),
    Target(Box<Target>),
}
struct TargetList(Vec<Target>);

struct Comprehension(AssignmentExpr, CompFor);
struct CompFor {
    async_: bool,
    target_list: TargetList,
    or_test: OrTest,
    comp_iter: Option<CompIter>,
}
enum CompIter {
    CompFor(Box<CompFor>),
    CompIf(Box<CompIf>),
}
struct CompIf {
    or_test: OrTest,
    comp_iter: Option<CompIter>,
}

struct OrTest(OrTestEnum, Box<AndTest>);
enum OrTestEnum {
    AndTest(Box<AndTest>),
    OrTest(Box<OrTest>),
}

struct AndTest(AndTestEnum, Box<NotTest>);
enum AndTestEnum {
    NotTest(Box<NotTest>),
    AndTest(Box<AndTest>),
}

struct NotTest(NotTestEnum, Box<NotTest>);
enum NotTestEnum {
    Comparison(Comparison),
    NotStr,
}

struct ParenthForm(Option<StarredExpr>);

enum FlexibleExpr {
    AssignmentExpr(AssignmentExpr),
    StarredExpr(StarredExpr),
}
struct FlexibleExprList(FlexibleExpr, Vec<FlexibleExpr>);

struct ListDisplay(Option<VecOr<FlexibleExprList, Comprehension>>);
struct SetDisplay(Option<VecOr<FlexibleExprList, Comprehension>>);

struct DictDisplay(LBRACEOp, Or<DictItem, DictComprehension>, RBRACEOp);
struct DictItemList(DictItem, Vec<(COMMAOp, DictItem)>, Option<COMMAOp>);
struct DictItem(Expr, COLONOp, Or<Expr, DOUBLESTAROp>, OrExpr);
struct DictComprehension(Expr, COLONOp, Expr, CompFor);

struct GeneratorExpr(LPAROp, Expr, CompFor, RPAROp);

struct YieldAtom(LPAROp, YieldExpr, RPAROp);
struct YieldFrom(Expr);
enum YieldExpr {
    YieldList(YieldList),
    YieldFrom(YieldFrom),
}
enum YieldListEnum {
    ExprList(ExprList),
    StarredExpr(StarredExpr),
}
struct YieldList(YieldListEnum, Option<StarredExprList>);

enum Enclosure {
    ParenthForm(ParenthForm),
    ListDisplay(ListDisplay),
    DictDisplay(DictDisplay),
    SetDisplay(SetDisplay),
    GeneratorExpr(GeneratorExpr),
    YieldAtom(YieldAtom),
}

enum Atom {
    True,
    False,
    None,
    Elipsis,
    Identifier(Identifier),
    Literal(Literal),
    Enclosure(Enclosure),
}

enum Primary {
    Atom(Box<Atom>),
    AttributeRef(Box<AttributeRef>),
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
