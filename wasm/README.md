# reframe-wasm

WebAssembly bindings for [Reframe](https://github.com/GoPlasmatic/reframe) - an enterprise-grade, bidirectional SWIFT MT ↔ ISO 20022 transformation service.

## Features

- **Three Engine Architecture**: Transform, Generate, and Validate engines matching Reframe's server architecture
- **Full Plugin Support**: All SWIFT MT and MX message handling functions (`parse_mt`, `parse_mx`, `validate_mt`, `validate_mx`, `generate_mt`, `generate_mx`, `publish_mt`, `publish_mx`, `detect`)
- **Execution Tracing**: Step-by-step debugging with `*_with_trace` methods for [dataflow-ui](https://github.com/GoPlasmatic/dataflow-ui) integration
- **Promise-based API**: All async operations return JavaScript Promises
- **Raw String Payloads**: Payloads are passed as raw strings - parsing is handled by the parse plugins in workflows

## Installation

```bash
npm install @goplasmatic/reframe-wasm
```

## Quick Start

```typescript
import init, { ReframeEngine } from '@goplasmatic/reframe-wasm';

// Initialize WASM module
await init();

// Load workflows (from your workflow package)
const workflows = {
  transform: [...transformWorkflows],
  generate: [...generateWorkflows],
  validate: [...validateWorkflows]
};
const engine = new ReframeEngine(JSON.stringify(workflows));

// Transform a SWIFT MT message to ISO 20022
// Payload is a raw string - parsing is done by the parse_mt plugin in the workflow
const mtMessage = "{1:F01BANKBEBBAXXX0000000000}{2:I103BANKDEFFXXXXN}{4:\n:20:REFERENCE123\n:23B:CRED\n:32A:230101EUR1000,00\n:50K:/12345678\nJOHN DOE\n:59:/87654321\nJANE DOE\n:71A:SHA\n-}";

const result = await engine.transform(mtMessage);
console.log(JSON.parse(result));
```

## API Reference

### ReframeEngine

The main engine class that wraps three dataflow-rs engines.

#### Constructor

```typescript
constructor(workflows_json: string): ReframeEngine
```

Creates a new engine with the given workflows. Accepts either:
- An object with `transform`, `generate`, `validate` keys (each an array of workflows)
- A flat array of workflows (all go to transform engine)

#### Methods

| Method | Description |
|--------|-------------|
| `transform(payload: string): Promise<string>` | Transform a message (MT → MX or MX → MT) |
| `transform_with_trace(payload: string): Promise<string>` | Transform with execution trace |
| `generate(payload: string): Promise<string>` | Generate a sample message |
| `generate_with_trace(payload: string): Promise<string>` | Generate with execution trace |
| `validate(payload: string): Promise<string>` | Validate a message |
| `validate_with_trace(payload: string): Promise<string>` | Validate with execution trace |
| `process(payload: string): Promise<string>` | Alias for `transform` (dataflow-ui compatibility) |
| `process_with_trace(payload: string): Promise<string>` | Alias for `transform_with_trace` |
| `get_workflows(): string` | Get original workflow definitions |
| `workflow_count(): number` | Get total workflow count |
| `transform_workflow_count(): number` | Get transform workflow count |
| `generate_workflow_count(): number` | Get generate workflow count |
| `validate_workflow_count(): number` | Get validate workflow count |
| `workflow_ids(): string` | Get workflow IDs as JSON |
| `function_names(): string` | Get registered function names as JSON |

### Standalone Functions

```typescript
// One-off transformation
function transform_message(workflows_json: string, payload: string): Promise<string>;

// One-off validation
function validate_message(workflows_json: string, payload: string): Promise<string>;

// One-off generation
function generate_message(workflows_json: string, payload: string): Promise<string>;
```

## Payload Handling

**Important:** All payloads are passed as raw strings. The parsing is handled by the parse plugins (`parse_mt`, `parse_mx`) in the workflows, not by the engine itself.

```javascript
// MT message - raw string, parsed by parse_mt plugin
const mtMessage = "{1:F01BANKBEBBAXXX...}{2:I103...}{4:\n:20:REF\n-}";
await engine.transform(mtMessage);

// MX message - raw XML string, parsed by parse_mx plugin
const mxMessage = "<?xml version='1.0'?><Document>...</Document>";
await engine.transform(mxMessage);
```

## Integration with dataflow-ui

The WASM library is designed to work with [dataflow-ui](https://github.com/GoPlasmatic/dataflow-ui) for workflow visualization and debugging.

```tsx
import { WorkflowVisualizer, DebuggerProvider, DataflowEngine } from '@goplasmatic/dataflow-ui';
import init, { ReframeEngine } from '@goplasmatic/reframe-wasm';

// Initialize WASM
await init();

// Create adapter implementing DataflowEngine interface
class ReframeEngineAdapter implements DataflowEngine {
  private engine: ReframeEngine;

  constructor(workflows: Workflow[]) {
    this.engine = new ReframeEngine(JSON.stringify({ transform: workflows }));
  }

  async processWithTrace(payload: string) {
    const result = await this.engine.process_with_trace(payload);
    return JSON.parse(result);
  }

  dispose() {
    this.engine.free();
  }
}

// Use in React component
function ReframeDebugView({ workflows }) {
  return (
    <DebuggerProvider engineFactory={(wfs) => new ReframeEngineAdapter(wfs)}>
      <WorkflowVisualizer workflows={workflows} debugMode={true} />
    </DebuggerProvider>
  );
}
```

## Registered Functions

The following plugin functions are automatically registered:

| Function | Description |
|----------|-------------|
| `detect` | Generic payload detection and analysis |
| `parse_mt` | Parse SWIFT MT message |
| `validate_mt` | Validate SWIFT MT message |
| `generate_mt` | Generate SWIFT MT message |
| `publish_mt` | Format SWIFT MT message for output |
| `parse_mx` | Parse ISO 20022 MX message |
| `validate_mx` | Validate ISO 20022 MX message |
| `generate_mx` | Generate ISO 20022 MX message |
| `publish_mx` | Format ISO 20022 MX message for output |

## Building from Source

### Prerequisites

- Rust 1.87+ (edition 2024)
- wasm-pack (`cargo install wasm-pack`)

### Build

```bash
cd wasm
wasm-pack build --target web --out-dir pkg
```

### Test

```bash
wasm-pack test --headless --chrome
```

### Publish

```bash
./publish.sh
cd pkg && npm publish --access public
```

## Local Development

To use local versions of dependencies during development, uncomment the patch section in `Cargo.toml`:

```toml
[patch.crates-io]
swift-mt-message = { path = "../../SwiftMTMessage" }
mx-message = { path = "../../MXMessage" }
dataflow-rs = { path = "../../dataflow-rs" }
datalogic-rs = { path = "../../datalogic-rs" }
```

## License

Apache-2.0

## Related Projects

- [Reframe](https://github.com/GoPlasmatic/reframe) - Server-side transformation service
- [dataflow-rs](https://github.com/GoPlasmatic/dataflow-rs) - Workflow engine
- [dataflow-ui](https://github.com/GoPlasmatic/dataflow-ui) - Workflow visualization
- [swift-mt-message](https://github.com/GoPlasmatic/SwiftMTMessage) - SWIFT MT parsing
- [mx-message](https://github.com/GoPlasmatic/MXMessage) - ISO 20022 parsing
