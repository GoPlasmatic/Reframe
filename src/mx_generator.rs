use mx_message::mx_envelope::MxEnvelope;
use mx_message::document::*;
use mx_message::header::*;
use quick_xml::se::to_string as xml_to_string;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_path_to_error::deserialize;
use tracing::{debug, error};

/// Generic ISO 20022 Message envelope structure for JSON parsing
#[derive(Debug, Serialize, Deserialize)]
pub struct GenericMxEnvelope {
    #[serde(rename = "AppHdr")]
    pub app_hdr: Value,
    #[serde(rename = "Document")]
    pub document: Value,
}

/// Generate pacs.008 XML from parsed JSON
fn generate_pacs008_xml(app_hdr: &Value, document: &Value) -> Result<String, Box<dyn std::error::Error>> {
    debug!("Parsing pacs.008 header");
    let header_str = serde_json::to_string(app_hdr)?;
    let hd = &mut serde_json::Deserializer::from_str(&header_str);
    let header: bah_pacs_008_001_08::BusinessApplicationHeaderV02 = deserialize(hd)
        .map_err(|e| {
            let path = e.path().to_string();
            let inner_err = e.into_inner().to_string();
            format!("Failed to parse pacs.008 header at path '{}': {}", path, inner_err)
        })?;
    
    debug!("Parsing pacs.008 document");
    let doc_content = document
        .get("FIToFICstmrCdtTrf")
        .ok_or_else(|| "Missing FIToFICstmrCdtTrf in Document")?;
    
    // Log the document structure for debugging
    debug!("Document content: {}", serde_json::to_string_pretty(doc_content)?);
    
    let doc_str = serde_json::to_string(doc_content)?;
    let dd = &mut serde_json::Deserializer::from_str(&doc_str);
    let doc: pacs_008_001_08::FIToFICustomerCreditTransferV08 = deserialize(dd)
        .map_err(|e| {
            let path = e.path().to_string();
            let inner_err = e.into_inner().to_string();
            error!("Failed to parse pacs.008 document at path '{}': {}", path, inner_err);
            error!("Document JSON was: {}", doc_str);
            format!("Failed to parse pacs.008 document at path '{}': {}", path, inner_err)
        })?;
    
    // Create envelope
    let envelope = MxEnvelope::new(header, doc, "urn:iso:std:iso:20022:tech:xsd:pacs.008.001.08".to_string());
    
    // Serialize to XML
    let xml_declaration = r#"<?xml version="1.0" encoding="UTF-8"?>"#;
    let xml_body = xml_to_string(&envelope)?;
    
    Ok(format!("{}\n{}", xml_declaration, xml_body))
}

/// Generate pacs.002 XML from parsed JSON
fn generate_pacs002_xml(app_hdr: &Value, document: &Value) -> Result<String, Box<dyn std::error::Error>> {
    debug!("Parsing pacs.002 header");
    let header_str = serde_json::to_string(app_hdr)?;
    let hd = &mut serde_json::Deserializer::from_str(&header_str);
    let header: bah_pacs_002_001_10::BusinessApplicationHeaderV02 = deserialize(hd)
        .map_err(|e| {
            let path = e.path().to_string();
            let inner_err = e.into_inner().to_string();
            format!("Failed to parse pacs.002 header at path '{}': {}", path, inner_err)
        })?;
    
    debug!("Parsing pacs.002 document");
    let doc_content = document
        .get("FIToFIPmtStsRpt")
        .ok_or_else(|| "Missing FIToFIPmtStsRpt in Document")?;
    
    let doc_str = serde_json::to_string(doc_content)?;
    let dd = &mut serde_json::Deserializer::from_str(&doc_str);
    let doc: pacs_002_001_10::FIToFIPaymentStatusReportV10 = deserialize(dd)
        .map_err(|e| {
            let path = e.path().to_string();
            let inner_err = e.into_inner().to_string();
            error!("Failed to parse pacs.002 document at path '{}': {}", path, inner_err);
            format!("Failed to parse pacs.002 document at path '{}': {}", path, inner_err)
        })?;
    
    let envelope = MxEnvelope::new(header, doc, "urn:iso:std:iso:20022:tech:xsd:pacs.002.001.10".to_string());
    let xml_declaration = r#"<?xml version="1.0" encoding="UTF-8"?>"#;
    let xml_body = xml_to_string(&envelope)?;
    
    Ok(format!("{}\n{}", xml_declaration, xml_body))
}

