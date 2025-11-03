use crate::deepseek::error::{DeepSeekError, Result};
use anyhow::Context;
use rquest::header::{HeaderValue, AUTHORIZATION};
use wasmtime::{Caller, Config, Engine, Linker, Memory, Module, Store, TypedFunc};

// Helper to get current Unix timestamp
// fn get_timestamp() -> u64 {
//     SystemTime::now()
//         .duration_since(UNIX_EPOCH)
//         .unwrap()
//         .as_secs()
// }

const FAKE_HEADERS: &[(&str, &str)] = &[
    ("Accept", "*/*"),
    ("Accept-Encoding", "gzip, deflate, br, zstd"),
    ("Accept-Language", "zh-CN,zh;q=0.9,en;q=0.8"),
    ("Origin", "https://chat.deepseek.com"),
    ("Pragma", "no-cache"),
    ("Priority", "u=1, i"),
    ("Referer", "https://chat.deepseek.com/"),
    ("Sec-Ch-Ua", "\"Chromium\";v=\"134\", \"Not:A-Brand\";v=\"24\", \"Google Chrome\";v=\"134\""),
    ("Sec-Ch-Ua-Mobile", "?0"),
    ("Sec-Ch-Ua-Platform", "\"macOS\""),
    ("Sec-Fetch-Dest", "empty"),
    ("Sec-Fetch-Mode", "cors"),
    ("Sec-Fetch-Site", "same-origin"),
    ("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/134.0.0.0 Safari/537.36"),
    ("X-App-Version", "20241129.1"),
    ("X-Client-Locale", "zh-CN"),
    ("X-Client-Platform", "web"),
    ("X-Client-Version", "1.0.0-always"),
];

pub struct DeepSeekHash {
    store: Store<()>, // Store for the WASM instance
    memory: Memory,
    // Exported functions from WASM
    wasm_solve: TypedFunc<(i32, i32, i32, i32, i32, f64), ()>,
    wbindgen_add_to_stack_pointer: TypedFunc<i32, i32>,
    wbindgen_export_0: TypedFunc<(i32, i32), i32>,
    #[allow(dead_code)]
    wbindgen_export_1: TypedFunc<(i32, i32, i32, i32), i32>,
    offset: usize,
}

impl DeepSeekHash {
    pub async fn new(wasm_path: &str) -> Result<Self> {
        let mut config = Config::new();
        config.wasm_bulk_memory(true);
        let engine = Engine::new(&config)?;
        let module = Module::from_file(&engine, wasm_path)?;

        let mut linker = Linker::new(&engine);

        // Define the `wbg_rand` function that `wasm-bindgen` expects
        linker.func_wrap("wbg", "__wbg_random_c860375d405066c3", || -> f64 {
            rand::random::<f64>()
        })?;

        // Define the `wbg_log` function that `wasm-bindgen` expects
        linker.func_wrap(
            "wbg",
            "__wbg_log_0400000000000000",
            |mut caller: Caller<'_, ()>, ptr: i32, len: i32| {
                let memory = match caller.get_export("memory") {
                    Some(wasmtime::Extern::Memory(mem)) => mem,
                    _ => panic!("failed to find host memory"),
                };
                let data = memory
                    .data(&caller)
                    .get(ptr as usize..ptr as usize + len as usize)
                    .unwrap();
                let s = std::str::from_utf8(data).unwrap();
                println!("WASM Log: {}", s);
            },
        )?;

        let mut store = Store::new(&engine, ());
        let instance = linker.instantiate(&mut store, &module)?;

        let memory = instance
            .get_memory(&mut store, "memory")
            .context("failed to find host memory")?;

        let wasm_solve = instance
            .get_typed_func::<(i32, i32, i32, i32, i32, f64), ()>(&mut store, "wasm_solve")?;
        let wbindgen_add_to_stack_pointer =
            instance.get_typed_func::<i32, i32>(&mut store, "__wbindgen_add_to_stack_pointer")?;
        let wbindgen_export_0 =
            instance.get_typed_func::<(i32, i32), i32>(&mut store, "__wbindgen_export_0")?;
        let wbindgen_export_1 = instance
            .get_typed_func::<(i32, i32, i32, i32), i32>(&mut store, "__wbindgen_export_1")?;

