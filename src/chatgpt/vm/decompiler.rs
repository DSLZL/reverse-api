// Bytecode decompiler for ChatGPT's JavaScript VM
use crate::chatgpt::utils::error::{ChatGptError, Result};
use crate::chatgpt::utils::helpers::Utils;
use base64::{engine::general_purpose, Engine as _};
use serde_json::Value;
use std::collections::HashMap;

/// VM bytecode decompiler
pub struct Decompiler {
    /// Opcode to operation name mapping
    mapping: HashMap<String, String>,
    /// Generated JavaScript code
    decompiled: String,
    /// Variable value tracking
    array_dict: HashMap<String, String>,
    // XOR keys
    // xor_key: String,
    // xor_key2: String,
    // State tracking
    // found: bool,
}

impl Default for Decompiler {
    fn default() -> Self {
        let mut mapping = HashMap::new();

        // Opcode mapping from Python decompiler
        mapping.insert("1".to_string(), "XOR_STR".to_string());
        mapping.insert("2".to_string(), "SET_VALUE".to_string());
        mapping.insert("3".to_string(), "BTOA".to_string());
        mapping.insert("4".to_string(), "BTOA_2".to_string());
        mapping.insert("5".to_string(), "ADD_OR_PUSH".to_string());
        mapping.insert("6".to_string(), "ARRAY_ACCESS".to_string());
        mapping.insert("7".to_string(), "CALL".to_string());
        mapping.insert("8".to_string(), "COPY".to_string());
        mapping.insert("10".to_string(), "window".to_string());
        mapping.insert("11".to_string(), "GET_SCRIPT_SRC".to_string());
        mapping.insert("12".to_string(), "GET_MAP".to_string());
        mapping.insert("13".to_string(), "TRY_CALL".to_string());
        mapping.insert("14".to_string(), "JSON_PARSE".to_string());
        mapping.insert("15".to_string(), "JSON_STRINGIFY".to_string());
        mapping.insert("17".to_string(), "CALL_AND_SET".to_string());
        mapping.insert("18".to_string(), "ATOB".to_string());
        mapping.insert("19".to_string(), "BTOA_3".to_string());
        mapping.insert("20".to_string(), "IF_EQUAL_CALL".to_string());
        mapping.insert("21".to_string(), "IF_DIFF_CALL".to_string());
        mapping.insert("22".to_string(), "TEMP_STACK_CALL".to_string());
        mapping.insert("23".to_string(), "IF_DEFINED_CALL".to_string());
        mapping.insert("24".to_string(), "BIND_METHOD".to_string());
        mapping.insert("27".to_string(), "REMOVE_OR_SUBTRACT".to_string());
        mapping.insert("29".to_string(), "LESS_THAN".to_string());
        mapping.insert("31".to_string(), "INCREMENT".to_string());
        mapping.insert("32".to_string(), "DECREMENT_AND_EXEC".to_string());
        mapping.insert("33".to_string(), "MULTIPLY".to_string());
        mapping.insert("34".to_string(), "MOVE".to_string());

        Self {
            mapping,
            decompiled: "var mem = {};\n".to_string(),
            array_dict: HashMap::new(),
            // xor_key: String::new(),
            // xor_key2: String::new(),
            // found: false,
        }
    }
}

impl Decompiler {
    /// Create new decompiler instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Decompile VM bytecode to JavaScript
    pub fn decompile_vm(turnstile: &str, token: &str) -> Result<String> {
        let mut decompiler = Self::new();

        // Initialize with browser environment simulation
        decompiler.decompiled = r#"const { JSDOM } = require("jsdom");
const dom = new JSDOM("<!DOCTYPE html><p>Hello world</p>", { url: "https://chatgpt.com/" });
const window = dom.window;
var mem = {};
"#
        .to_string();

        // Decode and XOR the turnstile data
        let decoded = general_purpose::STANDARD
            .decode(turnstile)
            .map_err(|e| ChatGptError::vm_execution(format!("Base64 decode failed: {}", e)))?;

        let xor_result = Utils::xor_strings(&String::from_utf8_lossy(&decoded), token);

        // Parse bytecode
        let bytecode: Value = serde_json::from_str(&xor_result)
            .map_err(|e| ChatGptError::vm_execution(format!("JSON parse failed: {}", e)))?;

        decompiler.decompile_bytecode(&bytecode)?;

        Ok(decompiler.decompiled)
    }

    /// Decompile bytecode array
    fn decompile_bytecode(&mut self, bytecode: &Value) -> Result<()> {
        let instructions = bytecode
            .as_array()
            .ok_or_else(|| ChatGptError::vm_execution("Bytecode is not an array"))?;

        for instruction in instructions {
            let inst_array = instruction
                .as_array()
                .ok_or_else(|| ChatGptError::vm_execution("Instruction is not an array"))?;

            if inst_array.is_empty() {
                continue;
            }

            let opcode_str: String = if let Some(s) = inst_array[0].as_str() {
                s.to_string()
            } else if let Some(n) = inst_array[0].as_i64() {
                n.to_string()
            } else {
                return Err(ChatGptError::vm_execution("Invalid opcode"));
            };

            let opcode = opcode_str.as_str();

            let args: Vec<String> = inst_array[1..]
                .iter()
                .map(|v| match v {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Null => "null".to_string(),
                    _ => v.to_string(),
                })
                .collect();

            self.handle_operation(opcode, &args)?;
        }

        Ok(())
    }