/// Generate pacs.004 XML from parsed JSON
fn generate_pacs004_xml(app_hdr: &Value, document: &Value) -> Result<String, Box<dyn std::error::Error>> {
    debug!("Parsing pacs.004 header");
    let header_str = serde_json::to_string(app_hdr)?;
    let hd = &mut serde_json::Deserializer::from_str(&header_str);
    let header: bah_pacs_004_001_09::BusinessApplicationHeaderV02 = deserialize(hd)
        .map_err(|e| {
            let path = e.path().to_string();
            let inner_err = e.into_inner().to_string();
            format!("Failed to parse pacs.004 header at path '{}': {}", path, inner_err)
        })?;
    
    debug!("Parsing pacs.004 document");
    let doc_content = document
        .get("PmtRtr")
        .ok_or_else(|| "Missing PmtRtr in Document")?;
    
    let doc_str = serde_json::to_string(doc_content)?;
    let dd = &mut serde_json::Deserializer::from_str(&doc_str);
    let doc: pacs_004_001_09::PaymentReturnV09 = deserialize(dd)
        .map_err(|e| {
            let path = e.path().to_string();
            let inner_err = e.into_inner().to_string();
            error!("Failed to parse pacs.004 document at path '{}': {}", path, inner_err);
            format!("Failed to parse pacs.004 document at path '{}': {}", path, inner_err)
        })?;
    
    let envelope = MxEnvelope::new(header, doc, "urn:iso:std:iso:20022:tech:xsd:pacs.004.001.09".to_string());
    let xml_declaration = r#"<?xml version="1.0" encoding="UTF-8"?>"#;
    let xml_body = xml_to_string(&envelope)?;
    
    Ok(format!("{}\n{}", xml_declaration, xml_body))
}

/// Generate pacs.009 XML from parsed JSON
fn generate_pacs009_xml(app_hdr: &Value, document: &Value) -> Result<String, Box<dyn std::error::Error>> {
    debug!("Parsing pacs.009 header");
    let header_str = serde_json::to_string(app_hdr)?;
    let hd = &mut serde_json::Deserializer::from_str(&header_str);
    let header: bah_pacs_009_001_08::BusinessApplicationHeaderV02 = deserialize(hd)
        .map_err(|e| {
            let path = e.path().to_string();
            let inner_err = e.into_inner().to_string();
            format!("Failed to parse pacs.009 header at path '{}': {}", path, inner_err)
        })?;
    
    debug!("Parsing pacs.009 document");
    let doc_content = document
        .get("FICdtTrf")
        .ok_or_else(|| "Missing FICdtTrf in Document")?;
    
    let doc_str = serde_json::to_string(doc_content)?;
    let dd = &mut serde_json::Deserializer::from_str(&doc_str);
    let doc: pacs_009_001_08::FinancialInstitutionCreditTransferV08 = deserialize(dd)
        .map_err(|e| {
            let path = e.path().to_string();
            let inner_err = e.into_inner().to_string();
            error!("Failed to parse pacs.009 document at path '{}': {}", path, inner_err);
            format!("Failed to parse pacs.009 document at path '{}': {}", path, inner_err)
        })?;
    
    let envelope = MxEnvelope::new(header, doc, "urn:iso:std:iso:20022:tech:xsd:pacs.009.001.08".to_string());
    let xml_declaration = r#"<?xml version="1.0" encoding="UTF-8"?>"#;
    let xml_body = xml_to_string(&envelope)?;
    
    Ok(format!("{}\n{}", xml_declaration, xml_body))
}

/// Generate camt.052 XML from parsed JSON
fn generate_camt052_xml(app_hdr: &Value, document: &Value) -> Result<String, Box<dyn std::error::Error>> {
    debug!("Parsing camt.052 header");
    let header_str = serde_json::to_string(app_hdr)?;
    let hd = &mut serde_json::Deserializer::from_str(&header_str);
    let header: bah_camt_052_001_08::BusinessApplicationHeaderV02 = deserialize(hd)
        .map_err(|e| {
            let path = e.path().to_string();
            let inner_err = e.into_inner().to_string();
            format!("Failed to parse camt.052 header at path '{}': {}", path, inner_err)
        })?;
    
    debug!("Parsing camt.052 document");
    let doc_content = document
        .get("BkToCstmrAcctRpt")
        .ok_or_else(|| "Missing BkToCstmrAcctRpt in Document")?;
    
    let doc_str = serde_json::to_string(doc_content)?;
    let dd = &mut serde_json::Deserializer::from_str(&doc_str);
    let doc: camt_052_001_08::BankToCustomerAccountReportV08 = deserialize(dd)
        .map_err(|e| {
            let path = e.path().to_string();
            let inner_err = e.into_inner().to_string();
            error!("Failed to parse camt.052 document at path '{}': {}", path, inner_err);
            format!("Failed to parse camt.052 document at path '{}': {}", path, inner_err)
        })?;
    
    let envelope = MxEnvelope::new(header, doc, "urn:iso:std:iso:20022:tech:xsd:camt.052.001.08".to_string());
    let xml_declaration = r#"<?xml version="1.0" encoding="UTF-8"?>"#;
    let xml_body = xml_to_string(&envelope)?;
    
    Ok(format!("{}\n{}", xml_declaration, xml_body))
}

