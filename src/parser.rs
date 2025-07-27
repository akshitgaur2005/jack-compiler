use crate::tokenizer::{Token, TokenType, Keyword};
use std::fmt::Debug;

#[derive(Debug, PartialEq, Clone)]
pub struct ClassNode {
    pub name: String,
    pub var_decs: Vec<ClassVarDecNode>,
    pub subroutine_decs: Vec<SubroutineDecNode>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ClassVarDecNode {
    pub kind: ClassVarKind,
    pub var_type: Type,
    pub names: Vec<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ClassVarKind {
    Static,
    Field,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Int,
    Char,
    Boolean,
    ClassName(String),
}

#[derive(Debug, PartialEq, Clone)]
pub struct SubroutineDecNode {
    pub kind: SubroutineKind,
    pub return_type: Option<Type>, // None for void
    pub name: String,
    pub parameters: Vec<(Type, String)>,
    pub body: SubroutineBodyNode,
}

#[derive(Debug, PartialEq, Clone)]
pub enum SubroutineKind {
    Constructor,
    Function,
    Method,
}

#[derive(Debug, PartialEq, Clone)]
pub struct SubroutineBodyNode {
    pub var_decs: Vec<VarDecNode>,
    pub statements: Vec<StatementNode>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct VarDecNode {
    pub var_type: Type,
    pub names: Vec<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum StatementNode {
    Let(LetStatementNode),
    If(IfStatementNode),
    While(WhileStatementNode),
    Do(DoStatementNode),
    Return(ReturnStatementNode),
}

#[derive(Debug, PartialEq, Clone)]
pub struct LetStatementNode {
    pub var_name: String,
    pub index_expr: Option<Box<ExpressionNode>>,
    pub value_expr: Box<ExpressionNode>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct IfStatementNode {
    pub condition: Box<ExpressionNode>,
    pub if_block: Vec<StatementNode>,
    pub else_block: Option<Vec<StatementNode>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct WhileStatementNode {
    pub condition: Box<ExpressionNode>,
    pub body: Vec<StatementNode>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct DoStatementNode {
    pub call: SubroutineCallNode,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ReturnStatementNode {
    pub value: Option<Box<ExpressionNode>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ExpressionNode {
    pub initial_term: Box<TermNode>,
    pub operations: Vec<(char, Box<TermNode>)>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TermNode {
    IntConst(u16),
    StrConst(String),
    KeywordConst(Keyword),
    VarName(String),
    ArrayAccess(String, Box<ExpressionNode>),
    SubroutineCall(SubroutineCallNode),
    Parenthesized(Box<ExpressionNode>),
    UnaryOp(char, Box<TermNode>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct SubroutineCallNode {
    pub receiver: Option<String>,
    pub name: String,
    pub args: Vec<ExpressionNode>,
}


// Parser Implementation
pub struct Parser<'a> {
    tokens: &'a [Token],
    position: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Parser { tokens, position: 0 }
    }

    pub fn parse_class(&mut self) -> Result<ClassNode, String> {
        self.expect_keyword(Keyword::Class)?;
        let name = self.expect_identifier()?;
        self.expect_symbol('{')?;

        let mut var_decs = Vec::new();
        while self.peek_keyword(&[Keyword::Static, Keyword::Field]) {
            var_decs.push(self.parse_class_var_dec()?);
        }

        let mut subroutine_decs = Vec::new();
        while self.peek_keyword(&[Keyword::Constructor, Keyword::Function, Keyword::Method]) {
            subroutine_decs.push(self.parse_subroutine_dec()?);
        }

        self.expect_symbol('}')?;

        Ok(ClassNode { name, var_decs, subroutine_decs })
    }

    fn parse_class_var_dec(&mut self) -> Result<ClassVarDecNode, String> {
        let kind = match self.expect_one_of_keywords(&[Keyword::Static, Keyword::Field])? {
            Keyword::Static => ClassVarKind::Static,
            Keyword::Field => ClassVarKind::Field,
            _ => unreachable!(),
        };
        let var_type = self.parse_type()?;
        let mut names = vec![self.expect_identifier()?];

        while self.match_symbol(',') {
            names.push(self.expect_identifier()?);
        }
        self.expect_symbol(';')?;

        Ok(ClassVarDecNode { kind, var_type, names })
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        if let Some(token) = self.peek() {
            match &token.token_type {
                TokenType::Keyword(k) => match k {
                    Keyword::Int => { self.advance(); Ok(Type::Int) },
                    Keyword::Char => { self.advance(); Ok(Type::Char) },
                    Keyword::Boolean => { self.advance(); Ok(Type::Boolean) },
                    _ => Err(format!("Expected type keyword, found {:?}", k))
                },
                TokenType::Identifier(_) => {
                    let name = self.expect_identifier()?;
                    Ok(Type::ClassName(name))
                },
                _ => Err(format!("Expected type, found {:?}", token)),
            }
        } else {
            Err("Expected type, found EOF".to_string())
        }
    }

    fn parse_subroutine_dec(&mut self) -> Result<SubroutineDecNode, String> {
        let kind = match self.expect_one_of_keywords(&[Keyword::Constructor, Keyword::Function, Keyword::Method])? {
            Keyword::Constructor => SubroutineKind::Constructor,
            Keyword::Function => SubroutineKind::Function,
            Keyword::Method => SubroutineKind::Method,
            _ => unreachable!(),
        };

        let return_type = if self.match_keyword(Keyword::Void) {
            None
        } else {
            Some(self.parse_type()?)
        };

        let name = self.expect_identifier()?;
        self.expect_symbol('(')?;
        let parameters = self.parse_parameter_list()?;
        self.expect_symbol(')')?;
        let body = self.parse_subroutine_body()?;

        Ok(SubroutineDecNode { kind, return_type, name, parameters, body })
    }

    fn parse_parameter_list(&mut self) -> Result<Vec<(Type, String)>, String> {
        let mut params = Vec::new();
        if !self.peek_symbol(')') {
            let p_type = self.parse_type()?;
            let p_name = self.expect_identifier()?;
            params.push((p_type, p_name));

            while self.match_symbol(',') {
                let p_type = self.parse_type()?;
                let p_name = self.expect_identifier()?;
                params.push((p_type, p_name));
            }
        }
        Ok(params)
    }

    fn parse_subroutine_body(&mut self) -> Result<SubroutineBodyNode, String> {
        self.expect_symbol('{')?;
        let mut var_decs = Vec::new();
        while self.peek_keyword(&[Keyword::Var]) {
            var_decs.push(self.parse_var_dec()?);
        }
        let statements = self.parse_statements()?;
        self.expect_symbol('}')?;
        Ok(SubroutineBodyNode { var_decs, statements })
    }

    fn parse_var_dec(&mut self) -> Result<VarDecNode, String> {
        self.expect_keyword(Keyword::Var)?;
        let var_type = self.parse_type()?;
        let mut names = vec![self.expect_identifier()?];

        while self.match_symbol(',') {
            names.push(self.expect_identifier()?);
        }
        self.expect_symbol(';')?;
        Ok(VarDecNode { var_type, names })
    }

    fn parse_statements(&mut self) -> Result<Vec<StatementNode>, String> {
        let mut statements = Vec::new();
        while self.is_statement() {
            statements.push(self.parse_statement()?);
        }
        Ok(statements)
    }

    fn is_statement(&self) -> bool {
        self.peek_keyword(&[Keyword::Let, Keyword::If, Keyword::While, Keyword::Do, Keyword::Return])
    }

    fn parse_statement(&mut self) -> Result<StatementNode, String> {
        if let Some(token) = self.peek() {
            if let TokenType::Keyword(k) = &token.token_type {
                return match k {
                    Keyword::Let => Ok(StatementNode::Let(self.parse_let_statement()?)),
                    Keyword::If => Ok(StatementNode::If(self.parse_if_statement()?)),
                    Keyword::While => Ok(StatementNode::While(self.parse_while_statement()?)),
                    Keyword::Do => Ok(StatementNode::Do(self.parse_do_statement()?)),
                    Keyword::Return => Ok(StatementNode::Return(self.parse_return_statement()?)),
                    _ => Err(format!("Expected statement keyword, found {:?}", k)),
                };
            }
        }
        Err("Expected statement, but found nothing or not a keyword".to_string())
    }

    fn parse_let_statement(&mut self) -> Result<LetStatementNode, String> {
        self.expect_keyword(Keyword::Let)?;
        let var_name = self.expect_identifier()?;
        let mut index_expr = None;
        if self.match_symbol('[') {
            index_expr = Some(Box::new(self.parse_expression()?));
            self.expect_symbol(']')?;
        }
        self.expect_symbol('=')?;
        let value_expr = Box::new(self.parse_expression()?);
        self.expect_symbol(';')?;
        Ok(LetStatementNode { var_name, index_expr, value_expr })
    }

    fn parse_if_statement(&mut self) -> Result<IfStatementNode, String> {
        self.expect_keyword(Keyword::If)?;
        self.expect_symbol('(')?;
        let condition = Box::new(self.parse_expression()?);
        self.expect_symbol(')')?;
        self.expect_symbol('{')?;
        let if_block = self.parse_statements()?;
        self.expect_symbol('}')?;

        let mut else_block = None;
        if self.match_keyword(Keyword::Else) {
            self.expect_symbol('{')?;
            else_block = Some(self.parse_statements()?);
            self.expect_symbol('}')?;
        }
        Ok(IfStatementNode { condition, if_block, else_block })
    }

    fn parse_while_statement(&mut self) -> Result<WhileStatementNode, String> {
        self.expect_keyword(Keyword::While)?;
        self.expect_symbol('(')?;
        let condition = Box::new(self.parse_expression()?);
        self.expect_symbol(')')?;
        self.expect_symbol('{')?;
        let body = self.parse_statements()?;
        self.expect_symbol('}')?;
        Ok(WhileStatementNode { condition, body })
    }

    fn parse_do_statement(&mut self) -> Result<DoStatementNode, String> {
        self.expect_keyword(Keyword::Do)?;
        let call = self.parse_subroutine_call()?;
        self.expect_symbol(';')?;
        Ok(DoStatementNode { call })
    }

    fn parse_return_statement(&mut self) -> Result<ReturnStatementNode, String> {
        self.expect_keyword(Keyword::Return)?;
        let value = if !self.peek_symbol(';') {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };
        self.expect_symbol(';')?;
        Ok(ReturnStatementNode { value })
    }

    fn parse_expression(&mut self) -> Result<ExpressionNode, String> {
        let initial_term = Box::new(self.parse_term()?);
        let mut operations = Vec::new();
        while let Some(op) = self.peek_op() {
            self.advance();
            let term = Box::new(self.parse_term()?);
            operations.push((op, term));
        }
        Ok(ExpressionNode { initial_term, operations })
    }

    fn parse_term(&mut self) -> Result<TermNode, String> {
        if let Some(token) = self.peek() {
            return match token.token_type.clone() {
                TokenType::IntConst(val) => {
                    self.advance();
                    Ok(TermNode::IntConst(val))
                },
                TokenType::StrConst(s) => {
                    self.advance();
                    Ok(TermNode::StrConst(s))
                },
                TokenType::Keyword(k @ Keyword::True) |
                TokenType::Keyword(k @ Keyword::False) |
                TokenType::Keyword(k @ Keyword::Null) |
                TokenType::Keyword(k @ Keyword::This) => {
                    self.advance();
                    Ok(TermNode::KeywordConst(k))
                },
                TokenType::Identifier(_) => {
                    if self.peek_next_symbol('.') || self.peek_next_symbol('(') {
                        Ok(TermNode::SubroutineCall(self.parse_subroutine_call()?))
                    } else if self.peek_next_symbol('[') {
                        let name = self.expect_identifier()?;
                        self.expect_symbol('[')?;
                        let expr = self.parse_expression()?;
                        self.expect_symbol(']')?;
                        Ok(TermNode::ArrayAccess(name, Box::new(expr)))
                    } else {
                        Ok(TermNode::VarName(self.expect_identifier()?))
                    }
                },
                TokenType::Symbol('(') => {
                    self.advance();
                    let expr = self.parse_expression()?;
                    self.expect_symbol(')')?;
                    Ok(TermNode::Parenthesized(Box::new(expr)))
                },
                TokenType::Symbol(op @ '-') | TokenType::Symbol(op @ '~') => {
                    self.advance();
                    let term = self.parse_term()?;
                    Ok(TermNode::UnaryOp(op, Box::new(term)))
                }
                _ => Err(format!("Unexpected token in term: {:?}", token))
            }
        }
        Err("Unexpected EOF in term".to_string())
    }

    fn parse_subroutine_call(&mut self) -> Result<SubroutineCallNode, String> {
        let first_identifier = self.expect_identifier()?;
        let (receiver, name) = if self.match_symbol('.') {
            (Some(first_identifier), self.expect_identifier()?)
        } else {
            (None, first_identifier)
        };

        self.expect_symbol('(')?;
        let args = self.parse_expression_list()?;
        self.expect_symbol(')')?;

        Ok(SubroutineCallNode { receiver, name, args })
    }

    fn parse_expression_list(&mut self) -> Result<Vec<ExpressionNode>, String> {
        let mut expressions = Vec::new();
        if !self.peek_symbol(')') {
            expressions.push(self.parse_expression()?);
            while self.match_symbol(',') {
                expressions.push(self.parse_expression()?);
            }
        }
        Ok(expressions)
    }

    // Utility functions
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn peek_next(&self) -> Option<&Token> {
        self.tokens.get(self.position + 1)
    }

    fn advance(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
    }

    fn expect_identifier(&mut self) -> Result<String, String> {
        if let Some(token) = self.peek() {
            if let TokenType::Identifier(name) = &token.token_type {
                let name_clone = name.clone();
                self.advance();
                return Ok(name_clone);
            }
            return Err(format!("Expected identifier, but found {:?}", token));
        }
        Err("Expected identifier, but found EOF".to_string())
    }

    fn expect_keyword(&mut self, expected: Keyword) -> Result<(), String> {
        if let Some(token) = self.peek() {
            if let TokenType::Keyword(k) = &token.token_type {
                 if *k == expected {
                    self.advance();
                    return Ok(());
                }
            }
            return Err(format!("Expected keyword {:?}, but found {:?}", expected, token));
        }
        Err(format!("Expected keyword {:?}, but found EOF", expected))
    }

    fn expect_one_of_keywords(&mut self, expected: &[Keyword]) -> Result<Keyword, String> {
        if let Some(token) = self.peek() {
            if let TokenType::Keyword(k) = &token.token_type {
                if expected.contains(k) {
                    let keyword_clone = k.clone();
                    self.advance();
                    return Ok(keyword_clone);
                }
            }
            return Err(format!("Expected one of {:?}, but found {:?}", expected, token));
        }
        Err(format!("Expected one of {:?}, but found EOF", expected))
    }

    fn match_keyword(&mut self, expected: Keyword) -> bool {
        if let Some(token) = self.peek() {
             if let TokenType::Keyword(k) = &token.token_type {
                if *k == expected {
                    self.advance();
                    return true;
                }
            }
        }
        false
    }

    fn peek_keyword(&self, keywords: &[Keyword]) -> bool {
        if let Some(token) = self.peek() {
            if let TokenType::Keyword(k) = &token.token_type {
                return keywords.contains(k);
            }
        }
        false
    }

    fn expect_symbol(&mut self, expected: char) -> Result<(), String> {
        if let Some(token) = self.peek() {
            if token.token_type == TokenType::Symbol(expected) {
                self.advance();
                Ok(())
            } else {
                Err(format!("Expected symbol '{}', but found {:?}", expected, token))
            }
        } else {
            Err(format!("Expected symbol '{}', but found EOF", expected))
        }
    }

    fn match_symbol(&mut self, expected: char) -> bool {
        if let Some(token) = self.peek() {
            if token.token_type == TokenType::Symbol(expected) {
                self.advance();
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn peek_symbol(&self, symbol: char) -> bool {
        if let Some(token) = self.peek() {
            if let TokenType::Symbol(s) = token.token_type {
                return s == symbol;
            }
        }
        false
    }

    fn peek_next_symbol(&self, symbol: char) -> bool {
        if let Some(token) = self.peek_next() {
            if let TokenType::Symbol(s) = token.token_type {
                return s == symbol;
            }
        }
        false
    }

    fn peek_op(&self) -> Option<char> {
        if let Some(token) = self.peek() {
            if let TokenType::Symbol(s) = token.token_type {
                if "+-*/&|<>=".contains(s) {
                    return Some(s);
                }
            }
        }
        None
    }
}
