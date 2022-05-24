#![allow(dead_code)]

use crate::library::doc::DocType;
use crate::library::doc::DocType::*;
use crate::library::go_parser::go_type::*;
use crate::library::go_parser::helper::*;
use crate::library::lexeme::definition::TokenKind::Identifiers;
use crate::library::lexeme::definition::TokenType::*;
use crate::library::lexeme::definition::{TokenKind, TokenType};
use crate::library::lexeme::token::Token;

#[derive(Debug)]
struct SymbolTable {
    symbol_type: TokenType,
    symbol_modifier: Modifier,
    id_name: String,
    is_assigned: bool,
    is_ptr: bool,
    assigned_val: String,
}

#[derive(Debug)]
struct CStructMem {
    name: String,
    member_type: TokenType,
    identifier: String,
}

impl Clone for SymbolTable {
    fn clone(&self) -> SymbolTable {
        let id = self.id_name.clone();
        let val = self.assigned_val.clone();
        SymbolTable {
            assigned_val: val,
            id_name: id,
            symbol_modifier: self.symbol_modifier,
            symbol_type: self.symbol_type,
            is_ptr: self.is_ptr,
            is_assigned: self.is_assigned,
        }
    }
}

impl Clone for CStructMem {
    fn clone(&self) -> CStructMem {
        CStructMem {
            name: self.name.clone(),
            member_type: self.member_type,
            identifier: self.identifier.clone(),
        }
    }
}

struct Parser {
    from: usize,
    //for symbol table
    once_warned: bool,
    //default false
    in_block_stmnt: bool,
    //default false
    in_expr: bool,
    //default false
    in_switch: bool,
    //defalt false
    strict: bool,
    //default true
    in_main: bool,
    sym_tab: Vec<SymbolTable>,
    // structure book keeping
    struct_mem: Vec<CStructMem>,
    typde_def_table: Vec<String>,
}

pub fn init_parser(lexeme: &Vec<Token>, strict_parser: bool) -> Vec<String> {
    let mut stream: Vec<String> = Vec::new();
    stream.push(CRUST.get_doc().to_string());

    stream.push("package main\n".to_string());
    let mut parser = Parser {
        from: 0,
        once_warned: false,
        in_block_stmnt: false,
        in_expr: false,
        in_switch: false,
        strict: strict_parser,
        in_main: false,
        sym_tab: Vec::new(),
        struct_mem: Vec::new(),
        typde_def_table: Vec::new(),
    };
    stream.append(&mut parser.parse_program(&lexeme));
    stream
}

