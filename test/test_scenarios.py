#!/usr/bin/env python3
"""
Test MT and MX message scenarios using the Reframe transformation service.
This script generates messages from scenarios and tests their bidirectional transformation.

Updated to work with the new SampleGenerationResponse structure:
- Changed from 'transformed_message' and 'generated_message' to 'result' field
- Changed from 'scenario_used' to 'scenario' field
- Error handling now uses 'errors' array with ReframeError objects
"""

import json
import requests
import argparse
import sys
from datetime import datetime
from pathlib import Path
from typing import List, Dict, Any, Optional, Tuple
import time
from collections import defaultdict
import re
from tabulate import tabulate

class ScenarioTester:
    def __init__(self, base_url: str = "http://localhost:3000", debug: bool = False):
        self.base_url = base_url
        self.debug = debug
        self.results = []
        self.statistics = {
            "total": 0,
            "generation_success": 0,
            "transformation_success": 0,
            "validation_success": 0,
            "roundtrip_success": 0,
            "by_doc_type": defaultdict(int),
            "by_message_type": defaultdict(int),
            "errors": []
        }
    
    def discover_message_types(self) -> Dict[str, List[str]]:
        """Discover all available message types from scenario directories"""
        message_types = {"MT": [], "MX": []}
        
        # Discover MT message types
        mt_dir = Path("scenarios/SwiftMTMessage")
        if mt_dir.exists():
            for item in sorted(mt_dir.iterdir()):
                if item.is_dir() and not item.name.startswith('.'):
                    # Check if it has an index.json file
                    if (item / "index.json").exists():
                        message_types["MT"].append(item.name.upper())
        
        # Discover MX message types - try both local and parent directory
        mx_dirs = [Path("scenarios/MXMessage"), Path("../scenarios/MXMessage")]
        for mx_dir in mx_dirs:
            if mx_dir.exists():
                for item in sorted(mx_dir.iterdir()):
                    if item.is_dir() and not item.name.startswith('.'):
                        # Convert directory name to MX format (e.g., pacs008 -> pacs.008)
                        name = item.name
                        if name.startswith(('pacs', 'pain', 'camt')):
                            # Insert dot before numbers
                            formatted_name = name[:4] + '.' + name[4:]
                            if (item / "index.json").exists():
                                message_types["MX"].append(formatted_name)
                break  # Use first existing directory
        
        return message_types
    
    def load_scenarios_from_index(self, message_type: str) -> List[Dict[str, str]]:
        """Load scenarios from index.json file for the message type"""
        # Determine if MT or MX
        if message_type.upper().startswith("MT"):
            # MT message
            clean_type = message_type.lower()
            scenario_dirs = [Path(f"scenarios/SwiftMTMessage/{clean_type}")]
        else:
            # MX message - remove dots for directory name
            clean_type = message_type.replace(".", "")
            # Try both local and parent directory
            scenario_dirs = [
                Path(f"scenarios/MXMessage/{clean_type}"),
                Path(f"../scenarios/MXMessage/{clean_type}")
            ]
        
        # Find the first existing directory
        scenario_dir = None
        for dir_path in scenario_dirs:
            if dir_path.exists():
                scenario_dir = dir_path
                break
        
        if not scenario_dir:
            if self.debug:
                print(f"DEBUG: No scenario directory found for {message_type}")
            return []
        
        index_file = scenario_dir / "index.json"
        
        if not index_file.exists():
            if self.debug:
                print(f"DEBUG: No index.json found at {index_file}")
            return []
        
        try:
            with open(index_file, 'r') as f:
                data = json.load(f)
                scenarios = data.get("scenarios", [])
                
                scenario_list = []
                for scenario in scenarios:
                    if isinstance(scenario, dict):
                        # New format with file and description
                        filename = scenario.get("file", "")
                        if filename.endswith(".json"):
                            filename = filename[:-5]  # Remove .json extension
                        scenario_list.append({
                            "name": filename,
                            "description": scenario.get("description", "")
                        })
                    elif isinstance(scenario, str):
                        # Old format - just scenario names (string)
                        if scenario.endswith(".json"):
                            scenario = scenario[:-5]
                        scenario_list.append({
                            "name": scenario,
                            "description": ""
                        })
                
                if self.debug:
                    print(f"DEBUG: Loaded {len(scenario_list)} scenarios from {index_file}")
                
                return scenario_list
        except Exception as e:
            if self.debug:
                print(f"DEBUG: Error loading scenarios from {index_file}: {e}")
            return []
    
    def generate_message(self, message_type: str, scenario: str) -> Tuple[Optional[Any], str]:
        """Generate a message (MT or MX) using the sample generator API"""
        try:
            config = {"scenario": scenario} if scenario != "default" else {}
            
            response = requests.post(
                f"{self.base_url}/generate/sample",
                json={
                    "message_type": message_type,
                    "config": config
                }
            )
            
            if response.status_code == 200:
                result = response.json()
                if result.get("success"):
                    self.statistics["generation_success"] += 1
                    # Updated to use new SampleGenerationResponse structure
                    message = result.get("result")
                    format_type = "MT" if message_type.upper().startswith("MT") else "MX"
                    
                    # Store scenario info if available
                    if self.debug and result.get("scenario"):
                        print(f"DEBUG: Used scenario: {result.get('scenario')}")
                    
                    return message, format_type
                else:
                    errors = result.get('errors', [])
                    if errors:
                        error = errors[0].get('message', 'Unknown error')
                    else:
                        error = 'Unknown error'
                    if self.debug:
                        print(f"DEBUG: Generation failed: {error}")
                    return None, ""
            else:
                error = f"HTTP {response.status_code}: {response.text}"
                if self.debug:
                    print(f"DEBUG: Generation HTTP error: {error}")
                return None, ""
        except Exception as e:
            if self.debug:
                print(f"DEBUG: Generation exception: {str(e)}")
            return None, ""
    
    def validate_message(self, message: Any, format_type: str) -> Tuple[bool, List[str]]:
        """Validate the generated message using the validation API"""
        errors = []
        
        if format_type == "MT":
            # Call MT validation endpoint
            try:
                response = requests.post(
                    f"{self.base_url}/validate/mt",
                    json={"message": message}
                )
                if response.status_code == 200:
                    result = response.json()
                    if not result.get("success"):
                        api_errors = result.get("errors", [])
                        for err in api_errors:
                            errors.append(f"{err.get('code', 'UNKNOWN')}: {err.get('message', 'Unknown error')}")
                    return result.get("success", False), errors
            except Exception as e:
                errors.append(f"Validation API error: {str(e)}")
                return False, errors
                
        elif format_type == "MX":
            # Call MX validation endpoint
            try:
                # Convert to string if needed
                message_str = message
                if isinstance(message, dict):
                    import json as json_module
                    message_str = json_module.dumps(message)
                    
                response = requests.post(
                    f"{self.base_url}/validate/mx",
                    json={"message": message_str}
                )
                if response.status_code == 200:
                    result = response.json()
                    if not result.get("success"):
                        api_errors = result.get("errors", [])
                        for err in api_errors:
                            errors.append(f"{err.get('code', 'UNKNOWN')}: {err.get('message', 'Unknown error')}")
                    return result.get("success", False), errors
            except Exception as e:
                errors.append(f"Validation API error: {str(e)}")
                return False, errors
                
        return False, ["Unknown format type"]
    
    def transform_mt_to_mx(self, mt_message: str) -> Optional[str]:
        """Transform MT message to MX"""
        try:
            response = requests.post(
                f"{self.base_url}/transform/mt-to-mx",
                json={"message": mt_message}
            )
            
            if response.status_code == 200:
                result = response.json()
                if result.get("success"):
                    self.statistics["transformation_success"] += 1
                    return result.get("result")
            return None
        except Exception as e:
            if self.debug:
                print(f"DEBUG: MT to MX transformation error: {str(e)}")
            return None
    
    def transform_mx_to_mt(self, mx_message: Any) -> Optional[str]:
        """Transform MX message to MT"""
        try:
            # Convert JSON to string if needed
            if isinstance(mx_message, dict):
                import json as json_module
                mx_message_str = json_module.dumps(mx_message)
            else:
                mx_message_str = mx_message
            
            response = requests.post(
                f"{self.base_url}/transform/mx-to-mt",
                json={"message": mx_message_str}
            )
            
            if response.status_code == 200:
                result = response.json()
                if result.get("success"):
                    self.statistics["transformation_success"] += 1
                    return result.get("result")
            elif self.debug:
                print(f"DEBUG: MX to MT transformation failed with status {response.status_code}")
                print(f"DEBUG: Response: {response.text[:500]}")
            return None
        except Exception as e:
            if self.debug:
                print(f"DEBUG: MX to MT transformation error: {str(e)}")
            return None
    
    def test_roundtrip(self, original: Any, format_type: str) -> bool:
        """Test roundtrip transformation (MT->MX->MT or MX->MT->MX)"""
        try:
            if format_type == "MT":
                # MT -> MX -> MT
                mx_result = self.transform_mt_to_mx(original)
                if mx_result:
                    mt_result = self.transform_mx_to_mt(mx_result)
                    if mt_result:
                        # Basic check - both should be MT messages
                        return bool(mt_result and ":" in mt_result)
            elif format_type == "MX":
                # MX -> MT -> MX (if supported)
                mt_result = self.transform_mx_to_mt(original)
                if mt_result:
                    mx_result = self.transform_mt_to_mx(mt_result)
                    if mx_result:
                        # Basic check - transformation completed
                        return bool(mx_result)
            return False
        except Exception as e:
            if self.debug:
                print(f"DEBUG: Roundtrip test error: {str(e)}")
            return False
    
    def test_scenario(self, message_type: str, scenario_info: Dict[str, str], sample_num: int = 1) -> Dict[str, Any]:
        """Test a single scenario"""
        scenario = scenario_info["name"]
        description = scenario_info.get("description", "")
        
        result = {
            "message_type": message_type,
            "scenario": scenario,
            "description": description,
            "sample": sample_num,
            "generation": "❌",
            "validation": "❌",
            "transformation": "❌",
            "roundtrip": "❌",
            "errors": []
        }
        
        # Generate message
        message, format_type = self.generate_message(message_type, scenario)
        if not message:
            result["errors"].append("Generation failed")
            return result
        
        result["generation"] = "✅"
        
        # Validate message
        is_valid, validation_errors = self.validate_message(message, format_type)
        if is_valid:
            result["validation"] = "✅"
            self.statistics["validation_success"] += 1
        else:
            result["validation"] = "❌"
            result["errors"].extend(validation_errors)
        
        # Test transformation
        if format_type == "MT":
            transformed = self.transform_mt_to_mx(message)
            if transformed:
                result["transformation"] = "✅"
            else:
                result["errors"].append("MT to MX transformation failed")
        elif format_type == "MX":
            # Debug: Show message type
            if self.debug:
                print(f"DEBUG: MX message type: {type(message)}")
                if isinstance(message, dict):
                    print(f"DEBUG: MX message keys: {list(message.keys())[:5]}")
                elif isinstance(message, str):
                    print(f"DEBUG: MX message preview: {message[:200]}")
            
            # Test MX to MT transformation
            transformed = self.transform_mx_to_mt(message)
            if transformed:
                result["transformation"] = "✅"
            else:
                result["transformation"] = "❌"
                result["errors"].append("MX to MT transformation failed")
        
        # Test roundtrip only if validation passed
        if result["validation"] == "✅":
            if self.test_roundtrip(message, format_type):
                result["roundtrip"] = "✅"
                self.statistics["roundtrip_success"] += 1
            else:
                result["roundtrip"] = "⚠️"  # Warning - partial support
        else:
            result["roundtrip"] = "—"  # Skip roundtrip if validation failed
        
        self.statistics["by_message_type"][message_type] += 1
        
        return result
    
    def test_message_type(self, message_type: str, scenarios: Optional[List[str]] = None, 
                          sample_count: int = 1) -> List[Dict[str, Any]]:
        """Test all scenarios for a message type with multiple samples"""
        results = []
        
        # Load scenarios from index.json
        scenario_infos = self.load_scenarios_from_index(message_type)
        
        if not scenario_infos:
            print(f"Warning: No scenarios found for {message_type}")
            return results
        
        # Filter scenarios if specific ones requested
        if scenarios:
            scenario_infos = [s for s in scenario_infos if s["name"] in scenarios]
            if not scenario_infos:
                print(f"Warning: None of the specified scenarios found for {message_type}")
                return results
        
        print(f"\nTesting {message_type} with {len(scenario_infos)} scenario(s), {sample_count} sample(s) each...")
        
        for scenario_info in scenario_infos:
            for sample_num in range(1, sample_count + 1):
                self.statistics["total"] += 1
                result = self.test_scenario(message_type, scenario_info, sample_num)
                results.append(result)
                
                # Small delay between tests
                if sample_count > 1 or len(scenario_infos) > 1:
                    time.sleep(0.1)
        
        return results
    
    def print_results_table(self, results: List[Dict[str, Any]]):
        """Print results in a formatted table"""
        if not results:
            print("No results to display")
            return
        
        # Group results by message type for better readability
        grouped_results = defaultdict(list)
        for r in results:
            grouped_results[r["message_type"]].append(r)
        
        # Prepare table data
        table_data = []
        for msg_type in sorted(grouped_results.keys()):
            for r in grouped_results[msg_type]:
                scenario_display = r["scenario"]
                if len(scenario_display) > 30:
                    scenario_display = scenario_display[:27] + "..."
                
                row = [
                    r["message_type"],
                    scenario_display,
                    r["sample"],
                    r["generation"],
                    r["validation"],
                    r["transformation"],
                    r["roundtrip"]
                ]
                
                # Add error summary
                if r["errors"]:
                    # Show first error code if available
                    error_summary = r["errors"][0][:20] if r["errors"] else "Error"
                    row.append(error_summary)
                else:
                    row.append("")
                
                table_data.append(row)
        
        # Print table
        headers = ["Message Type", "Scenario", "Sample", "Generator", "Validator", "Transform", "Round Trip", "Errors"]
        print("\n" + tabulate(table_data, headers=headers, tablefmt="grid"))
        
        # Print detailed errors if any
        errors_found = [r for r in results if r["errors"]]
        if errors_found and self.debug:
            print("\nDetailed Errors:")
            for r in errors_found:
                print(f"  {r['message_type']} / {r['scenario']} (Sample {r['sample']}): {', '.join(r['errors'])}")
    
    def print_summary(self):
        """Print test summary"""
        print("\n" + "="*80)
        print("TEST SUMMARY")
        print("="*80)
        
        total = self.statistics["total"]
        if total == 0:
            print("No tests were run")
            return
        
        print(f"Total tests: {total}")
        print(f"Generation success: {self.statistics['generation_success']}/{total} ({100*self.statistics['generation_success']/total:.1f}%)")
        print(f"Validation success: {self.statistics['validation_success']}/{total} ({100*self.statistics['validation_success']/total:.1f}%)")
        print(f"Transformation success: {self.statistics['transformation_success']}/{total} ({100*self.statistics['transformation_success']/total:.1f}%)")
        print(f"Round trip success: {self.statistics['roundtrip_success']}/{total} ({100*self.statistics['roundtrip_success']/total:.1f}%)")
        
        if self.statistics["by_message_type"]:
            print("\nTests by Message Type:")
            # Separate MT and MX for clarity
            mt_types = {k: v for k, v in self.statistics["by_message_type"].items() if k.startswith("MT")}
            mx_types = {k: v for k, v in self.statistics["by_message_type"].items() if not k.startswith("MT")}
            
            if mt_types:
                print("  MT Messages:")
                for msg_type, count in sorted(mt_types.items()):
                    print(f"    {msg_type}: {count}")
            
            if mx_types:
                print("  MX Messages:")
                for msg_type, count in sorted(mx_types.items()):
                    print(f"    {msg_type}: {count}")
    
    def export_results(self, results: List[Dict[str, Any]], filename: Optional[str] = None):
        """Export results to JSON file"""
        if filename is None:
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            filename = f"test_results_{timestamp}.json"
        
        # Ensure logs directory exists
        log_dir = Path("test/logs")
        log_dir.mkdir(parents=True, exist_ok=True)
        
        filepath = log_dir / filename
        
        export_data = {
            "timestamp": datetime.now().isoformat(),
            "base_url": self.base_url,
            "statistics": dict(self.statistics),
            "results": results
        }
        
        # Convert defaultdicts to regular dicts
        export_data["statistics"]["by_doc_type"] = dict(export_data["statistics"]["by_doc_type"])
        export_data["statistics"]["by_message_type"] = dict(export_data["statistics"]["by_message_type"])
        
        with open(filepath, 'w') as f:
            json.dump(export_data, f, indent=2)
        
        print(f"\nResults exported to: {filepath}")