        Ok(Self {
            store,
            memory,
            wasm_solve,
            wbindgen_add_to_stack_pointer,
            wbindgen_export_0,
            wbindgen_export_1,
            offset: 0,
        })
    }

    // Helper to encode string into WASM memory, similar to TypeScript's encodeString
    fn encode_string(&mut self, text: &str) -> Result<i32> {
        let bytes = text.as_bytes();
        let byte_length = bytes.len();

        // Allocate memory for the string
        let ptr = self
            .wbindgen_export_0
            .call(&mut self.store, (byte_length as i32, 1))?;

        // Write the bytes to memory
        self.memory.write(&mut self.store, ptr as usize, bytes)?;

        // Store the length for later use
        self.offset = byte_length;

        Ok(ptr)
    }

    pub fn calculate_hash(
        &mut self,
        algorithm: &str,
        challenge: &str,
        salt: &str,
        difficulty: f64,
        expire_at: i64,
    ) -> Result<Option<f64>> {
        if algorithm != "DeepSeekHashV1" {
            return Err(DeepSeekError::ApiError(format!(
                "Unsupported algorithm: {}",
                algorithm
            )));
        }

        let prefix = format!("{}_{}_", salt, expire_at);

        let stack_ptr = self
            .wbindgen_add_to_stack_pointer
            .call(&mut self.store, -16)?;

        let challenge_ptr = self.encode_string(challenge)?;
        let challenge_len = self.offset as i32;

        let prefix_ptr = self.encode_string(&prefix)?;
        let prefix_len = self.offset as i32;

        self.wasm_solve.call(
            &mut self.store,
            (
                stack_ptr,
                challenge_ptr,
                challenge_len,
                prefix_ptr,
                prefix_len,
                difficulty,
            ),
        )?;

        let mut result_bytes = [0u8; 8];
        self.memory
            .read(&mut self.store, (stack_ptr + 8) as usize, &mut result_bytes)?;
        let value = f64::from_le_bytes(result_bytes);

        let mut status_bytes = [0u8; 4];
        self.memory
            .read(&mut self.store, stack_ptr as usize, &mut status_bytes)?;
        let status = i32::from_le_bytes(status_bytes);

        self.wbindgen_add_to_stack_pointer
            .call(&mut self.store, 16)?;

        if status == 0 {
            Ok(None)
        } else {
            Ok(Some(value))
        }
    }
}

pub struct DeepSeekSignature {
    // This struct might not be needed if DeepSeekHash handles everything
    // Keeping it for now to match the original structure, but it can be removed later.
    pub session: String,
}

impl Default for DeepSeekSignature {
    fn default() -> Self {
        Self::new()
    }
}

impl DeepSeekSignature {
    pub fn new() -> Self {
        DeepSeekSignature {
            session: "".to_string(),
        }
    }

    pub async fn get_token(&self, apikey: &str) -> std::result::Result<String, DeepSeekError> {
        let url = "https://chat.deepseek.com/api/v0/users/current";
        let mut headers = rquest::header::HeaderMap::new();
        headers.insert(AUTHORIZATION, format!("Bearer {}", apikey).parse().unwrap());
        // Add FAKE_HEADERS
        for (key, value) in FAKE_HEADERS {
            headers.insert(*key, HeaderValue::from_static(value));
        }

        let client = rquest::Client::new();
        let resp = client.get(url).headers(headers).send().await?;

        let response_text = resp.text().await?;
        let json: serde_json::Value = serde_json::from_str(&response_text)?;

        // Check for error response
        if let Some(code) = json["code"].as_i64() {
            if code != 0 {
                let msg = json["msg"].as_str().unwrap_or("Unknown error");
                return Err(DeepSeekError::ApiError(format!(
                    "API error (code {}): {}",
                    code, msg
                )));
            }
        }

        // Try to get token from data.biz_data.token
        if let Some(token) = json["data"]["biz_data"]["token"].as_str() {
            Ok(token.to_string())
        } else {
            Err(DeepSeekError::ApiError(format!(
                "Failed to get token from response: {}",
                response_text
            )))
        }
    }
}
