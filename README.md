# Reframe: Open-Source SWIFT MT to ISO 20022 Transformation

**Reframe is an enterprise-grade, high-performance REST API that seamlessly converts legacy SWIFT MT messages into the modern ISO 20022 XML format. Built on a foundation of transparency and open-source principles, Reframe empowers financial institutions to accelerate their transition to CBPR+ with confidence.**

[![CI/CD Pipeline](https://github.com/GoPlasmatic/Reframe/actions/workflows/deploy-azure.yml/badge.svg?branch=main)](https://github.com/GoPlasmatic/Reframe/actions/workflows/deploy-azure.yml)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

**Live Demo**: [http://reframe-api-prod.eastus.azurecontainer.io:3000](http://reframe-api-prod.eastus.azurecontainer.io:3000)

---

## Why Reframe? The Value for Your Business

In an era of evolving payment standards, Reframe offers a strategic advantage by simplifying the complexities of ISO 20022 migration.

*   ✅ **Accelerate CBPR+ Compliance**: Effortlessly transform SWIFT MT messages to the ISO 20022 standard, ensuring you meet regulatory deadlines and stay ahead in the market.
*   🤝 **Full Transparency, Zero Black Boxes**: As an open-source solution, Reframe provides complete visibility into its conversion logic. The transformation rules are defined in simple JSON, allowing for easy auditing, customization, and trust.
*   ⚙️ **Reduce Operational Risk**: Our robust, schema-validated engine minimizes the risk of manual errors and ensures the integrity of your payment messages.
*   🚀 **Boost Efficiency**: Built in Rust, Reframe is designed for high-throughput, low-latency processing, handling your message volumes with ease.
*   🌐 **Comprehensive Message Coverage**: Full support for the entire lifecycle of **MT103, MT202, and MT205** messages, including normal payments, cover payments, rejections, and returns.

---

## A Modern, Transparent Technology Stack

Reframe combines cutting-edge technology with a commitment to openness, delivering a powerful and maintainable solution.

*   **Core Engine**: A high-performance Rust application using the Axum framework provides a robust and scalable API.
*   **Transparent Workflow Engine**: Powered by `dataflow-rs`, Reframe's logic is not hidden in compiled code. It's defined in external JSON files, making the transformation process transparent and easily adaptable.
*   **Integrated Web UI**: A modern React-based interface for easy testing, demonstration, and manual conversions.
*   **Containerized & Cloud-Ready**: Shipped as a single Docker container, ready for deployment on-premises or in the cloud.

---

## Streamlined Maintenance and Operations

We designed Reframe to be as simple to operate as it is powerful.

*   **Simple Deployment**: The entire application—API and web UI—is packaged into a single container. Run it with a single `docker run` command.
*   **Maintain with Ease**: Need to tweak the mapping for a specific field? Simply update a JSON file. No need to recompile or redeploy the entire application.
*   **Automated CI/CD**: A production-ready GitHub Actions pipeline is included for automated testing, building, and deployment to Azure.
*   **Built-in Monitoring**: A `/health` endpoint provides simple, effective monitoring for integration with your existing infrastructure.

---

## Supported Transformations

Reframe offers complete, production-ready support for the following message types:

| SWIFT Message Family | ISO 20022 Format | Scenarios Supported |
|----------------------|------------------|---------------------|
| **MT103** | `pacs.008`, `pacs.002`, `pacs.004` | Normal, STP, Rejection, Return |
| **MT202** | `pacs.009`, `pacs.002`, `pacs.004` | Normal, Cover, Rejection, Return |
| **MT205** | `pacs.009`, `pacs.002`, `pacs.004` | Normal, Cover, Rejection, Return |

---

## Getting Started

### Quick Start

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/Plasmatic/Reframe.git
    cd Reframe
    ```
2.  **Build and run with Docker:**
    ```bash
    docker build -t reframe .
    docker run -p 3000:3000 reframe
    ```
3.  **Access the application** at `http://localhost:3000`.

### API Usage

Convert any supported message with a simple POST request. Reframe automatically detects the message type and applies the correct transformation.

**POST** `/reframe`

```bash
curl -X POST http://localhost:3000/reframe \
  -H "Content-Type: text/plain" \
  --data-binary @path/to/your/mt_message.txt
```

**Example: MT103 to pacs.008**
```
# Request Body
{1:F01BNPAFRPPXXX0000000000}{2:O1031234240101DEUTDEFFXXXX12345678952401011234N}{3:{103:EBA}}{4:
:20:FT21001234567890
:23B:CRED
:32A:240101USD1000,00
:50K:/1234567890
ACME CORPORATION
123 MAIN STREET
NEW YORK NY 10001
:52A:BNPAFRPPXXX
:57A:DEUTDEFFXXX
:59:/DE89370400440532013000
MUELLER GMBH
HAUPTSTRASSE 1
10115 BERLIN
:70:PAYMENT FOR INVOICE 12345
:71A:OUR
-}
```
---

## Open Source and Contributing

Reframe is an open-source project licensed under the Apache 2.0 License. We believe in the power of community and welcome contributions. Please feel free to open issues or submit pull requests.

1.  Fork the Project
2.  Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3.  Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4.  Push to the Branch (`git push origin feature/AmazingFeature`)
5.  Open a Pull Request