def main():
    parser = argparse.ArgumentParser(
        description="Test MT and MX message scenarios",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Test all MX message types (default)
  python test_scenarios.py
  
  # Test all MX message types with all scenarios
  python test_scenarios.py --all-mx
  
  # Test specific message type with all scenarios
  python test_scenarios.py --message-type pacs.008
  
  # Test specific scenarios
  python test_scenarios.py -m pacs.008 -s standard high_value
  
  # Test with multiple samples per scenario
  python test_scenarios.py -m pacs.008 --sample-count 3
  
  # List all available message types
  python test_scenarios.py --list-types
  
  # Test with debug output
  python test_scenarios.py -m pacs.009 --debug
  
  # Export results to JSON
  python test_scenarios.py -m camt.054 --export
        """
    )
    
    parser.add_argument("--message-type", "-m", 
                       help="Message type to test (e.g., MT103, pacs.008)")
    parser.add_argument("--scenario", "-s", nargs="+", 
                       help="Specific scenario(s) to test")
    parser.add_argument("--sample-count", "-c", type=int, default=1,
                       help="Number of samples to generate per scenario (default: 1)")
    parser.add_argument("--all-mx", action="store_true",
                       help="Test all MX message types with all their scenarios")
    parser.add_argument("--all-mt", action="store_true",
                       help="Test all MT message types with all their scenarios")
    parser.add_argument("--debug", "-d", action="store_true",
                       help="Enable debug output")
    parser.add_argument("--export", "-e", action="store_true",
                       help="Export results to JSON file")
    parser.add_argument("--base-url", "-u", default="http://localhost:3000",
                       help="Base URL of the transformation service")
    parser.add_argument("--list-types", "-l", action="store_true",
                       help="List all available message types")
    parser.add_argument("--list-scenarios", action="store_true",
                       help="List scenarios for a message type (requires --message-type)")
    
    args = parser.parse_args()
    
    # Create tester
    tester = ScenarioTester(args.base_url, args.debug)
    
    # List available message types
    if args.list_types:
        print("Discovering available message types...")
        types = tester.discover_message_types()
        
        print("\n" + "="*50)
        print("AVAILABLE MESSAGE TYPES")
        print("="*50)
        
        if types["MT"]:
            print("\nMT Messages:")
            for mt_type in types["MT"]:
                print(f"  {mt_type}")
        
        if types["MX"]:
            print("\nMX Messages:")
            for mx_type in types["MX"]:
                print(f"  {mx_type}")
        
        print(f"\nTotal: {len(types['MT'])} MT types, {len(types['MX'])} MX types")
        sys.exit(0)
    
    # List scenarios for a message type
    if args.list_scenarios:
        if not args.message_type:
            print("Error: --list-scenarios requires --message-type")
            sys.exit(1)
        
        scenarios = tester.load_scenarios_from_index(args.message_type)
        if scenarios:
            print(f"\nScenarios for {args.message_type}:")
            print("="*50)
            for i, scenario in enumerate(scenarios, 1):
                desc = f" - {scenario['description']}" if scenario['description'] else ""
                print(f"{i:3}. {scenario['name']}{desc}")
            print(f"\nTotal: {len(scenarios)} scenarios")
        else:
            print(f"No scenarios found for {args.message_type}")
        sys.exit(0)
    
    # Determine what to test
    message_types_to_test = []
    
    if args.message_type:
        # Test specific message type
        message_types_to_test = [args.message_type]
    elif args.all_mx:
        # Test all MX types
        types = tester.discover_message_types()
        message_types_to_test = types["MX"]
        if not message_types_to_test:
            print("No MX message types found")
            sys.exit(1)
        print(f"Testing all {len(message_types_to_test)} MX message types...")
    elif args.all_mt:
        # Test all MT types
        types = tester.discover_message_types()
        message_types_to_test = types["MT"]
        if not message_types_to_test:
            print("No MT message types found")
            sys.exit(1)
        print(f"Testing all {len(message_types_to_test)} MT message types...")
    else:
        # Default: test all MX types
        types = tester.discover_message_types()
        message_types_to_test = types["MX"]
        if not message_types_to_test:
            print("No MX message types found. Use --message-type to specify a type.")
            sys.exit(1)
        print(f"Testing all {len(message_types_to_test)} MX message types (default behavior)...")
        print("Use --message-type to test a specific type, or --list-types to see all available types.\n")
    
    # Run tests for all selected message types
    all_results = []
    for msg_type in message_types_to_test:
        results = tester.test_message_type(
            msg_type,
            args.scenario,
            args.sample_count
        )
        all_results.extend(results)
    
    # Print results table
    tester.print_results_table(all_results)
    
    # Print summary
    tester.print_summary()
    
    # Export if requested
    if args.export:
        tester.export_results(all_results)
    
    # Exit with appropriate code
    success_rate = tester.statistics["validation_success"] / max(tester.statistics["total"], 1)
    sys.exit(0 if success_rate >= 0.95 else 1)

if __name__ == "__main__":
    main()