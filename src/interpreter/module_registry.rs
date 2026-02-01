use crate::ast::ast::Stmt;
use crate::std::math::*;
use crate::std::string::*;
use crate::std::time::*;
use crate::std::io::*;
use crate::std::json::*;
use std::collections::HashMap;
use std::path::{PathBuf};
use std::fs;
use crate::ast::ast::{Program, Ident};
use crate::interpreter::obj::Object;
use crate::errors::RuntimeError;

pub struct ModuleRegistry {
    loaded_modules: HashMap<String, Module>,
    stdlib: HashMap<String, Module>,
    base_path: PathBuf,
}

#[derive(Clone, Debug)]
pub struct Module {
    pub name: String,
    pub exports: HashMap<String, Object>,
}

impl ModuleRegistry {
    pub fn new(base_path: PathBuf) -> Self {
        let mut registry = ModuleRegistry {
            loaded_modules: HashMap::new(),
            stdlib: HashMap::new(),
            base_path,
        };
        
        registry.load_stdlib();
        
        registry
    }
    
    fn load_stdlib(&mut self) {
        // String modules
        let mut string_exports = HashMap::new();
        
        string_exports.insert("join".to_string(), create_builtin("join", 2, 2, string_join));
        
        self.stdlib.insert("std.string".to_string(), Module {
            name: "std.string".to_string(),
            exports: string_exports,
        });

        // Math modules
        let mut math_exports = HashMap::new();
        
        math_exports.insert("clamp".to_string(), create_builtin("clamp", 3, 3, math_clamp));
        math_exports.insert("random".to_string(), create_builtin("random", 0, 2, math_random));
        math_exports.insert("round".to_string(), create_builtin("round", 1, 1, math_round));

        self.stdlib.insert("std.math".to_string(), Module {
            name: "std.math".to_string(),
            exports: math_exports,
        });

        // Time modules
        let mut time_exports = HashMap::new();

        time_exports.insert("now".to_string(), create_builtin("now", 0, 0, time_now));
        time_exports.insert("sleep".to_string(), create_builtin("sleep", 1, 1, time_sleep));

        self.stdlib.insert("std.time".to_string(), Module {
            name: "std.time".to_string(),
            exports: time_exports,
        });

        // IO modules
        let mut io_exports = HashMap::new();
        
        io_exports.insert("read_file".to_string(), create_builtin("read_file", 1, 1, io_read_file));
        io_exports.insert("write_file".to_string(), create_builtin("write_file", 2, 2, io_write_file));
        io_exports.insert("append_file".to_string(), create_builtin("append_file", 2, 2, io_append_file));

        io_exports.insert("exists".to_string(), create_builtin("exists", 1, 1, io_exists));
        io_exports.insert("is_file".to_string(), create_builtin("is_file", 1, 1, io_is_file));

        io_exports.insert("is_dir".to_string(), create_builtin("is_dir", 1, 1, io_is_dir));

        io_exports.insert("list_dir".to_string(), create_builtin("list_dir", 1, 1, io_list_dir));

        self.stdlib.insert("std.io".to_string(), Module {
            name: "std.io".to_string(),
            exports: io_exports,
        });

        // JSON modules
        let mut json_exports = HashMap::new();
        
        json_exports.insert("serialize".to_string(), create_builtin("serialize", 1, 1, json_serialize));
        json_exports.insert("deserialize".to_string(), create_builtin("deserialize", 1, 1, json_deserialize));

        self.stdlib.insert("std.json".to_string(), Module {
            name: "std.json".to_string(),
            exports: json_exports,
        });
    }
    
    pub fn load_module(&mut self, path: &[String]) -> Result<Module, RuntimeError> {
        let module_path = path.join(".");
        
        // Check if already loaded
        if let Some(module) = self.loaded_modules.get(&module_path) {
            return Ok(module.clone());
        }
        
        // Check stdlib
        if let Some(module) = self.stdlib.get(&module_path) {
            return Ok(module.clone());
        }
        
        // Load user module
        self.load_user_module(path)
    }
    
    fn load_user_module(&mut self, path: &[String]) -> Result<Module, RuntimeError> {
        let mut file_path = self.base_path.clone();
        
        // Build file path from module path
        for part in path {
            file_path.push(part);
        }
        file_path.set_extension("giu");
        
        let source = fs::read_to_string(&file_path)
            .map_err(|e| RuntimeError::InvalidOperation(
                format!("Failed to load module '{}': {}", path.join("."), e)
            ))?;
        
        let module = self.parse_and_extract_module(&source, path)?;
        
        let module_path = path.join(".");
        self.loaded_modules.insert(module_path.clone(), module.clone());
        
        Ok(module)
    }
    
    fn parse_and_extract_module(&mut self, source: &str, path: &[String]) -> Result<Module, RuntimeError> {
        use crate::{Lexer, Parser, Tokens};
        use std::rc::Rc;
        use std::cell::RefCell;
        
        let token_vec = Lexer::lex_tokens(source.as_bytes())
            .map_err(|e| RuntimeError::InvalidOperation(
                format!("Failed to lex module: {:?}", e)
            ))?
            .1;
        
        let tokens = Tokens::new(&token_vec);
        
        let program = Parser::parse_tokens(tokens)
            .map_err(|e| RuntimeError::InvalidOperation(
                format!("Failed to parse module: {:?}", e)
            ))?
            .1;
        
        // Create evaluator with shared registry reference
        let registry_rc = Rc::new(RefCell::new(ModuleRegistry::new(self.base_path.clone())));
        
        // Copy loaded modules to avoid re-parsing
        for (key, val) in &self.loaded_modules {
            registry_rc.borrow_mut().loaded_modules.insert(key.clone(), val.clone());
        }
        
        let exports = self.extract_exports(program, registry_rc)?;
        
        Ok(Module {
            name: path.join("."),
            exports,
        })
    }
    
    fn extract_exports(&mut self, program: Program, registry: std::rc::Rc<std::cell::RefCell<ModuleRegistry>>) -> Result<HashMap<String, Object>, RuntimeError> {
        use crate::interpreter::eval::Evaluator;
        
        let mut evaluator = Evaluator::new(registry);
        let mut exports = HashMap::new();
        
        for stmt in program {
            match stmt.clone() {
                Stmt::StructStmt { name, .. } => {
                    let _obj = evaluator.eval_statement(stmt);
                    let Ident(struct_name) = name;
                    
                    if let Some(struct_obj) = evaluator.env.borrow().get(&struct_name) {
                        exports.insert(struct_name.clone(), struct_obj);
                    }
                }
                Stmt::LetStmt(ident, _) => {
                    evaluator.eval_statement(stmt);
                    let Ident(var_name) = ident;
                    
                    if let Some(obj) = evaluator.env.borrow().get(&var_name) {
                        // Export functions and other values
                        exports.insert(var_name, obj);
                    }
                }
                _ => {
                    evaluator.eval_statement(stmt);
                }
            }
        }
        
        Ok(exports)
    }
}

fn create_builtin(name: &str, min: usize, max: usize, func: fn(Vec<Object>) -> Result<Object, RuntimeError>) -> Object {
    Object::BuiltinStd(name.to_string(), min, max, func)
}