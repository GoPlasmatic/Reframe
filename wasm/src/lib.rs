//! WebAssembly bindings for Reframe SWIFT MT ↔ ISO 20022 transformation engine.
//!
//! This crate provides WASM bindings that allow using Reframe from JavaScript/TypeScript,
//! particularly for integration with dataflow-ui for workflow visualization and debugging.
//!
//! # Features
//!
//! - **Three Engine Architecture**: Transform, Generate, and Validate engines matching Reframe's architecture
//! - **Full Plugin Support**: All SWIFT MT and MX message handling functions
//! - **Execution Tracing**: Step-by-step debugging with `*_with_trace` methods for dataflow-ui integration
//! - **Promise-based API**: Async operations return JavaScript Promises
//!
//! # Usage
//!
//! ```javascript
//! import init, { ReframeEngine } from '@goplasmatic/reframe-wasm';
//!
//! await init();
//!
//! // Load workflows (array of workflow definitions)
//! const engine = new ReframeEngine(JSON.stringify(workflows));
//!
//! // Transform a SWIFT MT message to ISO 20022
//! // Payload is a raw string - parsing is done by the parse_mt/parse_mx plugins
//! const payload = "{1:F01BANKBEBBAXXX0000000000}{2:I103BANKDEFFXXXXN}{4:\n:20:REF123\n-}";
//! const result = await engine.transform(payload);
//! console.log(JSON.parse(result));
//!
//! // Get execution trace for debugging
//! const trace = await engine.transform_with_trace(payload);
//! console.log(JSON.parse(trace)); // Use with dataflow-ui DebuggerProvider
//! ```

mod detection;

use dataflow_rs::{AsyncFunctionHandler, Engine, Message, Workflow};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

// Import plugin registration functions
use mx_message::plugin::register_mx_functions;
use swift_mt_message::plugin::register_swift_mt_functions;

/// Initialize the WASM module.
///
/// This is automatically called when the module loads.
/// Sets up the panic hook for better error messages in the browser console.
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// Register all Reframe plugin functions for use in engines.
///
/// This includes:
/// - Generic detection function (`detect`)
/// - SWIFT MT functions (`parse_mt`, `publish_mt`, `validate_mt`, `generate_mt`)
/// - ISO 20022 MX functions (`parse_mx`, `publish_mx`, `validate_mx`, `generate_mx`)
fn register_all_functions() -> HashMap<String, Box<dyn AsyncFunctionHandler + Send + Sync>> {
    let mut custom_functions: HashMap<String, Box<dyn AsyncFunctionHandler + Send + Sync>> =
        HashMap::new();

    // Register generic detection function
    custom_functions.insert("detect".to_string(), Box::new(detection::Detect));

    // Register all MT plugin functions
    let mt_functions = register_swift_mt_functions();
    for (name, handler) in mt_functions {
        custom_functions.insert(name.to_string(), handler);
    }

    // Register all MX plugin functions
    let mx_functions = register_mx_functions();
    for (name, handler) in mx_functions {
        custom_functions.insert(name.to_string(), handler);
    }

    custom_functions
}

/// A WebAssembly-compatible Reframe engine.
///
/// Wraps three dataflow-rs engines (transform, generate, validate) to provide
/// the full Reframe functionality for SWIFT MT ↔ ISO 20022 transformations.
///
/// # Example
/// ```javascript
/// // Create engine with workflow definitions
/// const engine = new ReframeEngine(JSON.stringify({
///     transform: [...transformWorkflows],
///     generate: [...generateWorkflows],
///     validate: [...validateWorkflows]
/// }));
///
/// // Or with a flat array (all workflows go to transform engine)
/// const engine = new ReframeEngine(JSON.stringify(workflows));
///
/// // Process a raw SWIFT MT message (parsing done by parse_mt plugin)
/// const result = await engine.transform("{1:F01BANKBEBBAXXX...}");
/// ```
#[wasm_bindgen]
pub struct ReframeEngine {
    transform_engine: Arc<Engine>,
    generate_engine: Arc<Engine>,
    validate_engine: Arc<Engine>,
    workflows_json: String,
}

