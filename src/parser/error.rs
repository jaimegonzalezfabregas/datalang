use crate::{lexer::Lexogram, utils::print_hilighted};

#[derive(Debug)]

pub struct FailureExplanation {
    pub lex_pos: usize,
    pub if_it_was: String,
    pub failed_because: String,
    pub parent_failure: Vec<FailureExplanation>,
}

impl FailureExplanation {
    pub fn print(
        self,
        lex_list: &Vec<Lexogram>,
        original_string: &String,
        indentation: String,
    ) -> String {
        println!("{self:?}");

        let mut ret = format!(
            "{indentation}Error trying to read a \x1b[1m{}\x1b[0m failed because:\n",
            self.if_it_was,
        );
        let error_lex = &lex_list[self.lex_pos];

        if !self.parent_failure.is_empty() {
            for parent in self.parent_failure {
                ret += &parent.print(
                    lex_list,
                    original_string,
                    indentation.clone() + "\x1b[90m| \x1b[0m".into(),
                );
            }
        } else {
            ret += &format!(
                "{indentation}\x1b[1m{}\x1b[0m starting at:\n{indentation}{}\n{indentation}\n",
                self.failed_because,
                print_hilighted(
                    original_string,
                    error_lex.pos_s,
                    error_lex.pos_f,
                    indentation.clone(),
                )
            );
        };

        ret
    }
}

#[derive(Debug)]
pub enum ParserError {
    Custom(String),
    SyntaxError(FailureExplanation),
}

impl From<String> for ParserError {
    fn from(e: String) -> Self {
        Self::Custom(e)
    }
}
impl ParserError {
    pub fn print(self, lexic: &Vec<Lexogram>, commands: &String) -> String {
        match self {
            ParserError::Custom(str) => format!("custom error on parsing: {str}"),
            ParserError::SyntaxError(e) => e.print(lexic, commands, "".into()),
        }
    }
}
