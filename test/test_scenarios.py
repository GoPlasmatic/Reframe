#!/usr/bin/env python3
"""
Refactored test script for MT and MX message scenarios using the Reframe transformation service.

This refactored version provides:
- Better separation of concerns
- Cleaner code organization
- Easier maintenance and extension
- More robust error handling
"""

import json
import requests
import argparse
import sys
import time
from datetime import datetime
from pathlib import Path
from typing import List, Dict, Any, Optional, Tuple, Protocol
from collections import defaultdict
from dataclasses import dataclass, field
from enum import Enum
from tabulate import tabulate


# ==================== Configuration ====================

class TestStatus(Enum):
    """Test result statuses"""
    SUCCESS = "✅"
    FAILURE = "❌"
    WARNING = "⚠️"
    SKIPPED = "—"


@dataclass
class APIEndpoints:
    """API endpoint configuration"""
    base_url: str
    generate_sample: str = "/generate/sample"
    validate_mt: str = "/validate/mt"
    validate_mx: str = "/validate/mx"
    transform_mt_to_mx: str = "/transform/mt-to-mx"
    transform_mx_to_mt: str = "/transform/mx-to-mt"
    
    def __post_init__(self):
        """Ensure all endpoints are properly formatted"""
        for attr in ['generate_sample', 'validate_mt', 'validate_mx', 
                     'transform_mt_to_mx', 'transform_mx_to_mt']:
            endpoint = getattr(self, attr)
            if not endpoint.startswith('/'):
                setattr(self, attr, '/' + endpoint)


@dataclass
class ScenarioMapping:
    """Scenario mapping configuration"""
    mappings: Dict[str, Dict[str, str]] = field(default_factory=dict)
    
    @classmethod
    def load_default(cls):
        """Load default scenario mappings"""
        return cls(mappings={
            "MT101": {
                "bulk": "standard",
                "pain001": "standard",
                "default": "standard"
            },
            "MT103": {
                "high_value": "high_value",
                "remittance": "remittance_enhanced",
                "stp": "stp",
                "cbpr_standard": "standard",
                "cbpr_high_value": "high_value",
                "cbpr_remittance": "remittance_enhanced",
                "rejt": "rejection",
                "retn": "return",
                "default": "standard"
            },
            "MT192": {
                "cancellation": "request_cancellation",
                "camt056": "request_cancellation",
                "default": "request_cancellation"
            },
            "MT196": {
                "resolution": "answer_cancellation",
                "camt029": "answer_cancellation",
                "customer_resolution": "answer_cancellation",
                "default": "answer_cancellation"
            },
            "MT202": {
                "cov": "cbpr_cov_standard",
                "cover": "cbpr_cov_standard",
                "core": "cbpr_cov_standard",
                "rejt": "cbpr_cov_standard",
                "retn": "cbpr_cov_standard",
                "default": "cbpr_cov_standard"
            },
            "MT205": {
                "cov": "bank_transfer_cover",
                "serial": "bank_transfer_non_cover",
                "cover": "bank_transfer_cover",
                "rejt": "rejection_payment",
                "retn": "return_payment",
                "default": "bank_transfer_non_cover"
            },
            "MT292": {
                "cancellation": "fi_cancellation_request",
                "camt056": "fi_cancellation_request",
                "default": "fi_cancellation_request"
            },
            "MT296": {
                "resolution": "cancellation_accepted",
                "camt029": "cancellation_accepted",
                "fi_resolution": "cancellation_accepted",
                "default": "cancellation_accepted"
            },
            "MT900": {
                "debit": "basic_debit_confirmation",
                "camt054": "basic_debit_confirmation",
                "default": "basic_debit_confirmation"
            },
            "MT910": {
                "credit": "basic_credit_confirmation",
                "camt054": "basic_credit_confirmation",
                "default": "basic_credit_confirmation"
            },
            # MX message mappings
            "PACS.002": {
                "rejt": "stop_payment_rejected",
                "mt103rejt": "stop_payment_rejected",
                "default": "stop_payment_rejected"
            },
            "PACS.004": {
                "mt103retn": "cbpr_compliant_return",
                "mt202retn": "cbpr_compliant_return",
                "mt205retn": "cbpr_compliant_return",
                "default": "cbpr_compliant_return"
            },
            "PACS.008": {
                "cbpr_standard": "cbpr_business_payment",
                "cbpr_stp": "cbpr_commission_payment",
                "stp": "cbpr_commission_payment",
                "default": "cbpr_business_payment"
            },
            "PACS.009": {
                "core": "bank_transfer_non_cover",
                "cov": "bank_transfer_cover",
                "cover": "bank_transfer_cover",
                "adv": "bank_transfer_non_cover",
                "serial": "bank_transfer_non_cover",
                "default": "bank_transfer_non_cover"
            },
            "CAMT.052": {
                "mt942": "daily_balance_report",
                "default": "daily_balance_report"
            },
            "CAMT.053": {
                "mt940": "daily_account_statement",
                "default": "daily_account_statement"
            },
            "CAMT.107": {
                "mt110": "cbpr_cross_border_cheque",
                "default": "cbpr_cross_border_cheque"
            },
            "CAMT.108": {
                "mt111": "cbpr_lost_cheque_cancellation",
                "default": "cbpr_lost_cheque_cancellation"
            },
            "CAMT.109": {
                "mt112": "cbpr_stop_confirmation_report",
                "default": "cbpr_stop_confirmation_report"
            }
        })
    
    def get_fallback(self, message_type: str, scenario: str) -> str:
        """Get fallback scenario for a given message type and scenario"""
        msg_type_upper = message_type.upper()
        scenario_lower = scenario.lower()
        
        type_mappings = self.mappings.get(msg_type_upper, {})
        
        # Try to find best match
        for keyword, mapped_scenario in type_mappings.items():
            if keyword != "default" and keyword in scenario_lower:
                return mapped_scenario
        
        # Return default for message type or generic default
        return type_mappings.get("default", "standard")