/// Generate camt.053 XML from parsed JSON
fn generate_camt053_xml(app_hdr: &Value, document: &Value) -> Result<String, Box<dyn std::error::Error>> {
    debug!("Parsing camt.053 header");
    let header_str = serde_json::to_string(app_hdr)?;
    let hd = &mut serde_json::Deserializer::from_str(&header_str);
    let header: bah_camt_053_001_08::BusinessApplicationHeaderV02 = deserialize(hd)
        .map_err(|e| {
            let path = e.path().to_string();
            let inner_err = e.into_inner().to_string();
            format!("Failed to parse camt.053 header at path '{}': {}", path, inner_err)
        })?;
    
    debug!("Parsing camt.053 document");
    let doc_content = document
        .get("BkToCstmrStmt")
        .ok_or_else(|| "Missing BkToCstmrStmt in Document")?;
    
    let doc_str = serde_json::to_string(doc_content)?;
    let dd = &mut serde_json::Deserializer::from_str(&doc_str);
    let doc: camt_053_001_08::BankToCustomerStatementV08 = deserialize(dd)
        .map_err(|e| {
            let path = e.path().to_string();
            let inner_err = e.into_inner().to_string();
            error!("Failed to parse camt.053 document at path '{}': {}", path, inner_err);
            format!("Failed to parse camt.053 document at path '{}': {}", path, inner_err)
        })?;
    
    let envelope = MxEnvelope::new(header, doc, "urn:iso:std:iso:20022:tech:xsd:camt.053.001.08".to_string());
    let xml_declaration = r#"<?xml version="1.0" encoding="UTF-8"?>"#;
    let xml_body = xml_to_string(&envelope)?;
    
    Ok(format!("{}\n{}", xml_declaration, xml_body))
}

/// Generate camt.107 XML from parsed JSON
fn generate_camt107_xml(app_hdr: &Value, document: &Value) -> Result<String, Box<dyn std::error::Error>> {
    debug!("Parsing camt.107 header");
    let header_str = serde_json::to_string(app_hdr)?;
    let hd = &mut serde_json::Deserializer::from_str(&header_str);
    let header: bah_camt_107_001_01::BusinessApplicationHeaderV02 = deserialize(hd)
        .map_err(|e| {
            let path = e.path().to_string();
            let inner_err = e.into_inner().to_string();
            format!("Failed to parse camt.107 header at path '{}': {}", path, inner_err)
        })?;
    
    debug!("Parsing camt.107 document");
    let doc_content = document
        .get("ChqPresntmntNtfctn")
        .ok_or_else(|| "Missing ChqPresntmntNtfctn in Document")?;
    
    let doc_str = serde_json::to_string(doc_content)?;
    let dd = &mut serde_json::Deserializer::from_str(&doc_str);
    let doc: camt_107_001_01::ChequePresentmentNotificationV01 = deserialize(dd)
        .map_err(|e| {
            let path = e.path().to_string();
            let inner_err = e.into_inner().to_string();
            error!("Failed to parse camt.107 document at path '{}': {}", path, inner_err);
            format!("Failed to parse camt.107 document at path '{}': {}", path, inner_err)
        })?;
    
    let envelope = MxEnvelope::new(header, doc, "urn:iso:std:iso:20022:tech:xsd:camt.107.001.01".to_string());
    let xml_declaration = r#"<?xml version="1.0" encoding="UTF-8"?>"#;
    let xml_body = xml_to_string(&envelope)?;
    
    Ok(format!("{}\n{}", xml_declaration, xml_body))
}

