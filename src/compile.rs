use std::collections::HashMap;

use crate::parse::{Expression, ExpressionType, Statement, StatementType, VariableType};

#[derive(Debug)]
pub struct ClassBytecode {
    pub name: Option<String>,
    pub extends: Option<String>,
    pub bytecode: Vec<Instruction>,
    pub functions: HashMap<String, usize>,
}

#[derive(Debug)]
pub enum Instruction {
    PushSelf,
    PushInt(i64),
    PushFloat(f64),
    PushString(String),
    Store,
}

#[derive(Debug)]
pub enum CompileError {
    InvalidAnnotation {
        name: String,
    },
    InvalidClassName {
        line: usize,
        column: usize,
    },
    InvalidExtends {
        line: usize,
        column: usize,
    },
    NotImplemented {
        line: usize,
        column: usize,
        message: String,
    },
}

pub type AnnotationHandler = Box<dyn Fn()>;

struct VariableInfo {
    index: usize,
    konst: bool,
    ty: VariableType,
}

struct Compiler {
    annotation_handlers: HashMap<String, AnnotationHandler>,
    class_name: Option<String>,
    extends: Option<String>,
    non_class_name_statement_seen: bool,
    init_instructions: Vec<Instruction>,
}

impl Compiler {
    pub fn new(annotation_handlers: HashMap<String, AnnotationHandler>) -> Compiler {
        Compiler {
            annotation_handlers,

            class_name: None,
            extends: None,
            non_class_name_statement_seen: false,
            init_instructions: Vec::new(),
        }
    }

    pub fn emit_class_bytecode(
        mut self,
        statements: Vec<Statement>,
    ) -> Result<ClassBytecode, CompileError> {
        for statement in statements {
            self.handle_statement(statement)?;
        }

        Ok(ClassBytecode {
            name: self.class_name,
            extends: self.extends,
            bytecode: self.init_instructions,
            functions: HashMap::new(),
        })
    }

    fn handle_statement(&mut self, statement: Statement) -> Result<(), CompileError> {
        match statement.ty {
            StatementType::Annotation { name, target, .. } => {
                let handler = self
                    .annotation_handlers
                    .get(&name)
                    .ok_or(CompileError::InvalidAnnotation { name })?;

                handler();
                self.handle_statement(*target)?;
            }
            StatementType::ClassName(name) => {
                if self.non_class_name_statement_seen {
                    return Err(CompileError::InvalidClassName {
                        line: statement.line,
                        column: statement.column,
                    });
                }

                if let Some(_) = self.class_name {
                    return Err(CompileError::InvalidClassName {
                        line: statement.line,
                        column: statement.column,
                    });
                }

                let _ = self.class_name.insert(name);
            }
            StatementType::Extends(name) => {
                if self.non_class_name_statement_seen {
                    return Err(CompileError::InvalidExtends {
                        line: statement.line,
                        column: statement.column,
                    });
                }

                if let Some(_) = self.extends {
                    return Err(CompileError::InvalidExtends {
                        line: statement.line,
                        column: statement.column,
                    });
                }

                let _ = self.extends.insert(name);
            }
            StatementType::Var {
                konst: _,
                identifier,
                ty: _,
                value,
            } => {
                let Some(val) = value else { return Ok(()) };

                self.on_init(Instruction::PushSelf);
                self.on_init(Instruction::PushString(identifier));

                let mut expression_instructions = Vec::new();
                Self::evaluate_expression(&mut expression_instructions, val)?;
                self.init_instructions.extend(expression_instructions);

                self.on_init(Instruction::Store);
            }
            other => {
                return Err(CompileError::NotImplemented {
                    line: statement.line,
                    column: statement.column,
                    message: format!("{:?} not implemented yet", other),
                })
            }
        }

        Ok(())
    }

    fn on_init(&mut self, instruction: Instruction) {
        self.init_instructions.push(instruction);
    }

    fn evaluate_expression(
        instructions: &mut Vec<Instruction>,
        expression: Expression,
    ) -> Result<(), CompileError> {
        match expression.ty {
            ExpressionType::String(str) => {
                instructions.push(Instruction::PushString(str));
            }
            ExpressionType::Integer(i) => {
                instructions.push(Instruction::PushInt(i));
            }
            ExpressionType::Float(f) => {
                instructions.push(Instruction::PushFloat(f));
            }
            other => {
                return Err(CompileError::NotImplemented {
                    line: expression.line,
                    column: expression.column,
                    message: format!("{:?} not implemented yet", other),
                })
            }
        }

        Ok(())
    }
}

pub fn compile(
    statements: Vec<Statement>,
    annotation_handlers: HashMap<String, AnnotationHandler>,
) -> Result<ClassBytecode, CompileError> {
    Compiler::new(annotation_handlers).emit_class_bytecode(statements)
}
