use mx_message::document::*;
use mx_message::header::*;
use quick_xml::se::to_string as xml_to_string;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, error};

/// ISO 20022 Message envelope structure
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "Message")]
pub struct Iso20022Message {
    #[serde(rename = "AppHdr")]
    pub app_hdr: Value,
    #[serde(rename = "Document")]
    pub document: Value,
}

/// Generate MX message XML from JSON data
pub fn generate_mx_from_json(
    message_type: &str,
    json_data: &Value,
) -> Result<String, Box<dyn std::error::Error>> {
    debug!("Generating {} from JSON data", message_type);

    // Extract AppHdr and Document from the generated data
    let app_hdr = json_data
        .get("AppHdr")
        .ok_or("Missing AppHdr in generated data")?;
    let document = json_data
        .get("Document")
        .ok_or("Missing Document in generated data")?;

    // Generate XML based on message type
    let xml_string = match message_type {
        "pacs.008" => generate_pacs008_xml(app_hdr, document)?,
        "pacs.009" => generate_pacs009_xml(app_hdr, document)?,
        "pacs.002" => generate_pacs002_xml(app_hdr, document)?,
        "pacs.003" => generate_pacs003_xml(app_hdr, document)?,
        "pacs.004" => generate_pacs004_xml(app_hdr, document)?,
        "camt.052" => generate_camt052_xml(app_hdr, document)?,
        "camt.053" => generate_camt053_xml(app_hdr, document)?,
        "camt.054" => generate_camt054_xml(app_hdr, document)?,
        "camt.056" => generate_camt056_xml(app_hdr, document)?,
        "camt.029" => generate_camt029_xml(app_hdr, document)?,
        "camt.025" => generate_camt025_xml(app_hdr, document)?,
        "camt.057" => generate_camt057_xml(app_hdr, document)?,
        "camt.060" => generate_camt060_xml(app_hdr, document)?,
        "camt.107" => generate_camt107_xml(app_hdr, document)?,
        "camt.108" => generate_camt108_xml(app_hdr, document)?,
        "camt.109" => generate_camt109_xml(app_hdr, document)?,
        "pain.001" => generate_pain001_xml(app_hdr, document)?,
        "pain.008" => generate_pain008_xml(app_hdr, document)?,
        "pain.002" => generate_pain002_xml(app_hdr, document)?,
        _ => {
            error!("Unsupported MX message type: {}", message_type);
            return Err(format!("Unsupported MX message type: {}", message_type).into());
        }
    };

    debug!("Successfully generated {} message", message_type);
    Ok(xml_string)
}

fn generate_pacs008_xml(app_hdr: &Value, document: &Value) -> Result<String, Box<dyn std::error::Error>> {
    // Parse header
    let header: bah_pacs_008_001_08::BusinessApplicationHeaderV02 = 
        serde_json::from_value(app_hdr.clone())?;
    
    // Parse document
    let doc_content = document
        .get("FIToFICstmrCdtTrf")
        .ok_or("Missing FIToFICstmrCdtTrf in Document")?;
    let doc: pacs_008_001_08::FIToFICustomerCreditTransferV08 = 
        serde_json::from_value(doc_content.clone())?;
    
    // Create complete message
    let message = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<Message xmlns="urn:iso:std:iso:20022:tech:xsd:head.001.001.02">
{}
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:pacs.008.001.08">
<FIToFICstmrCdtTrf>{}</FIToFICstmrCdtTrf>
</Document>
</Message>"#,
        xml_to_string(&header)?,
        xml_to_string(&doc)?
    );
    
    Ok(message)
}

fn generate_pacs009_xml(app_hdr: &Value, document: &Value) -> Result<String, Box<dyn std::error::Error>> {
    let header: bah_pacs_009_001_08::BusinessApplicationHeaderV02 = 
        serde_json::from_value(app_hdr.clone())?;
    
    let doc_content = document
        .get("FICdtTrf")
        .ok_or("Missing FICdtTrf in Document")?;
    let doc: pacs_009_001_08::FinancialInstitutionCreditTransferV08 = 
        serde_json::from_value(doc_content.clone())?;
    
    let message = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<Message xmlns="urn:iso:std:iso:20022:tech:xsd:head.001.001.02">
{}
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:pacs.009.001.08">
<FICdtTrf>{}</FICdtTrf>
</Document>
</Message>"#,
        xml_to_string(&header)?,
        xml_to_string(&doc)?
    );
    
    Ok(message)
}