#[wasm_bindgen]
impl ReframeEngine {
    /// Create a new ReframeEngine from workflow definitions.
    ///
    /// # Arguments
    /// * `workflows_json` - JSON string containing workflow definitions. Can be:
    ///   - An object with `transform`, `generate`, `validate` keys (each an array of workflows)
    ///   - A flat array of workflows (all go to transform engine, others get empty)
    ///
    /// # Example
    /// ```javascript
    /// // Option 1: Categorized workflows
    /// const engine = new ReframeEngine(JSON.stringify({
    ///     transform: [{id: "mt-to-mx", ...}],
    ///     generate: [{id: "gen-mt103", ...}],
    ///     validate: [{id: "validate-mt", ...}]
    /// }));
    ///
    /// // Option 2: Flat array (all as transform workflows)
    /// const engine = new ReframeEngine(JSON.stringify([{id: "workflow1", ...}]));
    /// ```
    #[wasm_bindgen(constructor)]
    pub fn new(workflows_json: &str) -> Result<ReframeEngine, String> {
        let workflows_value: Value = serde_json::from_str(workflows_json)
            .map_err(|e| format!("Invalid workflows JSON: {}", e))?;

        // Parse workflows based on structure
        let (transform_workflows, generate_workflows, validate_workflows) =
            if let Some(obj) = workflows_value.as_object() {
                // Object format with categorized workflows
                let transform = parse_workflow_array(obj.get("transform"))?;
                let generate = parse_workflow_array(obj.get("generate"))?;
                let validate = parse_workflow_array(obj.get("validate"))?;
                (transform, generate, validate)
            } else if let Some(arr) = workflows_value.as_array() {
                // Flat array - all go to transform engine
                let transform = parse_workflow_vec(arr)?;
                (transform, Vec::new(), Vec::new())
            } else {
                return Err("Workflows must be an object or array".to_string());
            };

        // Create engines with their own function registries
        // (trait objects can't be cloned, so we register separately for each engine)
        let transform_engine = Engine::new(transform_workflows, Some(register_all_functions()));
        let generate_engine = Engine::new(generate_workflows, Some(register_all_functions()));
        let validate_engine = Engine::new(validate_workflows, Some(register_all_functions()));

        Ok(ReframeEngine {
            transform_engine: Arc::new(transform_engine),
            generate_engine: Arc::new(generate_engine),
            validate_engine: Arc::new(validate_engine),
            workflows_json: workflows_json.to_string(),
        })
    }

    /// Transform a message (MT → MX or MX → MT).
    ///
    /// The payload is stored as a raw string and should be parsed by the
    /// parse_mt or parse_mx plugins in the workflow.
    ///
    /// # Arguments
    /// * `payload` - Raw string payload to transform (not parsed by the engine)
    ///
    /// # Returns
    /// A Promise that resolves to the transformed message as a JSON string
    ///
    /// # Example
    /// ```javascript
    /// // SWIFT MT message (parsed by parse_mt plugin)
    /// const mtMessage = "{1:F01BANKBEBBAXXX0000000000}{2:I103BANKDEFFXXXXN}{4:\n:20:REF123\n-}";
    /// const result = await engine.transform(mtMessage);
    /// const output = JSON.parse(result);
    /// console.log(output.context.data.result);
    /// ```
    #[wasm_bindgen]
    pub fn transform(&self, payload: &str) -> js_sys::Promise {
        self.process_with_engine(&self.transform_engine, payload, false)
    }

    /// Transform a message with execution trace for debugging.
    ///
    /// Returns step-by-step execution information suitable for dataflow-ui visualization.
    /// The payload is stored as a raw string and should be parsed by the parse plugins.
    ///
    /// # Arguments
    /// * `payload` - Raw string payload to transform (not parsed by the engine)
    ///
    /// # Returns
    /// A Promise that resolves to the execution trace as a JSON string
    ///
    /// # Example
    /// ```javascript
    /// const trace = await engine.transform_with_trace(mtMessage);
    /// const traceData = JSON.parse(trace);
    /// console.log(traceData.steps); // Array of execution steps
    /// // Use with dataflow-ui DebuggerProvider
    /// ```
    #[wasm_bindgen]
    pub fn transform_with_trace(&self, payload: &str) -> js_sys::Promise {
        self.process_with_engine(&self.transform_engine, payload, true)
    }

