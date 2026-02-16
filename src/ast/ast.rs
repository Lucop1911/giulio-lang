use num_bigint::BigInt;
use std::hash::{Hash, Hasher};

pub type Program = Vec<Stmt>;

#[derive(PartialEq, Debug, Clone, Hash)]
pub enum Stmt {
    LetStmt(Ident, Expr),
    AssignStmt(Ident, Expr),
    FieldAssignStmt {
        object: Box<Expr>,
        field: String,
        value: Box<Expr>,
    },
    IndexAssignStmt {
        target: Box<Expr>,
        index: Box<Expr>,
        value: Box<Expr>,
    },
    ReturnStmt(Expr),
    ExprStmt(Expr),
    ExprValueStmt(Expr),
    FnStmt {
        name: Ident,
        params: Vec<Ident>,
        body: Program,
    },
    StructStmt {
        name: Ident,
        fields: Vec<(Ident, Expr)>,
        methods: Vec<(Ident, Expr)>,
    },
    ImportStmt {
        path: Vec<String>,
        items: ImportItems,
    },
    BreakStmt,
    ContinueStmt,
    ThrowStmt(Expr),
}

#[derive(PartialEq, Debug, Clone, Hash)]
pub enum Expr {
    IdentExpr(Ident),
    LitExpr(Literal),
    PrefixExpr(Prefix, Box<Expr>),
    InfixExpr(Infix, Box<Expr>, Box<Expr>),
    IfExpr {
        cond: Box<Expr>,
        consequence: Program,
        alternative: Option<Program>,
    },
    FnExpr {
        params: Vec<Ident>,
        body: Program,
    },
    CallExpr {
        function: Box<Expr>,
        arguments: Vec<Expr>,
    },
    ArrayExpr(Vec<Expr>),
    HashExpr(Vec<(Expr, Expr)>),
    IndexExpr {
        array: Box<Expr>,
        index: Box<Expr>,
    },
    MethodCallExpr{
        object: Box<Expr>,
        method: String,
        arguments: Vec<Expr>
    },
    StructLiteral {
        name: Ident,
        fields: Vec<(Ident, Expr)>,
    },
    ThisExpr,
    FieldAccessExpr {
        object: Box<Expr>,
        field: String,
    },
    WhileExpr {
        cond: Box<Expr>,
        body: Program,
    },
    ForExpr {
        ident: Ident,
        iterable: Box<Expr>,
        body: Program,
    },
    CStyleForExpr {
        init: Option<Box<Stmt>>,
        cond: Option<Box<Expr>>,
        update: Option<Box<Stmt>>,
        body: Program,
    },
    TryCatchExpr {
        try_body: Program,
        catch_ident: Option<Ident>,
        catch_body: Option<Program>,
        finally_body: Option<Program>,
    },
    AsyncFnExpr {
        params: Vec<Ident>,
        body: Program,
    },
    AwaitExpr(Box<Expr>),
}

#[derive(PartialEq, Debug, Clone)]
pub enum Literal {
    IntLiteral(i64),
    BigIntLiteral(BigInt),
    FloatLitera(f64),
    BoolLiteral(bool),
    StringLiteral(String),
    NullLiteral,
}

impl Hash for Literal {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Literal::IntLiteral(i) => i.hash(state),
            Literal::BigIntLiteral(b) => b.hash(state),
            Literal::FloatLitera(f) => f.to_bits().hash(state),
            Literal::BoolLiteral(b) => b.hash(state),
            Literal::StringLiteral(s) => s.hash(state),
            Literal::NullLiteral => "null".hash(state),
        }
    }
}

#[derive(PartialEq, Debug, Eq, Clone, Hash)]
pub struct Ident(pub String);

#[derive(PartialEq, Debug, Clone, Hash)]
pub enum Prefix {
    PrefixPlus,
    PrefixMinus,
    Not,
}

#[derive(PartialEq, Debug, Clone, Hash)]
pub enum Infix {
    Plus,
    Minus,
    Divide,
    Multiply,
    Modulo,
    Equal,
    NotEqual,
    GreaterThanEqual,
    LessThanEqual,
    GreaterThan,
    LessThan,
    And,
    Or,
}

#[derive(PartialEq, PartialOrd, Debug, Clone)]
pub enum Precedence {
    PLowest,
    POr,           // Lowest logical operator
    PAnd,          // Higher than OR
    PEquals,       // ==, !=
    PLessGreater,  // <, >, <=, >=
    PSum,          // +, -
    PProduct,      // *, /, %
    PPrefix,       // !, -, +
    PCall,         // function calls
    PIndex,        // array[index]
}

#[derive(PartialEq, Debug, Clone, Hash)]
pub enum ImportItems {
    All,
    Specific(Vec<String>),
    Single(String),
}