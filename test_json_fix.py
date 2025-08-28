#!/usr/bin/env python3
import json
import sys

# Read and fix the JSON
with open("/Users/codetiger/Development/Plasmatic/Reframe/workflows/reverse/pacs004/07-party-fields-mapping.json", "r") as f:
    content = f.read()

# Parse to check validity
try:
    data = json.loads(content)
    
    # Find and fix Field 50 mapping for MT202
    for task in data["tasks"]:
        if task["id"] == "map_field_50_agent_mt202":
            # Simplify logic: use InstgAgt.FinInstnId.BICFI
            task["function"]["input"]["mappings"][0]["logic"] = {
                "if": [
                    {
                        "and": [
                            {
                                "!": [
                                    {
                                        "or": [
                                            {"startsWith": [{"var": "data.ISO20022_MX.document.TxInf.OrgnlGrpInf.OrgnlMsgNmId"}, "pacs.008"]},
                                            {"startsWith": [{"var": "data.ISO20022_MX.document.TxInf.OrgnlGrpInf.OrgnlMsgNmId"}, "MT103"]},
                                            {
                                                "and": [
                                                    {"!": [{"var": "data.ISO20022_MX.document.TxInf.OrgnlGrpInf.OrgnlMsgNmId"}]},
                                                    {
                                                        "!": [
                                                            {
                                                                "and": [
                                                                    {"var": "data.ISO20022_MX.document.TxInf.RtrChain.Dbtr.Agt"},
                                                                    {"var": "data.ISO20022_MX.document.TxInf.RtrChain.Cdtr.Agt"}
                                                                ]
                                                            }
                                                        ]
                                                    }
                                                ]
                                            }
                                        ]
                                    }
                                ]
                            },
                            {"var": "data.ISO20022_MX.document.TxInf.InstgAgt.FinInstnId.BICFI"}
                        ]
                    },
                    {"A": {"bic": {"var": "data.ISO20022_MX.document.TxInf.InstgAgt.FinInstnId.BICFI"}}},
                    None
                ]
            }
            
        if task["id"] == "map_field_58_agent_mt202":
            # Simplify logic: use InstdAgt.FinInstnId.BICFI
            task["function"]["input"]["mappings"][0]["logic"] = {
                "if": [
                    {
                        "and": [
                            {
                                "!": [
                                    {
                                        "or": [
                                            {"startsWith": [{"var": "data.ISO20022_MX.document.TxInf.OrgnlGrpInf.OrgnlMsgNmId"}, "pacs.008"]},
                                            {"startsWith": [{"var": "data.ISO20022_MX.document.TxInf.OrgnlGrpInf.OrgnlMsgNmId"}, "MT103"]},
                                            {
                                                "and": [
                                                    {"!": [{"var": "data.ISO20022_MX.document.TxInf.OrgnlGrpInf.OrgnlMsgNmId"}]},
                                                    {
                                                        "!": [
                                                            {
                                                                "and": [
                                                                    {"var": "data.ISO20022_MX.document.TxInf.RtrChain.Dbtr.Agt"},
                                                                    {"var": "data.ISO20022_MX.document.TxInf.RtrChain.Cdtr.Agt"}
                                                                ]
                                                            }
                                                        ]
                                                    }
                                                ]
                                            }
                                        ]
                                    }
                                ]
                            },
                            {"var": "data.ISO20022_MX.document.TxInf.InstdAgt.FinInstnId.BICFI"}
                        ]
                    },
                    {"A": {"bic": {"var": "data.ISO20022_MX.document.TxInf.InstdAgt.FinInstnId.BICFI"}}},
                    None
                ]
            }
    
    # Write back the fixed JSON
    with open("/Users/codetiger/Development/Plasmatic/Reframe/workflows/reverse/pacs004/07-party-fields-mapping.json", "w") as f:
        json.dump(data, f, indent=4)
    
    print("✅ JSON fixed successfully")
    
except json.JSONDecodeError as e:
    print(f"❌ JSON error: {e}")
    sys.exit(1)