/// Generate camt.108 XML from parsed JSON
fn generate_camt108_xml(app_hdr: &Value, document: &Value) -> Result<String, Box<dyn std::error::Error>> {
    debug!("Parsing camt.108 header");
    let header_str = serde_json::to_string(app_hdr)?;
    let hd = &mut serde_json::Deserializer::from_str(&header_str);
    let header: bah_camt_108_001_01::BusinessApplicationHeaderV02 = deserialize(hd)
        .map_err(|e| {
            let path = e.path().to_string();
            let inner_err = e.into_inner().to_string();
            format!("Failed to parse camt.108 header at path '{}': {}", path, inner_err)
        })?;
    
    debug!("Parsing camt.108 document");
    let doc_content = document
        .get("ChqCxlOrStopReq")
        .ok_or_else(|| "Missing ChqCxlOrStopReq in Document")?;
    
    let doc_str = serde_json::to_string(doc_content)?;
    let dd = &mut serde_json::Deserializer::from_str(&doc_str);
    let doc: camt_108_001_01::ChequeCancellationOrStopRequestV01 = deserialize(dd)
        .map_err(|e| {
            let path = e.path().to_string();
            let inner_err = e.into_inner().to_string();
            error!("Failed to parse camt.108 document at path '{}': {}", path, inner_err);
            format!("Failed to parse camt.108 document at path '{}': {}", path, inner_err)
        })?;
    
    let envelope = MxEnvelope::new(header, doc, "urn:iso:std:iso:20022:tech:xsd:camt.108.001.01".to_string());
    let xml_declaration = r#"<?xml version="1.0" encoding="UTF-8"?>"#;
    let xml_body = xml_to_string(&envelope)?;
    
    Ok(format!("{}\n{}", xml_declaration, xml_body))
}

/// Generate camt.109 XML from parsed JSON
fn generate_camt109_xml(app_hdr: &Value, document: &Value) -> Result<String, Box<dyn std::error::Error>> {
    debug!("Parsing camt.109 header");
    let header_str = serde_json::to_string(app_hdr)?;
    let hd = &mut serde_json::Deserializer::from_str(&header_str);
    let header: bah_camt_109_001_01::BusinessApplicationHeaderV02 = deserialize(hd)
        .map_err(|e| {
            let path = e.path().to_string();
            let inner_err = e.into_inner().to_string();
            format!("Failed to parse camt.109 header at path '{}': {}", path, inner_err)
        })?;
    
    debug!("Parsing camt.109 document");
    let doc_content = document
        .get("ChqCxlOrStopRpt")
        .ok_or_else(|| "Missing ChqCxlOrStopRpt in Document")?;
    
    let doc_str = serde_json::to_string(doc_content)?;
    let dd = &mut serde_json::Deserializer::from_str(&doc_str);
    let doc: camt_109_001_01::ChequeCancellationOrStopReportV01 = deserialize(dd)
        .map_err(|e| {
            let path = e.path().to_string();
            let inner_err = e.into_inner().to_string();
            error!("Failed to parse camt.109 document at path '{}': {}", path, inner_err);
            format!("Failed to parse camt.109 document at path '{}': {}", path, inner_err)
        })?;
    
    let envelope = MxEnvelope::new(header, doc, "urn:iso:std:iso:20022:tech:xsd:camt.109.001.01".to_string());
    let xml_declaration = r#"<?xml version="1.0" encoding="UTF-8"?>"#;
    let xml_body = xml_to_string(&envelope)?;
    
    Ok(format!("{}\n{}", xml_declaration, xml_body))
}

/// Generate camt.054 XML from parsed JSON
fn generate_camt054_xml(app_hdr: &Value, document: &Value) -> Result<String, Box<dyn std::error::Error>> {
    debug!("Parsing camt.054 header");
    let header_str = serde_json::to_string(app_hdr)?;
    let hd = &mut serde_json::Deserializer::from_str(&header_str);
    let header: bah_camt_052_001_08::BusinessApplicationHeaderV02 = deserialize(hd)
        .map_err(|e| {
            let path = e.path().to_string();
            let inner_err = e.into_inner().to_string();
            format!("Failed to parse camt.054 header at path '{}': {}", path, inner_err)
        })?;
    
    debug!("Parsing camt.054 document");
    let doc_content = document
        .get("BkToCstmrDbtCdtNtfctn")
        .ok_or_else(|| "Missing BkToCstmrDbtCdtNtfctn in Document")?;
    
    let doc_str = serde_json::to_string(doc_content)?;
    let dd = &mut serde_json::Deserializer::from_str(&doc_str);
    let doc: camt_054_001_08::BankToCustomerDebitCreditNotificationV08 = deserialize(dd)
        .map_err(|e| {
            let path = e.path().to_string();
            let inner_err = e.into_inner().to_string();
            error!("Failed to parse camt.054 document at path '{}': {}", path, inner_err);
            format!("Failed to parse camt.054 document at path '{}': {}", path, inner_err)
        })?;
    
    let envelope = MxEnvelope::new(header, doc, "urn:iso:std:iso:20022:tech:xsd:camt.054.001.08".to_string());
    let xml_declaration = r#"<?xml version="1.0" encoding="UTF-8"?>"#;
    let xml_body = xml_to_string(&envelope)?;
    
    Ok(format!("{}\n{}", xml_declaration, xml_body))
}

