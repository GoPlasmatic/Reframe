# Reframe Architecture Document

## Overview

Reframe is an enterprise-grade bidirectional SWIFT MT ↔ ISO 20022 transformation service built in Rust. It leverages a suite of custom-built libraries to provide high-performance, transparent, and configurable message transformations for financial institutions.

## Core Libraries

### 1. **Dataflow-rs** (Workflow Engine)
- **Purpose**: JSON-based workflow orchestration engine
- **Role**: Executes transformation pipelines defined in JSON configuration files
- **Location**: `../dataflow-rs`

### 2. **Datalogic-rs** (Logic Engine)
- **Purpose**: JSONLogic implementation for declarative transformation rules
- **Role**: Evaluates conditional logic and business rules during transformations
- **Location**: `../datalogic-rs`

### 3. **Datafake-rs** (Data Generation)
- **Purpose**: Test data generation from JSON schemas
- **Role**: Creates realistic sample messages for testing and validation
- **Location**: `../datafake-rs`

### 4. **SwiftMTMessage** (MT Message Library)
- **Purpose**: SWIFT MT message parsing and generation
- **Role**: Handles MT message structure, validation, and serialization
- **Location**: `../SwiftMTMessage`

### 5. **MXMessage** (ISO 20022 Library)
- **Purpose**: ISO 20022 (MX) message parsing and generation
- **Role**: Manages MX message schemas, validation, and XML serialization
- **Location**: `../MXMessage`

## Architecture Diagram

```mermaid
graph TB
    subgraph "External Clients"
        CLIENT[REST API Clients]
        WEB[Web UI]
        CLI[CLI Tools]
    end

    subgraph "Reframe Core Service"
        subgraph "API Layer"
            HTTP[Axum HTTP Server<br/>REST Endpoints]
        end

        subgraph "Processing Layer"
            ROUTER[Message Router]
            FWD_ENGINE[Forward Engine<br/>MT → MX]
            REV_ENGINE[Reverse Engine<br/>MX → MT]
        end

        subgraph "Parsers & Publishers"
            PARSE_MT[ParseMT Module]
            PARSE_MX[ParseMX Module]
            PUB_MT[PublishMT Module]
            PUB_MX[PublishMX Module]
            SAMPLE_GEN[Sample Generator]
        end
    end

    subgraph "Core Libraries"
        subgraph "Workflow Orchestration"
            DATAFLOW[Dataflow-rs<br/>Workflow Engine]
            DATALOGIC[Datalogic-rs<br/>Logic Evaluator]
        end

        subgraph "Message Libraries"
            SWIFT_MT[SwiftMTMessage<br/>MT Parser/Generator]
            MX_MSG[MXMessage<br/>MX Parser/Generator]
        end

        subgraph "Test Data"
            DATAFAKE[Datafake-rs<br/>Data Generator]
        end
    end

    subgraph "Configuration & Data"
        WORKFLOWS[Workflow JSON Files<br/>Transformation Rules]
        SCENARIOS[Scenario JSON Files<br/>Test Data Schemas]
    end

    %% Client connections
    CLIENT --> HTTP
    WEB --> HTTP
    CLI --> HTTP

    %% HTTP to routers
    HTTP --> ROUTER

    %% Router to engines
    ROUTER -->|MT Input| FWD_ENGINE
    ROUTER -->|MX Input| REV_ENGINE

    %% Forward Engine flow
    FWD_ENGINE --> PARSE_MT
    PARSE_MT --> SWIFT_MT
    FWD_ENGINE --> DATAFLOW
    DATAFLOW --> DATALOGIC
    DATAFLOW --> WORKFLOWS
    FWD_ENGINE --> PUB_MX
    PUB_MX --> MX_MSG

    %% Reverse Engine flow
    REV_ENGINE --> PARSE_MX
    PARSE_MX --> MX_MSG
    REV_ENGINE --> DATAFLOW
    REV_ENGINE --> PUB_MT
    PUB_MT --> SWIFT_MT

    %% Sample generation
    SAMPLE_GEN --> DATAFAKE
    DATAFAKE --> SCENARIOS
    SAMPLE_GEN --> SWIFT_MT
    SAMPLE_GEN --> MX_MSG

    %% Library dependencies
    DATAFLOW -.->|uses| DATALOGIC
    PARSE_MT -.->|uses| SWIFT_MT
    PARSE_MX -.->|uses| MX_MSG
    PUB_MT -.->|uses| SWIFT_MT
    PUB_MX -.->|uses| MX_MSG

    classDef library fill:#e1f5fe,stroke:#01579b,stroke-width:2px
    classDef engine fill:#f3e5f5,stroke:#4a148c,stroke-width:2px
    classDef config fill:#fff9c4,stroke:#f57f17,stroke-width:2px
    classDef api fill:#e8f5e9,stroke:#1b5e20,stroke-width:2px

    class DATAFLOW,DATALOGIC,SWIFT_MT,MX_MSG,DATAFAKE library
    class FWD_ENGINE,REV_ENGINE engine
    class WORKFLOWS,SCENARIOS config
    class HTTP,ROUTER api
```

