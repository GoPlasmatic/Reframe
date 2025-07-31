#!/usr/bin/env python3
"""
Test MT message scenarios using the Reframe transformation service.
This script generates MT messages from scenarios and tests their transformation to ISO 20022.
"""

import json
import requests
import argparse
import sys
from datetime import datetime
from pathlib import Path
from typing import List, Dict, Any, Optional
import time
from collections import defaultdict
import re

class MTScenarioTester:
    def __init__(self, base_url: str = "http://localhost:3000"):
        self.base_url = base_url
        self.results = []
        self.statistics = {
            "total": 0,
            "generation_success": 0,
            "transformation_success": 0,
            "validation_success": 0,
            "by_doc_type": defaultdict(int),
            "by_method": defaultdict(int),
            "errors": []
        }
    
    def discover_scenarios(self, message_type: str) -> List[str]:
        """Load scenarios from index.json file for the message type"""
        message_type_lower = message_type.lower()
        
        # Try different possible paths
        possible_paths = [
            Path(f"/Users/codetiger/Development/Plasmatic/Reframe/test_scenarios/{message_type_lower}/index.json"),
            Path(f"../test_scenarios/{message_type_lower}/index.json"),
            Path(f"./test_scenarios/{message_type_lower}/index.json"),
            Path(f"test_scenarios/{message_type_lower}/index.json")
        ]
        
        for path in possible_paths:
            if path.exists():
                try:
                    with open(path, 'r') as f:
                        data = json.load(f)
                        scenarios = data.get("scenarios", [])
                        print(f"Loaded {len(scenarios)} scenarios from {path}")
                        return scenarios
                except Exception as e:
                    print(f"Error loading scenarios from {path}: {e}")
        
        # Fallback to default scenarios if index.json not found
        print(f"No index.json found for {message_type}, using default scenarios")
        return ["standard", "high_value", "remittance_enhanced", "cover_payment", "rejection", "return"]
    
    def generate_mt_message(self, message_type: str, scenario: str = "standard") -> Optional[str]:
        """Generate an MT message using the sample generator API"""
        try:
            response = requests.post(
                f"{self.base_url}/generate/mt-sample",
                json={
                    "message_type": message_type,
                    "config": {"scenario": scenario}
                }
            )
            
            if response.status_code == 200:
                result = response.json()
                if result.get("success"):
                    self.statistics["generation_success"] += 1
                    return result.get("transformed_message")
                else:
                    error = f"Generation failed: {result.get('error', 'Unknown error')}"
                    self.statistics["errors"].append({"scenario": scenario, "error": error})
                    return None
            else:
                error = f"HTTP {response.status_code}: {response.text}"
                self.statistics["errors"].append({"scenario": scenario, "error": error})
                return None
        except Exception as e:
            error = f"Exception during generation: {str(e)}"
            self.statistics["errors"].append({"scenario": scenario, "error": error})
            return None
    
    def transform_mt_to_mx(self, mt_message: str) -> Optional[Dict[str, Any]]:
        """Transform MT message to MX using the transformation API"""
        try:
            response = requests.post(
                f"{self.base_url}/transform/mt-to-mx",
                json={"message": mt_message}
            )
            
            if response.status_code == 200:
                result = response.json()
                if result.get("success"):
                    self.statistics["transformation_success"] += 1
                    return result
                else:
                    error = f"Transformation failed: {json.dumps(result.get('errors', []))}"
                    self.statistics["errors"].append({"error": error})
                    # Still return the result for partial info extraction
                    return result
            else:
                error = f"HTTP {response.status_code}: {response.text}"
                self.statistics["errors"].append({"error": error})
                return None
        except Exception as e:
            error = f"Exception during transformation: {str(e)}"
            self.statistics["errors"].append({"error": error})
            return None
    
    def extract_mx_info(self, transformation_result: Dict[str, Any]) -> Dict[str, Any]:
        """Extract key information from MX transformation result"""
        mx_info = {
            "valid": False,
            "doc_type": "unknown",
            "biz_svc": None,
            "msg_id": None,
            "errors": []
        }
        
        # Extract from transformed_message XML
        if "transformed_message" in transformation_result:
            xml = transformation_result["transformed_message"]
            
            # Handle null/None case
            if xml is None:
                return mx_info
            
            # Handle list case (API might return list with single string)
            if isinstance(xml, list) and len(xml) > 0:
                xml = xml[0]
            
            # Ensure xml is a string
            if not isinstance(xml, str):
                mx_info["errors"] = [str(xml)]
                return mx_info
            
            # Handle escaped XML (json string with \\n)
            if '\\n' in xml:
                xml = xml.replace('\\n', '\n')
            
            # Extract document type
            doc_types = ["pacs.008", "pacs.002", "pacs.004", "pacs.009", "camt.054", "camt.056"]
            for doc_type in doc_types:
                if doc_type in xml:
                    mx_info["doc_type"] = doc_type
                    break
            
            # Check for versioned elements
            if re.search(r"FIToFIPaymentStatusReport(V\d+)?", xml):
                mx_info["doc_type"] = "pacs.002"
            elif re.search(r"PaymentReturn(V\d+)?", xml):
                mx_info["doc_type"] = "pacs.004"
            
            # Extract BizSvc
            if "<BizSvc>" in xml:
                mx_info["biz_svc"] = xml.split("<BizSvc>")[1].split("</BizSvc>")[0]
            
            # Extract message ID
            if "<MsgId>" in xml:
                mx_info["msg_id"] = xml.split("<MsgId>")[1].split("</MsgId>")[0]
            
            mx_info["valid"] = True
        
        return mx_info
    
    def extract_mt_info(self, mt_message: str) -> Dict[str, Any]:
        """Extract key information from MT message"""
        mt_info = {
            "fields": {},
            "blocks": {},
            "message_type": None,
            "method": "normal"
        }
        
        # Parse fields
        field_pattern = r':(\d+[A-Z]?):(.*?)(?=\n:|$)'
        matches = re.findall(field_pattern, mt_message, re.MULTILINE | re.DOTALL)
        
        for field_tag, field_value in matches:
            mt_info["fields"][field_tag] = field_value.strip()
        
        # Determine message type
        if "I103" in mt_message:
            mt_info["message_type"] = "MT103"
        elif "I202" in mt_message:
            mt_info["message_type"] = "MT202"
        elif "I900" in mt_message:
            mt_info["message_type"] = "MT900"
        
        # Determine method based on content
        if "72" in mt_info["fields"]:
            field_72 = mt_info["fields"]["72"]
            if "/REJT/" in field_72:
                mt_info["method"] = "rejection"
            elif "/RETN/" in field_72:
                mt_info["method"] = "return"
        
        if "53" in mt_info["fields"] and "54" in mt_info["fields"]:
            mt_info["method"] = "cover"
        
        return mt_info
    
    def test_scenario(self, message_type: str, scenario: str) -> Dict[str, Any]:
        """Test a single scenario"""
        result = {
            "message_type": message_type,
            "scenario": scenario,
            "generation": "❌",
            "transformation": "❌",
            "validation": "❌",
            "mt_info": {},
            "mx_info": {},
            "error": None
        }
        
        # Generate MT message
        mt_message = self.generate_mt_message(message_type, scenario)
        if not mt_message:
            result["error"] = "Failed to generate MT message"
            return result
        
        result["generation"] = "✅"
        result["mt_info"] = self.extract_mt_info(mt_message)
        
        # Transform to MX
        transformation_result = self.transform_mt_to_mx(mt_message)
        if not transformation_result:
            result["error"] = "Failed to transform MT to MX"
            return result
        
        result["transformation"] = "✅"
        result["mx_info"] = self.extract_mx_info(transformation_result)
        
        # Validate
        if result["mx_info"]["valid"]:
            result["validation"] = "✅"
            self.statistics["validation_success"] += 1
            self.statistics["by_doc_type"][result["mx_info"]["doc_type"]] += 1
            self.statistics["by_method"][result["mt_info"]["method"]] += 1
        
        return result
    
    def test_message_type(self, message_type: str, scenarios: Optional[List[str]] = None):
        """Test all scenarios for a message type"""
        if scenarios is None:
            scenarios = self.discover_scenarios(message_type)
        
        print(f"\nTesting {message_type} with {len(scenarios)} scenarios...")
        print("="*80)
        
        for i, scenario in enumerate(scenarios):
            self.statistics["total"] += 1
            print(f"\n[{i+1}/{len(scenarios)}] Testing scenario: {scenario}")
            
            result = self.test_scenario(message_type, scenario)
            self.results.append(result)
            
            # Print result
            print(f"  Generation: {result['generation']}")
            print(f"  Transformation: {result['transformation']}")
            print(f"  Validation: {result['validation']}")
            
            if result["mx_info"]:
                print(f"  Document Type: {result['mx_info']['doc_type']}")
                print(f"  Business Service: {result['mx_info']['biz_svc']}")
            
            if result["error"]:
                print(f"  Error: {result['error']}")
            
            # Small delay to avoid overwhelming the service
            time.sleep(0.1)
    
    def print_summary(self):
        """Print test summary"""
        print("\n" + "="*80)
        print("TEST SUMMARY")
        print("="*80)
        
        total = self.statistics["total"]
        if total == 0:
            print("No tests were run")
            return
        
        print(f"Total scenarios tested: {total}")
        print(f"Generation success: {self.statistics['generation_success']}/{total} ({100*self.statistics['generation_success']/total:.1f}%)")
        print(f"Transformation success: {self.statistics['transformation_success']}/{total} ({100*self.statistics['transformation_success']/total:.1f}%)")
        print(f"Validation success: {self.statistics['validation_success']}/{total} ({100*self.statistics['validation_success']/total:.1f}%)")
        
        if self.statistics["by_doc_type"]:
            print("\nBy Document Type:")
            for doc_type, count in sorted(self.statistics["by_doc_type"].items()):
                print(f"  {doc_type}: {count}")
        
        if self.statistics["by_method"]:
            print("\nBy Method:")
            for method, count in sorted(self.statistics["by_method"].items()):
                print(f"  {method}: {count}")
        
        if self.statistics["errors"]:
            print(f"\nErrors encountered: {len(self.statistics['errors'])}")
            for i, error in enumerate(self.statistics['errors'][:5]):
                print(f"  {i+1}. {error}")
            if len(self.statistics['errors']) > 5:
                print(f"  ... and {len(self.statistics['errors']) - 5} more")
    
    def export_results(self, filename: Optional[str] = None):
        """Export results to JSON file"""
        if filename is None:
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            filename = f"scenario_test_results_{timestamp}.json"
        
        # Ensure logs directory exists
        log_dir = Path("test/logs")
        log_dir.mkdir(parents=True, exist_ok=True)
        
        filepath = log_dir / filename
        
        export_data = {
            "timestamp": datetime.now().strftime("%Y%m%d_%H%M%S"),
            "base_url": self.base_url,
            "statistics": dict(self.statistics),
            "results": self.results
        }
        
        with open(filepath, 'w') as f:
            json.dump(export_data, f, indent=2)
        
        print(f"\nResults exported to: {filepath}")

def main():
    parser = argparse.ArgumentParser(description="Test MT message scenarios")
    parser.add_argument("--message-type", "-m", default="MT103", help="Message type to test (e.g., MT103, MT202)")
    parser.add_argument("--scenarios", "-s", nargs="+", help="Specific scenarios to test")
    parser.add_argument("--base-url", "-u", default="http://localhost:3000", help="Base URL of the transformation service")
    parser.add_argument("--export", "-e", action="store_true", help="Export results to JSON file")
    
    args = parser.parse_args()
    
    # Create tester
    tester = MTScenarioTester(args.base_url)
    
    # Run tests
    tester.test_message_type(args.message_type, args.scenarios)
    
    # Print summary
    tester.print_summary()
    
    # Export results if requested
    if args.export:
        tester.export_results()
    
    # Exit with appropriate code
    success_rate = tester.statistics["validation_success"] / max(tester.statistics["total"], 1)
    sys.exit(0 if success_rate >= 0.95 else 1)

if __name__ == "__main__":
    main()