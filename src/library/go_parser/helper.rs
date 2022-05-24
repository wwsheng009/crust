use std::default;

use crate::library::lexeme::definition::TokenType::*;
use crate::library::lexeme::token::Token;

/**
 * skip_stmt:
 * forwards the lookahead by one statement
 * returns the lookahead at the lexeme after the semi-colon
 */

pub fn skip_stmt(lexeme: &Vec<Token>, mut lookahead: usize) -> usize {
    while lookahead < lexeme.len() && lexeme[lookahead].get_token_type() != Semicolon {
        lookahead += 1;
    }
    lookahead + 1
}
//如果只是单行的语句 
// 情况一
// for (i = 0;i < nm;i++)
// 		totalFlt += m_infoInit.initMin1[i].yFloatBottom;
// 情况二
// for (i = 0;i < nm;i++)
// 	{
//  }
pub fn skip_stmt_2(lexeme: &Vec<Token>, mut lookahead: usize) -> usize {
    let mut pre_line = lexeme[lookahead].get_token_line_num();
    while lookahead < lexeme.len() && lexeme[lookahead].get_token_type() != Semicolon {

        // if lexeme[lookahead].get_token_type() == LeftCurlyBrace{

        // }
        if lexeme[lookahead].get_token_line_num() != pre_line {
            break;
        }
        pre_line = lexeme[lookahead].get_token_line_num();
        lookahead += 1;
    }
    lookahead + 1
}



pub fn skip_for_condition(lexeme: &Vec<Token>, ahead: usize) -> usize {
    let mut lookahead = ahead;

    let mut paren = 1;

    //找到起始位置
    while lookahead < lexeme.len() &&  lexeme[lookahead].get_token_type() != LeftBracket{
        //需要跳过
        lookahead +=1;
            break;
    }
    //跳过第一个(
    lookahead +=1;
    
    if lookahead > lexeme.len() {
        return  lexeme.len();
    }
    while paren != 0 && lookahead < lexeme.len() {
        
        if lexeme[lookahead].get_token_type() == LeftBracket {
            paren += 1;
        }
        if lexeme[lookahead].get_token_type() == RightBracket {
            paren -= 1;
        }

        lookahead += 1;
        if lookahead < lexeme.len() && lexeme[lookahead].get_token_type() == LeftCurlyBrace {
            break;
        }

    }
    lookahead
}
pub fn skip_if_condition(lexeme: &Vec<Token>, ahead: usize) -> usize {
    let mut lookahead = ahead;

    let mut paren = 1;

    //找到起始位置
    while lookahead < lexeme.len() &&  lexeme[lookahead].get_token_type() != LeftBracket{
        //需要跳过
        lookahead +=1;
            break;
    }
    //跳过第一个(
    lookahead +=1;
    
    // while all braces are not closed
    // skip nested blocks if any
    let mut has_lc = false;
    for le in lexeme  {
        if le.get_token_type() == LeftCurlyBrace {
            has_lc = true;
            break;
        }
    }
    if lookahead > lexeme.len() {
        return  lexeme.len();
    }
    let mut prev_line = lexeme[lookahead].get_token_line_num();
    while paren != 0 && lookahead < lexeme.len() {
        
        if lexeme[lookahead].get_token_type() == LeftBracket {
            paren += 1;
        }
        if lexeme[lookahead].get_token_type() == RightBracket {
            paren -= 1;
        }
        
        if !has_lc &&  lexeme[lookahead].get_token_line_num() != prev_line {
            break;
        }
        if has_lc && lexeme[lookahead].get_token_type() == LeftCurlyBrace {
            break;
        }
        prev_line = lexeme[lookahead].get_token_line_num();
        lookahead += 1;
    }
    lookahead
}
/**
 * skip_block:
 * forwards the lookahead by one block
 * returns the lookahead at the lexeme after the closing brace
 */
pub fn skip_block(lexeme: &Vec<Token>, mut lookahead: usize) -> usize {
    if lookahead >= lexeme.len() {
        return lookahead;
    }
    let mut paren = 1;
    
    if lexeme[lookahead].get_token_value()== "{"{
        lookahead +=1
    }
    // while all braces are not closed
    // skip nested blocks if any
    while paren != 0 && lookahead < lexeme.len() {
        if lexeme[lookahead].get_token_type() == LeftCurlyBrace {
            paren += 1;
        }
        if lexeme[lookahead].get_token_type() == RightCurlyBrace {
            paren -= 1;
        }
        lookahead += 1;
    }
    lookahead
}

pub fn get_operator_as_fucn_name(token: &Token) -> &str {
    match token.get_token_type() {
        Plus => "_plus",
        Minus => "_minus",
        Multiplication => "_mult",
        Divide => "_div",
        PlusEqual => "_plus_eq",
        MinusEqual => "_minus_eq",
        MultiplyEqual => "_mult_eq",
        DivideEqual => "_div_eq",
        Increment => "_inc",
        Decrement => "_dec",
        Equal => "_eq",
        Assignment => "_assign",
        RightBracket => "_call",
        _ => "_misc_op",
    }
}
