//! WebAssembly tests for reframe-wasm
//!
//! Run with: wasm-pack test --headless --chrome

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

use reframe_wasm::ReframeEngine;

/// Test that the engine can be created with empty workflows
#[wasm_bindgen_test]
fn test_engine_creation_empty() {
    let engine = ReframeEngine::new("[]").expect("Failed to create engine");
    assert_eq!(engine.workflow_count(), 0);
}

/// Test that the engine can be created with categorized workflows
#[wasm_bindgen_test]
fn test_engine_creation_categorized() {
    let workflows = r#"{
        "transform": [],
        "generate": [],
        "validate": []
    }"#;
    let engine = ReframeEngine::new(workflows).expect("Failed to create engine");
    assert_eq!(engine.workflow_count(), 0);
    assert_eq!(engine.transform_workflow_count(), 0);
    assert_eq!(engine.generate_workflow_count(), 0);
    assert_eq!(engine.validate_workflow_count(), 0);
}

/// Test workflow IDs output
#[wasm_bindgen_test]
fn test_workflow_ids() {
    let engine = ReframeEngine::new("[]").expect("Failed to create engine");
    let ids = engine.workflow_ids();
    assert!(ids.contains("transform"));
    assert!(ids.contains("generate"));
    assert!(ids.contains("validate"));
}

/// Test function names output
#[wasm_bindgen_test]
fn test_function_names() {
    let engine = ReframeEngine::new("[]").expect("Failed to create engine");
    let names = engine.function_names();
    // Should contain registered functions
    assert!(names.contains("detect"));
    assert!(names.contains("parse_mt"));
    assert!(names.contains("parse_mx"));
}

/// Test engine with a simple workflow
#[wasm_bindgen_test]
fn test_engine_with_workflow() {
    let workflows = r#"[{
        "id": "test-workflow",
        "name": "Test Workflow",
        "priority": 1,
        "tasks": [{
            "id": "task1",
            "name": "Parse Payload",
            "function": {
                "name": "parse",
                "input": {}
            }
        }]
    }]"#;

    let engine = ReframeEngine::new(workflows).expect("Failed to create engine");
    assert_eq!(engine.workflow_count(), 1);
    assert_eq!(engine.transform_workflow_count(), 1);
}

/// Test get_workflows returns original JSON
#[wasm_bindgen_test]
fn test_get_workflows() {
    let workflows = "[]";
    let engine = ReframeEngine::new(workflows).expect("Failed to create engine");
    assert_eq!(engine.get_workflows(), workflows);
}

/// Test error handling for invalid JSON
#[wasm_bindgen_test]
fn test_invalid_json() {
    let result = ReframeEngine::new("not valid json");
    assert!(result.is_err());
}

/// Test error handling for invalid workflow structure
#[wasm_bindgen_test]
fn test_invalid_workflow_structure() {
    let result = ReframeEngine::new(r#"{"invalid": "structure"}"#);
    // Should succeed but with empty workflows since no valid arrays provided
    assert!(result.is_ok());
}