/// Generate camt.056 XML from parsed JSON
fn generate_camt056_xml(app_hdr: &Value, document: &Value) -> Result<String, Box<dyn std::error::Error>> {
    debug!("Parsing camt.056 header");
    let header_str = serde_json::to_string(app_hdr)?;
    let hd = &mut serde_json::Deserializer::from_str(&header_str);
    let header: bah_camt_056_001_08::BusinessApplicationHeaderV02 = deserialize(hd)
        .map_err(|e| {
            let path = e.path().to_string();
            let inner_err = e.into_inner().to_string();
            format!("Failed to parse camt.056 header at path '{}': {}", path, inner_err)
        })?;
    
    debug!("Parsing camt.056 document");
    let doc_content = document
        .get("FIToFIPmtCxlReq")
        .ok_or_else(|| "Missing FIToFIPmtCxlReq in Document")?;
    
    let doc_str = serde_json::to_string(doc_content)?;
    let dd = &mut serde_json::Deserializer::from_str(&doc_str);
    let doc: camt_056_001_08::FIToFIPaymentCancellationRequestV08 = deserialize(dd)
        .map_err(|e| {
            let path = e.path().to_string();
            let inner_err = e.into_inner().to_string();
            error!("Failed to parse camt.056 document at path '{}': {}", path, inner_err);
            format!("Failed to parse camt.056 document at path '{}': {}", path, inner_err)
        })?;
    
    let envelope = MxEnvelope::new(header, doc, "urn:iso:std:iso:20022:tech:xsd:camt.056.001.09".to_string());
    let xml_declaration = r#"<?xml version="1.0" encoding="UTF-8"?>"#;
    let xml_body = xml_to_string(&envelope)?;
    
    Ok(format!("{}\n{}", xml_declaration, xml_body))
}

/// Generate camt.057 XML from parsed JSON
fn generate_camt057_xml(app_hdr: &Value, document: &Value) -> Result<String, Box<dyn std::error::Error>> {
    debug!("Parsing camt.057 header");
    let header_str = serde_json::to_string(app_hdr)?;
    let hd = &mut serde_json::Deserializer::from_str(&header_str);
    let header: bah_camt_057_001_06::BusinessApplicationHeaderV02 = deserialize(hd)
        .map_err(|e| {
            let path = e.path().to_string();
            let inner_err = e.into_inner().to_string();
            format!("Failed to parse camt.057 header at path '{}': {}", path, inner_err)
        })?;
    
    debug!("Parsing camt.057 document");
    let doc_content = document
        .get("NtfctnToRcv")
        .ok_or_else(|| "Missing NtfctnToRcv in Document")?;
    
    let doc_str = serde_json::to_string(doc_content)?;
    let dd = &mut serde_json::Deserializer::from_str(&doc_str);
    let doc: camt_057_001_06::NotificationToReceiveV06 = deserialize(dd)
        .map_err(|e| {
            let path = e.path().to_string();
            let inner_err = e.into_inner().to_string();
            error!("Failed to parse camt.057 document at path '{}': {}", path, inner_err);
            format!("Failed to parse camt.057 document at path '{}': {}", path, inner_err)
        })?;
    
    let envelope = MxEnvelope::new(header, doc, "urn:iso:std:iso:20022:tech:xsd:camt.057.001.06".to_string());
    let xml_declaration = r#"<?xml version="1.0" encoding="UTF-8"?>"#;
    let xml_body = xml_to_string(&envelope)?;
    
    Ok(format!("{}\n{}", xml_declaration, xml_body))
}

