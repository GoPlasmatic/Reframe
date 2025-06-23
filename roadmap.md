# Reframe Product Roadmap

This document outlines the strategic direction and future development priorities for the Reframe project. Our goal is to evolve Reframe into a comprehensive, open-source solution for SWIFT message transformation.

## ✅ **Current Status: Complete Payment & Cancellation Ecosystem**

Reframe has achieved a major milestone: providing robust, production-ready transformation for the **complete payment and cancellation lifecycle** including MT103, MT202, MT205 payments and MT192, MT292, MT196, MT296 cancellation workflows. This comprehensive coverage establishes Reframe as a complete solution for modern payment processing scenarios.

*   **Full Payment Coverage**: Complete support for MT103, MT202, and MT205 with all variants (normal, cover, rejection, return)
*   **Complete Cancellation & Investigation Support**: MT192, MT292, MT196, MT296 workflows with camt.056.001.08 compliance
*   **End-to-End Lifecycle**: From payment initiation through processing, exceptions, and cancellations
*   **CBPR+ Compliance**: For payments, exceptions, and cancellation scenarios
*   **Transparent Workflows**: With consistent, auditable JSON-based logic across all message types
*   **High-Performance Engine**: Built in Rust for speed and reliability
*   **UETR Integration**: Full support for Unique End-to-End Transaction References in cancellation workflows

With this comprehensive payment and cancellation ecosystem complete, we are now focused on expanding Reframe's capabilities to cover additional corporate treasury and cash management use cases.

---

## 🚀 **What's Next: The Future of Reframe**

Our roadmap is focused on expanding into corporate treasury initiation and comprehensive cash management capabilities. We plan to tackle this in the following strategic phases.

### **Phase 1: Corporate & Treasury Payment Initiation**

The next major focus is to support key corporate and treasury payment initiation messages. This will enable straight-through processing for a broader range of corporate actions and complete the corporate payment ecosystem.

*   **Target Messages**: MT101 (Request for Transfer), MT102 (Multiple Customer Credit Transfer), MT104 (Direct Debit and Request for Direct Debit), MT107 (General Direct Debit).
*   **Business Value**: Enable corporate clients to initiate payments directly, reducing manual intervention and streamlining treasury operations.
*   **ISO 20022 Mapping**: Integration with pain.001 (Customer Credit Transfer Initiation) and pain.008 (Customer Direct Debit Initiation) schemas.

### **Phase 2: Advanced Investigation & Query Messages**

To complement the existing cancellation workflows, we will implement the remaining investigation and query messages for comprehensive exception handling.

*   **Target Messages**: MT195/295 (Queries), MT199/299 (Free Format Messages), additional MTnnn investigation messages.
*   **Business Value**: Complete the investigation ecosystem with query initiation, responses, and free-format communication capabilities.
*   **Integration**: Enhanced integration with existing cancellation workflows for comprehensive exception management.

### **Phase 3: Comprehensive Cash Management & Reporting**

This phase will introduce support for a wide array of cash management messages, transforming Reframe into a powerful tool for bank statement reporting and liquidity management.

*   **Target Messages**: MT9xx series (e.g., MT940/950 for Customer Statements, MT900/910 for Confirmations of Debit/Credit) and MT210 (Notice to Receive).
*   **Business Value**: Offer end-to-end visibility into cash positions and account movements, enabling better liquidity management and financial control.
*   **ISO 20022 Mapping**: Integration with camt.052, camt.053, camt.054 for comprehensive cash reporting.

### **Future Considerations: Beyond Core Payments**

*   **Securities & Trade Finance**: Explore the potential to support MT5xx and MT7xx message series.
*   **Enhanced Analytics & Reporting**: Develop a dedicated module for business intelligence on transformation activity.
*   **UI/UX Enhancements**: Continuously improve the web interface for a more intuitive user experience.
*   **Performance Optimization**: Further optimization for high-volume processing scenarios.

---

## **Recent Achievements: Version 1.5.4 Milestone**

The recent completion of the cancellation and investigation ecosystem represents a significant achievement:

*   **4 New Message Types**: MT192, MT292, MT196, MT296 fully implemented
*   **16 Additional Workflows**: Complete cancellation workflow coverage
*   **camt.056.001.08 Support**: New ISO 20022 schema for cancellation requests
*   **Enhanced Parser**: Extended message type detection and validation
*   **Production-Ready**: Complete with test data and comprehensive validation

This milestone positions Reframe as a comprehensive solution for the entire payment and cancellation lifecycle, providing financial institutions with a complete toolkit for SWIFT to ISO 20022 transformation.

---

## **How to Contribute**

Reframe is an open-source project, and we welcome community contributions. If you are interested in helping to implement any of the features on our roadmap, please open an issue on GitHub to start a discussion.