@dataclass
class TestResult:
    """Test result data structure"""
    message_type: str
    scenario: str
    description: str = ""
    sample: int = 1
    generation: TestStatus = TestStatus.FAILURE
    validation: TestStatus = TestStatus.FAILURE
    transformation: TestStatus = TestStatus.FAILURE
    roundtrip: TestStatus = TestStatus.FAILURE
    errors: List[str] = field(default_factory=list)
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for export"""
        return {
            "message_type": self.message_type,
            "scenario": self.scenario,
            "description": self.description,
            "sample": self.sample,
            "generation": self.generation.value,
            "validation": self.validation.value,
            "transformation": self.transformation.value,
            "roundtrip": self.roundtrip.value,
            "errors": self.errors
        }


# ==================== API Client ====================

class ReframeAPIClient:
    """Client for interacting with Reframe API"""
    
    def __init__(self, endpoints: APIEndpoints, debug: bool = False):
        self.endpoints = endpoints
        self.debug = debug
        self.session = requests.Session()
    
    def _log_debug(self, message: str):
        """Log debug message if debug mode is enabled"""
        if self.debug:
            print(f"DEBUG: {message}")
    
    def _make_request(self, method: str, endpoint: str, **kwargs) -> requests.Response:
        """Make HTTP request with error handling"""
        url = f"{self.endpoints.base_url}{endpoint}"
        
        if self.debug:
            self._log_debug(f"{method} {url}")
            if 'json' in kwargs:
                # Truncate large payloads for debug output
                debug_data = kwargs['json'].copy() if isinstance(kwargs['json'], dict) else kwargs['json']
                if isinstance(debug_data, dict):
                    for key in ['message', 'schema', 'config']:
                        if key in debug_data and len(str(debug_data[key])) > 500:
                            debug_data[key] = str(debug_data[key])[:500] + "..."
                self._log_debug(f"Request body: {json.dumps(debug_data, indent=2)}")
        
        try:
            response = self.session.request(method, url, **kwargs)
            
            if self.debug:
                self._log_debug(f"Response status: {response.status_code}")
                if response.status_code == 200:
                    try:
                        result = response.json()
                        # Truncate large responses
                        if 'result' in result and len(str(result['result'])) > 500:
                            debug_result = {**result, 'result': str(result['result'])[:500] + '...'}
                            self._log_debug(f"Response body: {json.dumps(debug_result, indent=2)}")
                        else:
                            self._log_debug(f"Response body: {json.dumps(result, indent=2)}")
                    except:
                        self._log_debug(f"Response body: {response.text[:500]}")
                else:
                    self._log_debug(f"Response body: {response.text[:500]}")
            
            return response
        except Exception as e:
            self._log_debug(f"Request failed: {str(e)}")
            raise
    
    def generate_sample(self, message_type: str, config: Dict[str, Any]) -> Optional[Any]:
        """Generate a sample message"""
        response = self._make_request(
            "POST",
            self.endpoints.generate_sample,
            json={"message_type": message_type, "config": config}
        )
        
        if response.status_code == 200:
            result = response.json()
            if result.get("success"):
                return result.get("result")
        return None
    
    def validate_mt(self, message: str) -> Tuple[bool, List[str]]:
        """Validate MT message"""
        response = self._make_request(
            "POST",
            self.endpoints.validate_mt,
            json={"message": message, "options": {"canonical": True}}
        )
        
        errors = []
        if response.status_code == 200:
            result = response.json()
            if not result.get("success"):
                api_errors = result.get("errors", [])
                for err in api_errors:
                    errors.append(f"{err.get('code', 'UNKNOWN')}: {err.get('message', 'Unknown error')}")
            return result.get("success", False), errors
        return False, [f"HTTP {response.status_code}"]
    
    def validate_mx(self, message: Any) -> Tuple[bool, List[str]]:
        """Validate MX message"""
        # Convert to string if needed
        message_str = json.dumps(message) if isinstance(message, dict) else message
        
        response = self._make_request(
            "POST",
            self.endpoints.validate_mx,
            json={"message": message_str, "options": {"canonical": True}}
        )
        
        errors = []
        if response.status_code == 200:
            result = response.json()
            if not result.get("success"):
                api_errors = result.get("errors", [])
                for err in api_errors:
                    errors.append(f"{err.get('code', 'UNKNOWN')}: {err.get('message', 'Unknown error')}")
            return result.get("success", False), errors
        return False, [f"HTTP {response.status_code}"]
    
    def transform_mt_to_mx(self, message: str) -> Optional[str]:
        """Transform MT to MX"""
        response = self._make_request(
            "POST",
            self.endpoints.transform_mt_to_mx,
            json={"message": message, "options": {"debug": True}}
        )
        
        if response.status_code == 200:
            result = response.json()
            if result.get("success"):
                return result.get("result")
        return None
    
    def transform_mx_to_mt(self, message: Any) -> Optional[str]:
        """Transform MX to MT"""
        # Convert to string if needed
        message_str = json.dumps(message) if isinstance(message, dict) else message
        
        response = self._make_request(
            "POST",
            self.endpoints.transform_mx_to_mt,
            json={"message": message_str, "options": {"debug": True}}
        )
        
        if response.status_code == 200:
            result = response.json()
            if result.get("success"):
                return result.get("result")
        return None


# ==================== Scenario Management ====================

class ScenarioManager:
    """Manages scenario discovery and loading"""
    
    def __init__(self, transformation_index_path: Path = Path("scenarios/transformation/index.json")):
        self.index_path = transformation_index_path
    
    def discover_message_types(self) -> Dict[str, List[str]]:
        """Discover all available message types from transformation scenarios"""
        message_types = {"MT": set(), "MX": set()}
        
        if not self.index_path.exists():
            return {"MT": [], "MX": []}
        
        try:
            with open(self.index_path, 'r') as f:
                data = json.load(f)
                
                # Process forward transformations (MT -> MX)
                for scenario in data.get("forward", []):
                    source = scenario.get("source", "")
                    if source:
                        message_types["MT"].add(source.upper())
                
                # Process reverse transformations (MX -> MT)
                for scenario in data.get("reverse", []):
                    source = scenario.get("source", "")
                    if source:
                        message_types["MX"].add(source)
        
        except Exception as e:
            print(f"Error loading transformation index: {e}")
            return {"MT": [], "MX": []}
        
        # Convert sets to sorted lists
        return {
            "MT": sorted(list(message_types["MT"])),
            "MX": sorted(list(message_types["MX"]))
        }
    
    def load_scenarios_for_type(self, message_type: str) -> List[Dict[str, str]]:
        """Load scenarios for a specific message type"""
        if not self.index_path.exists():
            return []
        
        try:
            with open(self.index_path, 'r') as f:
                data = json.load(f)
                
                scenario_list = []
                normalized_type = message_type.upper() if message_type.upper().startswith("MT") else message_type
                
                # Search in both forward and reverse transformations
                all_scenarios = data.get("forward", []) + data.get("reverse", [])
                
                for scenario in all_scenarios:
                    source = scenario.get("source", "")
                    normalized_source = source.upper() if source.upper().startswith("MT") else source
                    
                    if normalized_source == normalized_type:
                        file_path = scenario.get("file", "")
                        if file_path:
                            filename = Path(file_path).stem
                            scenario_list.append({
                                "name": filename,
                                "description": scenario.get("description", ""),
                                "target": scenario.get("target", ""),
                                "file": file_path
                            })
                
                return scenario_list
        except Exception:
            return []


# ==================== Test Engine ====================

class MessageGenerator:
    """Handles message generation logic"""
    
    def __init__(self, api_client: ReframeAPIClient, scenario_mapping: ScenarioMapping):
        self.api = api_client
        self.mapping = scenario_mapping
    
    def generate(self, message_type: str, scenario_path: str) -> Tuple[Optional[Any], str]:
        """Generate a message using schema or fallback to standard scenarios"""
        
        # Try to load transformation scenario
        if "forward/" in scenario_path or "reverse/" in scenario_path:
            scenario_file = Path(f"scenarios/transformation/{scenario_path}.json")
        else:
            scenario_file = Path(f"scenarios/transformation/{Path(scenario_path).stem}.json")
        
        if scenario_file.exists():
            try:
                with open(scenario_file, 'r') as f:
                    scenario_data = json.load(f)
                    
                    # Try schema-based generation
                    if "schema" in scenario_data:
                        message = self.api.generate_sample(
                            message_type,
                            {"schema": scenario_data["schema"]}
                        )
                        if message:
                            format_type = "MT" if message_type.upper().startswith("MT") else "MX"
                            return message, format_type
            except Exception:
                pass
        
        # Fallback to standard scenario
        fallback_scenario = self.mapping.get_fallback(message_type, Path(scenario_path).stem)
        message = self.api.generate_sample(message_type, {"scenario": fallback_scenario})
        
        if message:
            format_type = "MT" if message_type.upper().startswith("MT") else "MX"
            return message, format_type
        
        return None, ""


class ScenarioTester:
    """Main test orchestrator"""
    
    def __init__(self, api_client: ReframeAPIClient, scenario_manager: ScenarioManager,
                 message_generator: MessageGenerator):
        self.api = api_client
        self.scenarios = scenario_manager
        self.generator = message_generator
        self.statistics = defaultdict(int)
    
    def test_roundtrip(self, message: Any, format_type: str) -> bool:
        """Test roundtrip transformation"""
        try:
            if format_type == "MT":
                # MT -> MX -> MT
                mx_result = self.api.transform_mt_to_mx(message)
                if mx_result:
                    mt_result = self.api.transform_mx_to_mt(mx_result)
                    return bool(mt_result and ":" in mt_result)
            elif format_type == "MX":
                # MX -> MT -> MX
                mt_result = self.api.transform_mx_to_mt(message)
                if mt_result:
                    mx_result = self.api.transform_mt_to_mx(mt_result)
                    return bool(mx_result)
            return False
        except Exception:
            return False
    
    def test_scenario(self, message_type: str, scenario_info: Dict[str, str]) -> TestResult:
        """Test a single scenario"""
        result = TestResult(
            message_type=message_type,
            scenario=scenario_info["name"],
            description=scenario_info.get("description", "")
        )
        
        # Generate message
        scenario_path = scenario_info.get("file", scenario_info["name"])
        message, format_type = self.generator.generate(message_type, scenario_path)
        
        if not message:
            result.errors.append("Generation failed")
            return result
        
        result.generation = TestStatus.SUCCESS
        self.statistics["generation_success"] += 1
        
        # Validate message
        if format_type == "MT":
            is_valid, errors = self.api.validate_mt(message)
        else:
            is_valid, errors = self.api.validate_mx(message)
        
        if is_valid:
            result.validation = TestStatus.SUCCESS
            self.statistics["validation_success"] += 1
        else:
            result.errors.extend(errors)
        
        # Test transformation
        if format_type == "MT":
            transformed = self.api.transform_mt_to_mx(message)
            if transformed:
                result.transformation = TestStatus.SUCCESS
                self.statistics["transformation_success"] += 1
            else:
                result.errors.append("MT to MX transformation failed")
        else:
            transformed = self.api.transform_mx_to_mt(message)
            if transformed:
                result.transformation = TestStatus.SUCCESS
                self.statistics["transformation_success"] += 1
            else:
                result.errors.append("MX to MT transformation failed")
        
        # Test roundtrip if validation passed
        if result.validation == TestStatus.SUCCESS:
            if self.test_roundtrip(message, format_type):
                result.roundtrip = TestStatus.SUCCESS
                self.statistics["roundtrip_success"] += 1
            else:
                result.roundtrip = TestStatus.WARNING
        else:
            result.roundtrip = TestStatus.SKIPPED
        
        self.statistics["total"] += 1
        self.statistics[f"by_type_{message_type}"] += 1
        
        return result
    
    def test_message_type(self, message_type: str, 
                          specific_scenarios: Optional[List[str]] = None,
                          sample_count: int = 1) -> List[TestResult]:
        """Test all scenarios for a message type"""
        results = []
        
        # Load scenarios
        scenarios = self.scenarios.load_scenarios_for_type(message_type)
        if not scenarios:
            print(f"Warning: No scenarios found for {message_type}")
            return results
        
        # Filter if specific scenarios requested
        if specific_scenarios:
            scenarios = [s for s in scenarios if s["name"] in specific_scenarios]
            if not scenarios:
                print(f"Warning: None of the specified scenarios found for {message_type}")
                return results
        
        print(f"\nTesting {message_type} with {len(scenarios)} scenario(s), {sample_count} sample(s) each...")
        
        for scenario_info in scenarios:
            for sample_num in range(1, sample_count + 1):
                result = self.test_scenario(message_type, scenario_info)
                result.sample = sample_num
                results.append(result)
                
                # Small delay between tests
                if sample_count > 1 or len(scenarios) > 1:
                    time.sleep(0.1)
        
        return results


# ==================== Output and Reporting ====================

class ResultsReporter:
    """Handles results reporting and output"""
    
    @staticmethod
    def print_table(results: List[TestResult]):
        """Print results in a formatted table"""
        if not results:
            print("No results to display")
            return
        
        # Group by message type
        grouped = defaultdict(list)
        for r in results:
            grouped[r.message_type].append(r)
        
        # Prepare table data
        table_data = []
        for msg_type in sorted(grouped.keys()):
            for r in grouped[msg_type]:
                scenario_display = r.scenario
                if len(scenario_display) > 30:
                    scenario_display = scenario_display[:27] + "..."
                
                row = [
                    r.message_type,
                    scenario_display,
                    r.sample,
                    r.generation.value,
                    r.validation.value,
                    r.transformation.value,
                    r.roundtrip.value
                ]
                
                # Add error summary
                if r.errors:
                    error_summary = r.errors[0][:20] if r.errors else "Error"
                    row.append(error_summary)
                else:
                    row.append("")
                
                table_data.append(row)
        
        headers = ["Message Type", "Scenario", "Sample", "Generator", 
                   "Validator", "Transform", "Round Trip", "Errors"]
        print("\n" + tabulate(table_data, headers=headers, tablefmt="grid"))
    
    @staticmethod
    def print_summary(statistics: Dict[str, int]):
        """Print test summary"""
        print("\n" + "="*80)
        print("TEST SUMMARY")
        print("="*80)
        
        total = statistics.get("total", 0)
        if total == 0:
            print("No tests were run")
            return
        
        print(f"Total tests: {total}")
        for metric in ["generation_success", "validation_success", 
                       "transformation_success", "roundtrip_success"]:
            count = statistics.get(metric, 0)
            percentage = 100 * count / total
            print(f"{metric.replace('_', ' ').title()}: {count}/{total} ({percentage:.1f}%)")
        
        # Print by message type
        mt_types = {}
        mx_types = {}
        for key, value in statistics.items():
            if key.startswith("by_type_"):
                msg_type = key.replace("by_type_", "")
                if msg_type.startswith("MT"):
                    mt_types[msg_type] = value
                else:
                    mx_types[msg_type] = value
        
        if mt_types or mx_types:
            print("\nTests by Message Type:")
            if mt_types:
                print("  MT Messages:")
                for msg_type, count in sorted(mt_types.items()):
                    print(f"    {msg_type}: {count}")
            if mx_types:
                print("  MX Messages:")
                for msg_type, count in sorted(mx_types.items()):
                    print(f"    {msg_type}: {count}")
    
    @staticmethod
    def export_results(results: List[TestResult], statistics: Dict[str, int], 
                       base_url: str, filename: Optional[str] = None):
        """Export results to JSON file"""
        if filename is None:
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            filename = f"test_results_{timestamp}.json"
        
        log_dir = Path("test/logs")
        log_dir.mkdir(parents=True, exist_ok=True)
        filepath = log_dir / filename
        
        export_data = {
            "timestamp": datetime.now().isoformat(),
            "base_url": base_url,
            "statistics": dict(statistics),
            "results": [r.to_dict() for r in results]
        }
        
        with open(filepath, 'w') as f:
            json.dump(export_data, f, indent=2)
        
        print(f"\nResults exported to: {filepath}")


# ==================== Main Application ====================

def main():
    parser = argparse.ArgumentParser(
        description="Test MT and MX message scenarios",
        formatter_class=argparse.RawDescriptionHelpFormatter
    )
    
    parser.add_argument("--message-type", "-m", help="Message type to test")
    parser.add_argument("--scenario", "-s", nargs="+", help="Specific scenario(s)")
    parser.add_argument("--sample-count", "-c", type=int, default=1, help="Samples per scenario")
    parser.add_argument("--all-mx", action="store_true", help="Test all MX types")
    parser.add_argument("--all-mt", action="store_true", help="Test all MT types")
    parser.add_argument("--debug", "-d", action="store_true", help="Enable debug output")
    parser.add_argument("--export", "-e", action="store_true", help="Export results to JSON")
    parser.add_argument("--base-url", "-u", default="http://localhost:3000", help="API base URL")
    parser.add_argument("--list-types", "-l", action="store_true", help="List message types")
    parser.add_argument("--list-scenarios", action="store_true", help="List scenarios for type")
    
    args = parser.parse_args()
    
    # Initialize components
    endpoints = APIEndpoints(base_url=args.base_url)
    api_client = ReframeAPIClient(endpoints, debug=args.debug)
    scenario_manager = ScenarioManager()
    scenario_mapping = ScenarioMapping.load_default()
    message_generator = MessageGenerator(api_client, scenario_mapping)
    tester = ScenarioTester(api_client, scenario_manager, message_generator)
    
    # Handle listing operations
    if args.list_types:
        types = scenario_manager.discover_message_types()
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
    
    if args.list_scenarios:
        if not args.message_type:
            print("Error: --list-scenarios requires --message-type")
            sys.exit(1)
        
        scenarios = scenario_manager.load_scenarios_for_type(args.message_type)
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
        message_types_to_test = [args.message_type]
    elif args.all_mx:
        types = scenario_manager.discover_message_types()
        message_types_to_test = types["MX"]
        print(f"Testing all {len(message_types_to_test)} MX message types...")
    elif args.all_mt:
        types = scenario_manager.discover_message_types()
        message_types_to_test = types["MT"]
        print(f"Testing all {len(message_types_to_test)} MT message types...")
    else:
        # Default: test all MX types
        types = scenario_manager.discover_message_types()
        message_types_to_test = types["MX"]
        print(f"Testing all {len(message_types_to_test)} MX message types (default)...")
    
    # Run tests
    all_results = []
    for msg_type in message_types_to_test:
        results = tester.test_message_type(msg_type, args.scenario, args.sample_count)
        all_results.extend(results)
    
    # Report results
    ResultsReporter.print_table(all_results)
    ResultsReporter.print_summary(tester.statistics)
    
    if args.export:
        ResultsReporter.export_results(all_results, tester.statistics, args.base_url)
    
    # Exit with appropriate code
    success_rate = tester.statistics.get("validation_success", 0) / max(tester.statistics.get("total", 1), 1)
    sys.exit(0 if success_rate >= 0.95 else 1)


if __name__ == "__main__":
    main()