/// Generate camt.029 XML from parsed JSON
fn generate_camt029_xml(app_hdr: &Value, document: &Value) -> Result<String, Box<dyn std::error::Error>> {
    debug!("Parsing camt.029 header");
    let header_str = serde_json::to_string(app_hdr)?;
    let hd = &mut serde_json::Deserializer::from_str(&header_str);
    let header: bah_camt_025_001_08::BusinessApplicationHeaderV02 = deserialize(hd)
        .map_err(|e| {
            let path = e.path().to_string();
            let inner_err = e.into_inner().to_string();
            format!("Failed to parse camt.029 header at path '{}': {}", path, inner_err)
        })?;
    
    debug!("Parsing camt.029 document");
    let doc_content = document
        .get("RsltnOfInvstgtn")
        .ok_or_else(|| "Missing RsltnOfInvstgtn in Document")?;
    
    let doc_str = serde_json::to_string(doc_content)?;
    let dd = &mut serde_json::Deserializer::from_str(&doc_str);
    let doc: camt_029_001_09::ResolutionOfInvestigationV09 = deserialize(dd)
        .map_err(|e| {
            let path = e.path().to_string();
            let inner_err = e.into_inner().to_string();
            error!("Failed to parse camt.029 document at path '{}': {}", path, inner_err);
            format!("Failed to parse camt.029 document at path '{}': {}", path, inner_err)
        })?;
    
    let envelope = MxEnvelope::new(header, doc, "urn:iso:std:iso:20022:tech:xsd:camt.029.001.09".to_string());
    let xml_declaration = r#"<?xml version="1.0" encoding="UTF-8"?>"#;
    let xml_body = xml_to_string(&envelope)?;
    
    Ok(format!("{}\n{}", xml_declaration, xml_body))
}

/// Generate pain.001 XML from parsed JSON
fn generate_pain001_xml(app_hdr: &Value, document: &Value) -> Result<String, Box<dyn std::error::Error>> {
    debug!("Parsing pain.001 header");
    let header_str = serde_json::to_string(app_hdr)?;
    let hd = &mut serde_json::Deserializer::from_str(&header_str);
    let header: bah_pain_001_001_09::BusinessApplicationHeaderV02 = deserialize(hd)
        .map_err(|e| {
            let path = e.path().to_string();
            let inner_err = e.into_inner().to_string();
            format!("Failed to parse pain.001 header at path '{}': {}", path, inner_err)
        })?;
    
    debug!("Parsing pain.001 document");
    let doc_content = document
        .get("CstmrCdtTrfInitn")
        .ok_or_else(|| "Missing CstmrCdtTrfInitn in Document")?;
    
    let doc_str = serde_json::to_string(doc_content)?;
    let dd = &mut serde_json::Deserializer::from_str(&doc_str);
    let doc: pain_001_001_09::CustomerCreditTransferInitiationV09 = deserialize(dd)
        .map_err(|e| {
            let path = e.path().to_string();
            let inner_err = e.into_inner().to_string();
            error!("Failed to parse pain.001 document at path '{}': {}", path, inner_err);
            format!("Failed to parse pain.001 document at path '{}': {}", path, inner_err)
        })?;
    
    let envelope = MxEnvelope::new(header, doc, "urn:iso:std:iso:20022:tech:xsd:pain.001.001.09".to_string());
    let xml_declaration = r#"<?xml version="1.0" encoding="UTF-8"?>"#;
    let xml_body = xml_to_string(&envelope)?;
    
    Ok(format!("{}\n{}", xml_declaration, xml_body))
}

/// Generate pain.002 XML from parsed JSON
fn generate_pain002_xml(app_hdr: &Value, document: &Value) -> Result<String, Box<dyn std::error::Error>> {
    debug!("Parsing pain.002 header");
    let header_str = serde_json::to_string(app_hdr)?;
    let hd = &mut serde_json::Deserializer::from_str(&header_str);
    let header: bah_pain_002_001_10::BusinessApplicationHeaderV02 = deserialize(hd)
        .map_err(|e| {
            let path = e.path().to_string();
            let inner_err = e.into_inner().to_string();
            format!("Failed to parse pain.002 header at path '{}': {}", path, inner_err)
        })?;
    
    debug!("Parsing pain.002 document");
    let doc_content = document
        .get("CstmrPmtStsRpt")
        .ok_or_else(|| "Missing CstmrPmtStsRpt in Document")?;
    
    let doc_str = serde_json::to_string(doc_content)?;
    let dd = &mut serde_json::Deserializer::from_str(&doc_str);
    let doc: pain_002_001_10::CustomerPaymentStatusReportV10 = deserialize(dd)
        .map_err(|e| {
            let path = e.path().to_string();
            let inner_err = e.into_inner().to_string();
            error!("Failed to parse pain.002 document at path '{}': {}", path, inner_err);
            format!("Failed to parse pain.002 document at path '{}': {}", path, inner_err)
        })?;
    
    let envelope = MxEnvelope::new(header, doc, "urn:iso:std:iso:20022:tech:xsd:pain.002.001.10".to_string());
    let xml_declaration = r#"<?xml version="1.0" encoding="UTF-8"?>"#;
    let xml_body = xml_to_string(&envelope)?;
    
    Ok(format!("{}\n{}", xml_declaration, xml_body))
}

