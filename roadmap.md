# Reframe Product Roadmap

This document outlines the strategic direction and future development priorities for the Reframe project. Our goal is to evolve Reframe into a comprehensive, open-source solution for SWIFT message transformation.

## ✅ **Current Status: Complete Core Payments Coverage**

Reframe has achieved its foundational goal: providing robust, production-ready transformation for the entire payment lifecycle of **MT103, MT202, and MT205** messages. This milestone establishes a solid platform for future expansion.

*   **Full CBPR+ Compliance**: For standard payments, cover payments, rejections, and returns.
*   **Transparent Workflows**: With consistent, auditable JSON-based logic.
*   **High-Performance Engine**: Built in Rust for speed and reliability.

With this core functionality complete, we are now focused on expanding Reframe's capabilities to cover a wider range of financial messaging use cases.

---

## 🚀 **What's Next: The Future of Reframe**

Our roadmap is focused on expanding message type coverage and deepening enterprise capabilities. We plan to tackle this in the following strategic phases.

### **Phase 1: Expanding Corporate & Treasury Payments**

The next major focus is to support key corporate and treasury payment initiation messages. This will enable straight-through processing for a broader range of corporate actions.

*   **Target Messages**: MT101 (Request for Transfer), MT102 (Multiple Customer Credit Transfer), MT104 (Direct Debit and Request for Direct Debit), MT107 (General Direct Debit).
*   **Business Value**: Enable corporate clients to initiate payments directly, reducing manual intervention and streamlining treasury operations.

### **Phase 2: Comprehensive Exceptions & Investigations**

To provide a complete end-to-end solution, we will implement the ISO 20022 `camt` messages for exceptions and investigations. This will automate the resolution of payment queries and cancellations.

*   **Target Messages**: MT192 (Request for Cancellation), MT196/296 (Answers), MT195/295 (Queries), MT199/299 (Free Format Messages).
*   **Business Value**: Reduce operational costs and risks associated with manual investigation and exception handling, providing faster resolution for payment issues.

### **Phase 3: Advanced Cash Management & Reporting**

This phase will introduce support for a wide array of cash management messages, transforming Reframe into a powerful tool for bank statement reporting and liquidity management.

*   **Target Messages**: MT9xx series (e.g., MT940/950 for Customer Statements, MT900/910 for Confirmations of Debit/Credit) and MT210 (Notice to Receive).
*   **Business Value**: Offer end-to-end visibility into cash positions and account movements, enabling better liquidity management and financial control.

### **Future Considerations: Beyond the Core**

*   **Securities & Trade Finance**: Explore the potential to support MT5xx and MT7xx message series.
*   **Enhanced Analytics & Reporting**: Develop a dedicated module for business intelligence on transformation activity.
*   **UI/UX Enhancements**: Continuously improve the web interface for a more intuitive user experience.

---

## **How to Contribute**

Reframe is an open-source project, and we welcome community contributions. If you are interested in helping to implement any of the features on our roadmap, please open an issue on GitHub to start a discussion.
