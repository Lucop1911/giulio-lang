use crate::ast::ast::Stmt;
use crate::std::math::*;
use crate::std::string::*;
use crate::std::time::*;
use crate::std::io::*;
use crate::std::json::*;
use std::collections::HashMap;
use std::path::{PathBuf};
use tokio::fs;
use std::sync::{Arc, Mutex};
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
    
    pub async fn load_module(module_registry_arc: Arc<Mutex<Self>>, path: &[String]) -> Result<Module, RuntimeError> {
        let module_path = path.join(".");
        
        let loaded_module = {
            let registry = module_registry_arc.lock().unwrap();
            registry.loaded_modules.get(&module_path).cloned()
        };

        if let Some(module) = loaded_module {
            return Ok(module);
        }
        
        let stdlib_module = {
            let registry = module_registry_arc.lock().unwrap();
            registry.stdlib.get(&module_path).cloned()
        };

        if let Some(module) = stdlib_module {
            return Ok(module);
        }
        
        ModuleRegistry::load_user_module(module_registry_arc, path).await
    }
    
    async fn load_user_module(module_registry_arc: Arc<Mutex<Self>>, path: &[String]) -> Result<Module, RuntimeError> {
        let base_path = { module_registry_arc.lock().unwrap().base_path.clone() };
        let mut file_path = base_path;
        
        for part in path {
            file_path.push(part);
        }
        file_path.set_extension("giu");
        
        let source = fs::read_to_string(&file_path).await
            .map_err(|e| RuntimeError::InvalidOperation(
                format!("Failed to load module '{}': {}", path.join("."), e)
            ))?;
        
        let module = ModuleRegistry::parse_and_extract_module(Arc::clone(&module_registry_arc), &source, path).await?;
        
        let module_path = path.join(".");
        module_registry_arc.lock().unwrap().loaded_modules.insert(module_path.clone(), module.clone());
        
        Ok(module)
    }
    
    async fn parse_and_extract_module(module_registry_arc: Arc<Mutex<Self>>, source: &str, path: &[String]) -> Result<Module, RuntimeError> {
        use crate::{Lexer, Parser, Tokens};
        
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
        
        let base_path = { module_registry_arc.lock().unwrap().base_path.clone() };
        let registry_arc_for_eval = Arc::new(Mutex::new(ModuleRegistry::new(base_path)));
        
        let loaded_modules_for_eval = { module_registry_arc.lock().unwrap().loaded_modules.clone() };
        for (key, val) in loaded_modules_for_eval {
            registry_arc_for_eval.lock().unwrap().loaded_modules.insert(key.clone(), val.clone());
        }
        
        let exports = ModuleRegistry::extract_exports(program, registry_arc_for_eval).await?;
        
        Ok(Module {
            name: path.join("."),
            exports,
        })
    }
    
    async fn extract_exports(program: Program, registry: Arc<Mutex<Self>>) -> Result<HashMap<String, Object>, RuntimeError> {
        use crate::interpreter::eval::Evaluator;
        
        let mut evaluator = Evaluator::new(registry);
        let mut exports = HashMap::new();
        
        for stmt in program {
            match stmt.clone() {
                Stmt::StructStmt { name, .. } => {
                    let _obj = evaluator.eval_statement(stmt).await;
                    let Ident(struct_name) = name;
                    
                    if let Some(struct_obj) = evaluator.env.lock().unwrap().get(&struct_name) {
                        exports.insert(struct_name.clone(), struct_obj);
                    }
                }
                Stmt::LetStmt(ident, _) => {
                    evaluator.eval_statement(stmt).await;
                    let Ident(var_name) = ident;
                    
                    if let Some(obj) = evaluator.env.lock().unwrap().get(&var_name) {
                        exports.insert(var_name, obj);
                    }
                }
                Stmt::FnStmt { name, params: _, body: _ } => {
                    evaluator.eval_statement(stmt).await;
                    let Ident(fn_name) = name;

                    if let Some(obj) = evaluator.env.lock().unwrap().get(&fn_name) {
                        exports.insert(fn_name, obj);
                    }
                }
                _ => {
                    evaluator.eval_statement(stmt).await;
                }
            }
        }
        
        Ok(exports)
    }
}

fn create_builtin(name: &str, min: usize, max: usize, func: fn(Vec<Object>) -> Result<Object, RuntimeError>) -> Object {
    Object::BuiltinStd(name.to_string(), min, max, func)
}