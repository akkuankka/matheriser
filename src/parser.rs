/* parsing is done by recursive descent */
use logos::Logos;
use std::mem;

#[derive(Logos, Debug, PartialEq, Clone)]
enum Token {
    #[regex(r"[ \t\n\f]+", logos::skip)]
    #[error]
    Error,

    /// A number literal
    #[regex(r"[0-9]+", |lex| lex.slice().parse())]
    INumber(i64),
    #[regex(r"[0-9]+\.[0-9]+", |lex| lex.slice().parse())]
    #[regex(r"\.[0-9]+", |lex| {["0", lex.slice()].concat().parse()})]
    FNumber(f64),

    #[regex(r"[\(\)\*\+-/\^]", |lex| lex.slice().chars().nth(0))]
    Operator(char),

    #[regex(r":[a-zA-Z]+", |lex| {
        let thing = lex.slice()[1..].to_string();
        thing
    })]
    Symbol(String),

    #[regex(r"[a-zA-Z]+", |lex| lex.slice().to_string())]
    Word(String),

    EOF,
}

fn tokenise(stringin: &str) -> Vec<Token> {
    let mut r: Vec<Token> = Token::lexer(stringin).collect();
    r.push(Token::EOF);
    r.reverse();
    r
}

struct Parser {
    current: Token,
    stack: Vec<Token>,
}

impl Parser {
    fn new(tok: Vec<Token>) -> Self {
        let mut tok = tok;
        // println!("tokens are {:?}", tok);
        Parser {
            current: tok.pop().unwrap_or(Token::EOF), // should this be Err, Maybe
            stack: tok,
        }
    }
    ///puts the next element of the stack into current
    fn next(&mut self) -> Result<(), String> {
        match self.stack.pop() {
            Some(t) => {
                self.current = t;
                Ok(())
            }
            None => Err("got to the end of the file while still parsing".to_string()),
        }
    }

    fn pop(&mut self) -> Result<Token, String> {
        match self.stack.pop() {
            Some(t) => {
                let mut res = t;
                mem::swap(&mut self.current, &mut res);
                Ok(res)
            }
            None => Err("got to the end of the file while still parsing".to_string()),
        }
    }

    ///checks if current is a particular token
    fn test(&self, tok: &Token) -> bool {
        self.current == *tok
    }

    fn test_set(&self, toks: &[Token]) -> bool {
        let mut result = false;
        for i in toks {
            result |= *i == self.current
        }
        result
    }

    #[allow(dead_code)] // We're going to use this later to do parsing of custom functions I think
    fn is_word(&self) -> bool {
        match self.current {
            Token::Word(_) => true,
            _ => false,
        }
    }

    // fn expect(&self, tok: &Token) -> Result<(), String> {
    //     if !(self.current == *tok) {
    //         Err(format!("Expected token {:?}, got {:?}", self.current, tok))
    //     } else {
    //         Ok(())
    //     }
    // }
    /// if the token is what we asked for, go next
    fn consume(&mut self, tok: &Token) -> bool {
        if self.test(&tok) {
            let _ = self.next(); // we know this is fine because test is true
            true
        } else {
            false
        }
    }

