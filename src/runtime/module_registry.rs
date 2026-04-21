use crate::std::math::*;
use crate::std::string::*;
use crate::std::time::*;
use crate::std::io::*;
use crate::std::json::*;
use crate::std::http::*;
use crate::std::env::*;
use std::path::PathBuf;
use tokio::fs;
use std::sync::{Arc, Mutex};
use crate::ast::ast::Program;
use crate::runtime::obj::{Object, HashMap};
use crate::runtime::runtime_errors::RuntimeError;
use ahash::HashMapExt;

#[cfg(feature = "wasm")]
use crate::wasm::{WasmRuntime, WasmStore};

pub struct ModuleRegistry {
    pub(crate) loaded_modules: HashMap<String, Module>,
    stdlib: HashMap<String, Module>,
    pub(crate) base_path: PathBuf,
    #[cfg(feature = "wasm")]
    pub(crate) wasm_runtime: Option<WasmRuntime>,
    #[cfg(feature = "wasm")]
    pub(crate) wasm_store: Option<WasmStore>,
}

#[derive(Clone, Debug)]
pub struct Module {
    pub name: String,
    pub exports: HashMap<String, Object>,
}

impl ModuleRegistry {
    pub fn new(base_path: PathBuf) -> Self {
        #[cfg(feature = "wasm")]
        let (wasm_runtime, wasm_store) = match WasmRuntime::new() {
            Ok(runtime) => {
                let store = runtime.create_store();
                (Some(runtime), Some(store))
            }
            Err(e) => {
                eprintln!("Warning: Failed to initialize WASM runtime: {}", e);
                (None, None)
            }
        };
        
        let mut registry = ModuleRegistry {
            loaded_modules: HashMap::new(),
            stdlib: HashMap::new(),
            base_path,
            #[cfg(feature = "wasm")]
            wasm_runtime,
            #[cfg(feature = "wasm")]
            wasm_store,
        };
        
        registry.load_stdlib();
        
        registry
    }
    