impl Parser {
    fn parse_program(&mut self, lexeme: &Vec<Token>) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();
        let mut head: usize = 0;
        let mut lookahead: usize;
        let mut temp_lexeme: Vec<Token> = Vec::new();
        while head < lexeme.len() {
            lookahead = head;
            // println!("current index:{}", head);
            // if head == 9 {
            //     println!("current index:{}", head);
            // }
            //match over token kind and token type

            //如果是自定义类型，这里无法识别
            match lexeme[head].get_type() {
                (TokenKind::DataTypes, TokenType::Typedef) => {
                    //typedef STRUCT struct_t;
                    while lexeme[head].get_token_type() != Semicolon {
                        temp_lexeme.push(lexeme[head].clone());
                        head += 1;
                    }
                    temp_lexeme.push(lexeme[head].clone());
                    stream.append(&mut self.parse_typdef(&temp_lexeme));
                    head += 1;
                    // println!("{}", lexeme[head].get_token_value());
                    temp_lexeme.clear();
                }
                // matches any datatype
                (TokenKind::DataTypes, _) | (TokenKind::Modifiers, _) => {
                    // println!("{:#?}", lexeme[head]);
                    //if token is modifiers , move lookahead pointer to next lexeme
                    if lexeme[head].get_token_type() == Signed
                        || lexeme[head].get_token_type() == Unsigned
                    {
                        lookahead += 1;
                    }
                    //To see whats after the given identifier
                    //ex : int a = 0; int a;
                    //     int foo(){}
                    lookahead += 2;
                    if lookahead >= lexeme.len() {
                        head +=1;
                        continue;
                    }
                    //A quick hack to prevent parser going into infinite loop on processing
                    if lookahead < lexeme.len() && lexeme[lookahead].get_token_type() == ScopeResolution {
                        while lookahead < lexeme.len() &&lexeme[lookahead].get_token_type() != LeftBracket {
                            lookahead += 1;
                        }
                    }
                    match lexeme[lookahead].get_token_type() {
                        // function declaration
                        LeftBracket => {
                            //inside the function
                            self.in_block_stmnt = true;
                            //move till end of function argument declaration
                            while lexeme[lookahead].get_token_type() != RightBracket {
                                lookahead += 1;
                            }
                            //move ahead of )
                            lookahead += 1;

                            if lexeme[lookahead].get_token_type() == Semicolon {
                                //parse function block
                                while head < lookahead {
                                    let l: Token = lexeme[head].clone();
                                    temp_lexeme.push(l);
                                    head += 1;
                                }
                                self.in_block_stmnt = false;
                                stream.append(&mut self.parse_function(&temp_lexeme));
                                temp_lexeme.clear();
                            } else {
                                // skip function body declaration
                                if lexeme[lookahead].get_token_type() != LeftCurlyBrace {
                                    lookahead += 1;
                                    // head = lookahead;
                                    //FIXME : Why is continue here ?
                                    //continue; ??
                                }
                                // advance lookahead to end of block
                                lookahead = skip_block(&lexeme, lookahead + 1);
                                // collect entire function block
                                while head < lookahead {
                                    let l: Token = lexeme[head].clone();
                                    temp_lexeme.push(l);
                                    head += 1;
                                }

                                //parse function block
                                stream.append(&mut self.parse_function(&temp_lexeme));
                                temp_lexeme.clear();

                                self.in_block_stmnt = false;
                            }
                        }

                        //array declaration found
                        LeftSquareBracket => {
                            lookahead = skip_stmt(&lexeme, lookahead);

                            // collect variable declaration
                            while head != lookahead {
                                let l: Token = lexeme[head].clone();
                                temp_lexeme.push(l);
                                head += 1;
                            }
                            // parse declaration
                            stream.append(&mut self.parse_array_declaration(&temp_lexeme));
                            temp_lexeme.clear();
                        }

                        // variable declaration or declaration + assignment
                        Semicolon | Comma | Assignment => {
                            lookahead = skip_stmt(&lexeme, lookahead);

                            // collect variable declaration
                            while head < lexeme.len() && head != lookahead {
                                let l: Token = lexeme[head].clone();
                                temp_lexeme.push(l);
                                head += 1;
                            }
                            //  println!(" {:?}", temp_lexeme);
                            // parse declaration
                            stream.append(&mut self.parse_declaration(&temp_lexeme));
                            temp_lexeme.clear();
                        }
                        Identifier => {
                            //in case of pointer declaration : int *a;
                            while lexeme[head].get_token_type() != Semicolon {
                                temp_lexeme.push(lexeme[head].clone());
                                head += 1;
                            }
                            temp_lexeme.push(lexeme[head].clone());
                            stream.append(&mut self.parse_declaration(&temp_lexeme));
                            head += 1;
                            // println!("{}", lexeme[head].get_token_value());
                            temp_lexeme.clear();
                        }

                        _ => {
                            // 防止无限循环
                            head += 1;
                        }
                    };
                }

                // matches if statement
                (TokenKind::Keyword, KeywordIf) => {
                    // let mut temp_lexeme: Vec<Token> = Vec::new();

                    // move lookahead past conditon

                    lookahead = skip_if_condition(lexeme, lookahead);
                    // let mut braceketPair = 0;
                    // while lookahead < lexeme.len()
                    //     && lexeme[lookahead].get_token_type() != RightBracket
                    // {
                    //     if lexeme[lookahead].get_token_type() == LeftBracket {
                    //         braceketPair+=1
                    //     }
                    //     lookahead += 1;
                    // }
                    // while lexeme[lookahead + 1].get_token_type() == RightBracket {
                    //     lookahead += 1;
                    // }
                    // lookahead += 1;

                    // move lookahead past block
                    if lookahead < lexeme.len()
                        && lexeme[lookahead].get_token_kind() == TokenKind::Comments
                    {
                        lookahead += 1;
                    }
                    if lookahead < lexeme.len()
                        && lexeme[lookahead].get_token_type() == LeftCurlyBrace
                    {
                        lookahead = skip_block(&lexeme, lookahead + 1);
                    }
                    // move lookahead past block for 'if' without braces
                    else {
                        lookahead = skip_stmt(&lexeme, lookahead);
                    }
                    // collect if block
                    while head < lexeme.len() && head < lookahead {
                        let l: Token = lexeme[head].clone();
                        temp_lexeme.push(l);
                        head += 1;
                    }

                    // parse if
                    stream.append(&mut self.parse_if(&temp_lexeme));
                    temp_lexeme.clear();
                }

                (TokenKind::Keyword, KeywordElse) => {
                    if let Some(t) = stream.last() {
                        if t == "\n" {
                            stream.pop();
                        }
                    }
                    stream.push("else".to_string());
                    head += 1;
                    lookahead = head;
                    if lexeme[head].get_token_type() == KeywordIf {
                        continue;
                    } else {
                        if lexeme[lookahead].get_token_type() == LeftCurlyBrace {
                            head += 1;
                            lookahead = skip_block(&lexeme, head) - 1;
                        } else {
                            lookahead = skip_stmt(&lexeme, lookahead);
                        }

                        while head < lookahead {
                            let l: Token = lexeme[head].clone();
                            temp_lexeme.push(l);
                            head += 1;
                        }
                        //** parse else body
                        stream.push("{\n".to_string());
                        stream.append(&mut self.parse_program(&temp_lexeme));
                        temp_lexeme.clear();
                        stream.push("}\n".to_string());
                    }
                }

                (TokenKind::Keyword, KeywordSwitch) => {
                    while lookahead < lexeme.len() && lexeme[lookahead].get_token_type() != LeftCurlyBrace {
                        lookahead += 1;
                    }
                    lookahead += 1;

                    lookahead = skip_block(&lexeme, lookahead);
                    while head < lookahead && head < lexeme.len() {
                        let l: Token = lexeme[head].clone();
                        temp_lexeme.push(l);
                        head += 1;
                    }
                    self.in_switch = true;

                    stream.append(&mut self.parse_switch(&temp_lexeme));
                    temp_lexeme.clear();
                    self.in_switch = false;
                }

                (TokenKind::Keyword, KeywordWhile) => {
                    // let mut temp_lexeme: Vec<Token> = Vec::new();

                    // move lookahead past conditon
                    while lexeme[lookahead].get_token_type() != RightBracket {
                        lookahead += 1;
                    }
                    lookahead += 1;

                    // move lookahead past block
                    if lexeme[lookahead].get_token_type() == LeftCurlyBrace {
                        lookahead = skip_block(&lexeme, lookahead + 1);
                    }
                    // move lookahead past block for 'if' without braces
                    else {
                        lookahead = skip_stmt(&lexeme, lookahead);
                    }
                    // collect if block
                    while head < lookahead {
                        let l: Token = lexeme[head].clone();
                        temp_lexeme.push(l);
                        head += 1;
                    }

                    let was_in_switch: bool;
                    was_in_switch = self.in_switch;
                    self.in_switch = false;

                    // parse if
                    stream.append(&mut self.parse_while(&temp_lexeme));
                    self.in_switch = was_in_switch;
                    temp_lexeme.clear();
                }

                // matches do while statement
                (TokenKind::Keyword, KeywordDo) => {
                    // move lookahead past block
                    lookahead = skip_block(&lexeme, lookahead + 2);
                    lookahead = skip_stmt(&lexeme, lookahead);

                    // collect while block
                    while head < lookahead {
                        let l: Token = lexeme[head].clone();
                        temp_lexeme.push(l);
                        head += 1;
                    }
                    // parse while
                    let was_in_switch: bool;
                    was_in_switch = self.in_switch;
                    self.in_switch = false;

                    stream.append(&mut self.parse_dowhile(&temp_lexeme));
                    temp_lexeme.clear();

                    self.in_switch = was_in_switch;
                }
                (TokenKind::Keyword, Using) => {
                    stream
                        .push("//FIXME: Convert the below statement manually,\n/**\n".to_string());
                    while lexeme[head].get_token_type() != Semicolon {
                        stream.push(lexeme[head].get_token_value());
                        head += 1;
                    }
                    stream.push(lexeme[head].get_token_value());
                    stream.push("\n */".to_string());
                    head += 1;
                }
                // matches for statement
                (_, KeywordFor) => {
                    // let mut pre_line = lexeme[lookahead].get_token_line_num();
                    let end = skip_for_condition(lexeme, lookahead);
                    while lookahead < lexeme.len()
                        && lookahead < end //lexeme[lookahead].get_token_type() != RightBracket
                    {
                        lookahead += 1;
                    }
                    // lookahead += 1;
                    if lookahead < lexeme.len()
                        && lexeme[lookahead].get_token_kind() == TokenKind::Comments
                    {
                        // stream.push(lexeme[lookahead].get_token_value());
                        lookahead += 1
                    }
                    if lookahead < lexeme.len()
                        && lexeme[lookahead].get_token_type() == LeftCurlyBrace
                    {
                        lookahead = skip_block(&lexeme, lookahead);
                    } else {
                        lookahead = skip_stmt(&lexeme, lookahead);
                    }

                    while head < lookahead && head < lexeme.len() {
                        let l: Token = lexeme[head].clone();
                        temp_lexeme.push(l);
                        head += 1;
                    }

                    let was_in_switch: bool;
                    was_in_switch = self.in_switch;
                    self.in_switch = false;

                    stream.append(&mut self.parse_for(&temp_lexeme));
                    temp_lexeme.clear();
                    self.in_switch = was_in_switch;
                }

                // matches single and multi-line comment
                (TokenKind::Comments, _) => {
                    //如果是在block后面的注释
                    if lexeme[head].get_token_kind() == TokenKind::Comments
                        && (stream.last() == Some(&";".to_string())
                            || stream.last() == Some(&"}".to_string()))
                    {
                        stream.push("\n".to_string())
                    }
                    stream.push(lexeme[head].get_token_value() + "\n");
                    head += 1;
                }

                // assignment statements
                (_, Identifier) => {
                    // let mut temp_lexeme: Vec<Token> = Vec::new();
                    //identifier = expr
                    //identifier()
                    //identifier+expr
                    //identifier OP_INC|OP_DEC; =>postfix
                    if head + 1 >= lexeme.len() {
                        head +=1;
                        continue;
                    }
                    match lexeme[head + 1].get_type() {
                        (TokenKind::Identifiers, Identifier) => {
                            if self
                                .typde_def_table
                                .contains(&lexeme[head].get_token_value())
                            {
                                lookahead = skip_stmt(&lexeme, lookahead);

                                // collect variable declaration
                                while head != lookahead {
                                    let l: Token = lexeme[head].clone();
                                    temp_lexeme.push(l);
                                    head += 1;
                                }
                                //  println!(" {:?}", temp_lexeme);
                                // parse declaration
                                stream.append(&mut self.parse_declaration(&temp_lexeme));
                            } else {
                                while lexeme[head].get_token_type() != Semicolon {
                                    temp_lexeme.push(lexeme[head].clone());
                                    head += 1;
                                }
                                temp_lexeme.push(lexeme[head].clone());
                                head += 1;
                                stream.append(&mut self.parse_class_decl(&temp_lexeme));
                            }
                            temp_lexeme.clear();
                        }
                        // i = 100, j = 100;
                        (TokenKind::AssignmentOperators, Assignment) => {
                            // move lookahead past statement
                            if head + 3 < lexeme.len() && lexeme[head + 3].get_token_type() == Comma
                            {
                                lookahead = head + 3;
                                while head < lookahead + 1 {
                                    let l: Token = lexeme[head].clone();
                                    temp_lexeme.push(l);
                                    head += 1;
                                }
                                stream.append(&mut self.parse_assignment(&temp_lexeme));
                                temp_lexeme.clear();
                            } else {
                                lookahead = skip_stmt(&lexeme, lookahead);
                                // collect statement
                                while head < lexeme.len() && head < lookahead {
                                    let l: Token = lexeme[head].clone();
                                    temp_lexeme.push(l);
                                    head += 1;
                                }

                                // parse assignment
                                stream.append(&mut self.parse_assignment(&temp_lexeme));
                                temp_lexeme.clear();
                            }
                            stream.push("\n".to_string());
                        }
                        (TokenKind::UnaryOperators, _) => {
                            if self.in_expr != true {
                                stream.push(lexeme[head].get_token_value());
                                stream.push(match lexeme[head + 1].get_token_type() {
                                    Increment => "++".to_string(),
                                    Decrement => "--".to_string(),
                                    _ => " ;".to_string(),
                                });
                                head += 2;
                            } else {
                                head += 2;
                            }
                        }
                        (TokenKind::BinaryOperators, _) => {
                            lookahead = skip_stmt(&lexeme, lookahead);

                            //check if overloaded operators is in effect like << >>
                            if lexeme[head + 2].get_token_type() == StringValue
                                || lexeme[head + 2].get_token_type() == CharValue
                            {
                                stream.push(
                                    "\n//This statement need to be handled manually \n".to_string(),
                                );
                                while head < lookahead {
                                    stream.push(lexeme[head].get_token_value());
                                    head += 1;
                                }
                            } else {
                                // move lookahead past statement
                                // collect statement
                                while head < lookahead && head < lexeme.len() {
                                    let l: Token = lexeme[head].clone();
                                    temp_lexeme.push(l);
                                    head += 1;
                                }

                                // parse assignment
                                stream.append(&mut self.parse_expr(&temp_lexeme));
                                temp_lexeme.clear();
                            }
                        }
                        (TokenKind::SpecialChars, LeftBracket) => {
                            // let mut lbcount = 0;
                            while lexeme[head].get_token_type() != RightBracket {
                                if lexeme[head].get_token_type() == Arrow {
                                    stream.push(".".to_string());
                                } else {
                                    stream.push(lexeme[head].get_token_value());
                                }
                                // if lexeme[head].get_token_type() == LeftBracket{
                                //     lbcount +=1;
                                // }
                                head += 1;
                            }
                            stream.push(lexeme[head].get_token_value());
                            // for _ in 0..lbcount {
                            //    stream.push(")".to_string());
                            // }
                            head += 1;
                        }

                        // a -> x->y(); ==> a.x.y()
                        (TokenKind::SpecialChars, Arrow) => {
                            //insert the previous identifier token
                            stream.push(lexeme[head].get_token_value());
                            stream.push(".".to_string());
                            head += 2;
                        }
                        // (_, LEFT_SBRACKET) => {
                        //     while lexeme[head].get_token_type() != RIGHT_SBRACKET {
                        //         stream.push(lexeme[head].get_token_value());
                        //         head += 1;
                        //     }
                        //     stream.push(lexeme[head].get)
                        // }
                        (_, _) => {
                            if lexeme[head].get_token_type() != RightCurlyBrace {
                                stream.push(lexeme[head].get_token_value());
                            }
                            head += 1;
                        }
                    };
                }

                (TokenKind::UnaryOperators, _) => {
                    stream.push(lexeme[head + 1].get_token_value());
                    stream.push(match lexeme[head].get_token_type() {
                        Increment => "++".to_string(),
                        Decrement => "--".to_string(),
                        _ => " ;".to_string(),
                    });
                    head += 2;
                }

                //结构
                (_, KeywordStruct) => {
                    // struct a {  }
                    if lexeme[head + 2].get_token_type() == LeftCurlyBrace {
                        // lexeme[head + 1].get_token_type() == LeftCurlyBrace {
                        //struct A{};
                        while lexeme[head].get_token_type() != RightCurlyBrace {
                            temp_lexeme.push(lexeme[head].clone());
                            head += 1;
                        }
                        //push the right curly brace
                        temp_lexeme.push(lexeme[head].clone());
                        stream.append(&mut self.parse_struct(&temp_lexeme));
                        temp_lexeme.clear();
                        head += 2; //skip semicolon
                    } else {
                        //struct variable declaration

                        while lexeme[head].get_token_type() != Semicolon {
                            temp_lexeme.push(lexeme[head].clone());
                            head += 1;
                        }
                        temp_lexeme.push(lexeme[head].clone());
                        head += 1;
                        stream.append(&mut self.parse_struct_decl(&temp_lexeme));
                        temp_lexeme.clear();
                    }
                }

                (_, KeywordUnion) => {
                    if lexeme[head + 2].get_token_type() == LeftCurlyBrace {
                        stream.push(UNION.get_doc().to_string());
                        //struct A{};
                        while lexeme[head].get_token_type() != RightCurlyBrace {
                            temp_lexeme.push(lexeme[head].clone());
                            head += 1;
                        }
                        //push the right curly brace
                        temp_lexeme.push(lexeme[head].clone());
                        stream.append(&mut self.parse_union(&temp_lexeme));
                        temp_lexeme.clear();
                        head += 2; //skip semicolon
                    } else {
                        //struct variable declaration

                        while lexeme[head].get_token_type() != Semicolon {
                            temp_lexeme.push(lexeme[head].clone());
                            head += 1;
                        }
                        temp_lexeme.push(lexeme[head].clone());
                        head += 1;
                        stream.append(&mut self.parse_union_decl(&temp_lexeme));
                        temp_lexeme.clear();
                    }
                }

                (_, KeywordClass) => {
                    // class 偏移第二个是{
                    // class B {
                    //    int aa, bb;
                    // };
                    if lexeme[head + 2].get_token_type() == LeftCurlyBrace {
                        //struct A{};
                        while lexeme[head].get_token_type() != RightCurlyBrace
                            || lexeme[head + 1].get_token_type() != Semicolon
                        {
                            temp_lexeme.push(lexeme[head].clone());
                            head += 1;
                        }

                        //push the right curly brace
                        temp_lexeme.push(lexeme[head].clone());
                        stream.append(&mut self.parse_class(&temp_lexeme));
                        temp_lexeme.clear();
                        head += 2; //skip semicolon
                    } else {
                        // 实际情况会比较复杂，
                        // 防止无线限循环
                        head += 1;
                    }
                }

                (_, KeywordEnum) => {
                    while lexeme[head].get_token_type() != Semicolon {
                        temp_lexeme.push(lexeme[head].clone());
                        // stream.push(lexeme[head].get_token_value());
                        head += 1;
                    }
                    stream.append(&mut self.parse_enum(&temp_lexeme));
                    temp_lexeme.clear();
                    head += 1;
                }
                (_, KeywordReturn) => {
                    let mut t = head;
                    // stream.push(NO_RETURN.get_doc().to_string());

                    while t < lexeme.len() && lexeme[t].get_token_type() != Semicolon {
                        t += 1;
                    }

                    if t != lexeme.len() - 1 {
                        while t < lexeme.len() && lexeme[head].get_token_type() != Semicolon {
                            // println!("{:?}", lexeme[head]);

                            stream.push(lexeme[head].get_token_value());
                            head += 1;
                        }
                        stream.push(lexeme[head].get_token_value());
                        head += 1;
                    } else {
                        //convert to shorthand notation
                        // head += 1;
                        if self.in_main {
                            // stream.push("std::process::exit(".to_string());
                            while t < lexeme.len() && lexeme[head].get_token_type() != Semicolon {
                                stream.push(lexeme[head].get_token_value());
                                head += 1;
                            }
                            stream.push(");".to_string());
                        } else {
                            while t < lexeme.len() && lexeme[head].get_token_type() != Semicolon {
                                stream.push(lexeme[head].get_token_value());
                                head += 1;
                            }
                        }
                        stream.push("\n".to_string());
                        head += 1;
                    }
                }
                (_, HeaderInclude) => {
                    if self.once_warned == false {
                        stream.push(INCLUDE_STMT.get_doc().to_string());
                    } else {
                        stream.pop();
                        stream.push("* >>>>>>>>".to_string());
                    }

                    while lexeme[head].get_token_type() != GreaterThan {
                        stream.push(lexeme[head].get_token_value());
                        head += 1;
                    }
                    stream.push(lexeme[head].get_token_value() + "\n");
                    stream.push("**/\n".to_string());
                    head += 1;

                    self.once_warned = true;
                }
                (_, LeftCurlyBrace) => {
                    if head != 0 {
                        stream.push("{\n".to_string());
                        lookahead = head;
                        lookahead = skip_block(&lexeme, lookahead) - 1;
                        // collect while body
                        let mut temp_lexeme: Vec<Token> = Vec::new();
                        while head < lookahead {
                            let l: Token = lexeme[head].clone();
                            temp_lexeme.push(l);
                            head += 1;
                        }
                        stream.append(&mut self.parse_program(&temp_lexeme));
                        stream.push("}\n".to_string());
                        temp_lexeme.clear();
                    }
                    head += 1;
                }
                // if all fails
                (_, _) => {
                    if lexeme[head].get_token_type() != RightCurlyBrace {
                        if lexeme[head].get_token_type() == KeywordBreak {
                            // if !self.in_switch {
                                stream.push(lexeme[head].get_token_value());
                            // }
                        } else {
                            if lexeme[head].get_token_kind() == TokenKind::Comments
                                && stream.last() == Some(&";".to_string())
                            {
                                stream.push("\n".to_string())
                            }
                            stream.push(lexeme[head].get_token_value());
                        }
                    }
                    head += 1;
                }
            };
        }
        //return the rust lexeme to main
        stream
    }

    /**
     * print_lexemes: DEBUG_ONLY
     * prints the lexemes in the lexeme vector
     * from index start to end
     */
    fn print_lexemes(lexeme: &Vec<Token>, start: usize, end: usize) {
        println!("----------lexeme-start------------");
        for i in start..end {
            println!("{}", lexeme[i]);
        }
        println!("----------lexeme-end------------");
    }

    /**
     * parse_function:
     * parse c/c++ function into rust equivalent function
     */
    fn parse_function(&mut self, lexeme: &Vec<Token>) -> Vec<String> {
        let mut temp_lexeme: Vec<Token> = Vec::new();
        let mut head: usize;
        let mut lookahead: usize = 1;
        let mut stream: Vec<String> = Vec::new();
        let mut in_impl: bool = false;
        //if the function has scope resolution we need build a impl `class|struct`

        let mut fucntion_name: String = "".to_string();

        stream.push("func".to_string());
        if lexeme[2].get_token_type() == ScopeResolution {
            stream.push("(".to_string());
            stream.push(lexeme[1].get_token_value());
            stream.push(")".to_string());
            // stream.push("{".to_string());
            in_impl = true;
            lookahead = 3;
        }
        //move lookahead to functiion arguments
        //FIXME : i will f up when overloading operator() in cpp
        let mut warn_operator_overload = false;
        while lexeme[lookahead].get_token_type() != LeftBracket {
            match lexeme[lookahead].get_token_kind() {
                TokenKind::UnaryOperators
                | TokenKind::BinaryOperators
                | TokenKind::AssignmentOperators => {
                    warn_operator_overload = true;
                    fucntion_name.push_str(get_operator_as_fucn_name(&lexeme[lookahead]))
                }
                _ => fucntion_name.push_str(lexeme[lookahead].get_token_value().as_str()),
            }
            lookahead += 1
        }
        lookahead += 1;
        head = lookahead;

        if warn_operator_overload {
            stream.push(DocType::OPERATOR_OVERLOAD.get_doc().to_string());
        }

        stream.push(fucntion_name);
        stream.push("(".to_string());

        // parse arguments differently for functions that are not main
        // since rust does not have arguments or return type for main
        if lexeme[1].get_token_type() != Main {
            // collect arguments
            while lexeme[lookahead].get_token_type() != RightBracket {
                lookahead += 1;
            }
            while head < lookahead {
                let l: Token = lexeme[head].clone();
                temp_lexeme.push(l);
                head += 1;
            }
            // parse arguments
            stream.append(&mut self.parse_arguments(&temp_lexeme));
            temp_lexeme.clear();

            stream.push(")".to_string());

            // parse return type
            if let Some(rust_type) = parse_type(lexeme[0].get_token_type(), Modifier::Default) {
                if rust_type != "void".to_string() {
                    stream.push("".to_string());
                    stream.push(rust_type);
                }
            }

            stream.push("{\n".to_string());
        }
        // declare argc and argv inside main, if required
        else {
            //parsing main function
            self.in_main = true;
            stream.push(")".to_string());
            stream.push("{".to_string());
            if lexeme[head].get_token_type() != RightBracket {
                if self.strict == false {
                    //stream.push(NO_STRICT.get_doc().to_string());
                    stream.push("let mut argv: Vec<_> = std::env::args().collect();".to_string());
                    stream.push("let mut argc = argv.len();".to_string());
                } else {
                    //stream.push(STRICT.get_doc().to_string());

                    stream.push("let argv: Vec<_> = std::env::args().collect();".to_string());
                    stream.push("let argc = argv.len();".to_string());
                }
            }
        }
        while head < lexeme.len() && lexeme[head].get_token_type() != LeftCurlyBrace {
            // if head >= lexeme.len() {
            //     break;
            // }
            head += 1
        }
        head += 1;

        // collect function body
        // len - 1  so that '}' is excluded
        while head < lexeme.len() - 1 {
            let l: Token = lexeme[head].clone();
            temp_lexeme.push(l);
            head += 1;
        }
        // parse function body
        stream.append(&mut self.parse_program(&temp_lexeme));
        stream.push("}".to_string());
        // if in_impl {
        //     stream.push("}".to_string());
        // }
        self.in_main = false;
        stream
    }

    /**
     * parse-arguments:
     * parse c/c++ formal arguments in the function signature
     * into rust equivalent arguments
     */
    fn parse_arguments(&mut self, lexeme: &Vec<Token>) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();
        let mut head: usize = 0;
        let mut temp_lexeme: Vec<Token> = Vec::new();
        while head < lexeme.len() {
            if lexeme[head].get_token_type() == Comma {
                let mut arg_decl = self.parse_argument_declaration(&temp_lexeme);

                stream.append(&mut arg_decl);
                stream.push(",".to_string());
                temp_lexeme.clear();
                head += 1;
                continue;
            }

            temp_lexeme.push(lexeme[head].clone());
            head += 1;
        }
        if !temp_lexeme.is_empty() {
            let mut arg_decl = self.parse_argument_declaration(&temp_lexeme);
            stream.append(&mut arg_decl);
        }
        stream
    }
    //参数声明
    fn parse_argument_declaration(&mut self, lexeme: &Vec<Token>) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();
        let mut is_const = false;
        let mut typ_index = 0;
        if lexeme[0].get_token_type() == KeywordConst {
            is_const = true;
            typ_index = 1;
        }
        let arg_type;
        // get the rust type
        if let Some(rust_type) = parse_type(lexeme[typ_index].get_token_type(), Modifier::Default) {
            arg_type = rust_type;
        } else {
            // if type parser dint return Some type, then it must be user defined type.
            //TODO : should check the typedef table
            arg_type = lexeme[typ_index].get_token_value();
        }
        let mut identifier_idx = 1;

        let mut reference = false;
        let reference_idx = 1 + typ_index;
        if reference_idx < lexeme.len() && lexeme[reference_idx].get_token_kind() != Identifiers {
            if lexeme[reference_idx].get_token_type() == BitwiseAnd
                || lexeme[reference_idx].get_token_type() == Multiplication
            {
                reference = true;
            }
            identifier_idx = reference_idx + 1;
        }
        if reference
            && reference_idx < lexeme.len() - 1
            && lexeme[reference_idx + 1].get_token_kind() != Identifiers
        {
            if lexeme[reference_idx + 1].get_token_type() == BitwiseAnd
                || lexeme[reference_idx + 1].get_token_type() == Multiplication
            {
                reference = true;
            }
            identifier_idx = identifier_idx + 1;
        }

        if identifier_idx < lexeme.len() {
            let identifier = lexeme[identifier_idx].get_token_value();

            stream.push(identifier);
            stream.push(" ".to_string());
        }

        if reference {
            stream.push("*".to_string());
            if !self.strict && !is_const {
                stream.push("".to_string());
            }
        }
        stream.push(arg_type);
        stream
    }

    /**
     * parse_declaration:
     * parse c/c++ declaration into rust
     * equivalent statements */
    fn parse_declaration_inline(&mut self, lexeme: &Vec<Token>) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();

        //  let mut sym_tab: Vec<SymbolTable> = Vec::new();
        //self.sym_tab.clear();
        let mut sym: SymbolTable = SymbolTable {
            symbol_type: Others,
            symbol_modifier: Modifier::Default,
            id_name: "undefined_var".to_string(),
            is_assigned: false,
            is_ptr: false,
            assigned_val: "NONE".to_string(),
        };

        //check if there is any modifier present

        let (token_kind, token_type) = lexeme[0].get_type();

        let mut type_index = 0;
        if token_kind == TokenKind::Modifiers {
            //type name can be found in next lexeme
            type_index = 1;
            match token_type {
                Signed => {
                    sym.symbol_modifier = Modifier::Signed;
                }
                Unsigned => {
                    sym.symbol_modifier = Modifier::Unsigned;
                }
                KeywordStatic => {
                    sym.symbol_modifier = Modifier::Static;
                }
                KeywordConst => {
                    sym.symbol_modifier = Modifier::Const;
                }
                _ => {}
            }
        }

        let type_token = &lexeme[type_index];
        let typdef_type = type_token.get_token_value(); //get the type name

        let mut head: usize = type_index + 1;
        //let sym_idx:usize=0;
        while head < lexeme.len() {
            match lexeme[head].get_token_type() {
                Identifier => sym.id_name = lexeme[head].get_token_value(),

                Assignment => {
                    sym.is_assigned = true;
                    sym.assigned_val = "".to_string();
                    head += 1;
                    let mut br = 0;
                    if sym.is_ptr == true {
                        if lexeme[head].get_token_type() == Null {
                            while lexeme[head].get_token_type() != Semicolon
                                && lexeme[head].get_token_type() != Comma
                            {
                                head += 1;
                            }
                            sym.is_assigned = false;
                        } else {
                            head += 1;
                        }
                    }
                    let mut temp_lex: Vec<Token> = Vec::new();
                    while lexeme[head].get_token_type() != Semicolon
                        && !(br == 0 && lexeme[head].get_token_type() == Comma)
                    {
                        if lexeme[head].get_token_type() == LeftBracket {
                            br += 1;
                        }
                        if lexeme[head].get_token_type() == RightBracket {
                            br -= 1;
                        }
                        temp_lex.push(lexeme[head].clone());
                        //parse assigned value for expression

                        head += 1;
                    }
                    temp_lex.push(lexeme[head].clone());
                    let a_val = self.parse_expr(&temp_lex);
                    let mut a_value = String::new();
                    for val in a_val {
                        a_value = a_value + &val;
                    }
                    sym.assigned_val.push_str(a_value.as_str());

                    continue;
                }

                Semicolon | Comma => {
                    // used enum value in the symbol table
                    sym.symbol_type = type_token.get_token_type();
                    //       println!("SYM TYPE {}",sym.typ);
                    self.sym_tab.push(sym.clone());
                }
                //int * a ;
                Multiplication => {
                    sym.is_ptr = true;
                }
                _ => {
                    sym.assigned_val.push_str(&lexeme[head].get_token_value());
                }
            };
            head += 1;
        }

        // if self.strict == false {
        //     stream.push(NO_STRICT.get_doc().to_string());
        // } else {
        //     stream.push(STRICT.get_doc().to_string());
        // }

        //from `from` start declaration statement generation
        let (_, sym_table_right) = self.sym_tab.split_at(self.from);
        for i in sym_table_right {
            // get identifier
            //for declaration out of any blocks(global)
            self.from += 1;
            // match i.symbol_modifier {
            //     Modifier::Const => {
            //         stream.push("const".to_string());
            //     }
            //     _ => {
            //         if self.strict == false {
            //             if self.in_block_stmnt == true {
            //                 stream.push("var".to_string());
            //             } else {
            //                 stream.push("var".to_string());
            //             }
            //         } else {
            //             if self.in_block_stmnt == true {
            //                 stream.push("var".to_string());
            //             } else {
            //                 stream.push("var".to_string());
            //             }
            //         }
            //     }
            // }
            stream.push(i.id_name.clone());
            // stream.push(" ".to_string());

            // get the rust type
            // if let Some(rust_type) = parse_type(i.symbol_type, i.symbol_modifier) {
            //     if rust_type == "_".to_string() {
            //         //not able to find the type, let the rust compiler do the type inference.
            //         stream.pop();
            //     } else {
            //         stream.push(rust_type);
            //     }
            // } else {
            //     // if type parser dint return Some type, then it must be user defined type.
            //     //TODO : should check the typedef table
            //     stream.push(typdef_type.clone());
            // }

            // take care of assignment
            if i.is_assigned {
                stream.push(":=".to_string());
                if i.is_ptr == true {
                    stream.push("&".to_string());
                }
                // if self.strict == false && i.is_ptr == true {
                //     stream.push("".to_string());
                // }

                stream.push((&i.assigned_val).to_string());
            }
            if i.is_ptr == true {
                stream.push("&".to_string());

                if self.strict == false {
                    stream.push("&".to_string());
                }
            }
        }
        stream
    }
    /**
     * parse_declaration:
     * parse c/c++ declaration into rust
     * equivalent statements */
    fn parse_declaration(&mut self, lexeme: &Vec<Token>) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();

        //  let mut sym_tab: Vec<SymbolTable> = Vec::new();
        //self.sym_tab.clear();
        let mut sym: SymbolTable = SymbolTable {
            symbol_type: Others,
            symbol_modifier: Modifier::Default,
            id_name: "undefined_var".to_string(),
            is_assigned: false,
            is_ptr: false,
            assigned_val: "NONE".to_string(),
        };

        //check if there is any modifier present

        let (token_kind, token_type) = lexeme[0].get_type();

        let mut type_index = 0;
        if token_kind == TokenKind::Modifiers {
            //type name can be found in next lexeme
            type_index = 1;
            match token_type {
                Signed => {
                    sym.symbol_modifier = Modifier::Signed;
                }
                Unsigned => {
                    sym.symbol_modifier = Modifier::Unsigned;
                }
                KeywordStatic => {
                    sym.symbol_modifier = Modifier::Static;
                }
                KeywordConst => {
                    sym.symbol_modifier = Modifier::Const;
                }
                _ => {}
            }
        }

        let type_token = &lexeme[type_index];
        let typdef_type = type_token.get_token_value(); //get the type name

        let mut head: usize = type_index + 1;
        //let sym_idx:usize=0;
        while head < lexeme.len() {
            match lexeme[head].get_token_type() {
                Identifier => sym.id_name = lexeme[head].get_token_value(),

                Assignment => {
                    sym.is_assigned = true;
                    sym.assigned_val = "".to_string();
                    // head += 1;
                    let mut br = 0;
                    if sym.is_ptr == true {
                        if lexeme[head].get_token_type() == Null {
                            while lexeme[head].get_token_type() != Semicolon
                                && lexeme[head].get_token_type() != Comma
                            {
                                head += 1;
                            }
                            sym.is_assigned = false;
                        } else {
                            head += 1;
                        }
                    }else{
                        head+=1;
                    }
                    let mut temp_lex: Vec<Token> = Vec::new();
                    while head < lexeme.len()
                        && lexeme[head].get_token_type() != Semicolon
                        && !(br == 0 && lexeme[head].get_token_type() == Comma)
                    {
                        if lexeme[head].get_token_type() == LeftBracket {
                            br += 1;
                        }
                        if lexeme[head].get_token_type() == RightBracket {
                            br -= 1;
                        }
                        temp_lex.push(lexeme[head].clone());
                        //parse assigned value for expression

                        head += 1;
                    }
                    if head < lexeme.len() {
                        temp_lex.push(lexeme[head].clone());
                    }
                    let a_val = self.parse_expr(&temp_lex);
                    let mut a_value = String::new();
                    for val in a_val {
                        a_value = a_value + &val;
                    }
                    sym.assigned_val.push_str(a_value.as_str());

                    continue;
                }

                Semicolon | Comma => {
                    // used enum value in the symbol table
                    sym.symbol_type = type_token.get_token_type();
                    //       println!("SYM TYPE {}",sym.typ);
                    self.sym_tab.push(sym.clone());
                }
                //int * a ;
                Multiplication => {
                    sym.is_ptr = true;
                }
                _ => {
                    sym.assigned_val.push_str(&lexeme[head].get_token_value());
                }
            };
            head += 1;
        }

        // if self.strict == false {
        //     stream.push(NO_STRICT.get_doc().to_string());
        // } else {
        //     stream.push(STRICT.get_doc().to_string());
        // }

        //from `from` start declaration statement generation
        let (_, sym_table_right) = self.sym_tab.split_at(self.from);
        for i in sym_table_right {
            // get identifier
            //for declaration out of any blocks(global)
            self.from += 1;
            match i.symbol_modifier {
                Modifier::Const => {
                    stream.push("const".to_string());
                }
                _ => {
                    if self.strict == false {
                        if self.in_block_stmnt == true {
                            stream.push("var".to_string());
                        } else {
                            stream.push("var".to_string());
                        }
                    } else {
                        if self.in_block_stmnt == true {
                            stream.push("var".to_string());
                        } else {
                            stream.push("var".to_string());
                        }
                    }
                }
            }
            stream.push(i.id_name.clone());
            // stream.push(" ".to_string());

            if i.is_ptr == true {
                stream.push("*".to_string());

                // if self.strict == false {
                //     stream.push("&".to_string());
                // }
            }
            // get the rust type
            if let Some(rust_type) = parse_type(i.symbol_type, i.symbol_modifier) {
                if rust_type == "_".to_string() {
                    //not able to find the type, let the rust compiler do the type inference.
                    stream.pop();
                } else {
                    stream.push(rust_type);
                }
            } else {
                // if type parser dint return Some type, then it must be user defined type.
                //TODO : should check the typedef table
                stream.push(typdef_type.clone());
            }

            // take care of assignment
            if i.is_assigned {
                stream.push("=".to_string());
                if i.is_ptr == true {
                    stream.push("&".to_string());
                }
                // if self.strict == false && i.is_ptr == true {
                //     stream.push("mut".to_string());
                // }

                stream.push((&i.assigned_val).to_string());
            }
            // 有可能会重复
            if sym_table_right.len() >= 1 {
                stream.push("\n".to_string());
            }
        }
        stream
    }

    /* parse simple typedef definition of form
     * typedef typename newtype;
     */
    fn parse_typdef(&mut self, lexeme: &Vec<Token>) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();
        stream.push("type".to_string());
        stream.push(lexeme[2].get_token_value() + "=");
        self.typde_def_table.push(lexeme[2].get_token_value());
        if let Some(typ) = parse_type(lexeme[1].get_token_type(), Modifier::Default) {
            stream.push(typ);
        } else {
            stream.push(lexeme[1].get_token_value());
        }
        stream.push(";".to_string());
        return stream;
    }

    /**
     * parse_if:
     * parse c/c++ if statements into rust
     * equivalent statements
     */
    fn parse_if(&mut self, lexeme: &Vec<Token>) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();
        let mut head: usize = 0;
        if lexeme.len() <= 0 {
            return stream;
        }
        stream.push("\nif".to_string());
        // stream.push("(".to_string());
        head += 1;

        // head += 1;
        // condition
        // let mut lbcount = 0;
        let lookahead = skip_if_condition(lexeme, head);
        head += 1;
        //skip '('
        while lookahead >= 1 && head < lookahead - 1 {
            stream.push(lexeme[head].get_token_value());
            head += 1
        }

        // while lexeme[head].get_token_type() != RightBracket {
        //     if lexeme[head].get_token_type() == LeftBracket {
        //         lbcount += 1
        //     }
        //     if lexeme[head].get_token_type() == Arrow {
        //         stream.push(".".to_string());
        //     } else {
        //         stream.push(lexeme[head].get_token_value());
        //     }
        //     head += 1;
        // }
        // while lexeme[head + 1].get_token_type() == RightBracket {
        //     head += 1;
        // }
        // head += 1;
        head = lookahead;
        // for _ in 0..lbcount {
        //     stream.push(")".to_string());
        // }
        // stream.push(")".to_string());
        // stream.push("== true".to_string());
        stream.push("{\n".to_string());
        if head < lexeme.len() && lexeme[head].get_token_kind() == TokenKind::Comments {
            stream.push(lexeme[head].get_token_value());
            stream.push("\n".to_string());
            head += 1;
        }
        if head < lexeme.len() && lexeme[head].get_token_type() == LeftCurlyBrace {
            head += 1;
        }

        // collect if body
        let mut temp_lexeme: Vec<Token> = Vec::new();
        while head < lexeme.len() {
            let l: Token = lexeme[head].clone();
            temp_lexeme.push(l);
            head += 1;
        }
        // parse if body
        stream.append(&mut self.parse_program(&temp_lexeme));
        if let Some(t) = temp_lexeme.last() {
            if t.get_token_value() != ";" {
                stream.push("\n".to_string());
            }
        }
        stream.push("}".to_string());

        stream.push("\n".to_string());
        stream
    }

    /**
     * parse_while:
     * parse c/c++ while statements into rust
     * equivalent statements
     */
    fn parse_while(&mut self, lexeme: &Vec<Token>) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();
        let mut head: usize = 0;
        let mut no_cond = false;
        head += 1;

        //skip '('
        head += 1;
        // condition
        let mut cond_stream: Vec<String> = Vec::new();
        while lexeme[head].get_token_type() != RightBracket {
            cond_stream.push(lexeme[head].get_token_value());
            head += 1;
        }
        if cond_stream.len() == 1
            && (cond_stream[0] == "1".to_string() || cond_stream[0] == "true".to_string())
        {
            no_cond = true;
        }
        head += 1;

        if lexeme[head].get_token_type() == LeftCurlyBrace {
            head += 1;
        }

        // collect while body
        let mut temp_lexeme: Vec<Token> = Vec::new();
        while head < lexeme.len() {
            let l: Token = lexeme[head].clone();
            temp_lexeme.push(l);
            head += 1;
        }
        // parse while body
        let mut body_stream = &mut self.parse_program(&temp_lexeme);

        if no_cond == true {
            stream.push("for".to_string());
        } else {
            stream.push("for".to_string());
            // stream.push("".to_string());
            stream.append(&mut cond_stream);
            // stream.push("".to_string());
            // stream.push("== true".to_string());
        }
        stream.push("{".to_string());
        stream.append(&mut body_stream);

        stream.push("}".to_string());
        stream
    }

    /**
     * parse_dowhile:
     * parse c/c++ do while statements into rust
     * equivalent statements
     */
    fn parse_dowhile(&mut self, lexeme: &Vec<Token>) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();
        let mut temp_stream: Vec<String> = Vec::new();
        let mut head: usize = 0;
        let mut lookahead: usize;

        head += 2;
        lookahead = head;

        lookahead = skip_block(&lexeme, lookahead) - 1;
        // collect while body
        let mut temp_lexeme: Vec<Token> = Vec::new();
        while head < lookahead {
            let l: Token = lexeme[head].clone();
            temp_lexeme.push(l);
            head += 1;
        }
        // parse while body

        temp_stream.append(&mut self.parse_program(&temp_lexeme));
        temp_lexeme.clear();

        head += 3;
        if (lexeme[head].get_token_value() == String::from("1")
            || lexeme[head].get_token_value() == String::from("true"))
            && lexeme[head + 1].get_token_type() == RightBracket
        {
            stream.push("for".to_string());
            stream.push("{".to_string());
            stream.append(&mut temp_stream);

            stream.push("}".to_string());
        } else {
            stream.push("for".to_string());
            // stream.push("(".to_string());
            while lexeme[head].get_token_type() != RightBracket {
                stream.push(lexeme[head].get_token_value());
                head += 1;
            }
            // stream.push(")".to_string());
            stream.push("{".to_string());
            stream.append(&mut temp_stream);
            // 
            // stream.push("== true".to_string());
            stream.push("}".to_string());
            // stream.push("{".to_string());
            // stream.push("}".to_string());
        }
        stream.push(";".to_string());
        stream
    }

    fn parse_switch(&mut self, lexeme: &Vec<Token>) -> Vec<String> {
        let mut head: usize = 2;
        let mut lookahead: usize = 2;
        let mut stream: Vec<String> = Vec::new();
        let mut temp_lexeme: Vec<Token> = Vec::new();

        stream.push("switch".to_string());

        // find starting of switch block
        while lookahead < lexeme.len() && lexeme[lookahead].get_token_type() != LeftCurlyBrace {
            lookahead += 1;
        }
        // {
        // move back to find the variable/result to be matched
        lookahead -= 1;
        // single variable
        if lookahead - head == 1 {
            stream.push(lexeme[lookahead - 1].get_token_value());
        }
        // expression
        else {
            while head < lookahead {
                let l: Token = lexeme[head].clone();
                temp_lexeme.push(l);
                head += 1;
            }
            head -= 1;
            stream.append(&mut self.parse_program(&temp_lexeme));

            temp_lexeme.clear();
        }
        // move forward to the starting of the block
        head += 3;
        stream.push("{\n".to_string());

        //head is at case
        lookahead = skip_block(&lexeme, head) - 1;
        while head < lookahead {
            let l: Token = lexeme[head].clone();
            temp_lexeme.push(l);
            head += 1;
        }
        stream.append(&mut self.parse_case(&temp_lexeme));
        stream.push("}".to_string());
        stream
    }

    fn parse_case(&mut self, lexeme: &Vec<Token>) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();
        //head is at case
        let mut head: usize = 0;
        let mut lookahead: usize;
        let mut temp_lexeme: Vec<Token> = Vec::new();
        let mut def: bool = false;
        //look whether default case is handled for exaustive search
        while head < lexeme.len() {
            if lexeme[head].get_token_type() == KeywordDefault {
                stream.push("default".to_string());
                def = true;
            } else {
                stream.push(lexeme[head].get_token_value());
                head += 1; //head is at matching value
                stream.push(lexeme[head].get_token_value());
            }

            head += 1; // head is at :
            stream.push(":".to_string());

            // either brace or no brace
            head += 1;
            if   head < lexeme.len() && lexeme[head].get_token_type() == LeftCurlyBrace {
                head += 1;
                lookahead = skip_block(&lexeme, head) - 1;
            } else {
                lookahead = head;
                while lookahead < lexeme.len()
                    && lexeme[lookahead].get_token_type() != KeywordCase
                    && lexeme[lookahead].get_token_type() != KeywordDefault
                {
                    lookahead += 1;
                }
            }
            while head < lexeme.len() && head < lookahead {
                let l: Token = lexeme[head].clone();
                temp_lexeme.push(l);
                head += 1;
            }
            stream.push("\n".to_string());
            stream.append(&mut self.parse_program(&temp_lexeme));
            stream.push("\n".to_string());

            if head < lexeme.len() && lexeme[head].get_token_type() == RightCurlyBrace {
                head += 1;
            }
            temp_lexeme.clear();
        }
        // if def == false {
        //     stream.push("_".to_string());
        //     stream.push("=>".to_string());
        //     stream.push("{".to_string());
        //     stream.push("}".to_string());
        // }
        stream
    }

    /**
     * parse_for:
     * parse c/c++ do while statements into rust
     * equivalent statements
     *
     * Identify infinite loops and replace for with loop{}
     */
    fn parse_for(&mut self, lexeme: &Vec<Token>) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();
        let mut head: usize = 0;
        let mut lookahead: usize;
        let mut temp_lexeme: Vec<Token> = Vec::new();

        // while lexeme[head].get_token_type() != LeftCurlyBrace {
        //     stream.push(lexeme[head].get_token_value());
        //     head += 1;
        // }

        // head += 1;
        //在for (int i =0; )找到(
        while lexeme[head].get_token_type() != LeftBracket {
            head += 1;
        }
        head += 1;
        lookahead = head;

        //for (int i =0; )第一个节点是数据类型吗
        let decl: bool = if lexeme[head].get_token_kind() == TokenKind::DataTypes {
            true
        } else {
            false
        };
        // let mut no_init:bool; //no initialization
        let mut no_cond: bool = false; //if no condition to terminate，没有结束条件，
        let mut no_updation: bool = false; //no inc/dec of loop counter，没有计数器，

        let mut declare: Vec<String> = Vec::new();
        let mut body: Vec<String> = Vec::new();
        let mut updation: Vec<String> = Vec::new();
        let mut term_cond: Vec<String> = Vec::new();
        // initial assignment
        lookahead = skip_stmt(&lexeme, lookahead);

        //incase of initialization expressio for (;i<10;i++) ; common case
        //for (int i = 0; i < 100; i++) {，在这里找到int i = 0;并生成数据声明 let i:i32 = 0;
        if head + 1 < lookahead {
            while head < lookahead && head < lexeme.len() {
                let l: Token = lexeme[head].clone();
                temp_lexeme.push(l);
                head += 1;
            }

            if decl == true {
                // stream.append(&mut self.parse_declaration(&temp_lexeme));
                declare.append(&mut self.parse_declaration_inline(&temp_lexeme))
                // println!("{:?}", stream);
            } else {
                // stream.append(&mut self.parse_assignment(&temp_lexeme));
                declare.append(&mut self.parse_assignment(&temp_lexeme));
            }
        } else {
            head += 1;
            // no_init = true;
        }
        temp_lexeme.clear();

        // terminating condition
        // /for (int i = 0; i < 100; i++) {，在这里找到结束条件表达式i < 100;生成while i < 100;最
        lookahead = skip_stmt(&lexeme, lookahead);

        if head + 1 < lookahead {
            while head < lookahead - 1 && head < lexeme.len() {
                term_cond.push(lexeme[head].get_token_value());
                head += 1;
            }
        } else {
            no_cond = true;
        }
        head += 1;
        temp_lexeme.clear();

        lookahead = head;
        // update expression
        // 如果有）会有错误

        let end = skip_for_condition(lexeme,0);
        while lookahead < lexeme.len() && lookahead < end-1  {
            let l: Token = lexeme[lookahead].clone();
            temp_lexeme.push(l);
            lookahead += 1;
        }
        // while lookahead < lexeme.len() && lexeme[lookahead].get_token_type() != RightBracket {
        //     let l: Token = lexeme[lookahead].clone();
        //     temp_lexeme.push(l);
        //     lookahead += 1;
        // }
        //no_updation
        if head == lookahead {
            no_updation = true;
        } else {
            temp_lexeme.push(Token::new(
                String::from(";"),
                TokenKind::SpecialChars,
                Semicolon,
                0,
                0,
            ));
            updation.append(&mut self.parse_program(&temp_lexeme));
            if let Some(t) = updation.last() {
                if t == ";" {
                    updation.pop();
                }
            }
            temp_lexeme.clear();
        }
        head = lookahead;
        head += 1;
        if head < lexeme.len() && lexeme[head].get_token_kind() == TokenKind::Comments {
            head += 1;
        }
        if head < lexeme.len() && lexeme[head].get_token_type() == LeftCurlyBrace {
            head += 1;
            lookahead = skip_block(&lexeme, head);
        } else {
            lookahead = skip_stmt(&lexeme, head);
        }

        // lookahead = skip_block(&lexeme, lookahead);
        while head < lookahead && head < lexeme.len() {
            let l: Token = lexeme[head].clone();
            temp_lexeme.push(l);
            head += 1;
        }
        body.append(&mut self.parse_program(&temp_lexeme));

        if no_cond == true {
            stream.push("for".to_string());
        } else {
            stream.push("for".to_string());
            stream.append(&mut declare);

            stream.append(&mut term_cond); //append termianating condition
            stream.push(";".to_string());
        }

        if no_updation != true {
            stream.append(&mut updation);
        }
        stream.push("{\n".to_string());
        stream.append(&mut body);
        stream.push("}\n".to_string());

        stream
    }

    /* parse_assignment:
     * parse c/c++ assignment statements into rust equivalent code
     * compound assignments must be converted to declarations
     * as rust doesnt support compound assignment
     */
    fn parse_assignment(&mut self, lexeme: &Vec<Token>) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();
        // let mut lookahead = lexeme.len();
        let mut thead: usize = 2;
        let mut lexeme1: Vec<Token> = Vec::new();

        let mut n = 2;
        let m = 3;

        let mut tstream: Vec<String> = Vec::new();
        if n < lexeme.len() && lexeme[n].get_token_kind() == TokenKind::UnaryOperators {
            while lexeme[thead].get_token_type() != Semicolon {
                lexeme1.push(lexeme[thead].clone());
                thead += 1;
            }
            lexeme1.push(lexeme[thead].clone());
            stream.push(lexeme[0].get_token_value());
            stream.push(lexeme[1].get_token_value());
            stream.append(&mut self.parse_expr(&lexeme1));
        } else if m < lexeme.len() && lexeme[m].get_token_kind() == TokenKind::UnaryOperators {
            while lexeme[thead].get_token_type() != Semicolon {
                lexeme1.push(lexeme[thead].clone());
                thead += 1;
            }
            lexeme1.push(lexeme[thead].clone());
            stream.push(lexeme[0].get_token_value());
            stream.push(lexeme[1].get_token_value());
            stream.append(&mut self.parse_expr(&lexeme1));
        } else if m < lexeme.len() && lexeme[m].get_token_kind() == TokenKind::BinaryOperators {
            while thead < lexeme.len() && lexeme[thead].get_token_type() != Semicolon {
                lexeme1.push(lexeme[thead].clone());
                thead += 1;
            }
            if thead < lexeme1.len(){lexeme1.push(lexeme[thead].clone())};
            stream.push(lexeme[0].get_token_value());
            stream.push(lexeme[1].get_token_value());
            stream.append(&mut self.parse_expr(&lexeme1));
        } else if n < lexeme.len() && lexeme[n].get_token_type() == BitwiseAnd {
            stream.push(lexeme[0].get_token_value());
            stream.push(lexeme[1].get_token_value());

            while thead < lexeme.len() && lexeme[thead].get_token_type() != Semicolon {
                stream.push(lexeme[thead].get_token_value());
                thead += 1;
            }
        } else {
            if m < lexeme.len() && lexeme[m].get_token_type() == Assignment {
                while thead < lexeme.len() && lexeme[thead].get_token_type() != Semicolon
                    && lexeme[thead].get_token_type() != Comma
                {
                    lexeme1.push(lexeme[thead].clone());
                    thead += 1;
                }
                lexeme1.push(lexeme[thead].clone());
                stream.append(&mut self.parse_program(&lexeme1));
            }
            stream.push(lexeme[0].get_token_value());
            stream.push(lexeme[1].get_token_value());
            if n < lexeme.len() {
                if lexeme[n].get_token_kind() == TokenKind::UnaryOperators {
                    stream.push(lexeme[m].get_token_value());
                } else {
                    //println!("{:#?}", lexeme);
                    stream.push(lexeme[n].get_token_value());
                    n += 1;
                    if n < lexeme.len() && lexeme[n].get_token_type() == LeftBracket
                        || n < lexeme.len() && lexeme[n].get_token_type() == LeftSquareBracket
                    {
                        while n < lexeme.len() && lexeme[n].get_token_type() != Semicolon {
                            stream.push(lexeme[n].get_token_value());
                            n += 1;
                        }
                    } else {
                        //复制过去
                        // fcl.line = new float[m_MaxPoint];
                        while n < lexeme.len() && lexeme[n].get_token_type() != Semicolon {
                            if lexeme[n].get_token_type() != Comma {
                                stream.push(lexeme[n].get_token_value());
                            }

                            n += 1;
                        }
                    }
                    stream.push(";".to_string()); //不能直接加\n，因为可能是for 行内赋值
                }
            }
        }
        if tstream.len() > 0 {
            stream.append(&mut tstream);
        }
        stream
    }

    /* parse_expr:
     * parse c/c++ expression statements into rust equivalent code
     */
    fn parse_expr(&mut self, lexeme: &Vec<Token>) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();
        // let mut lookahead = lexeme.len();
        let mut tstream: Vec<String> = Vec::new();
        let mut thead: usize = 0;

        let mut prev_id = " ".to_string();
        let mut typ = Others;
        //a=b+c++;
        // let mut pre_line = lexeme[thead].get_token_line_num();

        // 一般是int i = 100;
        // 也有可能是这样的int i = 100,j=200;
        while thead < lexeme.len() && lexeme[thead].get_token_type() != Semicolon
        //&& lexeme[thead].get_token_type() != Comma
        {
            // if lexeme[thead].get_token_line_num() != pre_line {
            //     break;
            // }
            // pre_line = lexeme[thead].get_token_line_num();

            if lexeme[thead].get_token_kind() == TokenKind::UnaryOperators {
                if lexeme[thead].get_token_type() == SizeOf {
                    //println!(" {:?} ",lexeme);
                    stream.push("len(".to_string());
                    thead += 2;
                    if lexeme[thead].get_token_kind() == TokenKind::DataTypes {
                        if let Some(t) =
                            parse_type(lexeme[thead].get_token_type(), Modifier::Default)
                        {
                            stream.push(t)
                        }
                    } else {
                        stream.push(lexeme[thead].get_token_value());
                    }
                    stream.push(")".to_string());
                    thead += 1;
                } else {
                    //println!(" 1542 :unop");
                    //incase of post
                    if typ == Identifier {
                        tstream.push(prev_id.clone());
                        tstream.push(match lexeme[thead].get_token_type() {
                            Increment => "+=1".to_string(),
                            Decrement => "-=1".to_string(),
                            _ => " ;".to_string(),
                        });
                        tstream.push(";".to_string());

                        thead += 1;
                    //continue;
                    }
                    // incase of pre
                    else {
                        stream.push("(".to_string());
                        stream.push(lexeme[thead + 1].get_token_value());
                        stream.push(match lexeme[thead].get_token_type() {
                            Increment => "++".to_string(),
                            Decrement => "--".to_string(),
                            _ => " ;".to_string(),
                        });
                        stream.push(")".to_string());
                        thead += 1;
                    }
                }
            } else {
                if lexeme[thead].get_token_type() == TokenType::Arrow {
                    stream.push(".".to_string());
                } else {
                    stream.push(lexeme[thead].get_token_value());
                }
            }

            typ = lexeme[thead].get_token_type();
            prev_id = lexeme[thead].get_token_value();

            thead += 1;
            if thead == lexeme.len() {
                // println!("{:#?}", lexeme[thead - 1]);
                println!(
                    "1663-出错,没有在行{}找到分号或是逗号",
                    lexeme[thead - 1].get_token_line_num()
                );
            }
        }
        stream.push(";".to_string());
        if tstream.len() > 0 {
            stream.append(&mut tstream);
        }
        stream
    }

    fn parse_array_declaration(&mut self, lexeme: &Vec<Token>) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();
        let mut typ: String = " ".to_string();

        //int a[10];
        if let Some(t) = parse_type(lexeme[0].get_token_type(), Modifier::Default) {
            typ = t;
        }
        // if self.strict == true {
        //     // stream.push(STRICT.get_doc().to_string());
        //     stream.push("".to_string());
        // } else {
        //     // stream.push(NO_STRICT.get_doc().to_string());
        //     stream.push("".to_string());
        // }

        let mut head = 0;
        stream.push(lexeme[head + 1].get_token_value());
        stream.push(":=".to_string());
        // xxx[12]
        if lexeme[head + 4].get_token_value() == "]" {
            stream.push("[".to_string() + &lexeme[head + 3].get_token_value()[..] + "]" + &typ[..]);
            head = 5;
        //xxx[]
        } else {
            stream.push("[".to_string() + "]" + &typ[..]);
            head = 4;
        }

        let mut lookahead = head;
        while lexeme[lookahead].get_token_type() != Semicolon {
            lookahead += 1;
        }
        let mut temp_lexeme: Vec<Token> = Vec::new();
        if lexeme[head].get_token_type() == Comma {
            temp_lexeme.push(lexeme[0].clone());
            //move to next
            head += 1;
            while lexeme[head].get_token_type() != Semicolon {
                temp_lexeme.push(lexeme[head].clone());
                head += 1;
            }
            stream.push(";".to_string());
            temp_lexeme.push(lexeme[head].clone());
            stream.append(&mut self.parse_program(&temp_lexeme));
        } else if lexeme[head].get_token_type() == Assignment {
            // let mut has_left = false;

            while lexeme[head].get_token_type() != Semicolon
                && lexeme[head].get_token_type() != RightCurlyBrace
            {
                //  static char THIS_FILE[] = __FILE__;//如果只有一个赋值，强制加上一个[
                if lexeme[head].get_token_type() == Assignment {
                    // stream.push(lexeme[head].get_token_value());
                    stream.push("{".to_string());
                } else if lexeme[head].get_token_kind() == TokenKind::Comments {
                    stream.push(lexeme[head].get_token_value() + "\n");
                } else {
                    stream.push(match lexeme[head].get_token_type() {
                        LeftCurlyBrace => {
                            // 不重复增加
                            "".to_string()
                        }
                        _ => lexeme[head].get_token_value(),
                    });
                }

                head += 1;
            }
            stream.push("}".to_string());
            stream.push(";".to_string());
        } else {
            stream.push(";".to_string());
        }

        stream
    }

    // not tested
    fn parse_struct(&mut self, lexeme: &Vec<Token>) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();
        let mut head: usize = 0;
        stream.push("type".into());
        //go的语法是关键字在中间
        //push the struct id_name
        stream.push(lexeme[head + 1].get_token_value()); //push the struct name
                                                         //一般都是struct
        stream.push(lexeme[head].get_token_value()); //push the keyword struct

        head += 1;
        let name = lexeme[head].get_token_value();
        stream.push("{".to_string());
        head += 2;
        //收集一行定义的所有内容

        let mut temp_lexeme: Vec<Token> = Vec::new();
        while head < lexeme.len() && lexeme[head].get_token_type() != RightCurlyBrace {
            //到下一个标记符之前，
            while head < (lexeme.len() - 1) && lexeme[head].get_token_type() != Semicolon {
                if lexeme[head].get_token_kind() == TokenKind::Comments {
                    stream.push("\n".to_string() + &lexeme[head].get_token_value());
                } else {
                    temp_lexeme.push(lexeme[head].clone());
                }
                head += 1
            }
            //不需要加入；分隔号
            temp_lexeme.push(lexeme[head].clone());
            head += 1;
            //结束位置不对，不是以；结束，有可能是注释
            // if temp_lexeme.iter().last().unwrap().get_token_type() != Semicolon {
            //     // head += 1;
            //     head -= 1;
            //     temp_lexeme.clear();
            //     continue;
            // }
            stream.push("\n".to_string());
            stream.append(&mut self.parse_struct_inbody_decl(&temp_lexeme, &name));
            temp_lexeme.clear();
            // stream.push("\n".into());
            while head < (lexeme.len() - 1) && lexeme[head].get_token_kind() == TokenKind::Comments
            {
                stream.push(lexeme[head].get_token_value());
                head += 1;
            }
        }
        stream.push("\n".to_string());

        //最后一个}
        if head < lexeme.len() {
            stream.push(lexeme[head].get_token_value() + "\n");
        }
        stream
    }

    // not tested 结构体的内部字段列表
    fn parse_struct_inbody_decl(&mut self, lexeme: &Vec<Token>, name: &String) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();
        let mut head = 0;
        if lexeme.len() < 2 {
            return stream;
        }
        // static CMapStringToPtr m_LetterTable
        if lexeme[head].get_token_value() == "static" {
            head += 1
        }
        let mut is_reference = false;
        //第一个是类型
        let mut rust_type = lexeme[head].get_token_value(); // "RUST_TYPE".to_string();//字段类型

        // let mut offset1 = 0;
        head +=1;
        if lexeme[head].get_token_kind() == TokenKind::BinaryOperators {
            //t* a
            
            // stream.push(lexeme[head + 2].get_token_value());
            // stream.push(lexeme[head + 1].get_token_value());
            // stream.push(lexeme[head].get_token_value());
            
            if lexeme[head].get_token_type() == TokenType::LessThan {
                
                while lexeme[head-1].get_token_type() != TokenType::GreaterThan {
                    rust_type+=&lexeme[head].get_token_value();
                    head +=1;
                }
            }else if lexeme[head].get_token_type() == TokenType::Multiplication{
                is_reference = true;
                head +=1;
            }
        }
        let identifier  = lexeme[head].get_token_value();
        //  else {
        //     // t a
        //     stream.push(lexeme[head].get_token_value()); //head+1是标识符，head是类型
        //     head +=1                                    // stream.push(lexeme[head].get_token_value());
        // }
        // stream.push(" ".to_string());
        stream.push(identifier.clone());
        if is_reference {
            stream.push("*".to_string());
        }
        

        let mut struct_memt = CStructMem {
            identifier: "NONE".to_string(),
            name: name.clone(),
            member_type: TokenType::Others,
        };

        // check is the array
        // float max[8];
        let mut array_start = head;
        let mut array_endpos = head;
        let mut is_array = false;
        let mut idx = head;
        while idx < lexeme.len() {
            if lexeme[idx].get_token_type() == LeftSquareBracket
            {
                array_start =idx;
                is_array = true;
            }
            if lexeme[idx].get_token_type() == RightSquareBracket {
           
            array_endpos = idx
            }
            idx+=1;
        }
        

        if let Some(rust_typ) = parse_type(lexeme[head].get_token_type(), Modifier::Default) {
            rust_type = rust_typ.clone();
            if is_array {
                stream.push("[".to_string());
                while array_start < array_endpos-1 {
                    stream.push(lexeme[array_start+1].get_token_value());
                    array_start+=1
                }
                stream.push("]".to_string());
            }
            stream.push(rust_typ);
            struct_memt.member_type = lexeme[head].get_token_type();
            struct_memt.identifier = lexeme[head + 1].get_token_value();
        } else {
            // 没有找到对应的类型，原样输出
            if is_array {
                stream.push("[".to_string());
                while array_start < array_endpos-1 {
                    stream.push(lexeme[array_start+1].get_token_value());
                    array_start+=1
                }
                stream.push("]".to_string());
            }
            stream.push(rust_type.clone());
            struct_memt.member_type = lexeme[head].get_token_type();
            struct_memt.identifier = lexeme[head + 1].get_token_value();
        }
        if is_array {
            head = array_endpos +1;
        }else{
            head +=1;
        }
    
        //go的每一个字段的类型定义后不需要特别的标识符
        // stream.push(" ".to_string());
        self.struct_mem.push(struct_memt.clone());

        // strcut a {
        // int a, b, c;
        // }
        while head < lexeme.len()  && lexeme[head].get_token_type() != Semicolon {
            if lexeme[head].get_token_type() == Comma {
                head += 1;
            }
            if lexeme[head + 1].get_token_type() == LeftSquareBracket
                && lexeme[head + 3].get_token_type() == RightSquareBracket
            {
                // int a[9], b[9];
                struct_memt.identifier = lexeme[head].get_token_value();
                stream.push("\n".to_string());
                stream.push(lexeme[head].get_token_value());
                stream.push("[".to_string());
                stream.push(lexeme[head + 2].get_token_value());
                stream.push("]".to_string());
                stream.push(rust_type.clone());
                // int a[9], b[9];  // b[9]是4个标识符合
                head += 4;
                // stream.push("\n".to_string());
                self.struct_mem.push(struct_memt.clone());
            } else {
                struct_memt.identifier = lexeme[head].get_token_value();
                stream.push("\n".to_string());
                // stream.push(identifier.clone());
                stream.push(lexeme[head].get_token_value());
                // stream.push(" ".to_string());
                stream.push(rust_type.clone());
                // int a, b, c;
                head += 1;
                self.struct_mem.push(struct_memt.clone());
            };
        }
        stream
    }

    // not tested
    fn parse_struct_decl(&mut self, lexeme: &Vec<Token>) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();

        // stream.push(STRUCT_INIT.get_doc().to_string());
        stream.push("var".to_string());
        let mut head = 1;
        //struct FilePointer fp;
        let struct_name = lexeme[head].get_token_value();
        head += 1;
        stream.push(lexeme[head].get_token_value()); //push the identifer => let a
        stream.push("=".to_string());
        stream.push(struct_name.clone());
        stream.push("{".to_string());

        for row in &self.struct_mem {
            if row.name == struct_name {
                stream.push(row.identifier.clone());
                stream.push(" ".to_string());
                stream.push(get_default_value_for(row.member_type));
                stream.push(" ".to_string());
            }
        }
        stream.push("}\n".to_string());

        stream
    }

    //parse tagged union
    fn parse_union(&mut self, lexeme: &Vec<Token>) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();
        let mut head: usize = 0;
        stream.push("enum".to_string()); //push the keyword union
        head += 1;
        //push the struct id_name
        stream.push(lexeme[head].get_token_value()); //push the struct name
        let name = lexeme[head].get_token_value();
        stream.push("{".to_string());
        head += 2;
        let mut temp_lexeme: Vec<Token> = Vec::new();
        while head < lexeme.len() && lexeme[head].get_token_type() != RightCurlyBrace {
            while head < lexeme.len() && lexeme[head].get_token_type() != Semicolon {
                temp_lexeme.push(lexeme[head].clone());
                head += 1
            }
            temp_lexeme.push(lexeme[head].clone());
            head += 1;
            stream.append(&mut self.parse_union_inbody_decl(&temp_lexeme, &name));
            temp_lexeme.clear();
        }
        stream.push(lexeme[head].get_token_value() + "\n");

        stream
    }

    /* parse union type declarations
     * input : union tag_name var [;= ...]
     * output : let [mut] variant_name = Sometype_variant
     */
    fn parse_union_decl(&mut self, lexeme: &Vec<Token>) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();
        let mut head: usize = 0;

        stream.push(UNION_DECL.get_doc().to_string());

        //push the keyword let
        stream.push("let".to_string());
        if !self.strict {
            stream.push("mut".to_string());
        }

        stream.push(lexeme[head + 2].get_token_value());
        head += 3;
        while lexeme[head].get_token_type() != Semicolon {
            stream.push(lexeme[head].get_token_value());
            head += 1;
        }

        stream.push(";".to_string());

        stream
    }
    /* parse union body into Some type body
     * return rust stream
     */
    fn parse_union_inbody_decl(&mut self, lexeme: &Vec<Token>, name: &String) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();
        let mut head = 0;
        //push the identifier
        stream.push(lexeme[head + 1].get_token_value());
        stream.push("(".to_string());
        let mut struct_memt = CStructMem {
            identifier: "NONE".to_string(),
            name: name.clone(),
            member_type: TokenType::Others,
        };
        let mut rust_type = "RUST_TYPE".to_string();
        //push the type
        if let Some(rust_typ) = parse_type(lexeme[head].get_token_type(), Modifier::Default) {
            rust_type = rust_typ.clone();
            stream.push(rust_typ);
            struct_memt.member_type = lexeme[head].get_token_type();
            struct_memt.identifier = lexeme[head + 1].get_token_value();
        }
        head += 2;
        stream.push("),".to_string());
        //update struct member table (may require for analysis

        self.struct_mem.push(struct_memt.clone());
        while lexeme[head].get_token_type() != Semicolon {
            if lexeme[head].get_token_type() == Comma {
                head += 1;
            }
            struct_memt.identifier = lexeme[head].get_token_value();
            stream.push(lexeme[head].get_token_value());
            stream.push("(".to_string());
            stream.push(rust_type.clone());
            head += 1;
            stream.push("),".to_string());
            self.struct_mem.push(struct_memt.clone());
        }
        stream
    }
    fn parse_enum(&mut self, lexeme: &Vec<Token>) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();
        let mut head: usize = 0;

        head += 1;
        let name = lexeme[head].get_token_value();

        stream.push("type".to_string());
        stream.push(name.clone());
        stream.push("int".to_string());
        stream.push(";".to_string());
        stream.push("const".to_string());
        stream.push("(\n".to_string());

        head += 2;
        let mut is_first = true;
        while lexeme[head].get_token_type() != RightCurlyBrace {
            if lexeme[head].get_token_type() != Comma {
                stream.push(name.to_uppercase() + "_" + &lexeme[head].get_token_value());
                if is_first {
                    stream.push(name.clone());
                    stream.push("=".to_string());
                    stream.push("iota".to_string());
                    is_first = false
                }
                stream.push("\n".to_string());
            }

            head += 1
        }
        stream.push(")\n".to_string());
        stream
    }
    // not tested
    fn parse_class(&mut self, lexeme: &Vec<Token>) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();
        let mut head: usize = 0;
        let mut method_stream: Vec<String> = Vec::new();
        stream.push("type".to_string()); //push the keyword struct

        head += 1;
        //push the struct id_name
        let class_name = lexeme[head].get_token_value();
        stream.push(class_name.clone()); //push the class name
        let name = lexeme[head].get_token_value();
        stream.push("struct".to_string()); //push the keyword struct
        stream.push("{\n".to_string());
        head += 2;
        let mut modifier: String = " ".to_string();
        let mut temp_lexeme: Vec<Token> = Vec::new();
        let mut tstream: Vec<String> = Vec::new();

        while head < lexeme.len() - 1
            && lexeme[head].get_token_type() != RightCurlyBrace
            && lexeme[head + 1].get_token_type() != Semicolon
        {
            match lexeme[head].get_type() {
                (TokenKind::Modifiers, _) => {
                    match lexeme[head].get_token_type() {
                        KeywordPublic => {
                            head += 2;
                            modifier = "".to_string();
                        }
                        KeywordProtected | keywordPrivate => {
                            head += 2;
                            modifier = "".to_string();
                        }
                        _ => {}
                    };
                }
                (_, Identifier) => {
                    if lexeme[head].get_token_value() == class_name {
                        tstream.push(CONSTRUCTOR.get_doc().to_string());
                        let mut lookahead = head;
                        while lexeme[lookahead].get_token_type() != LeftCurlyBrace {
                            lookahead += 1;
                        }
                        lookahead += 1;
                        lookahead = skip_block(lexeme, lookahead);
                        while head < lookahead {
                            tstream.push(lexeme[head].get_token_value());
                            head += 1;
                        }
                        tstream.push("\n **/\n".to_string());
                        continue;
                    }
                }

                _ => {}
            }

            if head < lexeme.len() - 2 && lexeme[head + 2].get_token_type() == LeftBracket {
                while lexeme[head].get_token_type() != RightCurlyBrace {
                    temp_lexeme.push(lexeme[head].clone());
                    head += 1;
                }
                temp_lexeme.push(lexeme[head].clone());
                head += 1;
                method_stream.append(&mut self.parse_method_decl(&temp_lexeme, &modifier));
                temp_lexeme.clear();
            } else {
                while head < lexeme.len()
                    && lexeme[head].get_token_type() != RightCurlyBrace
                    && lexeme[head].get_token_kind() != TokenKind::Modifiers
                {
                    while head < lexeme.len() - 1 && lexeme[head].get_token_type() != Semicolon {
                        temp_lexeme.push(lexeme[head].clone());
                        head += 1
                    }
                    temp_lexeme.push(lexeme[head].clone());
                    head += 1;
                    if head < lexeme.len() && lexeme[head].get_token_kind() == TokenKind::Comments {
                        temp_lexeme.push(lexeme[head].clone());
                        head += 1;
                    }
                    stream.append(&mut self.parse_class_inbody_decl(
                        &temp_lexeme,
                        &name,
                        &modifier,
                    ));
                    temp_lexeme.clear();
                }
            }
        }
        if head < lexeme.len() {
            stream.push(lexeme[head].get_token_value());
        }
        stream.push(
            "\n\n/**Method declarations are wrapped inside the impl block \
    \n * Which implements the corresponding structure\
    \n **/\n"
                .to_string(),
        );
        // stream.push("impl".to_string());
        // stream.push(name.clone());
        // stream.push("{\n".to_string());
        if tstream.len() > 0 {
            stream.append(&mut tstream);
        }
        stream.append(&mut method_stream);

        // stream.push("}\n".to_string());
        stream
    }

    // not tested
    fn parse_method_decl(&mut self, lexeme: &Vec<Token>, modifier: &String) -> Vec<String> {
        let mut temp_lexeme: Vec<Token> = Vec::new();
        let mut head: usize = 3;
        let mut lookahead: usize = head;
        let mut stream: Vec<String> = Vec::new();
        if modifier.len() > 1 {
            stream.push(modifier.clone());
        }
        stream.push("func".to_string());
        stream.push(lexeme[1].get_token_value());
        stream.push("(".to_string());
        stream.push("&self".to_string()); //first argument of method must be self, for sefety we consider reference/borrow
                                          // parse arguments differenly for functions that are not main
                                          // collect arguments
        while lexeme[lookahead].get_token_type() != RightBracket {
            lookahead += 1;
        }
        if head < lookahead {
            stream.push(",".to_string());
        }
        while head < lookahead {
            let l: Token = lexeme[head].clone();
            temp_lexeme.push(l);
            head += 1;
        }

        // parse arguments
        stream.append(&mut self.parse_arguments(&temp_lexeme));
        temp_lexeme.clear();

        stream.push(")".to_string());

        // parse return type
        if let Some(rust_type) = parse_type(lexeme[0].get_token_type(), Modifier::Default) {
            if rust_type != "void".to_string() {
                stream.push(".".to_string());
                stream.push(rust_type);
            }
        }

        stream.push("{".to_string());
        while lexeme[head].get_token_type() != LeftCurlyBrace {
            head += 1
        }
        head += 1;

        // collect function body
        // len - 1  so that '}' is excluded
        while head < lexeme.len() - 1 {
            let l: Token = lexeme[head].clone();
            temp_lexeme.push(l);
            head += 1;
        }
        // parse function body
        stream.append(&mut self.parse_program(&temp_lexeme));
        stream.push("}".to_string());
        stream
    }

    // not tested
    fn parse_class_inbody_decl(
        &mut self,
        lexeme: &Vec<Token>,
        name: &String,
        modifier: &String,
    ) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();
        let mut head = 0;
        //push the identifier
        if modifier.len() > 1 {
            stream.push(modifier.clone());
        }
        let mut struct_memt = CStructMem {
            identifier: "NONE".to_string(),
            name: name.clone(),
            member_type: TokenType::Others,
        };
        stream.push(lexeme[head + 1].get_token_value());
        // stream.push(" ".to_string());

        let mut rust_type: String = " ".to_string();
        if let Some(rust_typ) = parse_type(lexeme[0].get_token_type(), Modifier::Default) {
            rust_type = rust_typ.clone();
            stream.push(rust_typ);
            struct_memt.member_type = lexeme[0].get_token_type();
            struct_memt.identifier = lexeme[1].get_token_value();
        }

        self.struct_mem.push(struct_memt.clone());
        head += 2;
        while head < lexeme.len() && lexeme[head].get_token_type() != Semicolon {
            if lexeme[head].get_token_type() == Comma {
                head += 1;
            }
            stream.push(rust_type.clone());
            stream.push(lexeme[head].get_token_value());
            // stream.push(":".to_string());
            struct_memt.identifier = lexeme[head].get_token_value();
            self.struct_mem.push(struct_memt.clone());
            head += 1;
        }
        if head < lexeme.len() - 1 && lexeme[head + 1].get_token_kind() == TokenKind::Comments {
            stream.push(lexeme[head + 1].get_token_value());
        }
        stream.push("\n".to_string());
        stream
    }

    // not tested
    fn parse_class_decl(&mut self, lexeme: &Vec<Token>) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();

        // stream.push(STRUCT_INIT.get_doc().to_string());
        stream.push("var".to_string());
        let mut head = 0;
        //struct FilePointer fp;
        let struct_name = lexeme[head].get_token_value();
        head += 1;
        stream.push(lexeme[head].get_token_value()); //push the identifer => let a
        stream.push("=".to_string());
        stream.push(struct_name.clone());
        stream.push("{".to_string());

        for row in &self.struct_mem {
            if row.name == struct_name {
                stream.push(row.identifier.clone());
                stream.push(" ".to_string());
                stream.push(get_default_value_for(row.member_type));
                stream.push(" ".to_string());
            }
        }
        if self.struct_mem.len() == 0 {
            head += 2;
            while head < lexeme.len() - 2 {
                stream.push(lexeme[head].get_token_value());
                head += 1;
            }
        }
        stream.push("}\n".to_string());

        stream
    }
}