## Data Flow Diagrams

### Forward Transformation (MT → ISO 20022)

```mermaid
sequenceDiagram
    participant Client
    participant HTTP as HTTP Server
    participant Router
    participant FwdEngine as Forward Engine
    participant ParseMT
    participant SwiftMT as SwiftMTMessage
    participant Dataflow as Dataflow-rs
    participant Datalogic as Datalogic-rs
    participant PublishMX
    participant MXMsg as MXMessage

    Client->>HTTP: POST /transform/mt-to-mx
    HTTP->>Router: Route MT message
    Router->>FwdEngine: Process MT
    
    FwdEngine->>ParseMT: Parse MT message
    ParseMT->>SwiftMT: Validate & structure
    SwiftMT-->>ParseMT: Parsed MT
    ParseMT-->>FwdEngine: MT data structure
    
    FwdEngine->>Dataflow: Execute workflow
    Dataflow->>Datalogic: Evaluate conditions
    Datalogic-->>Dataflow: Logic results
    Dataflow-->>FwdEngine: Transformed data
    
    FwdEngine->>PublishMX: Generate MX
    PublishMX->>MXMsg: Create ISO 20022
    MXMsg-->>PublishMX: XML message
    PublishMX-->>FwdEngine: MX output
    
    FwdEngine-->>Router: MX message
    Router-->>HTTP: Response
    HTTP-->>Client: ISO 20022 XML
```

### Reverse Transformation (ISO 20022 → MT)

```mermaid
sequenceDiagram
    participant Client
    participant HTTP as HTTP Server
    participant Router
    participant RevEngine as Reverse Engine
    participant ParseMX
    participant MXMsg as MXMessage
    participant Dataflow as Dataflow-rs
    participant Datalogic as Datalogic-rs
    participant PublishMT
    participant SwiftMT as SwiftMTMessage

    Client->>HTTP: POST /transform/mx-to-mt
    HTTP->>Router: Route MX message
    Router->>RevEngine: Process MX
    
    RevEngine->>ParseMX: Parse MX message
    ParseMX->>MXMsg: Validate XML
    MXMsg-->>ParseMX: Parsed MX
    ParseMX-->>RevEngine: MX data structure
    
    RevEngine->>Dataflow: Execute workflow
    Dataflow->>Datalogic: Evaluate conditions
    Datalogic-->>Dataflow: Logic results
    Dataflow-->>RevEngine: Transformed data
    
    RevEngine->>PublishMT: Generate MT
    PublishMT->>SwiftMT: Create SWIFT MT
    SwiftMT-->>PublishMT: MT message
    PublishMT-->>RevEngine: MT output
    
    RevEngine-->>Router: MT message
    Router-->>HTTP: Response
    HTTP-->>Client: SWIFT MT text
```

### Sample Generation Flow

```mermaid
sequenceDiagram
    participant Client
    participant HTTP as HTTP Server
    participant SampleGen as Sample Generator
    participant Datafake as Datafake-rs
    participant SwiftMT as SwiftMTMessage
    participant MXMsg as MXMessage
    participant Scenarios as Scenario Files

    Client->>HTTP: POST /generate/sample
    HTTP->>SampleGen: Generate sample
    
    SampleGen->>Scenarios: Load scenario
    Scenarios-->>SampleGen: Schema definition
    
    SampleGen->>Datafake: Generate data
    Datafake-->>SampleGen: Generated values
    
    alt MT Message
        SampleGen->>SwiftMT: Generate MT
        SwiftMT-->>SampleGen: MT message
    else MX Message
        SampleGen->>MXMsg: Generate MX
        MXMsg-->>SampleGen: MX message
    end
    
    SampleGen-->>HTTP: Sample message
    HTTP-->>Client: Generated sample
```

## Library Dependencies and Interactions

### 1. **Dataflow-rs ↔ Datalogic-rs**
- Dataflow-rs orchestrates workflow execution
- Calls Datalogic-rs for conditional logic evaluation
- Datalogic-rs provides JSONLogic operators for transformations