    fn load_stdlib(&mut self) {
        // String modules
        let mut string_exports = HashMap::new();
        
        string_exports.insert("join".to_string(), create_builtin("join", 2, 2, string_join));
        string_exports.insert("reverse".to_string(), create_builtin("reverse", 1, 1, string_reverse));
        string_exports.insert("repeat".to_string(), create_builtin("repeat", 2, 2, string_repeat));
        string_exports.insert("chars".to_string(), create_builtin("chars", 1, 1, string_chars));
        
        self.stdlib.insert("std::string".to_string(), Module {
            name: "std::string".to_string(),
            exports: string_exports,
        });

        // Math modules
        let mut math_exports = HashMap::new();
        
        math_exports.insert("clamp".to_string(), create_builtin("clamp", 3, 3, math_clamp));
        math_exports.insert("random".to_string(), create_builtin("random", 0, 2, math_random));
        math_exports.insert("round".to_string(), create_builtin("round", 1, 1, math_round));
        math_exports.insert("floor".to_string(), create_builtin("floor", 1, 1, math_floor));
        math_exports.insert("ceil".to_string(), create_builtin("ceil", 1, 1, math_ceil));
        math_exports.insert("sqrt".to_string(), create_builtin("sqrt", 1, 1, math_sqrt));
        math_exports.insert("sin".to_string(), create_builtin("sin", 1, 1, math_sin));
        math_exports.insert("cos".to_string(), create_builtin("cos", 1, 1, math_cos));
        math_exports.insert("tan".to_string(), create_builtin("tan", 1, 1, math_tan));
        math_exports.insert("log".to_string(), create_builtin("log", 1, 1, math_log));
        math_exports.insert("log10".to_string(), create_builtin("log10", 1, 1, math_log10));
        math_exports.insert("abs".to_string(), create_builtin("abs", 1, 1, math_abs_int));
        math_exports.insert("min".to_string(), create_builtin("min", 2, 2, math_min_int));
        math_exports.insert("max".to_string(), create_builtin("max", 2, 2, math_max_int));
        math_exports.insert("PI".to_string(), math_pi());
        math_exports.insert("E".to_string(), math_e());

        self.stdlib.insert("std::math".to_string(), Module {
            name: "std::math".to_string(),
            exports: math_exports,
        });

        // Time modules
        let mut time_exports = HashMap::new();

        time_exports.insert("now".to_string(), create_builtin("now", 0, 0, time_now));
        time_exports.insert("sleep".to_string(), create_builtin_async("sleep", 1, 1, time_sleep_wrapper));

        self.stdlib.insert("std::time".to_string(), Module {
            name: "std::time".to_string(),
            exports: time_exports,
        });

        // IO modules
        let mut io_exports = HashMap::new();
        
        io_exports.insert("read_file".to_string(), create_builtin("read_file", 1, 1, io_read_file));
        io_exports.insert("read_file_async".to_string(), create_builtin_async("read_file_async", 1, 1, io_read_file_wrapper));
        io_exports.insert("write_file".to_string(), create_builtin("write_file", 2, 2, io_write_file));
        io_exports.insert("write_file_async".to_string(), create_builtin_async("write_file_async", 2, 2, io_write_file_wrapper));
        io_exports.insert("append_file".to_string(), create_builtin("append_file", 2, 2, io_append_file));
        io_exports.insert("append_file_async".to_string(), create_builtin_async("append_file_async", 2, 2, io_append_file_wrapper));

        io_exports.insert("exists".to_string(), create_builtin("exists", 1, 1, io_exists));
        io_exports.insert("is_file".to_string(), create_builtin("is_file", 1, 1, io_is_file));

        io_exports.insert("is_dir".to_string(), create_builtin("is_dir", 1, 1, io_is_dir));

        io_exports.insert("list_dir".to_string(), create_builtin("list_dir", 1, 1, io_list_dir));
        io_exports.insert("list_dir_async".to_string(), create_builtin_async("list_dir_async", 1, 1, io_list_dir_wrapper));

        io_exports.insert("create_dir".to_string(), create_builtin("create_dir", 1, 1, io_create_dir));
        io_exports.insert("create_dir_async".to_string(), create_builtin_async("create_dir_async", 1, 1, io_create_dir_wrapper));
        io_exports.insert("delete_file".to_string(), create_builtin("delete_file", 1, 1, io_delete_file));
        io_exports.insert("delete_file_async".to_string(), create_builtin_async("delete_file_async", 1, 1, io_delete_file_wrapper));
        io_exports.insert("delete_dir".to_string(), create_builtin("delete_dir", 1, 1, io_delete_dir));
        io_exports.insert("delete_dir_async".to_string(), create_builtin_async("delete_dir_async", 1, 1, io_delete_dir_wrapper));

        self.stdlib.insert("std::io".to_string(), Module {
            name: "std::io".to_string(),
            exports: io_exports,
        });

        // JSON modules
        let mut json_exports = HashMap::new();
        
        json_exports.insert("serialize".to_string(), create_builtin("serialize", 1, 1, json_serialize));
        json_exports.insert("deserialize".to_string(), create_builtin("deserialize", 1, 1, json_deserialize));

        self.stdlib.insert("std::json".to_string(), Module {
            name: "std::json".to_string(),
            exports: json_exports,
        });

        // HTTP modules
        let mut http_exports = HashMap::new();
        
        http_exports.insert("get".to_string(), create_builtin_async("get", 1, 1, http_get));
        http_exports.insert("post".to_string(), create_builtin_async("post", 2, 2, http_post));
        http_exports.insert("put".to_string(), create_builtin_async("put", 2, 2, http_put));
        http_exports.insert("delete".to_string(), create_builtin_async("delete", 1, 1, http_delete));

        self.stdlib.insert("std::http".to_string(), Module {
            name: "std::http".to_string(),
            exports: http_exports,
        });

        // Env modules
        let mut env_exports = HashMap::new();

        env_exports.insert("args".to_string(), create_builtin("args", 0, 0, env_args));

        self.stdlib.insert("std::env".to_string(), Module {
            name: "std::env".to_string(),
            exports: env_exports,
        });
    }
    
