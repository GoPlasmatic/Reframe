# 🗺️ Mapping Guide

Comprehensive guide to field mapping and business rule configuration in Reframe.

## Table of Contents
- [Overview](#overview)
- [Mapping Concepts](#mapping-concepts)
- [Forward Mapping (MT → ISO 20022)](#forward-mapping-mt--iso-20022)
- [Reverse Mapping (ISO 20022 → MT)](#reverse-mapping-iso-20022--mt)
- [Transformation Functions](#transformation-functions)
- [Business Rules](#business-rules)
- [Advanced Mapping Patterns](#advanced-mapping-patterns)
- [Troubleshooting](#troubleshooting)

---

## Overview

Reframe's mapping system uses [datalogic-rs](https://github.com/GoPlasmatic/datalogic-rs) (a Rust implementation of JSON Logic) for powerful, transparent transformations between SWIFT MT and ISO 20022 formats. All mapping logic is externalized in JSON workflow files powered by the [dataflow-rs](https://github.com/GoPlasmatic/dataflow-rs) workflow engine, making it completely auditable and customizable.

### Key Features

- **🔍 Transparent Mapping**: All field mappings visible in JSON workflow files
- **🔧 JSON Logic**: Powerful conditional and transformation logic via [datalogic-rs](https://github.com/GoPlasmatic/datalogic-rs)
- **📊 Bidirectional Support**: Dedicated workflows for each transformation direction
- **✅ Business Rules**: Complex validation and conditional logic
- **🎯 Message-Specific**: Tailored workflows for each message type
- **🔗 Task-Based Processing**: Sequential transformation tasks with conditions using [dataflow-rs](https://github.com/GoPlasmatic/dataflow-rs)

### Technology Stack

- **Workflow Engine**: [dataflow-rs](https://github.com/GoPlasmatic/dataflow-rs) - A powerful Rust-based workflow engine for data processing pipelines
- **Logic Engine**: [datalogic-rs](https://github.com/GoPlasmatic/datalogic-rs) - Rust implementation of JSON Logic for declarative transformations

---

## Mapping Concepts

### Workflow Structure

Each workflow file contains:
- **id**: Unique workflow identifier
- **name**: Human-readable workflow name
- **description**: Workflow purpose and details
- **priority**: Execution priority
- **condition**: JSON Logic condition for workflow execution (powered by datalogic-rs)
- **tasks**: Array of transformation tasks (executed by dataflow-rs)

### JSON Logic Notation

Reframe uses [datalogic-rs](https://github.com/GoPlasmatic/datalogic-rs) for conditions and transformations:

```json
{
  "var": "data.SwiftMT.fields.20.value"          // Variable access
}

{
  "if": [                                        // If-then-else
    {"var": "data.SwiftMT.fields.32A.amount"},
    {"var": "data.SwiftMT.fields.32A.amount"},
    0.0
  ]
}

{
  "cat": [                                       // String concatenation
    {"var": "data.SwiftMT.fields.20.value"},
    "-pacs.008-001"
  ]
}

{
  "==": [{"var": "SwiftMT.method"}, "normal"]   // Equality check
}

{
  "and": [                                       // Logical AND
    {"var": "data.SwiftMT.fields.53A"},
    {"!": {"var": "data.SwiftMT.fields.54A"}}
  ]
}
```

### Task Structure

```json
{
  "id": "task_identifier",
  "name": "Task Name",
  "description": "Task description",
  "condition": {                                 // Optional condition
    "==": [{"var": "SwiftMT.method"}, "normal"]
  },
  "function": {
    "name": "map",                               // Function type: map, validate, PublishMX
    "input": {
      "mappings": [...]                          // Function-specific configuration
    }
  }
}
```

---

## Forward Mapping (MT → ISO 20022)

### MT103 to pacs.008 Mapping

#### Business Application Header (BAH)

Example from `workflows/forward/MT103/bah-mapping.json`:

```json
{
  "id": "mt103-bah-mapper",
  "name": "MT103 Business Application Header Mapping for CBPR+",
  "priority": 2,
  "condition": {
    "and": [
      {"==": [{"var": "SwiftMT.message_type"}, "103"]},
      {"in": [{"var": "SwiftMT.method"}, ["normal", "stp"]]},
      {"==": [{"var": "progress.workflow_id"}, "parser"]},
      {"==": [{"var": "progress.status_code"}, 200]}
    ]
  },
  "tasks": [
    {
      "id": "prepare_bah_context",
      "name": "Prepare BAH Context",
      "function": {
        "name": "map",
        "input": {
          "mappings": [
            {
              "path": "temp_data",
              "logic": {
                "Sender": {
                  "if": [
                    {"var": "data.SwiftMT.basic_header.sender_bic.raw"},
                    {"var": "data.SwiftMT.basic_header.sender_bic.raw"},
                    {"if": [
                      {"==": [{"var": "SwiftMT.method"}, "stp"]},
                      "",
                      "NOTPROVIDED"
                    ]}
                  ]
                },
                "Receiver": {
                  "if": [
                    {"var": "data.SwiftMT.application_header.receiver_bic.raw"},
                    {"var": "data.SwiftMT.application_header.receiver_bic.raw"},
                    {"if": [
                      {"==": [{"var": "SwiftMT.method"}, "stp"]},
                      "",
                      "NOTPROVIDED"
                    ]}
                  ]
                }
              }
            }
          ]
        }
      }
    },
    {
      "id": "construct_business_application_header",
      "name": "Construct Business Application Header",
      "function": {
        "name": "map",
        "input": {
          "mappings": [
            {
              "path": "data.AppHdr",
              "logic": {
                "Fr": {
                  "FIId": {
                    "FinInstnId": {
                      "BICFI": {"var": "temp_data.Sender"}
                    }
                  }
                },
                "To": {
                  "FIId": {
                    "FinInstnId": {
                      "BICFI": {"var": "temp_data.Receiver"}
                    }
                  }
                },
                "BizMsgIdr": {
                  "cat": [
                    {"var": "data.SwiftMT.fields.20.value"},
                    "-pacs.008-001"
                  ]
                },
                "MsgDefIdr": "pacs.008.001.08",
                "BizSvc": "swift.cbprplus.02",
                "CreDt": {
                  "if": [
                    {"var": "data.SwiftMT.fields.32A.value_date"},
                    {
                      "cat": [
                        {"var": "data.SwiftMT.fields.32A.value_date"},
                        "T",
                        {"format_date": [{"now": []}, "HH:mm:ss"]},
                        "+00:00"
                      ]
                    },
                    "9999-12-31T00:00:00+00:00"
                  ]
                }
              }
            }
          ]
        }
      }
    }
  ]
}
```

#### Document Content Mapping

Example from `workflows/forward/MT103/document-mapping.json`:

```json
{
  "id": "mt103-document-mapper",
  "name": "MT103 to pacs.008 Document Mapping for CBPR+",
  "tasks": [
    {
      "id": "determine_settlement_method_and_agents",
      "name": "Determine Settlement Method and Agents",
      "function": {
        "name": "map",
        "input": {
          "mappings": [
            {
              "path": "temp_data.settlement_method",
              "logic": {
                "if": [
                  {"and": [
                    {"!": {"var": "data.SwiftMT.fields.53A"}},
                    {"!": {"var": "data.SwiftMT.fields.53B"}},
                    {"!": {"var": "data.SwiftMT.fields.53D"}},
                    {"!": {"var": "data.SwiftMT.fields.54A"}}
                  ]},
                  "INDA",
                  {
                    "if": [
                      {"var": "data.SwiftMT.fields.53A.bic.raw"},
                      {
                        "if": [
                          {"==": [
                            {"substr": [{"var": "data.SwiftMT.fields.53A.bic.raw"}, 0, 6]},
                            {"substr": [{"var": "temp_data.Sender"}, 0, 6]}
                          ]},
                          "INDA",
                          "COVE"
                        ]
                      },
                      "COVE"
                    ]
                  }
                ]
              }
            }
          ]
        }
      }
    },
    {
      "id": "construct_group_header",
      "name": "Construct Group Header",
      "function": {
        "name": "map",
        "input": {
          "mappings": [
            {
              "path": "data.Document.FIToFICstmrCdtTrf.GrpHdr",
              "logic": {
                "MsgId": {
                  "if": [
                    {"var": "data.SwiftMT.fields.20.value"},
                    {"var": "data.SwiftMT.fields.20.value"},
                    "NOTPROVIDED"
                  ]
                },
                "CreDtTm": {
                  "if": [
                    {"var": "data.SwiftMT.fields.32A.value_date"},
                    {
                      "cat": [
                        {"var": "data.SwiftMT.fields.32A.value_date"},
                        "T",
                        {"format_date": [{"now": []}, "HH:mm:ss"]},
                        "+00:00"
                      ]
                    },
                    {"format_date": [{"now": []}, "yyyy-MM-ddTHH:mm:ss+00:00"]}
                  ]
                },
                "NbOfTxs": "1",
                "TtlIntrBkSttlmAmt": {
                  "@Ccy": {
                    "if": [
                      {"var": "data.SwiftMT.fields.32A.currency"},
                      {"var": "data.SwiftMT.fields.32A.currency"},
                      "USD"
                    ]
                  },
                  "$value": {
                    "if": [
                      {"var": "data.SwiftMT.fields.32A.amount"},
                      {"var": "data.SwiftMT.fields.32A.amount"},
                      0.0
                    ]
                  }
                },
                "SttlmInf": {
                  "SttlmMtd": {"var": "temp_data.settlement_method"}
                }
              }
            }
          ]
        }
      }
    }
  ]
}
```

#### Party Information Mapping

```json
{
  "id": "map_credit_transfer_transaction_info",
  "function": {
    "name": "map",
    "input": {
      "mappings": [
        {
          "path": "data.Document.FIToFICstmrCdtTrf.CdtTrfTxInf",
          "logic": {
            "Dbtr": {
              "Nm": {
                "if": [
                  {"var": "data.SwiftMT.fields.50.K.lines"},
                  {
                    "if": [
                      {">": [{"length": {"var": "data.SwiftMT.fields.50.K.lines"}}, 1]},
                      {"var": "data.SwiftMT.fields.50.K.lines.1"},
                      {"var": "data.SwiftMT.fields.50.K.lines.0"}
                    ]
                  },
                  {
                    "if": [
                      {"var": "data.SwiftMT.fields.50A.name_and_address"},
                      {"var": "data.SwiftMT.fields.50A.name_and_address.0"},
                      "NOTPROVIDED"
                    ]
                  }
                ]
              }
            },
            "DbtrAcct": {
              "Id": {
                "Othr": {
                  "Id": {
                    "if": [
                      {"var": "data.SwiftMT.fields.50.K.lines"},
                      {
                        "if": [
                          {"starts_with": [{"var": "data.SwiftMT.fields.50.K.lines.0"}, "50K:/"]},
                          {"substr": [{"var": "data.SwiftMT.fields.50.K.lines.0"}, 5]},
                          "NOTPROVIDED"
                        ]
                      },
                      {
                        "if": [
                          {"var": "data.SwiftMT.fields.50A.account"},
                          {"var": "data.SwiftMT.fields.50A.account"},
                          "NOTPROVIDED"
                        ]
                      }
                    ]
                  }
                }
              }
            }
          }
        }
      ]
    }
  }
}
```

### Settlement Method Mapping

The CBPR+ 4-table decision logic for fields 53a/54a/55a determines settlement method:

```json
{
  "path": "temp_data.settlement_method",
  "logic": {
    "if": [
      // No correspondent fields present
      {"and": [
        {"!": {"var": "data.SwiftMT.fields.53A"}},
        {"!": {"var": "data.SwiftMT.fields.53B"}},
        {"!": {"var": "data.SwiftMT.fields.53D"}},
        {"!": {"var": "data.SwiftMT.fields.54A"}}
      ]},
      "INDA",  // Indirect settlement
      
      // Field 53B with party identifier
      {
        "if": [
          {"and": [
            {"var": "data.SwiftMT.fields.53B.party_identifier"},
            {"starts_with": [{"var": "data.SwiftMT.fields.53B.party_identifier"}, "/"]}
          ]},
          {
            "if": [
              {"starts_with": [{"var": "data.SwiftMT.fields.53B.party_identifier"}, "/C"]},
              "INGA",  // Indirect with guarantee
              "INDA"   // Indirect settlement
            ]
          },
          
          // Field 53A BIC comparison
          {
            "if": [
              {"and": [
                {"var": "data.SwiftMT.fields.53A.bic.raw"},
                {"or": [
                  // BIC matches sender
                  {"==": [
                    {"substr": [{"var": "data.SwiftMT.fields.53A.bic.raw"}, 0, 6]},
                    {"substr": [{"var": "temp_data.Sender"}, 0, 6]}
                  ]},
                  // BIC matches receiver
                  {"==": [
                    {"substr": [{"var": "data.SwiftMT.fields.53A.bic.raw"}, 0, 6]},
                    {"substr": [{"var": "temp_data.Receiver"}, 0, 6]}
                  ]}
                ]}
              ]},
              "INDA",  // Indirect settlement
              "COVE"   // Cover payment
            ]
          }
        ]
      }
    ]
  }
}
```

---

## Reverse Mapping (ISO 20022 → MT)

### pacs.008 to MT103 Mapping

Example from `workflows/reverse/pacs008/field-mapping.json`:

#### Header Construction

```json
{
  "id": "construct_swift_headers",
  "name": "Construct SWIFT Headers",
  "function": {
    "name": "map",
    "input": {
      "mappings": [
        {
          "path": "data.SwiftMT",
          "logic": {
            "message_type": "103",
            "basic_header": {
              "application_id": "F",
              "service_id": "01",
              "sender_bic": {
                "raw": {
                  "if": [
                    {"var": "temp_data.Sender"},
                    {"substr": [{"var": "temp_data.Sender"}, 0, 8]},
                    null
                  ]
                }
              },
              "logical_terminal": {
                "if": [
                  {"var": "temp_data.Sender"},
                  {"cat": [{"var": "temp_data.Sender"}, "AXXX"]},
                  null
                ]
              },
              "session_number": "0000",
              "sequence_number": "000000"
            },
            "application_header": {
              "direction": "I",
              "message_type": "103",
              "priority": "N",
              "receiver_bic": {
                "raw": {
                  "if": [
                    {"var": "temp_data.Receiver"},
                    {"substr": [{"var": "temp_data.Receiver"}, 0, 8]},
                    null
                  ]
                }
              }
            }
          }
        }
      ]
    }
  }
}
```

#### Field Mapping

```json
{
  "id": "map_mandatory_fields",
  "name": "Map Mandatory MT103 Fields",
  "function": {
    "name": "map",
    "input": {
      "mappings": [
        {
          "path": "data.SwiftMT.fields",
          "logic": {
            "20": {
              "value": {
                "if": [
                  {"var": "data.ISO20022_MX.document.CdtTrfTxInf.PmtId.InstrId"},
                  {"var": "data.ISO20022_MX.document.CdtTrfTxInf.PmtId.InstrId"},
                  "NOTPROVIDED"
                ]
              }
            },
            "23B": {
              "value": "CRED"
            },
            "32A": {
              "value_date": {
                "if": [
                  {"var": "data.ISO20022_MX.document.CdtTrfTxInf.IntrBkSttlmDt"},
                  {"substr": [{"var": "data.ISO20022_MX.document.CdtTrfTxInf.IntrBkSttlmDt"}, 0, 10]},
                  {"format_date": [{"now": []}, "yyyy-MM-dd"]}
                ]
              },
              "currency": {
                "if": [
                  {"var": "data.ISO20022_MX.document.CdtTrfTxInf.IntrBkSttlmAmt.@Ccy"},
                  {"var": "data.ISO20022_MX.document.CdtTrfTxInf.IntrBkSttlmAmt.@Ccy"},
                  "USD"
                ]
              },
              "amount": {
                "if": [
                  {"var": "data.ISO20022_MX.document.CdtTrfTxInf.IntrBkSttlmAmt.$value"},
                  {"var": "data.ISO20022_MX.document.CdtTrfTxInf.IntrBkSttlmAmt.$value"},
                  0.0
                ]
              }
            }
          }
        }
      ]
    }
  }
}
```

---

## Transformation Functions

### Built-in Functions

#### String Operations

```json
{
  "cat": [                    // Concatenate strings
    {"var": "field1"},
    "-",
    {"var": "field2"}
  ]
}

{
  "substr": [                 // Substring extraction
    {"var": "field"},
    0,                        // Start position
    8                         // Length
  ]
}

{
  "starts_with": [            // Check string prefix
    {"var": "field"},
    "/C"
  ]
}
```

#### Date/Time Functions

```json
{
  "format_date": [            // Format current date/time
    {"now": []},
    "yyyy-MM-dd"              // Format pattern
  ]
}

{
  "format_date": [
    {"now": []},
    "HH:mm:ss"
  ]
}
```

#### Conditional Logic

```json
{
  "if": [                     // If-then-else
    condition,
    then_value,
    else_value
  ]
}

{
  "and": [                    // Logical AND
    condition1,
    condition2,
    condition3
  ]
}

{
  "or": [                     // Logical OR
    condition1,
    condition2
  ]
}

{
  "!": condition              // Logical NOT
}
```

#### Comparison Operations

```json
{
  "==": [value1, value2]      // Equality
}

{
  ">": [value1, value2]       // Greater than
}

{
  "in": [value, array]        // Check if value in array
}

{
  "matches": [                // Regular expression match
    {"var": "field"},
    "^[A-Z]{3}$"
  ]
}
```

#### Array Operations

```json
{
  "length": {"var": "array"}  // Array length
}

{
  "merge": [                  // Merge arrays
    array1,
    array2
  ]
}
```

### Special Functions

#### PublishMX Function

Generates XML output for ISO 20022 messages:

```json
{
  "id": "generate_document_xml",
  "name": "Generate Document XML",
  "condition": {"==": [{"var": "SwiftMT.method"}, "normal"]},
  "function": {
    "name": "PublishMX",
    "input": {
      "input_field_name": "Document",
      "output_field_name": "document_xml",
      "source_format": "MT103.Document"
    }
  }
}
```

#### Validate Function

Validates business rules:

```json
{
  "id": "validate_preconditions",
  "name": "Validate Preconditions",
  "function": {
    "name": "validate",
    "input": {
      "rules": [
        {
          "path": "data.ISO20022_MX.document.CdtTrfTxInf.IntrBkSttlmAmt.@Ccy",
          "logic": {
            "!": {
              "in": [
                {"var": "data.ISO20022_MX.document.CdtTrfTxInf.IntrBkSttlmAmt.@Ccy"},
                ["XAU", "XAG", "XPD", "XPT"]
              ]
            }
          },
          "message": "T20054: Commodities currencies not allowed in Field 32A"
        }
      ]
    }
  }
}
```

---

## Business Rules

### Validation Rules

#### Precondition Validation

Example from `workflows/forward/MT103/precondition.json`:

```json
{
  "id": "mt103-precondition",
  "name": "MT103 Precondition Validation",
  "tasks": [
    {
      "id": "validate_mandatory_fields",
      "name": "Validate Mandatory Fields",
      "function": {
        "name": "validate",
        "input": {
          "rules": [
            {
              "path": "data.SwiftMT.fields.20",
              "logic": {"var": "data.SwiftMT.fields.20"},
              "message": "Field 20 (Transaction Reference) is mandatory"
            },
            {
              "path": "data.SwiftMT.fields.32A",
              "logic": {"var": "data.SwiftMT.fields.32A"},
              "message": "Field 32A (Value Date, Currency, Amount) is mandatory"
            },
            {
              "path": "data.SwiftMT.fields.50",
              "logic": {
                "or": [
                  {"var": "data.SwiftMT.fields.50K"},
                  {"var": "data.SwiftMT.fields.50A"},
                  {"var": "data.SwiftMT.fields.50F"}
                ]
              },
              "message": "Field 50 (Ordering Customer) is mandatory"
            }
          ]
        }
      }
    }
  ]
}
```

### Conditional Logic

#### Charge Bearer Mapping

```json
{
  "ChrgBr": {
    "if": [
      {"var": "data.SwiftMT.fields.71A.value"},
      {
        "if": [
          {"==": [{"var": "data.SwiftMT.fields.71A.value"}, "BEN"]},
          "CRED",
          {
            "if": [
              {"==": [{"var": "data.SwiftMT.fields.71A.value"}, "OUR"]},
              "DEBT",
              "SHAR"
            ]
          }
        ]
      },
      "SHAR"  // Default value
    ]
  }
}
```

#### Service Level Mapping

```json
{
  "Prty": {
    "if": [
      {"==": [{"var": "data.SwiftMT.fields.23B.value"}, "URGP"]},
      "URGT",
      "NORM"
    ]
  }
}
```

---

## Advanced Mapping Patterns

### Dynamic Field Selection

Select fields based on availability:

```json
{
  "Nm": {
    "if": [
      {"var": "data.SwiftMT.fields.50.K.lines"},
      // If 50K exists, use it
      {
        "if": [
          {">": [{"length": {"var": "data.SwiftMT.fields.50.K.lines"}}, 1]},
          {"var": "data.SwiftMT.fields.50.K.lines.1"},
          {"var": "data.SwiftMT.fields.50.K.lines.0"}
        ]
      },
      // Else check 50A
      {
        "if": [
          {"var": "data.SwiftMT.fields.50A.name_and_address"},
          {"var": "data.SwiftMT.fields.50A.name_and_address.0"},
          // Else check 50F
          {
            "if": [
              {"var": "data.SwiftMT.fields.50F.name_and_address"},
              {"var": "data.SwiftMT.fields.50F.name_and_address.0"},
              "NOTPROVIDED"
            ]
          }
        ]
      }
    ]
  }
}
```

### Complex Conditional Mapping

Example of instruction for creditor agent:

```json
{
  "InstrForCdtrAgt": {
    "if": [
      {"or": [
        {"var": "data.SwiftMT.fields.23E"},
        {"var": "data.SwiftMT.fields.72.acc_instructions"}
      ]},
      {
        "merge": [
          {
            "if": [
              {"var": "data.SwiftMT.fields.23E"},
              [{
                "Cd": {
                  "if": [
                    {"in": [{"var": "data.SwiftMT.fields.23E"}, ["CHQB", "HOLD", "PHOB", "TELB"]]},
                    {"var": "data.SwiftMT.fields.23E"},
                    ""
                  ]
                }
              }],
              []
            ]
          },
          {
            "InstrInf": {
              "if": [
                {"var": "data.SwiftMT.fields.72.lines"},
                {"var": "data.SwiftMT.fields.72.lines"},
                []
              ]
            }
          }
        ]
      },
      []
    ]
  }
}
```

### Array Processing

Processing charges information:

```json
{
  "ChrgsInf": {
    "if": [
      {"or": [
        {"var": "data.SwiftMT.fields.71F"},
        {"var": "data.SwiftMT.fields.71G"}
      ]},
      {
        "merge": [
          // Sender charges (71F)
          {
            "if": [
              {"var": "data.SwiftMT.fields.71F"},
              [{
                "Amt": {
                  "@Ccy": {"var": "data.SwiftMT.fields.71F.currency"},
                  "$value": {"var": "data.SwiftMT.fields.71F.amount"}
                },
                "Agt": {
                  "FinInstnId": {
                    "BICFI": {"var": "temp_data.Sender"}
                  }
                }
              }],
              []
            ]
          },
          // Receiver charges (71G)
          {
            "if": [
              {"var": "data.SwiftMT.fields.71G"},
              [{
                "Amt": {
                  "@Ccy": {"var": "data.SwiftMT.fields.71G.currency"},
                  "$value": {"var": "data.SwiftMT.fields.71G.amount"}
                },
                "Agt": {
                  "FinInstnId": {
                    "BICFI": {"var": "data.SwiftMT.fields.57A.bic.raw"}
                  }
                }
              }],
              []
            ]
          }
        ]
      },
      []
    ]
  }
}
```

---

## Troubleshooting

### Common Issues

#### 1. Field Not Found

**Problem**: Variable returns null/undefined

**Solution**: Use conditional logic with fallback:
```json
{
  "if": [
    {"var": "data.SwiftMT.fields.20.value"},
    {"var": "data.SwiftMT.fields.20.value"},
    "NOTPROVIDED"  // Fallback value
  ]
}
```

#### 2. Array Access

**Problem**: Accessing specific array elements

**Solution**: Check array length first:
```json
{
  "if": [
    {">": [{"length": {"var": "data.SwiftMT.fields.50.K.lines"}}, 1]},
    {"var": "data.SwiftMT.fields.50.K.lines.1"},  // Second line
    {"var": "data.SwiftMT.fields.50.K.lines.0"}   // First line
  ]
}
```

#### 3. Complex Conditions

**Problem**: Multiple conditions need evaluation

**Solution**: Use nested if statements or and/or operators:
```json
{
  "and": [
    {"var": "data.SwiftMT.fields.53A"},
    {"!": {"var": "data.SwiftMT.fields.54A"}},
    {"==": [{"var": "SwiftMT.method"}, "normal"]}
  ]
}
```

### Debugging Workflows

#### Enable Debug Logging

```bash
# Enable detailed workflow logging
RUST_LOG=debug,dataflow_rs=trace cargo run
```

#### Test Individual Workflows

Use the workflow test endpoint to validate specific workflows:

```bash
curl -X POST http://localhost:3000/test-workflow \
  -H "Content-Type: application/json" \
  -d '{
    "workflow": "MT103/document-mapping.json",
    "data": {
      "SwiftMT": {
        "fields": {
          "20": {"value": "TEST123"},
          "32A": {
            "value_date": "2024-01-15",
            "currency": "USD",
            "amount": 1000.50
          }
        }
      }
    }
  }'
```

### Performance Optimization

#### 1. Minimize Nested Conditions

Instead of deep nesting:
```json
{
  "if": [
    condition1,
    {
      "if": [
        condition2,
        {
          "if": [
            condition3,
            value3,
            value4
          ]
        },
        value2
      ]
    },
    value1
  ]
}
```

Use early returns:
```json
{
  "if": [
    {"!": condition1},
    value1,
    {
      "if": [
        {"!": condition2},
        value2,
        {
          "if": [
            condition3,
            value3,
            value4
          ]
        }
      ]
    }
  ]
}
```

#### 2. Cache Common Values

Store frequently used values in temp_data:
```json
{
  "path": "temp_data.sender_bic",
  "logic": {"var": "data.SwiftMT.basic_header.sender_bic.raw"}
}
```

Then reference throughout workflow:
```json
{
  "var": "temp_data.sender_bic"
}
```

---

## Next Steps

1. **[Architecture Guide](architecture.md)** - Understand the technical architecture
2. **[Message Formats](message-formats.md)** - Complete list of supported message types
3. **[Installation Guide](installation.md)** - Setup and configuration instructions

---

*Last updated: January 2024*