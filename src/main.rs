use axum::{
    extract::State,
    http::{header, StatusCode},
    response::Response,
    routing::{get, post},
    Router,
};
use dataflow_rs::engine::message::Message;
use dataflow_rs::{Engine, Workflow};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

mod parser;
use parser::ParserFunction;

mod publish;
use publish::PublishFunction;

// Application state
#[derive(Clone)]
struct AppState {
    engine: Arc<Mutex<Engine>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the dataflow engine
    let mut engine = Engine::new();

    // Register custom parse function
    engine.register_task_function("parse".to_string(), Box::new(ParserFunction));
    engine.register_task_function("publish".to_string(), Box::new(PublishFunction));

    // Add sample workflows
    setup_workflows(&mut engine).await?;

    // Create application state
    let state = AppState {
        engine: Arc::new(Mutex::new(engine)),
    };

    // Build the router
    let app = Router::new()
        .route("/reframe", post(process_data))
        .route("/health", get(health_check))
        .layer(ServiceBuilder::new().layer(CorsLayer::permissive()))
        .with_state(state);

    println!("🚀 Server starting on http://0.0.0.0:3000");

    // Start the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn setup_workflows(engine: &mut Engine) -> anyhow::Result<()> {
    let workflow1_str = r#"
    {
        "id": "mt103_to_pacs008_mapper",
        "name": "MT103 to pacs.008.001.13 Mapper",
        "tasks": [
            {
                "id": "parse_mt103",
                "name": "Parse MT103 Message",
                "function": {
                    "name": "parse",
                    "input": { 
                        "format": "SwiftMT",
                        "input_field_name": "payload",
                        "output_field_name": "SwiftMT"
                    }
                }
            },

            {
                "id": "map_group_header",
                "name": "Map Group Header",
                "function": {
                    "name": "map",
                    "input": {
                        "mappings": [
                            {
                                "path": "data.MX.FIToFICstmrCdtTrf.GrpHdr.MsgId",
                                "logic": { "var": "data.SwiftMT.20" }
                            },
                            {
                                "path": "data.MX.FIToFICstmrCdtTrf.GrpHdr.CreDtTm",
                                "logic": {
                                    "if": [
                                        { "var": "metadata.timestamp" },
                                        { "var": "metadata.timestamp" },
                                        "2024-01-01T00:00:00Z"
                                    ]
                                }
                            },
                            {
                                "path": "data.MX.FIToFICstmrCdtTrf.GrpHdr.NbOfTxs",
                                "logic": "1"
                            },
                            {
                                "path": "data.MX.FIToFICstmrCdtTrf.GrpHdr.CtrlSum",
                                "logic": {
                                    "if": [
                                        { "var": "data.SwiftMT.32A_amount" },
                                        { "var": "data.SwiftMT.32A_amount" },
                                        0.0
                                    ]
                                }
                            },
                            {
                                "path": "data.MX.FIToFICstmrCdtTrf.GrpHdr.InitgPty.Nm",
                                "logic": "Reframe Processing System"
                            },
                            {
                                "path": "data.MX.FIToFICstmrCdtTrf.GrpHdr.SttlmInf.SttlmMtd",
                                "logic": "CLRG"
                            }
                        ]
                    }
                }
            },
            {
                "id": "map_credit_transfer_info",
                "name": "Map Credit Transfer Transaction Information",
                "function": {
                    "name": "map",
                    "input": {
                        "mappings": [
                            {
                                "path": "data.MX.FIToFICstmrCdtTrf.CdtTrfTxInf.0.PmtId.InstrId",
                                "logic": { "var": "data.SwiftMT.20" }
                            },
                            {
                                "path": "data.MX.FIToFICstmrCdtTrf.CdtTrfTxInf.0.PmtId.EndToEndId",
                                "logic": { "var": "data.SwiftMT.20" }
                            },
                            {
                                "path": "data.MX.FIToFICstmrCdtTrf.CdtTrfTxInf.0.PmtId.UETR",
                                "logic": { "var": "data.SwiftMT.20" }
                            },
                            {
                                "path": "data.MX.FIToFICstmrCdtTrf.CdtTrfTxInf.0.IntrBkSttlmAmt.@Ccy",
                                "logic": {
                                    "if": [
                                        { "var": "data.SwiftMT.32A_currency" },
                                        { "var": "data.SwiftMT.32A_currency" },
                                        "USD"
                                    ]
                                }
                            },
                            {
                                "path": "data.MX.FIToFICstmrCdtTrf.CdtTrfTxInf.0.IntrBkSttlmAmt.$value",
                                "logic": {
                                    "if": [
                                        { "var": "data.SwiftMT.32A_amount" },
                                        { "var": "data.SwiftMT.32A_amount" },
                                        0.0
                                    ]
                                }
                            },
                            {
                                "path": "data.MX.FIToFICstmrCdtTrf.CdtTrfTxInf.0.IntrBkSttlmDt",
                                "logic": {
                                    "if": [
                                        { "var": "data.SwiftMT.32A_date" },
                                        { "var": "data.SwiftMT.32A_date" },
                                        "2024-01-01"
                                    ]
                                }
                            }
                        ]
                    }
                }
            },
            {
                "id": "map_charge_bearer",
                "name": "Map Charge Bearer",
                "function": {
                    "name": "map",
                    "input": {
                        "mappings": [
                            {
                                "path": "data.MX.FIToFICstmrCdtTrf.CdtTrfTxInf.0.ChrgBr",
                                "logic": {
                                    "if": [
                                        { "==": [{ "var": "data.SwiftMT.71A" }, "OUR"] },
                                        "DEBT",
                                        {
                                            "if": [
                                                { "==": [{ "var": "data.SwiftMT.71A" }, "BEN"] },
                                                "CRED",
                                                "SHAR"
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
                "id": "map_instructing_agent",
                "name": "Map Instructing Agent",
                "function": {
                    "name": "map",
                    "input": {
                        "mappings": [
                            {
                                "path": "data.MX.FIToFICstmrCdtTrf.CdtTrfTxInf.0.InstgAgt.FinInstnId.BICFI",
                                "logic": { "var": "data.SwiftMT.52A" }
                            },
                            {
                                "path": "data.MX.FIToFICstmrCdtTrf.CdtTrfTxInf.0.InstdAgt.FinInstnId.BICFI",
                                "logic": { "var": "data.SwiftMT.57A" }
                            }
                        ]
                    }
                }
            },
            {
                "id": "map_debtor_info",
                "name": "Map Debtor Information",
                "function": {
                    "name": "map",
                    "input": {
                        "mappings": [
                            {
                                "path": "data.MX.FIToFICstmrCdtTrf.CdtTrfTxInf.0.Dbtr.Nm",
                                "logic": { "var": "data.SwiftMT.50K" }
                            },
                            {
                                "path": "data.MX.FIToFICstmrCdtTrf.CdtTrfTxInf.0.DbtrAcct.Id.IBAN",
                                "logic": { "var": "data.SwiftMT.50A_account" }
                            },
                            {
                                "path": "data.MX.FIToFICstmrCdtTrf.CdtTrfTxInf.0.DbtrAgt.FinInstnId.BICFI",
                                "logic": { "var": "data.SwiftMT.52A" }
                            }
                        ]
                    }
                }
            },
            {
                "id": "map_creditor_info",
                "name": "Map Creditor Information",
                "function": {
                    "name": "map",
                    "input": {
                        "mappings": [
                            {
                                "path": "data.MX.FIToFICstmrCdtTrf.CdtTrfTxInf.0.Cdtr.Nm",
                                "logic": { "var": "data.SwiftMT.59" }
                            },
                            {
                                "path": "data.MX.FIToFICstmrCdtTrf.CdtTrfTxInf.0.CdtrAcct.Id.IBAN",
                                "logic": { "var": "data.SwiftMT.59_account" }
                            },
                            {
                                "path": "data.MX.FIToFICstmrCdtTrf.CdtTrfTxInf.0.CdtrAgt.FinInstnId.BICFI",
                                "logic": { "var": "data.SwiftMT.57A" }
                            }
                        ]
                    }
                }
            },
            {
                "id": "map_remittance_info",
                "name": "Map Remittance Information",
                "function": {
                    "name": "map",
                    "input": {
                        "mappings": [
                            {
                                "path": "data.MX.FIToFICstmrCdtTrf.CdtTrfTxInf.0.RmtInf.Ustrd.0",
                                "logic": { "var": "data.SwiftMT.70" }
                            }
                        ]
                    }
                }
            },
            {
                "id": "map_intermediary_agents",
                "name": "Map Intermediary Agents",
                "function": {
                    "name": "map",
                    "input": {
                        "mappings": [
                            {
                                "path": "data.MX.FIToFICstmrCdtTrf.CdtTrfTxInf.0.IntrmyAgt1.FinInstnId.BICFI",
                                "logic": { "var": "data.SwiftMT.56A" }
                            },
                            {
                                "path": "data.MX.FIToFICstmrCdtTrf.CdtTrfTxInf.0.IntrmyAgt2.FinInstnId.BICFI",
                                "logic": { "var": "data.SwiftMT.56C" }
                            }
                        ]
                    }
                }
            },
            {
                "id": "map_regulatory_reporting",
                "name": "Map Regulatory Reporting",
                "function": {
                    "name": "map",
                    "input": {
                        "mappings": [
                            {
                                "path": "data.MX.FIToFICstmrCdtTrf.CdtTrfTxInf.0.RgltryRptg.0.Dtls.0.Cd",
                                "logic": { "var": "data.SwiftMT.79" }
                            },
                            {
                                "path": "data.MX.FIToFICstmrCdtTrf.CdtTrfTxInf.0.SplmtryData.0.PlcAndNm",
                                "logic": "MT103_Original"
                            },
                            {
                                "path": "data.MX.FIToFICstmrCdtTrf.CdtTrfTxInf.0.SplmtryData.0.Envlp",
                                "logic": {}
                            }
                        ]
                    }
                }
            },
            {
                "id": "publish_mx_message",
                "name": "Publish MX Message",
                "function": {
                    "name": "publish",
                    "input": {
                        "input_field_name": "MX",
                        "output_format": "pacs.008.001.13"
                    }
                }
            }
        ]
    }
    "#;

    // Parse and add workflows to the engine
    let workflow1 = Workflow::from_json(workflow1_str)?;
    engine.add_workflow(&workflow1);

    println!("✅ MT103 to pacs.008.001.13 mapping workflow configured successfully");
    Ok(())
}

async fn process_data(
    State(state): State<AppState>,
    payload: String,
) -> Result<Response<String>, StatusCode> {
    let engine = state.engine.lock().await;

    // Create a message with the payload
    let mut message = Message::new(&Value::String(payload));

    // Process the message through workflows
    match engine.process_message(&mut message).await {
        Ok(_) => {
            // Check if we have a result and if it looks like XML
            if let Some(result_value) = message.data.get("result") {
                let result_string = result_value.as_str().unwrap_or("");
                Response::builder()
                    .header(header::CONTENT_TYPE, "application/xml")
                    .body(result_string.to_string())
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
            } else {
                // Return the full message data as JSON if no specific result
                Response::builder()
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(serde_json::to_string(&message.data).unwrap_or_else(|_| "{}".to_string()))
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
        Err(e) => {
            let error_response =
                serde_json::json!({"error": format!("Error processing data: {:?}", e)});
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header(header::CONTENT_TYPE, "application/json")
                .body(error_response.to_string())
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Health check endpoint
async fn health_check() -> Result<Response<String>, StatusCode> {
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(r#"{"status":"healthy","service":"reframe-api","version":"0.1.0"}"#.to_string())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(response)
}