### 2. **SwiftMTMessage Integration**
- Used by ParseMT for parsing incoming MT messages
- Used by PublishMT for generating MT output
- Provides MT message validation and structure definitions

### 3. **MXMessage Integration**
- Used by ParseMX for parsing ISO 20022 XML
- Used by PublishMX for generating MX output
- Provides MX schema validation and XML serialization

### 4. **Datafake-rs Integration**
- Used by Sample Generator for test data creation
- Reads scenario JSON schemas
- Generates realistic financial data using faker patterns

### 5. **Workflow Configuration**
- Dataflow-rs reads workflow JSON files
- Workflows define transformation pipelines
- Each workflow step can invoke Datalogic-rs for conditions

## Component Responsibilities

### Reframe Core
- **HTTP Server**: Request handling, routing, response formatting
- **Forward Engine**: Manages MT→MX transformation pipeline
- **Reverse Engine**: Manages MX→MT transformation pipeline
- **Message Router**: Determines transformation direction

### Library Responsibilities

| Library | Primary Responsibility | Key Features |
|---------|----------------------|--------------|
| **Dataflow-rs** | Workflow orchestration | JSON-based workflows, pipeline execution, task management |
| **Datalogic-rs** | Business logic evaluation | JSONLogic operators, conditional rules, data manipulation |
| **Datafake-rs** | Test data generation | Faker patterns, schema-based generation, realistic values |
| **SwiftMTMessage** | MT message handling | MT parsing, validation, generation, field definitions |
| **MXMessage** | MX message handling | XML parsing, schema validation, ISO 20022 structures |

## Deployment Architecture

```mermaid
graph TB
    subgraph "Production Environment"
        LB[Load Balancer]
        
        subgraph "Application Instances"
            INST1[Reframe Instance 1]
            INST2[Reframe Instance 2]
            INST3[Reframe Instance N]
        end
        
        subgraph "Shared Resources"
            CONFIG[Workflow Configs<br/>Mounted Volume]
            SCENARIOS_VOL[Scenario Files<br/>Mounted Volume]
        end
        
        subgraph "Embedded Libraries"
            direction LR
            LIBS[Dataflow-rs<br/>Datalogic-rs<br/>Datafake-rs<br/>SwiftMTMessage<br/>MXMessage]
        end
    end
    
    LB --> INST1
    LB --> INST2
    LB --> INST3
    
    INST1 --> CONFIG
    INST2 --> CONFIG
    INST3 --> CONFIG
    
    INST1 --> SCENARIOS_VOL
    INST2 --> SCENARIOS_VOL
    INST3 --> SCENARIOS_VOL
    
    INST1 -.->|contains| LIBS
    INST2 -.->|contains| LIBS
    INST3 -.->|contains| LIBS
    
    classDef instance fill:#bbdefb,stroke:#1565c0,stroke-width:2px
    classDef storage fill:#fff9c4,stroke:#f57f17,stroke-width:2px
    classDef libs fill:#f3e5f5,stroke:#4a148c,stroke-width:1px,stroke-dasharray: 5 5
    
    class INST1,INST2,INST3 instance
    class CONFIG,SCENARIOS_VOL storage
    class LIBS libs
```

## Key Design Patterns

### 1. **Engine Pattern**
- Separate engines for forward and reverse transformations
- Engines are initialized once and reused across requests
- Each engine maintains its own workflow pipeline

### 2. **Plugin Architecture**
- Libraries are plugged into the core service
- Each library has a specific interface and responsibility
- Easy to extend or replace individual components

### 3. **Configuration-Driven**
- Workflows defined in JSON for transparency
- Business rules externalized via Datalogic-rs
- Scenarios for test data also in JSON format

### 4. **Pipeline Processing**
- Dataflow-rs orchestrates multi-step transformations
- Each step can be independently configured
- Supports conditional branching via Datalogic-rs

## Performance Considerations

1. **Rust Performance**: All libraries built in Rust for maximum performance
2. **Stateless Design**: Each request is independent, enabling horizontal scaling
3. **Engine Reuse**: Engines initialized once, avoiding repeated setup overhead
4. **Efficient Parsing**: Specialized parsers for MT and MX formats
5. **Memory Safety**: Rust's ownership model prevents memory leaks

## Future Architecture Considerations

1. **Library Versioning**: Semantic versioning for coordinated updates
2. **Plugin System**: Dynamic library loading for runtime extensions
3. **Caching Layer**: Potential for workflow result caching
4. **Distributed Processing**: Message queue integration for async processing
5. **Monitoring Integration**: Metrics collection across all libraries