    /// Generate a sample message.
    ///
    /// The payload should be a JSON string containing a scenario schema with `source_type` field.
    /// The `source_type` is used to set `metadata.message_type` for workflow routing.
    ///
    /// # Arguments
    /// * `payload` - JSON string containing the scenario schema
    ///
    /// # Returns
    /// A Promise that resolves to the generated message as a JSON string
    ///
    /// # Example
    /// ```javascript
    /// const scenario = '{"source_type": "MT103", "schema": {...}}';
    /// const result = await engine.generate(scenario);
    /// const output = JSON.parse(result);
    /// console.log(output.data.result); // Generated MT103 message
    /// ```
    #[wasm_bindgen]
    pub fn generate(&self, payload: &str) -> js_sys::Promise {
        self.process_generate(&self.generate_engine, payload, false)
    }

    /// Generate a sample message with execution trace.
    ///
    /// # Arguments
    /// * `payload` - JSON string containing the scenario schema
    ///
    /// # Returns
    /// A Promise that resolves to the execution trace as a JSON string
    #[wasm_bindgen]
    pub fn generate_with_trace(&self, payload: &str) -> js_sys::Promise {
        self.process_generate(&self.generate_engine, payload, true)
    }

    /// Validate a message.
    ///
    /// The payload is stored as a raw string and should be parsed by the
    /// validate_mt or validate_mx plugins in the workflow.
    ///
    /// # Arguments
    /// * `payload` - Raw string payload to validate (not parsed by the engine)
    ///
    /// # Returns
    /// A Promise that resolves to the validation result as a JSON string
    ///
    /// # Example
    /// ```javascript
    /// const mtMessage = "{1:F01BANKBEBBAXXX0000000000}{2:I103BANKDEFFXXXXN}{4:\n:20:REF123\n-}";
    /// const result = await engine.validate(mtMessage);
    /// const validation = JSON.parse(result);
    /// console.log(validation.context.data.validation_result);
    /// ```
    #[wasm_bindgen]
    pub fn validate(&self, payload: &str) -> js_sys::Promise {
        self.process_with_engine(&self.validate_engine, payload, false)
    }

    /// Validate a message with execution trace.
    ///
    /// # Arguments
    /// * `payload` - Raw string payload to validate (not parsed by the engine)
    ///
    /// # Returns
    /// A Promise that resolves to the execution trace as a JSON string
    #[wasm_bindgen]
    pub fn validate_with_trace(&self, payload: &str) -> js_sys::Promise {
        self.process_with_engine(&self.validate_engine, payload, true)
    }

    /// Process a payload through the engine's workflows (for dataflow-ui compatibility).
    ///
    /// This is an alias for `transform` that matches the dataflow-ui DataflowEngine interface.
    /// The payload is stored as a raw string and should be parsed by a parse plugin.
    ///
    /// # Arguments
    /// * `payload` - Raw string payload to process (not parsed by the engine)
    ///
    /// # Returns
    /// A Promise that resolves to the processed message as a JSON string
    #[wasm_bindgen]
    pub fn process(&self, payload: &str) -> js_sys::Promise {
        self.transform(payload)
    }

    /// Process with trace (for dataflow-ui compatibility).
    ///
    /// This is an alias for `transform_with_trace` that matches the dataflow-ui DataflowEngine interface.
    /// The payload is stored as a raw string and should be parsed by a parse plugin.
    ///
    /// # Arguments
    /// * `payload` - Raw string payload to process (not parsed by the engine)
    ///
    /// # Returns
    /// A Promise that resolves to the execution trace as a JSON string
    #[wasm_bindgen]
    pub fn process_with_trace(&self, payload: &str) -> js_sys::Promise {
        self.transform_with_trace(payload)
    }

    /// Get the workflow definitions as a JSON string.
    ///
    /// # Returns
    /// The original workflow definitions JSON string
    #[wasm_bindgen]
    pub fn get_workflows(&self) -> String {
        self.workflows_json.clone()
    }

    /// Get the total number of workflows across all engines.
    #[wasm_bindgen]
    pub fn workflow_count(&self) -> usize {
        self.transform_engine.workflows().len()
            + self.generate_engine.workflows().len()
            + self.validate_engine.workflows().len()
    }

    /// Get the number of transform workflows.
    #[wasm_bindgen]
    pub fn transform_workflow_count(&self) -> usize {
        self.transform_engine.workflows().len()
    }

    /// Get the number of generate workflows.
    #[wasm_bindgen]
    pub fn generate_workflow_count(&self) -> usize {
        self.generate_engine.workflows().len()
    }

    /// Get the number of validate workflows.
    #[wasm_bindgen]
    pub fn validate_workflow_count(&self) -> usize {
        self.validate_engine.workflows().len()
    }

