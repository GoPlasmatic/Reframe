# Reframe Product Roadmap

This document outlines the strategic direction and future development priorities for the Reframe project. Our goal is to evolve Reframe into a comprehensive, open-source solution for bidirectional SWIFT message transformation.

## ✅ **Current Status: Complete Payment, Cancellation & Cheque Processing Ecosystem with Enhanced Bidirectional Support** ✨ **v2.0.4 MILESTONE**

Reframe has achieved major milestones: providing robust, production-ready transformation for the **complete payment, cancellation, and cheque processing lifecycle** including MT103, MT202, MT205 payments, MT192, MT292, MT196, MT296 cancellation workflows, **plus comprehensive bidirectional transformation capabilities including cheque processing**. This establishes Reframe as a complete solution for modern banking operations with extensive forward and reverse transformation support.

*   **Full Payment Coverage**: Complete support for MT103, MT202, and MT205 with all variants (normal, cover, rejection, return)
*   **Complete Cancellation & Investigation Support**: MT192, MT292, MT196, MT296 workflows with camt.056.001.08 compliance
*   **Cash Management Support**: MT900/MT910 confirmation messages with camt.054.001.08 compliance
*   **Enhanced Bidirectional Transformation**: Comprehensive ISO 20022 → SWIFT MT conversion covering 7 message families
*   **Cheque Processing Ecosystem** ✨ **NEW v2.0.4**: Complete cheque lifecycle support with camt.107/108/109 → MT110/111/112 transformations
*   **Payment Status Reporting** ✨ **NEW v2.0.4**: Enhanced rejection handling with pacs.002 → MT199/MT299 transformations
*   **End-to-End Lifecycle**: From payment initiation through processing, exceptions, cancellations, cheque processing, and comprehensive reverse conversion
*   **CBPR+ Compliance**: For payments, exceptions, cancellation scenarios, cheque processing, and reverse transformations
*   **Transparent Workflows**: With consistent, auditable JSON-based logic across all message types and directions
*   **High-Performance Engine**: Built in Rust for speed and reliability in both transformation directions
*   **UETR Integration**: Full support for Unique End-to-End Transaction References in cancellation workflows

With this comprehensive payment, cancellation, and cheque processing ecosystem complete and enhanced bidirectional transformation capabilities, we are now focused on expanding into additional corporate treasury and trade finance use cases.

---

## 🚀 **What's Next: The Future of Reframe**

Our roadmap is focused on completing the bidirectional transformation ecosystem and expanding into additional corporate treasury and cash management capabilities. We plan to tackle this in the following strategic phases.

### **Phase 1: Complete Bidirectional Transformation Coverage** ✅ **COMPLETED v2.0.4**

**Successfully completed reverse transformation support with major enhancements:**

#### **Completed Reverse Transformations** ✅ **DONE**
*   **Payment Status Reports**: ✅ pacs.002 → MT199/MT299 (Enhanced rejection and status reporting)
*   **Cheque Processing Lifecycle**: ✅ camt.107/108/109 → MT110/111/112 (Complete cheque presentment, cancellation, and reporting)
*   **Business Value**: Complete round-trip processing for payments, cheques, and status reporting
*   **Enhanced Error Handling**: Comprehensive validation for all reverse transformations

#### **Remaining Priority Transformations** ✨ **NEXT PHASE**
*   **Cancellation Round-trip**: camt.056 → MT192/MT292/MT196/MT296 (Cancellation Requests → Cancellation Messages)
*   **Cash Management Round-trip**: camt.054 → MT900/MT910 (Bank Notifications → Confirmation Messages)

#### **Bidirectional Web Interface**
*   **Enhanced UI**: Updated web interface with direction selection and format auto-detection
*   **Real-time Validation**: Live validation for both MT and ISO 20022 message formats
*   **Round-trip Testing**: Built-in tools to test forward and reverse transformations

### **Phase 2: Corporate & Treasury Payment Initiation**

Building on the solid bidirectional foundation, expand support for key corporate and treasury payment initiation messages.

*   **Target Messages**: MT101 (Request for Transfer), MT102 (Multiple Customer Credit Transfer), MT104 (Direct Debit and Request for Direct Debit), MT107 (General Direct Debit).
*   **Bidirectional Support**: Full forward (MT→ISO 20022) and reverse (ISO 20022→MT) transformation capabilities
*   **Business Value**: Enable corporate clients to initiate payments directly with complete system interoperability
*   **ISO 20022 Mapping**: Integration with pain.001 (Customer Credit Transfer Initiation) and pain.008 (Customer Direct Debit Initiation) schemas.

### **Phase 3: Advanced Investigation & Query Messages**

To complement the existing cancellation workflows, implement the remaining investigation and query messages with full bidirectional support.