    fn require(&mut self, tok: Token) -> Result<Token, String> {
        if self.consume(&tok) {
            Ok(tok)
        } else {
            Err(format!("Required token {:?}, got {:?}", self.current, tok))
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum UnaryOp {
    Neg,
    Word(String),
}
impl UnaryOp {
    fn from(tok: &Token) -> Result<UnaryOp, String> {
        match tok {
            Token::Operator('-') => Ok(UnaryOp::Neg),
            Token::Word(bla) => Ok(UnaryOp::Word(bla.clone())),
            _ => Err("Unexpected token parsing unary operator".to_string()),
        }
    }

    fn precedence(&self) -> u8 {
        2
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum BinaryOp {
    Plus,
    Minus,
    Mul,
    Div,
    Exp,
}

impl BinaryOp {
    fn from(tok: &Token) -> Result<Self, String> {
        match tok {
            Token::Operator(c) => Ok(match c {
                '+' => BinaryOp::Plus,
                '-' => BinaryOp::Minus,
                '/' => BinaryOp::Div,
                '*' => BinaryOp::Mul,
                '^' => BinaryOp::Exp,
                _ => return Err(format!("'{}' is not recognised as a binary operator", c)),
            }),
            _ => Err(String::from("Unexpected token parsing unary operator")),
        }
    }

    fn precedence(&self) -> u8 {
        match self {
            Self::Plus | Self::Minus => 1,
            Self::Mul | Self::Div => 3,
            Self::Exp => 4,
        }
    }
}

use crate::eval::Data;
#[derive(Debug, PartialEq)]
pub enum ExprTree {
    Val(Data),
    UNode(UnaryOp, Box<ExprTree>),
    BNode(BinaryOp, Box<ExprTree>, Box<ExprTree>),
}

impl ExprTree {
    fn make_leaf(tok: &Token) -> Result<Self, String> {
        match tok {
            Token::INumber(n) => Ok(ExprTree::Val((*n as i64).into())),
            Token::FNumber(n) => Ok(ExprTree::Val((*n as f64).into())),
            Token::Symbol(n) => Ok(ExprTree::Val((&*n.clone() as &str).to_string().into())),
            _ => Err("Tried to parse something that isn't a number as a number".to_string()),
        }
    }
    fn make_unary_node(op: UnaryOp, tree: ExprTree) -> ExprTree {
        ExprTree::UNode(op, tree.into())
    }
    fn make_binary_node(op: BinaryOp, lhs: ExprTree, rhs: ExprTree) -> ExprTree {
        ExprTree::BNode(op, lhs.into(), rhs.into())
    }
}

fn parse_expression(p: &mut Parser) -> Result<ExprTree, String> {
    let t = recognise(0, p)?;
    if p.current == Token::EOF {
        Ok(t)
    } else {
        Err("Finished parsing the expression, but there were still things on the stack".to_string())
    }
}

fn recognise(n: u8, p: &mut Parser) -> Result<ExprTree, String> {
    let mut t = parse_subexpression(p)?;
    while p.test_set(&[
        // Is `current` a binary operator?
        Token::Operator('+'),
        Token::Operator('-'),
        Token::Operator('*'),
        Token::Operator('/'),
        Token::Operator('^'),
    ]) && BinaryOp::from(&p.current)?.precedence() >= n
    {
        let op = BinaryOp::from(&p.pop()?)?;
        let q = match op {
            BinaryOp::Exp => op.precedence(), // right associative operators
            _ => op.precedence() + 1,         // everything else is left-associative
        };
        let t_ = recognise(q, p)?;
        t = ExprTree::make_binary_node(op, t, t_);
    }
    Ok(t)
}

fn parse_subexpression(p: &mut Parser) -> Result<ExprTree, String> {
    match p.current {
        Token::Operator('-') | Token::Word(_) => {
            let op = UnaryOp::from(&p.pop()?)?;
            let q = op.precedence();
            Ok(ExprTree::make_unary_node(op, recognise(q, p)?))
        }
        Token::Operator('(') => {
            let _ = p.next(); // we know this is safe to do because we know current is something
            let t = recognise(0, p);
            p.require(Token::Operator(')'))?;
            t
        }
        Token::FNumber(_) | Token::INumber(_) | Token::Symbol(_) => {
            Ok(ExprTree::make_leaf(&p.pop()?)?)
        }
        _ => Err(format!(
            "Got an unexpected token: {:?}, rest of stack is {:?}",
            p.current, p.stack
        )),
    }
}

pub fn parse_string(input: &str) -> Result<ExprTree, String> {
    let toks = tokenise(input);
    let mut parser = Parser::new(toks);
    parse_expression(&mut parser)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn correctly_parses_value() {
        assert_eq!(Ok(ExprTree::Val(3.into())), parse_string("3"))
    }
    #[test]
    fn correctly_parses_simple_tree() {
        assert_eq!(
            Ok(ExprTree::BNode(
                BinaryOp::Plus,
                ExprTree::Val(3.into()).into(),
                ExprTree::Val(4.into()).into()
            )),
            parse_string("3+4")
        )
    }
}

// enum OperatorStackMember {
//     Binary(BinaryOp),
//     Unary(UnaryOp),
//     Sentinel,
// }

/* impl OperatorStackMember {
    fn precedes(&self, op: &OperatorStackMember) -> bool {
        match self {
            Unary(_) => true,
            Binary(BinaryOp::Exp) => true,
            Binary(lhs) => match op {
                Unary(_) => false,
                Binary(rhs) => lhs.precedence() >= rhs.precedence(),
                Sentinel => true,
            },
            Sentinel => false,
        }
    }
}

fn pop_operator(
    operators: &mut Vec<OperatorStackMember>,
    operands: &mut Vec<ExprTree>,
) -> Result<(), String> {
    match operators
        .last()
        .ok_or("Unexpected end of operator stack".to_string())?
    {
        Binary(op) => {
            let rhs = operands
                .pop()
                .ok_or("Unexpected end of operand stack".to_string())?;
            let lhs = operands
                .pop()
                .ok_or("Unexpected end of operand stack".to_string())?;
            operators.pop();
            operands.push(ExprTree::make_binary_node(
                *op, // UNWRAP ARGUMENT: we already checked that there is an operator
                lhs,
                rhs,
            ))
        },
        Unary(op) => {
            let operand = operands.pop().ok_or("Unexpected end of operator stack".to_string())?;
            operands.push(ExprTree::make_unary_node(op, operand));
            operators.pop();
        },
        Sentinel => {return Err("Tried to use a parsing artifact as operator".to_string())}
    }
    Ok(())
}

fn push_operator(
    op: OperatorStackMember,
    operators: &mut Vec<OperatorStackMember>,
    operands: &mut Vec<ExprTree>,
) -> Result<(), String>{
    while operators.last().unwrap_or(&OperatorStackMember::Sentinel).precedes(&op) {
        pop_operator(operators, operands)?;
    }
    operators.push(op);
    Ok(())
}

pub fn parse_expression(p: &mut Parser) -> Result<ExprTree, String> {
    let mut operator_stack: Vec<OperatorStackMember> = vec![];
    let mut operand_stack: Vec<ExprTree> = vec![];
    operator_stack.push(OperatorStackMember::Sentinel);
    parse_binary_expression(p, &mut operator_stack, &mut operand_stack)?;
    match operand_stack.last().to_owned() {
        Some(t) => Ok(*t),
        None => Err("Somehow no tree was constructed, 100% programmer error".to_string()),
    }
}

fn parse_binary_expression(
    p: &mut Parser,
    optr_stk: &mut Vec<OperatorStackMember>,
    oprd_stk: &mut Vec<ExprTree>,
) -> Result<(), String> {
    parse_unary_expression(p, &mut optr_stk, &mut oprd_stk)?; // Parse LHS of the expression // by reference?
    while p.test_set(&[
        // Is `current` a binary operator?
        Token::Operator('+'),
        Token::Operator('-'),
        Token::Operator('*'),
        Token::Operator('/'),
        Token::Operator('^'),
    ]) {
        p.next();
        parse_unary_expression(p)?;
    }
    Ok(())
}

fn parse_unary_expression(
    p: &mut Parser,
    optr_stk: &mut Vec<OperatorStackMember>,
    oprd_stk: &mut Vec<ExprTree>,
) -> Result<(), String> {
    match p.current {
        Token::Operator('(') => {
            //Current is a bracket
            p.next(); // yeet the bracket
            parse_binary_expression(p); // parse what's in the bracket
            p.require(Token::Operator(')'))?; // there should be a closing bracket there
        }
        Token::Operator('-') => {
            //this and the following branch
            p.next(); // do the same thing, but are separated because different
            parse_unary_expression(p)?; // AST
        }
        Token::Word(_) => {
            p.next();
            parse_unary_expression(p)?;
        }
        Token::Number(_) => {
            // Finally, if there is a number, we don't care what it is yet
            p.next(); // move on
        }
        _ => return Err(String::from("Unexpected token")),
    }
    Ok(())
}
*/