    /// Get the list of all workflow IDs.
    ///
    /// # Returns
    /// JSON object with `transform`, `generate`, and `validate` arrays of workflow IDs
    #[wasm_bindgen]
    pub fn workflow_ids(&self) -> String {
        let transform_ids: Vec<&String> = self.transform_engine.workflows().keys().collect();
        let generate_ids: Vec<&String> = self.generate_engine.workflows().keys().collect();
        let validate_ids: Vec<&String> = self.validate_engine.workflows().keys().collect();

        serde_json::to_string(&json!({
            "transform": transform_ids,
            "generate": generate_ids,
            "validate": validate_ids
        }))
        .unwrap_or_else(|_| r#"{"transform":[],"generate":[],"validate":[]}"#.to_string())
    }

    /// Get the list of registered function names.
    ///
    /// # Returns
    /// JSON array of function names available in the engines
    #[wasm_bindgen]
    pub fn function_names(&self) -> String {
        // Create a temporary function registry to get names
        let functions = register_all_functions();
        let names: Vec<&String> = functions.keys().collect();
        serde_json::to_string(&names).unwrap_or_else(|_| "[]".to_string())
    }
}

impl ReframeEngine {
    /// Internal helper to process a payload through a specific engine.
    /// Payload is stored as a raw string - parsing is done by the parse plugins.
    fn process_with_engine(
        &self,
        engine: &Arc<Engine>,
        payload: &str,
        with_trace: bool,
    ) -> js_sys::Promise {
        // Store payload as a raw string - parsing is done by the parse plugins
        let mut message = Message::from_value(&Value::String(payload.to_string()));

        // Clone the Arc for the async block
        let engine = Arc::clone(engine);

        if with_trace {
            future_to_promise(async move {
                match engine.process_message_with_trace(&mut message).await {
                    Ok(trace) => serde_json::to_string(&trace)
                        .map(|s| JsValue::from_str(&s))
                        .map_err(|e| JsValue::from_str(&e.to_string())),
                    Err(e) => Err(JsValue::from_str(&e.to_string())),
                }
            })
        } else {
            future_to_promise(async move {
                match engine.process_message(&mut message).await {
                    Ok(()) => serde_json::to_string(&message)
                        .map(|s| JsValue::from_str(&s))
                        .map_err(|e| JsValue::from_str(&e.to_string())),
                    Err(e) => Err(JsValue::from_str(&e.to_string())),
                }
            })
        }
    }