/// Generate pain.008 XML from parsed JSON
fn generate_pain008_xml(app_hdr: &Value, document: &Value) -> Result<String, Box<dyn std::error::Error>> {
    debug!("Parsing pain.008 header");
    let header_str = serde_json::to_string(app_hdr)?;
    let hd = &mut serde_json::Deserializer::from_str(&header_str);
    let header: bah_pain_008_001_08::BusinessApplicationHeaderV02 = deserialize(hd)
        .map_err(|e| {
            let path = e.path().to_string();
            let inner_err = e.into_inner().to_string();
            format!("Failed to parse pain.008 header at path '{}': {}", path, inner_err)
        })?;
    
    debug!("Parsing pain.008 document");
    let doc_content = document
        .get("CstmrDrctDbtInitn")
        .ok_or_else(|| "Missing CstmrDrctDbtInitn in Document")?;
    
    let doc_str = serde_json::to_string(doc_content)?;
    let dd = &mut serde_json::Deserializer::from_str(&doc_str);
    let doc: pain_008_001_08::CustomerDirectDebitInitiationV08 = deserialize(dd)
        .map_err(|e| {
            let path = e.path().to_string();
            let inner_err = e.into_inner().to_string();
            error!("Failed to parse pain.008 document at path '{}': {}", path, inner_err);
            format!("Failed to parse pain.008 document at path '{}': {}", path, inner_err)
        })?;
    
    let envelope = MxEnvelope::new(header, doc, "urn:iso:std:iso:20022:tech:xsd:pain.008.001.08".to_string());
    let xml_declaration = r#"<?xml version="1.0" encoding="UTF-8"?>"#;
    let xml_body = xml_to_string(&envelope)?;
    
    Ok(format!("{}\n{}", xml_declaration, xml_body))
}

/// Generate pacs.003 XML from parsed JSON
fn generate_pacs003_xml(app_hdr: &Value, document: &Value) -> Result<String, Box<dyn std::error::Error>> {
    debug!("Parsing pacs.003 header");
    let header_str = serde_json::to_string(app_hdr)?;
    let hd = &mut serde_json::Deserializer::from_str(&header_str);
    let header: bah_pacs_003_001_08::BusinessApplicationHeaderV02 = deserialize(hd)
        .map_err(|e| {
            let path = e.path().to_string();
            let inner_err = e.into_inner().to_string();
            format!("Failed to parse pacs.003 header at path '{}': {}", path, inner_err)
        })?;
    
    debug!("Parsing pacs.003 document");
    let doc_content = document
        .get("FIToFICstmrDrctDbt")
        .ok_or_else(|| "Missing FIToFICstmrDrctDbt in Document")?;
    
    let doc_str = serde_json::to_string(doc_content)?;
    let dd = &mut serde_json::Deserializer::from_str(&doc_str);
    let doc: pacs_003_001_08::FIToFICustomerDirectDebitV08 = deserialize(dd)
        .map_err(|e| {
            let path = e.path().to_string();
            let inner_err = e.into_inner().to_string();
            error!("Failed to parse pacs.003 document at path '{}': {}", path, inner_err);
            format!("Failed to parse pacs.003 document at path '{}': {}", path, inner_err)
        })?;
    
    let envelope = MxEnvelope::new(header, doc, "urn:iso:std:iso:20022:tech:xsd:pacs.003.001.08".to_string());
    let xml_declaration = r#"<?xml version="1.0" encoding="UTF-8"?>"#;
    let xml_body = xml_to_string(&envelope)?;
    
    Ok(format!("{}\n{}", xml_declaration, xml_body))
}