    /// Handle a single operation
    fn handle_operation(&mut self, opcode: &str, args: &[String]) -> Result<()> {
        let operation = self
            .mapping
            .get(opcode)
            .cloned()
            .unwrap_or_else(|| format!("UNKNOWN_{}", opcode));

        match operation.as_str() {
            "SET_VALUE" => self.handle_set_value(args),
            "COPY" => self.handle_copy(args),
            "XOR_STR" => self.handle_xor(args),
            "ARRAY_ACCESS" => self.handle_array_access(args),
            "ADD_OR_PUSH" => self.handle_add_or_push(args),
            "CALL" => self.handle_call(args),
            "JSON_STRINGIFY" => self.handle_json_stringify(args),
            "IF_DEFINED_CALL" => self.handle_if_defined_call(args),
            "BTOA_3" => self.handle_btoa3(args),
            _ => self.handle_generic(operation.as_str(), args),
        }
    }

    fn handle_set_value(&mut self, args: &[String]) -> Result<()> {
        if args.len() < 2 {
            return Ok(());
        }

        let var_name = args[0].replace('.', "_");
        let value = &args[1];

        if value == "[]" {
            self.decompiled
                .push_str(&format!("var var_{} = [];\n", var_name));
            self.array_dict.insert(args[0].clone(), "[]".to_string());
        } else if value == "None" || value == "null" {
            self.decompiled
                .push_str(&format!("var var_{} = null;\n", var_name));
            self.array_dict.insert(args[0].clone(), "null".to_string());
        } else if let Ok(num) = value.parse::<f64>() {
            if num.fract() == 0.0 {
                self.decompiled
                    .push_str(&format!("var var_{} = {};\n", var_name, num as i64));
            } else {
                self.decompiled
                    .push_str(&format!("var var_{} = {};\n", var_name, num));
            }
            self.array_dict.insert(args[0].clone(), value.clone());
        } else {
            self.decompiled
                .push_str(&format!("var var_{} = \"{}\";\n", var_name, value));
            self.array_dict
                .insert(args[0].clone(), format!("\"{}\"", value));
        }

        Ok(())
    }

    fn handle_copy(&mut self, args: &[String]) -> Result<()> {
        if args.len() < 2 {
            return Ok(());
        }

        let dest = args[0].replace('.', "_");
        let src = args[1].replace('.', "_");

        self.decompiled
            .push_str(&format!("var var_{} = var_{};\n", dest, src));

        Ok(())
    }

    fn handle_xor(&mut self, args: &[String]) -> Result<()> {
        if args.len() < 2 {
            return Ok(());
        }

        let var_name = args[0].replace('.', "_");
        let key_name = args[1].replace('.', "_");

        // Track potential XOR keys
        self.decompiled.push_str(&format!(
            "var var_{} = XOR_STR(var_{}, var_{});\n",
            var_name, var_name, key_name
        ));

        Ok(())
    }

    fn handle_array_access(&mut self, args: &[String]) -> Result<()> {
        if args.len() < 3 {
            return Ok(());
        }

        let dest = args[0].replace('.', "_");
        let arr = args[1].replace('.', "_");
        let idx = args[2].replace('.', "_");

        self.decompiled
            .push_str(&format!("var var_{} = var_{}[var_{}];\n", dest, arr, idx));

        Ok(())
    }

    fn handle_add_or_push(&mut self, args: &[String]) -> Result<()> {
        if args.len() < 2 {
            return Ok(());
        }

        let var_name = args[0].replace('.', "_");
        let arg_name = args[1].replace('.', "_");

        self.decompiled.push_str(&format!(
            "var var_{} = Array.isArray(var_{}) ? (var_{}.push(var_{}), var_{}) : var_{} + var_{};\n",
            var_name, var_name, var_name, arg_name, var_name, var_name, arg_name
        ));

        Ok(())
    }

    fn handle_call(&mut self, args: &[String]) -> Result<()> {
        if args.is_empty() {
            return Ok(());
        }

        let func = args[0].replace('.', "_");
        let call_args: Vec<String> = args[1..]
            .iter()
            .map(|a| format!("var_{}", a.replace('.', "_")))
            .collect();

        self.decompiled
            .push_str(&format!("var_{}({});\n", func, call_args.join(", ")));

        Ok(())
    }

    fn handle_json_stringify(&mut self, args: &[String]) -> Result<()> {
        if args.is_empty() {
            return Ok(());
        }

        let var_name = args[0].replace('.', "_");
        self.decompiled.push_str(&format!(
            "var var_{} = JSON.stringify(var_{});\n",
            var_name, var_name
        ));

        Ok(())
    }

    fn handle_if_defined_call(&mut self, _args: &[String]) -> Result<()> {
        // Simplified implementation - track XOR key usage
        self.decompiled.push_str("// IF_DEFINED_CALL\n");
        Ok(())
    }

    fn handle_btoa3(&mut self, args: &[String]) -> Result<()> {
        if args.is_empty() {
            return Ok(());
        }

        let var_name = args[0].replace('.', "_");
        self.decompiled.push_str(&format!(
            "var var_{} = btoa(\"\" + var_{});\n",
            var_name, var_name
        ));

        Ok(())
    }

    fn handle_generic(&mut self, operation: &str, args: &[String]) -> Result<()> {
        let args_str = args.join(", ");
        self.decompiled
            .push_str(&format!("// {}: {}\n", operation, args_str));
        Ok(())
    }
}
