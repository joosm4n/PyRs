use crate::{pyrs_parser2::DynError, pyrs_tokentypes::*};

// #[derive(Debug, Clone, PartialEq)]
// pub struct Expr {
//     data: String,
//     kind: ExprKin

enum VecOr<T1, T2> {
    A(Vec<T1>),
    B(Vec<T2>),
}
enum Or<T1, T2> {
    A(T1),
    B(T2),
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

enum Expr {
    ConditionalExpr(ConditionalExpr),
    LambdaExpr(LambdaExpr),
}
struct ExprList(Vec<Expr>);

struct AttributeRef((Primary, Expr));

struct ConditionalExpr {
    // TODO:
}
struct LambdaExpr {
    // TODO:
}
struct OrExpr {
    // TODO:
}
enum StarredExpr {
    OrExpr(OrExpr),
    Expr(Expr),
}
struct StarredExprList(Vec<StarredExpr>);

struct AssignmentExpr {
    ident: Identifier,
    expr: Expr,
}

enum TupleSubscript {
    SingleSubscript(Vec<SingleSubscript>),
    StarredExpr(Vec<StarredExpr>),
}

struct ProperSlice((Option<Expr>, Option<Expr>, Option<Expr>));

enum SingleSubscript {
    ProperSlice(ProperSlice),
    AssignmentExpr(AssignmentExpr),
}

enum Subscript {
    Single(SingleSubscript),
    Tuple(TupleSubscript),
}
struct Subscription((Primary, Subscript));

enum Target {
    Identifier(Identifier),
    TargetList(Option<TargetList>),
    AttributeRef(AttributeRef),
    Subscription(Subscription),
    Target(Box<Target>),
}
struct TargetList(Vec<Target>);

struct Comprehension {
    assign: AssignmentExpr,
    comp_for: CompFor,
}
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

struct OrTest {
    first: OrTestEnum,
    and_test: Box<AndTest>,
}
enum OrTestEnum {
    AndTest(Box<AndTest>),
    OrTest(Box<OrTest>),
}

struct AndTest {
    first: AndTestEnum,
    not_test: Box<NotTest>,
}
enum AndTestEnum {
    NotTest(Box<NotTest>),
    AndTest(Box<AndTest>),
}

struct NotTest {
    first: NotTestEnum,
    not_test: Box<NotTest>,
}
enum NotTestEnum {
    Comparison(Comparison),
    NotStr,
}

struct ParenthForm {
    starred_expr: Option<StarredExpr>,
}

enum FlexibleExpr {
    AssignmentExpr(AssignmentExpr),
    StarredExpr(StarredExpr),
}
struct FlexibleExprList {
    first: FlexibleExpr,
    others: Vec<FlexibleExpr>,
}

struct ListDisplay(Option<VecOr<FlexibleExprList, Comprehension>>);
struct SetDisplay(Option<VecOr<FlexibleExprList, Comprehension>>);

struct DictDisplay(Option<DictDisplayEnum>);
enum DictDisplayEnum {
    DictItemList(DictItemList),
    DictComprehension(DictComprehension),
}
struct DictItemList {
    first: DictItem,
    other: Vec<DictItem>,
}
enum DictItemEnum {
    Expr(Expr),
    OrExpr(OrExpr),
}
struct DictItem {
    a: Expr,
    b: DictItemEnum,
}
struct DictComprehension {
    a: Expr,
    b: Expr,
    c: CompFor,
}

struct GeneratorExpr {
    expr: Expr,
    comp_for: CompFor,
}

struct YieldAtom(YieldExpr);
struct YieldFrom(Expr);
enum YieldExpr {
    YieldList(YieldList),
    YieldFrom(YieldFrom),
}
enum YieldListEnum {
    ExprList(ExprList),
    StarredExpr(StarredExpr),
}
struct YieldList {
    first: YieldListEnum,
    starred_expr_list: Option<StarredExprList>,
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExprOld {
    Atom,
    /**/ BuiltInConst,
    /*__*/ TrueConst,
    /*__*/ FalseConst,
    /*__*/ NoneConst,
    /*__*/ ElipsisConst,

    /**/ Identifier,

    /**/ Literal,
    /**/ Number,

    /**/ Enclosure,
    /*__*/ ParenthForm,

    /*--*/ ListDisplay,
    /*____*/ FlexibleExprList,

    /*____*/ Comprehension,
    /*______*/ CompFor,
    /*______*/ CompIter,
    /*______*/ CompIf,

    /*__*/ SetDisplay,
    /*____*/ // FlexibleExprList,
    /*____*/ // Comprehension,

    /*__*/ DictDisplay,
    /*____*/ DictItemList,
    /*______*/ DictItem,
    /*____*/ DictComprehension,

    /*__*/ GeneratorExpr,
    /*__*/ YieldAtom,

    Strings,
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

#[derive(Debug)]
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
