pub type Program = Vec<Stmt>;

#[derive(PartialEq, Debug, Clone)]
pub enum Stmt {
    LetStmt(Ident, Expr),
    AssignStmt(Ident, Expr),
    ReturnStmt(Expr),
    ExprStmt(Expr),
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
}

#[derive(PartialEq, Debug, Clone)]
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
}

#[derive(PartialEq, Debug, Clone)]
pub enum Literal {
    IntLiteral(i64),
    BoolLiteral(bool),
    StringLiteral(String),
    NullLiteral,
}

#[derive(PartialEq, Debug, Eq, Clone)]
pub struct Ident(pub String);

#[derive(PartialEq, Debug, Clone)]
pub enum Prefix {
    PrefixPlus,
    PrefixMinus,
    Not,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Infix {
    Plus,
    Minus,
    Divide,
    Multiply,
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
    PProduct,      // *, /
    PPrefix,       // !, -, +
    PCall,         // function calls
    PIndex,        // array[index]
}

#[derive(PartialEq, Debug, Clone)]
pub enum ImportItems {
    All,
    Specific(Vec<String>),
    Single(String),
}