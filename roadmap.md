# CBPR+ MT to MX Transformation Formats

This document outlines all ISO 20022 (MX) message formats required for CBPR+ MT to MX transformation compliance, based on the official CBPR+ Portfolio as of March 2025.

**Production Environment**: `http://reframe-api-prod.eastus.azurecontainer.io:3000`

---

## 🎯 Core Payment Message Formats

### Customer Payments (MT1xx equivalents)

| MT Message | ISO 20022 Equivalent | Current Status |
| -------------------- | --------------------------------------------------- | ----------------|
| **MT 101** | pain.001.001.09 (interbank) | ❌ **Not Implemented** |
| **MT 102 / MT 102 STP** | pacs.008.001.08 | ✅ **Complete** |
| **MT 103 / STP / REMIT** | pacs.008.001.08 | ✅ **Complete** |
| **MT 103/RETN** | pacs.004.001.09 | ❌ **Not Implemented** |
| **MT 104** | pain.008.001.08 / pacs.003.001.08 | ❌ **Not Implemented** |
| **MT 107** | pacs.003.001.08 | ❌ **Not Implemented** |
| **MT 192** | camt.055.001.08 and camt.056.001.08 | 🔄 **Partial** (camt.056 only) |
| **MT 196** | camt.029.001.09 (response) | 🔄 **In Progress** |
| **MT 190 / MT 191** | camt.105.001.02 / camt.106.001.02 | ❌ **Not Implemented** |
| **MT 110 / 111 / 112** | camt.107.001.01 / camt.108.001.01 / camt.109.001.01 | ❌ **Not Implemented** |

### Bank-to-Bank Payments (MT2xx equivalents)

| MT Message | ISO 20022 Equivalent | Current Status |
| -------------------------------- | --------------------------------- | ----------------|
| **MT 200 / 201 / 202 / 203 / 205** | pacs.009.001.08 | 🔄 **Partial** (MT 202 only) |
| **MT 202 with Reimbursement Agents** | pacs.009.001.08 - ADV | ❌ **Not Implemented** |
| **MT 202 COV / MT 205 COV** | pacs.009.001.08 COV | 🔄 **Partial** (MT 202 COV only) |
| **MT 202/RETN / MT 205/RETN** | pacs.004.001.09 | ❌ **Not Implemented** |
| **MT 204** | pacs.010.001.03 | ❌ **Not Implemented** |
| **MT 210** | camt.057.001.06 | ❌ **Not Implemented** |
| **MT 292** | camt.056.001.08 / camt.058.001.06 | ❌ **Not Implemented** |
| **MT 296** | camt.029.001.09 | ❌ **Not Implemented** |

---

## 📒 Cash Management Reporting (MT9xx equivalents)

| MT Message | ISO 20022 Equivalent | Current Status |
| ------------------ | -------------------- | ----------------|
| **MT 900 / 910** | camt.054.001.08 | ❌ **Not Implemented** |
| **MT 920** | camt.060.001.05 | ❌ **Not Implemented** |
| **MT 935 / 940 / 950** | camt.053.001.08 | 🔄 **Partial** (MT 940 only) |
| **MT 941 / 942** | camt.052.001.08 | ✅ **Complete** |

---

## ❓ Exceptions and Investigations

| MT Message | ISO 20022 Equivalent | Current Status |
| ---------------------------- | ---------------------------------------- | ----------------|
| **MT 195 / 295 (Query)** | camt.110.001.01 / camt.110.001.02 | 🔄 **In Progress** |
| **MT 196 / 296 (Response)** | camt.111.001.01 / camt.111.001.02 | 🔄 **In Progress** |
| **MT 199 / 299 (Investigation)** | camt.110 (request) / camt.111 (response) | 🔄 **In Progress** |

---

## 🛏️ Administrative and Notification Messages

| MT Message | ISO 20022 Equivalent | Current Status |
| --------------------------- | ------------------------------------------------ | ----------------|
| **MT 199 / 299** | admi.024.001.01 (Notification of Correspondence) | ❌ **Not Implemented** |
| **New ISO format** | camt.025.001.08 (Receipt) | ❌ **Not Implemented** |
| **Business Application Header** | head.001.001.02 | ❌ **Not Implemented** |

---

## 📝 Multiple & Single Transaction Formats

| Transaction Type | ISO 20022 Equivalent | Current Status |
| --------------------------- | ------------------------------------------------ | ----------------|
| **Multiple Transaction** | camt.105.001.02, camt.106.001.02 | ❌ **Not Implemented** |
| **Single Transaction** | camt.105.001.02, camt.106.001.02 | ❌ **Not Implemented** |

---

## 📊 Implementation Summary

| **Status** | **Count** | **Percentage** |
|------------|-----------|----------------|
| ✅ **Complete** | 7 | 22% |
| 🔄 **In Progress** | 6 | 19% |
| ❌ **Not Implemented** | 19 | 59% |
| **TOTAL** | **32** | **100%** |

---