fn generate_pacs002_xml(app_hdr: &Value, document: &Value) -> Result<String, Box<dyn std::error::Error>> {
    let header: bah_pacs_002_001_10::BusinessApplicationHeaderV02 = 
        serde_json::from_value(app_hdr.clone())?;
    
    let doc_content = document
        .get("FIToFIPmtStsRpt")
        .ok_or("Missing FIToFIPmtStsRpt in Document")?;
    let doc: pacs_002_001_10::FIToFIPaymentStatusReportV10 = 
        serde_json::from_value(doc_content.clone())?;
    
    let message = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<Message xmlns="urn:iso:std:iso:20022:tech:xsd:head.001.001.02">
{}
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:pacs.002.001.10">
<FIToFIPmtStsRpt>{}</FIToFIPmtStsRpt>
</Document>
</Message>"#,
        xml_to_string(&header)?,
        xml_to_string(&doc)?
    );
    
    Ok(message)
}

fn generate_pacs003_xml(_app_hdr: &Value, _document: &Value) -> Result<String, Box<dyn std::error::Error>> {
    // pacs.003 is not in the mx_message library yet, return JSON for now
    Err("pacs.003 XML generation not yet implemented".into())
}

fn generate_pacs004_xml(app_hdr: &Value, document: &Value) -> Result<String, Box<dyn std::error::Error>> {
    let header: bah_pacs_004_001_09::BusinessApplicationHeaderV02 = 
        serde_json::from_value(app_hdr.clone())?;
    
    let doc_content = document
        .get("PmtRtr")
        .ok_or("Missing PmtRtr in Document")?;
    let doc: pacs_004_001_09::PaymentReturnV09 = 
        serde_json::from_value(doc_content.clone())?;
    
    let message = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<Message xmlns="urn:iso:std:iso:20022:tech:xsd:head.001.001.02">
{}
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:pacs.004.001.09">
<PmtRtr>{}</PmtRtr>
</Document>
</Message>"#,
        xml_to_string(&header)?,
        xml_to_string(&doc)?
    );
    
    Ok(message)
}

fn generate_camt052_xml(app_hdr: &Value, document: &Value) -> Result<String, Box<dyn std::error::Error>> {
    let header: bah_camt_052_001_08::BusinessApplicationHeaderV02 = 
        serde_json::from_value(app_hdr.clone())?;
    
    let doc_content = document
        .get("BkToCstmrAcctRpt")
        .ok_or("Missing BkToCstmrAcctRpt in Document")?;
    let doc: camt_052_001_08::BankToCustomerAccountReportV08 = 
        serde_json::from_value(doc_content.clone())?;
    
    let message = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<Message xmlns="urn:iso:std:iso:20022:tech:xsd:head.001.001.02">
{}
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:camt.052.001.08">
<BkToCstmrAcctRpt>{}</BkToCstmrAcctRpt>
</Document>
</Message>"#,
        xml_to_string(&header)?,
        xml_to_string(&doc)?
    );
    
    Ok(message)
}

fn generate_camt053_xml(app_hdr: &Value, document: &Value) -> Result<String, Box<dyn std::error::Error>> {
    let header: bah_camt_053_001_08::BusinessApplicationHeaderV02 = 
        serde_json::from_value(app_hdr.clone())?;
    
    let doc_content = document
        .get("BkToCstmrStmt")
        .ok_or("Missing BkToCstmrStmt in Document")?;
    let doc: camt_053_001_08::BankToCustomerStatementV08 = 
        serde_json::from_value(doc_content.clone())?;
    
    let message = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<Message xmlns="urn:iso:std:iso:20022:tech:xsd:head.001.001.02">
{}
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:camt.053.001.08">
<BkToCstmrStmt>{}</BkToCstmrStmt>
</Document>
</Message>"#,
        xml_to_string(&header)?,
        xml_to_string(&doc)?
    );
    
    Ok(message)
}

fn generate_camt054_xml(_app_hdr: &Value, _document: &Value) -> Result<String, Box<dyn std::error::Error>> {
    // camt.054 is not in the mx_message library yet, return JSON for now
    Err("camt.054 XML generation not yet implemented".into())
}

fn generate_camt056_xml(_app_hdr: &Value, _document: &Value) -> Result<String, Box<dyn std::error::Error>> {
    // camt.056 is not in the mx_message library yet, return JSON for now
    Err("camt.056 XML generation not yet implemented".into())
}

fn generate_camt029_xml(_app_hdr: &Value, _document: &Value) -> Result<String, Box<dyn std::error::Error>> {
    // camt.029 is not in the mx_message library yet, return JSON for now
    Err("camt.029 XML generation not yet implemented".into())
}