*   **Target Messages**: MT195/295 (Queries), MT199/299 (Free Format Messages), additional MTnnn investigation messages.
*   **Bidirectional Coverage**: Complete round-trip transformation for all investigation scenarios
*   **Business Value**: Complete the investigation ecosystem with query initiation, responses, and free-format communication capabilities.
*   **Integration**: Enhanced integration with existing cancellation workflows for comprehensive exception management.

### **Phase 4: Comprehensive Cash Management & Reporting**

This phase will introduce support for a wide array of cash management messages with full bidirectional capabilities.

*   **Target Messages**: MT9xx series (e.g., MT940/950 for Customer Statements) and MT210 (Notice to Receive).
*   **Bidirectional Support**: Complete forward and reverse transformation capabilities for all cash management scenarios
*   **Business Value**: Offer end-to-end visibility into cash positions and account movements with complete system interoperability
*   **ISO 20022 Mapping**: Integration with camt.052, camt.053, camt.054 for comprehensive cash reporting.

### **Future Considerations: Beyond Core Payments**

*   **Securities & Trade Finance**: Explore bidirectional support for MT5xx and MT7xx message series.
*   **Enhanced Analytics & Reporting**: Develop a dedicated module for business intelligence on transformation activity across both directions.
*   **UI/UX Enhancements**: Continuously improve the web interface for intuitive bidirectional transformation experience.
*   **Performance Optimization**: Further optimization for high-volume processing scenarios in both directions.
*   **API Enhancements**: Advanced API features for bulk processing and transformation orchestration.

---

## **Recent Achievements: Version 2.0.4 Milestone** ✨ **MAJOR ADVANCEMENT**

The expansion of cheque processing and payment status reporting represents a significant leap forward:

*   **Cheque Processing Ecosystem**: Complete camt.107/108/109 → MT110/111/112 transformation support
*   **Enhanced Payment Status Reporting**: pacs.002 → MT199/MT299 for comprehensive rejection and status handling
*   **Expanded Message Coverage**: From 3 to 7 reverse transformation message families
*   **Production-Ready Cheque Support**: Full lifecycle cheque presentment, cancellation, and status reporting
*   **Advanced Field Mapping**: 1,600+ lines of new workflow mappings for complex banking operations
*   **Comprehensive Test Coverage**: Complete sample data and validation for all new message types
*   **Enhanced Architecture**: Improved parser and publisher modules for extended message support

This milestone establishes Reframe as a comprehensive solution for banking operations beyond core payments, including cheque processing and enhanced status reporting.

## **Previous Achievement: Version 2.0.2 Milestone**

The initial expansion of bidirectional transformation capabilities:

*   **Enhanced Reverse Transformation**: Added support for pacs.009 and pacs.004 reverse transformations
*   **Complete Financial Institution Support**: Full MT202/MT205 reverse transformation capabilities
*   **Return Message Processing**: Comprehensive support for payment return message reverse transformation
*   **Advanced Validation**: Enhanced validation framework for reverse transformations

---

## **Previous Major Achievement: Version 1.5.4 Milestone**

The completion of the cancellation and investigation ecosystem represented a significant achievement:

*   **4 New Message Types**: MT192, MT292, MT196, MT296 fully implemented
*   **16 Additional Workflows**: Complete cancellation workflow coverage
*   **camt.056.001.08 Support**: New ISO 20022 schema for cancellation requests
*   **Enhanced Parser**: Extended message type detection and validation
*   **Production-Ready**: Complete with test data and comprehensive validation

---

## **Technical Roadmap**

### **Phase 1 Technical Deliverables**
*   Enhanced parser modules for all ISO 20022 schemas
*   Reverse workflow mappings for pacs.009, pacs.002, pacs.004, camt.056, camt.054
*   Comprehensive test suite for bidirectional transformations
*   Performance benchmarking for round-trip processing

### **Phase 2 Technical Deliverables**
*   MT101/102/104/107 parser and workflow implementations
*   pain.001/pain.008 schema support with bidirectional mappings
*   Enhanced validation engine for payment initiation scenarios

### **Phase 3 Technical Deliverables**
*   Investigation message workflows with bidirectional support
*   Advanced error correlation across transformation directions
*   Enhanced monitoring for complex investigation scenarios

---

## **How to Contribute**

Reframe is an open-source project, and we welcome community contributions. Priority areas for contribution include:

1. **Reverse Transformation Mappings**: Help implement additional ISO 20022 → MT transformation workflows
2. **Test Data Development**: Create comprehensive test cases for bidirectional scenarios
3. **Documentation**: Improve documentation for bidirectional transformation use cases
4. **Performance Testing**: Conduct load testing for high-volume bidirectional processing

If you are interested in helping to implement any of the features on our roadmap, please open an issue on GitHub to start a discussion.