/// Generate admi.024 XML from parsed JSON
fn generate_admi024_xml(app_hdr: &Value, document: &Value) -> Result<String, Box<dyn std::error::Error>> {
    debug!("Parsing admi.024 header");
    let header_str = serde_json::to_string(app_hdr)?;
    let hd = &mut serde_json::Deserializer::from_str(&header_str);
    let header: bah_admi_024_001_01::BusinessApplicationHeaderV02 = deserialize(hd)
        .map_err(|e| {
            let path = e.path().to_string();
            let inner_err = e.into_inner().to_string();
            format!("Failed to parse admi.024 header at path '{}': {}", path, inner_err)
        })?;
    
    debug!("Parsing admi.024 document");
    let doc_content = document
        .get("NtfctnOfCorr")
        .ok_or_else(|| "Missing NtfctnOfCorr in Document")?;
    
    let doc_str = serde_json::to_string(doc_content)?;
    let dd = &mut serde_json::Deserializer::from_str(&doc_str);
    let doc: admi_024_001_01::NotificationOfCorrespondenceV01 = deserialize(dd)
        .map_err(|e| {
            let path = e.path().to_string();
            let inner_err = e.into_inner().to_string();
            error!("Failed to parse admi.024 document at path '{}': {}", path, inner_err);
            format!("Failed to parse admi.024 document at path '{}': {}", path, inner_err)
        })?;
    
    let envelope = MxEnvelope::new(header, doc, "urn:iso:std:iso:20022:tech:xsd:admi.024.001.01".to_string());
    let xml_declaration = r#"<?xml version="1.0" encoding="UTF-8"?>"#;
    let xml_body = xml_to_string(&envelope)?;
    
    Ok(format!("{}\n{}", xml_declaration, xml_body))
}

/// Generate MX message XML from JSON data
pub fn generate_mx_from_json(
    message_type: &str,
    json_data: &Value,
) -> Result<String, Box<dyn std::error::Error>> {
    debug!("Generating {} from JSON data", message_type);
    debug!("Input JSON structure: {}", serde_json::to_string_pretty(json_data)?);
    
    // First parse the JSON into a generic envelope to validate structure
    let envelope: GenericMxEnvelope = serde_json::from_value(json_data.clone())
        .map_err(|e| {
            error!("Failed to parse JSON into MX envelope structure: {}", e);
            format!("Invalid MX message structure: {}", e)
        })?;
    
    debug!("Successfully parsed generic envelope structure");
    debug!("AppHdr keys: {:?}", envelope.app_hdr.as_object().map(|o| o.keys().collect::<Vec<_>>()));
    debug!("Document keys: {:?}", envelope.document.as_object().map(|o| o.keys().collect::<Vec<_>>()));
    
    // Generate XML based on message type
    let xml_string = match message_type {
        "admi.024" => generate_admi024_xml(&envelope.app_hdr, &envelope.document)?,
        "pacs.002" => generate_pacs002_xml(&envelope.app_hdr, &envelope.document)?,
        "pacs.003" => generate_pacs003_xml(&envelope.app_hdr, &envelope.document)?,
        "pacs.004" => generate_pacs004_xml(&envelope.app_hdr, &envelope.document)?,
        "pacs.008" => generate_pacs008_xml(&envelope.app_hdr, &envelope.document)?,
        "pacs.009" => generate_pacs009_xml(&envelope.app_hdr, &envelope.document)?,
        "camt.029" => generate_camt029_xml(&envelope.app_hdr, &envelope.document)?,
        "camt.052" => generate_camt052_xml(&envelope.app_hdr, &envelope.document)?,
        "camt.053" => generate_camt053_xml(&envelope.app_hdr, &envelope.document)?,
        "camt.054" => generate_camt054_xml(&envelope.app_hdr, &envelope.document)?,
        "camt.056" => generate_camt056_xml(&envelope.app_hdr, &envelope.document)?,
        "camt.057" => generate_camt057_xml(&envelope.app_hdr, &envelope.document)?,
        "camt.107" => generate_camt107_xml(&envelope.app_hdr, &envelope.document)?,
        "camt.108" => generate_camt108_xml(&envelope.app_hdr, &envelope.document)?,
        "camt.109" => generate_camt109_xml(&envelope.app_hdr, &envelope.document)?,
        "pain.001" => generate_pain001_xml(&envelope.app_hdr, &envelope.document)?,
        "pain.002" => generate_pain002_xml(&envelope.app_hdr, &envelope.document)?,
        "pain.008" => generate_pain008_xml(&envelope.app_hdr, &envelope.document)?,
        
        // These message types are not yet implemented
        "camt.025" | "camt.060" => {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("XML generation for {} is not yet implemented", message_type)
            )));
        },
        
        _ => {
            error!("Unsupported MX message type: {}", message_type);
            return Err(format!("Unsupported MX message type: {}", message_type).into());
        }
    };
    
    debug!("Successfully generated {} XML message", message_type);
    Ok(xml_string)
}