fn generate_camt025_xml(_app_hdr: &Value, _document: &Value) -> Result<String, Box<dyn std::error::Error>> {
    // camt.025 is not in the mx_message library yet, return JSON for now
    Err("camt.025 XML generation not yet implemented".into())
}

fn generate_camt057_xml(_app_hdr: &Value, _document: &Value) -> Result<String, Box<dyn std::error::Error>> {
    // camt.057 is not in the mx_message library yet, return JSON for now
    Err("camt.057 XML generation not yet implemented".into())
}

fn generate_camt060_xml(_app_hdr: &Value, _document: &Value) -> Result<String, Box<dyn std::error::Error>> {
    // camt.060 is not in the mx_message library yet, return JSON for now
    Err("camt.060 XML generation not yet implemented".into())
}

fn generate_camt107_xml(app_hdr: &Value, document: &Value) -> Result<String, Box<dyn std::error::Error>> {
    let header: bah_camt_107_001_01::BusinessApplicationHeaderV02 = 
        serde_json::from_value(app_hdr.clone())?;
    
    let doc_content = document
        .get("ChqPrsntmntNtfctn")
        .ok_or("Missing ChqPrsntmntNtfctn in Document")?;
    let doc: camt_107_001_01::ChequePresentmentNotificationV01 = 
        serde_json::from_value(doc_content.clone())?;
    
    let message = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<Message xmlns="urn:iso:std:iso:20022:tech:xsd:head.001.001.02">
{}
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:camt.107.001.01">
<ChqPrsntmntNtfctn>{}</ChqPrsntmntNtfctn>
</Document>
</Message>"#,
        xml_to_string(&header)?,
        xml_to_string(&doc)?
    );
    
    Ok(message)
}

fn generate_camt108_xml(app_hdr: &Value, document: &Value) -> Result<String, Box<dyn std::error::Error>> {
    let header: bah_camt_108_001_01::BusinessApplicationHeaderV02 = 
        serde_json::from_value(app_hdr.clone())?;
    
    let doc_content = document
        .get("ChqCxlOrStopReq")
        .ok_or("Missing ChqCxlOrStopReq in Document")?;
    let doc: camt_108_001_01::ChequeCancellationOrStopRequestV01 = 
        serde_json::from_value(doc_content.clone())?;
    
    let message = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<Message xmlns="urn:iso:std:iso:20022:tech:xsd:head.001.001.02">
{}
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:camt.108.001.01">
<ChqCxlOrStopReq>{}</ChqCxlOrStopReq>
</Document>
</Message>"#,
        xml_to_string(&header)?,
        xml_to_string(&doc)?
    );
    
    Ok(message)
}

fn generate_camt109_xml(app_hdr: &Value, document: &Value) -> Result<String, Box<dyn std::error::Error>> {
    let header: bah_camt_109_001_01::BusinessApplicationHeaderV02 = 
        serde_json::from_value(app_hdr.clone())?;
    
    let doc_content = document
        .get("ChqCxlOrStopRpt")
        .ok_or("Missing ChqCxlOrStopRpt in Document")?;
    let doc: camt_109_001_01::ChequeCancellationOrStopReportV01 = 
        serde_json::from_value(doc_content.clone())?;
    
    let message = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<Message xmlns="urn:iso:std:iso:20022:tech:xsd:head.001.001.02">
{}
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:camt.109.001.01">
<ChqCxlOrStopRpt>{}</ChqCxlOrStopRpt>
</Document>
</Message>"#,
        xml_to_string(&header)?,
        xml_to_string(&doc)?
    );
    
    Ok(message)
}

fn generate_pain001_xml(_app_hdr: &Value, _document: &Value) -> Result<String, Box<dyn std::error::Error>> {
    // pain.001 is not in the mx_message library yet, return JSON for now
    Err("pain.001 XML generation not yet implemented".into())
}

fn generate_pain008_xml(_app_hdr: &Value, _document: &Value) -> Result<String, Box<dyn std::error::Error>> {
    // pain.008 is not in the mx_message library yet, return JSON for now
    Err("pain.008 XML generation not yet implemented".into())
}

fn generate_pain002_xml(_app_hdr: &Value, _document: &Value) -> Result<String, Box<dyn std::error::Error>> {
    // pain.002 is not in the mx_message library yet, return JSON for now
    Err("pain.002 XML generation not yet implemented".into())
}

/// Format XML string with proper indentation
pub fn format_xml_message(xml_string: &str) -> String {
    // For now, just return as-is. Could add proper XML formatting later
    xml_string.to_string()
}