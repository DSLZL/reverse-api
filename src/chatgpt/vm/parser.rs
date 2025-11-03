// JavaScript parsing utilities for extracting keys and analyzing code
use crate::chatgpt::utils::error::{ChatGptError, Result};
use regex::Regex;
use std::collections::HashMap;

/// JavaScript parser for analyzing decompiled code
pub struct Parser;

impl Parser {
    /// Find variable definition in JavaScript code
    ///
    /// This is a simplified version that uses regex patterns instead of full AST parsing
    pub fn find_var_definition(var_name: &str, start_line: usize, code: &str) -> Option<String> {
        let lines: Vec<&str> = code.lines().collect();

        // Search backwards from start_line
        for i in (0..start_line.min(lines.len())).rev() {
            let line = lines[i];

            // Look for variable declaration: var var_X = ...;
            let var_pattern = format!(r"var\s+var_{}\s*=\s*(.+?);", regex::escape(var_name));
            if let Ok(re) = Regex::new(&var_pattern) {
                if let Some(caps) = re.captures(line) {
                    let value = caps.get(1)?.as_str().trim();

                    // Skip complex operations
                    if value.contains("btoa")
                        || value.contains("XOR_STR")
                        || value.contains("doubleXOR")
                        || value.contains("singlebtoa")
                    {
                        continue;
                    }

                    return Some(value.to_string());
                }
            }
        }

        None
    }

    /// Parse assignments in decompiled JavaScript code
    pub fn parse_assignments(code: &str) -> HashMap<String, String> {
        let mut assignments = HashMap::new();

        // Find all variable assignments like: var_X = var_Y;
        let assign_re = Regex::new(r"var\s+var_(\w+)\s*=\s*(.+?);").unwrap();

        for caps in assign_re.captures_iter(code) {
            if let (Some(key), Some(value)) = (caps.get(1), caps.get(2)) {
                let key_str = key.as_str();
                let value_str = value.as_str().trim();

                // Skip function calls and complex expressions
                if !value_str.contains('(') || value_str.starts_with('"') {
                    assignments.insert(key_str.to_string(), value_str.to_string());
                }
            }
        }

        assignments
    }

    /// Extract XOR key from JavaScript code
    pub fn get_xor_key(js_code: &str) -> Option<String> {
        // Look for XOR_STR function calls: XOR_STR(var_X, "key") or XOR_STR(var_X, var_Y)
        let xor_pattern = Regex::new(r#"XOR_STR\s*\([^,]+,\s*([^)]+)\)"#).unwrap();

        for caps in xor_pattern.captures_iter(js_code) {
            if let Some(second_arg) = caps.get(1) {
                let arg = second_arg.as_str().trim();

                // If it's a string literal, return it
                if arg.starts_with('"') && arg.ends_with('"') {
                    return Some(arg.trim_matches('"').to_string());
                }

                // If it's a variable, try to find its value
                if arg.starts_with("var_") {
                    let var_name = arg.strip_prefix("var_").unwrap();
                    let var_pattern =
                        format!(r#"var\s+var_{}\s*=\s*"([^"]+)";"#, regex::escape(var_name));
                    if let Ok(re) = Regex::new(&var_pattern) {
                        if let Some(value_caps) = re.captures(js_code) {
                            return value_caps.get(1).map(|m| m.as_str().to_string());
                        }
                    }
                }
            }
        }

        None
    }

    /// Parse keys from decompiled code
    pub fn parse_keys(decompiled_code: &str) -> Result<(String, HashMap<String, String>)> {
        let assignments = Self::parse_assignments(decompiled_code);
        let xor_key = Self::get_xor_key(decompiled_code)
            .ok_or_else(|| ChatGptError::vm_execution("XOR key not found"))?;

        let mut parsed_keys = HashMap::new();
        let mut random_index = 1;

        for (key, value) in assignments {
            let parsed_value = if value.starts_with("Array") && !value.contains("location") {
                // Parse array operations: Array(5) : 1.5 + 2.5
                if let Some(sum_part) = value.split(") : ").nth(1) {
                    let numbers: Vec<f64> = sum_part
                        .split(" + ")
                        .filter_map(|s| s.trim().parse().ok())
                        .collect();

                    if numbers.len() >= 2 {
                        (numbers[0] + numbers[1]).to_string()
                    } else {
                        value.clone()
                    }
                } else {
                    value.clone()
                }
            } else if value.contains("location") {
                "location".to_string()
            } else if value.contains("cfIpLongitude") {
                "ipinfo".to_string()
            } else if value.contains("maxTouchPoints") {
                "vendor".to_string()
            } else if value.contains("history") {
                "history".to_string()
            } else if value.contains(r#"window["Object"]["keys"]"#) {
                "localstorage".to_string()
            } else if value.contains("createElement") {
                "element".to_string()
            } else if value.chars().all(|c| c.is_ascii_digit() || c == '.') {
                value.clone()
            } else if value.contains("random") {
                let result = format!("random_{}", random_index);
                random_index += 1;
                result
            } else if value.contains("doublexor") || value.contains("singlebtoa") {
                value.clone()
            } else {
                continue;
            };

            parsed_keys.insert(key, parsed_value);
        }

        Ok((xor_key, parsed_keys))
    }

    /// Analyze JavaScript structure for variable dependencies
    pub fn analyze_dependencies(code: &str) -> HashMap<String, Vec<String>> {
        let mut dependencies = HashMap::new();

        // Find variable declarations and their dependencies
        let var_re = Regex::new(r"var\s+var_(\w+)\s*=\s*(.+?);").unwrap();
        let ref_re = Regex::new(r"var_(\w+)").unwrap();

        for caps in var_re.captures_iter(code) {
            if let (Some(var_name), Some(expr)) = (caps.get(1), caps.get(2)) {
                let var = var_name.as_str().to_string();
                let expression = expr.as_str();

                // Find all variable references in the expression
                let mut deps = Vec::new();
                for ref_caps in ref_re.captures_iter(expression) {
                    if let Some(ref_var) = ref_caps.get(1) {
                        let ref_name = ref_var.as_str().to_string();
                        if ref_name != var {
                            deps.push(ref_name);
                        }
                    }
                }

                dependencies.insert(var, deps);
            }
        }

        dependencies
    }

    /// Extract string literals from code
    pub fn extract_strings(code: &str) -> Vec<String> {
        let mut strings = Vec::new();
        let string_re = Regex::new(r#""([^"\\]*(\\.[^"\\]*)*)""#).unwrap();

        for caps in string_re.captures_iter(code) {
            if let Some(s) = caps.get(1) {
                strings.push(s.as_str().to_string());
            }
        }

        strings
    }

    /// Check if code contains specific pattern
    pub fn contains_pattern(code: &str, pattern: &str) -> bool {
        if let Ok(re) = Regex::new(pattern) {
            re.is_match(code)
        } else {
            code.contains(pattern)
        }
    }

    /// Count occurrences of a pattern in code
    pub fn count_pattern(code: &str, pattern: &str) -> usize {
        if let Ok(re) = Regex::new(pattern) {
            re.find_iter(code).count()
        } else {
            code.matches(pattern).count()
        }
    }
}