    pub async fn load_module(module_registry_arc: Arc<Mutex<Self>>, path: &[String]) -> Result<Module, RuntimeError> {
        let module_path = path.join("::");
        
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
        if path.first().map(|s| s.as_str()) == Some("wasm") {
            return ModuleRegistry::load_wasm_module(module_registry_arc, &path[1..]).await;
        }

        let base_path = { module_registry_arc.lock().unwrap().base_path.clone() };
        let mut file_path = base_path;
        
        for part in path {
            if part == "super" {
                if !file_path.pop() {
                    return Err(RuntimeError::InvalidOperation(
                        "Cannot use 'super::' at root level".to_string()
                    ));
                }
            } else {
                file_path.push(part);
            }
        }
        file_path.set_extension("g");
        
        let source = fs::read_to_string(&file_path).await
            .map_err(|e| RuntimeError::InvalidOperation(
                format!("Failed to load module '{}': {}", path.join("::"), e)
            ))?;
        
        let module = ModuleRegistry::parse_and_extract_module(Arc::clone(&module_registry_arc), &source, path).await?;
        
        let module_path = path.join("::");
        module_registry_arc.lock().unwrap().loaded_modules.insert(module_path.clone(), module.clone());
        
        Ok(module)
    }
    
    async fn parse_and_extract_module(module_registry_arc: Arc<Mutex<Self>>, source: &str, path: &[String]) -> Result<Module, RuntimeError> {
        use crate::{Lexer, Parser};
        use crate::vm::compiler::compute_slots::compute_slots;
        
        let spanned_tokens = Lexer::lex_tokens(source.as_bytes())
            .map_err(|e| RuntimeError::InvalidOperation(
                format!("Failed to lex module: {}", e)
            ))?;
        
        let spanned = crate::lexer::token::SpannedTokens::new(&spanned_tokens);
        let tokens = spanned.to_tokens();
        
        let mut program = Parser::parse_tokens(tokens)
            .map_err(|e| RuntimeError::InvalidOperation(
                format!("Failed to parse module: {:?}", e)
            ))?
            .1;
        
        compute_slots(&mut program);
        
        let base_path = { module_registry_arc.lock().unwrap().base_path.clone() };
        let registry_arc_for_eval = Arc::new(Mutex::new(ModuleRegistry::new(base_path)));
        
        let loaded_modules_for_eval = { module_registry_arc.lock().unwrap().loaded_modules.clone() };
        let wasm_runtime_for_eval = { module_registry_arc.lock().unwrap().wasm_runtime.clone() };
        for (key, val) in loaded_modules_for_eval {
            registry_arc_for_eval.lock().unwrap().loaded_modules.insert(key.clone(), val.clone());
        }
        registry_arc_for_eval.lock().unwrap().wasm_runtime = wasm_runtime_for_eval;
        
        let exports = ModuleRegistry::extract_exports(program, registry_arc_for_eval).await?;
        
        Ok(Module {
            name: path.join("::"),
            exports,
        })
    }
    
    async fn extract_exports(_program: Program, _registry: Arc<Mutex<Self>>) -> Result<HashMap<String, Object>, RuntimeError> {
        Ok(HashMap::new())
    }
}

fn create_builtin(name: &str, min: usize, max: usize, func: fn(Vec<Object>) -> Result<Object, RuntimeError>) -> Object {
    Object::BuiltinStd(name.to_string(), min, max, func)
}

fn create_builtin_async(name: &str, min: usize, max: usize, func: fn(Vec<Object>) -> Result<Object, RuntimeError>) -> Object {
    Object::BuiltinStdAsync(name.to_string(), min, max, func)
}