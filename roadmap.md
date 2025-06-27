# Reframe Product Roadmap

This document outlines the strategic direction and future development priorities for the Reframe project. Our goal is to evolve Reframe into a comprehensive, open-source solution for bidirectional SWIFT message transformation.

## ✅ **Current Status: Complete Payment & Cancellation Ecosystem with Bidirectional Support**

Reframe has achieved major milestones: providing robust, production-ready transformation for the **complete payment and cancellation lifecycle** including MT103, MT202, MT205 payments and MT192, MT292, MT196, MT296 cancellation workflows, **plus initial bidirectional transformation capabilities**. This comprehensive coverage establishes Reframe as a complete solution for modern payment processing scenarios with both forward and reverse transformation support.

*   **Full Payment Coverage**: Complete support for MT103, MT202, and MT205 with all variants (normal, cover, rejection, return)
*   **Complete Cancellation & Investigation Support**: MT192, MT292, MT196, MT296 workflows with camt.056.001.08 compliance
*   **Cash Management Support**: MT900/MT910 confirmation messages with camt.054.001.08 compliance
*   **Bidirectional Transformation**: Initial support for ISO 20022 → SWIFT MT conversion (starting with pacs.008 → MT103)
*   **End-to-End Lifecycle**: From payment initiation through processing, exceptions, cancellations, and reverse conversion
*   **CBPR+ Compliance**: For payments, exceptions, cancellation scenarios, and reverse transformations
*   **Transparent Workflows**: With consistent, auditable JSON-based logic across all message types and directions
*   **High-Performance Engine**: Built in Rust for speed and reliability in both transformation directions
*   **UETR Integration**: Full support for Unique End-to-End Transaction References in cancellation workflows

With this comprehensive payment and cancellation ecosystem complete and bidirectional transformation capabilities initiated, we are now focused on expanding the reverse transformation coverage and additional corporate treasury use cases.

---

## 🚀 **What's Next: The Future of Reframe**

Our roadmap is focused on completing the bidirectional transformation ecosystem and expanding into additional corporate treasury and cash management capabilities. We plan to tackle this in the following strategic phases.

### **Phase 1: Complete Bidirectional Transformation Coverage** ✨ **HIGH PRIORITY**

The immediate focus is to achieve full bidirectional support for all currently supported message types, enabling complete round-trip transformation capabilities.

#### **Reverse Transformation Expansion**
*   **Target Transformations**: 
    - pacs.009 → MT202/MT205 (Financial Institution Credit Transfers)
    - pacs.002 → MT103REJT/MT202REJT/MT205REJT (Payment Status Reports → Rejection Messages)
    - pacs.004 → MT103RETN/MT202RETN/MT205RETN (Payment Returns → Return Messages)
    - camt.056 → MT192/MT292/MT196/MT296 (Cancellation Requests → Cancellation Messages)
    - camt.054 → MT900/MT910 (Bank Notifications → Confirmation Messages)
*   **Business Value**: Enable complete round-trip processing for all payment scenarios, supporting legacy system integration and migration scenarios.
*   **Enhanced Error Handling**: Comprehensive validation for reverse transformations with detailed error mapping.

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

## **Recent Achievements: Version 1.6.0 Milestone** ✨ **NEW**

The recent addition of bidirectional transformation capabilities represents a significant advancement:

*   **Reverse Transformation Engine**: Complete infrastructure for ISO 20022 → SWIFT MT conversion
*   **pacs.008 → MT103 Support**: First production-ready reverse transformation
*   **Intelligent Direction Detection**: Automatic detection of transformation direction based on content type
*   **Enhanced Workflow Architecture**: Separate forward and reverse workflow engines
*   **Comprehensive Error Handling**: Detailed error reporting for both transformation directions
*   **Updated Web Interface**: Enhanced UI supporting bidirectional transformation testing

This milestone positions Reframe as the first open-source solution offering complete bidirectional SWIFT ↔ ISO 20022 transformation capabilities.

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