    /// Internal helper to process a generate request.
    /// Parses the scenario to extract source_type and sets it as metadata.message_type.
    fn process_generate(
        &self,
        engine: &Arc<Engine>,
        payload: &str,
        with_trace: bool,
    ) -> js_sys::Promise {
        // Parse the scenario to extract source_type for workflow routing
        let scenario: Value = match serde_json::from_str(payload) {
            Ok(v) => v,
            Err(e) => {
                let error_msg = format!("Invalid scenario JSON: {}", e);
                return future_to_promise(async move { Err(JsValue::from_str(&error_msg)) });
            }
        };

        // Extract source_type from scenario
        let message_type = scenario
            .get("source_type")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Create message with the scenario as payload
        let mut message = Message::from_value(&scenario);

        // Set metadata.message_type for workflow condition routing
        if let Some(metadata) = message.metadata_mut().as_object_mut() {
            metadata.insert("message_type".to_string(), json!(message_type));
        }

        // Clone the Arc for the async block
        let engine = Arc::clone(engine);

        if with_trace {
            future_to_promise(async move {
                match engine.process_message_with_trace(&mut message).await {
                    Ok(trace) => serde_json::to_string(&trace)
                        .map(|s| JsValue::from_str(&s))
                        .map_err(|e| JsValue::from_str(&e.to_string())),
                    Err(e) => Err(JsValue::from_str(&e.to_string())),
                }
            })
        } else {
            future_to_promise(async move {
                match engine.process_message(&mut message).await {
                    Ok(()) => serde_json::to_string(&message)
                        .map(|s| JsValue::from_str(&s))
                        .map_err(|e| JsValue::from_str(&e.to_string())),
                    Err(e) => Err(JsValue::from_str(&e.to_string())),
                }
            })
        }
    }
}

/// Parse an optional workflow array value into a Vec<Workflow>.
fn parse_workflow_array(value: Option<&Value>) -> Result<Vec<Workflow>, String> {
    match value {
        Some(Value::Array(arr)) => parse_workflow_vec(arr),
        Some(Value::Null) | None => Ok(Vec::new()),
        _ => Err("Workflow value must be an array".to_string()),
    }
}

/// Parse a JSON array into a Vec<Workflow>.
fn parse_workflow_vec(arr: &[Value]) -> Result<Vec<Workflow>, String> {
    let mut workflows = Vec::with_capacity(arr.len());
    for (i, workflow_value) in arr.iter().enumerate() {
        let workflow_str = serde_json::to_string(workflow_value).map_err(|e| e.to_string())?;
        let workflow = Workflow::from_json(&workflow_str)
            .map_err(|e| format!("Invalid workflow at index {}: {}", i, e))?;
        workflows.push(workflow);
    }
    Ok(workflows)
}

/// Convenience function to transform a message with a one-off engine.
///
/// Creates an engine with the given workflows and transforms a single message.
/// Use ReframeEngine class for better performance when processing multiple messages.
/// The payload is stored as a raw string and should be parsed by a parse plugin.
///
/// # Arguments
/// * `workflows_json` - JSON string containing workflow definitions
/// * `payload` - Raw string payload to transform (not parsed by the engine)
///
/// # Returns
/// A Promise that resolves to the transformed message as a JSON string
///
/// # Example
/// ```javascript
/// const mtMessage = "{1:F01BANKBEBBAXXX...}";
/// const result = await transform_message(workflowsJson, mtMessage);
/// console.log(JSON.parse(result));
/// ```
#[wasm_bindgen]
pub fn transform_message(workflows_json: &str, payload: &str) -> js_sys::Promise {
    let engine_result = ReframeEngine::new(workflows_json);
    match engine_result {
        Ok(engine) => engine.transform(payload),
        Err(e) => future_to_promise(async move { Err(JsValue::from_str(&e)) }),
    }
}

/// Convenience function to validate a message with a one-off engine.
///
/// The payload is stored as a raw string and should be parsed by a validate plugin.
///
/// # Arguments
/// * `workflows_json` - JSON string containing validation workflow definitions
/// * `payload` - Raw string payload to validate (not parsed by the engine)
///
/// # Returns
/// A Promise that resolves to the validation result as a JSON string
#[wasm_bindgen]
pub fn validate_message(workflows_json: &str, payload: &str) -> js_sys::Promise {
    // For validation, we need to structure workflows correctly
    let workflows_value: Result<Value, _> = serde_json::from_str(workflows_json);
    match workflows_value {
        Ok(value) => {
            // Wrap as validate workflows if it's a flat array
            let structured = if value.is_array() {
                json!({ "validate": value })
            } else {
                value
            };
            let structured_json = serde_json::to_string(&structured).unwrap_or_default();
            let engine_result = ReframeEngine::new(&structured_json);
            match engine_result {
                Ok(engine) => engine.validate(payload),
                Err(e) => future_to_promise(async move { Err(JsValue::from_str(&e)) }),
            }
        }
        Err(e) => {
            let error_msg = format!("Invalid workflows JSON: {}", e);
            future_to_promise(async move { Err(JsValue::from_str(&error_msg)) })
        }
    }
}

/// Convenience function to generate a sample message with a one-off engine.
///
/// The payload is stored as a raw string containing generation parameters.
///
/// # Arguments
/// * `workflows_json` - JSON string containing generation workflow definitions
/// * `payload` - Raw string payload with generation parameters
///
/// # Returns
/// A Promise that resolves to the generated message as a JSON string
#[wasm_bindgen]
pub fn generate_message(workflows_json: &str, payload: &str) -> js_sys::Promise {
    // For generation, we need to structure workflows correctly
    let workflows_value: Result<Value, _> = serde_json::from_str(workflows_json);
    match workflows_value {
        Ok(value) => {
            // Wrap as generate workflows if it's a flat array
            let structured = if value.is_array() {
                json!({ "generate": value })
            } else {
                value
            };
            let structured_json = serde_json::to_string(&structured).unwrap_or_default();
            let engine_result = ReframeEngine::new(&structured_json);
            match engine_result {
                Ok(engine) => engine.generate(payload),
                Err(e) => future_to_promise(async move { Err(JsValue::from_str(&e)) }),
            }
        }
        Err(e) => {
            let error_msg = format!("Invalid workflows JSON: {}", e);
            future_to_promise(async move { Err(JsValue::from_str(&error_msg)) })
        }
    }
}
