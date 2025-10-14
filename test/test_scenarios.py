#!/usr/bin/env python3
"""
Test script for MT and MX message scenarios using the Reframe transformation service.

Test Flow for Each Scenario:
1. Generate sample message using the Sample Generation API with message type and scenario
2. Validate the generated message structure using validation API
3. Transform the message using the Transformation API
"""

import json
import requests
import argparse
import sys
import time
from datetime import datetime
from pathlib import Path
from typing import List, Dict, Any, Optional, Tuple
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
    generate_sample: str = "/api/generate"
    validate: str = "/api/validate"
    transform: str = "/api/transform"

    def __post_init__(self):
        """Ensure all endpoints are properly formatted"""
        for attr in ['generate_sample', 'validate', 'transform']:
            endpoint = getattr(self, attr)
            if not endpoint.startswith('/'):
                setattr(self, attr, '/' + endpoint)




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
    
    def generate_sample(self, message_type: str, scenario: str, debug: bool = False) -> Optional[Any]:
        """Generate a sample message"""
        payload = {
            "message_type": message_type,
            "scenario": scenario,
            "debug": debug
        }
        response = self._make_request(
            "POST",
            self.endpoints.generate_sample,
            json=payload
        )

        if response.status_code == 200:
            result = response.json()
            if result.get("success"):
                return result.get("result")
        return None
    
    def validate(self, message: str, canonical: bool = True, business_validation: bool = False) -> Tuple[bool, List[str]]:
        """Validate message (MT or MX)"""
        # Convert to string if needed
        message_str = json.dumps(message) if isinstance(message, dict) else message

        response = self._make_request(
            "POST",
            self.endpoints.validate,
            json={
                "message": message_str,
                "canonical": canonical,
                "business_validation": business_validation
            }
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
    
    def transform(self, message: str, debug: bool = False) -> Optional[Any]:
        """Transform message (MT↔MX or MX↔MT)"""
        # Handle different message formats
        if isinstance(message, list) and len(message) > 0:
            # Result from transformation is a list with XML strings
            message_str = message[0] if isinstance(message[0], str) else str(message[0])
        elif isinstance(message, dict):
            message_str = json.dumps(message)
        else:
            message_str = str(message)

        response = self._make_request(
            "POST",
            self.endpoints.transform,
            json={
                "message": message_str,
                "debug": debug,
                "validation": True
            }
        )

        if response.status_code == 200:
            result = response.json()
            if result.get("success"):
                return result.get("result")
        return None


# ==================== Scenario Management ====================

class ScenarioManager:
    """Manages scenario discovery and loading from packages"""

    def __init__(self, config_path: Path = None):
        if config_path is None:
            config_path = Path("reframe.config.json")

        self.packages = []
        self.index_paths = []

        # Load packages from config
        if config_path.exists():
            try:
                with open(config_path, 'r') as f:
                    config = json.load(f)
                    for pkg in config.get("packages", []):
                        if pkg.get("enabled", True):
                            pkg_path = Path(pkg["path"])
                            scenarios_path = pkg_path / "scenarios" / "index.json"
                            if scenarios_path.exists():
                                self.packages.append(pkg_path)
                                self.index_paths.append(scenarios_path)
            except Exception as e:
                print(f"Warning: Failed to load config: {e}")

        # If no packages loaded, try default paths
        if not self.index_paths:
            for default_path in ["../reframe-package-swift-cbpr/scenarios/index.json",
                                "scenarios/index.json"]:
                p = Path(default_path)
                if p.exists():
                    self.index_paths.append(p)
                    break
    
    def discover_message_types(self) -> Dict[str, List[str]]:
        """Discover all available message types from all package scenarios"""
        message_types = {"MT": set(), "MX": set()}

        if not self.index_paths:
            return {"MT": [], "MX": []}

        for index_path in self.index_paths:
            try:
                with open(index_path, 'r') as f:
                    data = json.load(f)

                    # Process outgoing transformations (MT -> MX)
                    for scenario in data.get("outgoing", []):
                        source = scenario.get("source", "")
                        if source:
                            message_types["MT"].add(source.upper())

                    # Process incoming transformations (MX -> MT)
                    for scenario in data.get("incoming", []):
                        source = scenario.get("source", "")
                        if source:
                            message_types["MX"].add(source)

            except Exception as e:
                print(f"Error loading transformation index from {index_path}: {e}")
                continue

        # Convert sets to sorted lists
        return {
            "MT": sorted(list(message_types["MT"])),
            "MX": sorted(list(message_types["MX"]))
        }
    
    def load_scenarios_for_type(self, message_type: str) -> List[Dict[str, str]]:
        """Load scenarios for a specific message type from all packages"""
        if not self.index_paths:
            return []

        scenario_list = []
        normalized_type = message_type.upper() if message_type.upper().startswith("MT") else message_type

        for index_path in self.index_paths:
            try:
                with open(index_path, 'r') as f:
                    data = json.load(f)

                    # Search in both outgoing (MT->MX) and incoming (MX->MT) transformations
                    all_scenarios = data.get("outgoing", []) + data.get("incoming", [])

                    for scenario in all_scenarios:
                        source = scenario.get("source", "")
                        normalized_source = source.upper() if source.upper().startswith("MT") else source

                        if normalized_source == normalized_type:
                            scenario_id = scenario.get("id", "")
                            if scenario_id:
                                scenario_list.append({
                                    "id": scenario_id,
                                    "description": scenario.get("description", ""),
                                    "target": scenario.get("target", "")
                                })

            except Exception as e:
                print(f"Error loading scenarios from {index_path}: {e}")
                continue

        return scenario_list


# ==================== Test Engine ====================

class ScenarioTester:
    """Main test orchestrator"""
    
    def __init__(self, api_client: ReframeAPIClient, scenario_manager: ScenarioManager):
        self.api = api_client
        self.scenarios = scenario_manager
        self.statistics = defaultdict(int)
    
    # Removed roundtrip testing methods
    
    def test_scenario(self, message_type: str, scenario_info: Dict[str, str]) -> TestResult:
        """Test a single scenario following the simplified 3-step validation flow"""
        result = TestResult(
            message_type=message_type,
            scenario=scenario_info["id"],
            description=scenario_info.get("description", "")
        )
        
        # Step 1: Generate sample message using Sample Generator API with message type and scenario
        scenario_id = scenario_info["id"]
        format_type = "MT" if message_type.upper().startswith("MT") else "MX"
        message = self.api.generate_sample(message_type, scenario_id)
        
        if not message:
            result.errors.append("Generation failed")
            # Still count this test in statistics
            self.statistics["total"] += 1
            self.statistics[f"by_type_{message_type}"] += 1
            # Mark other statuses as skipped
            result.validation = TestStatus.SKIPPED
            result.transformation = TestStatus.SKIPPED
            return result
        
        result.generation = TestStatus.SUCCESS
        self.statistics["generation_success"] += 1
        
        # Step 2: Validate the generated message structure using unified validation API
        is_valid, errors = self.api.validate(message, canonical=True, business_validation=(format_type == "MT"))

        if is_valid:
            result.validation = TestStatus.SUCCESS
            self.statistics["validation_success"] += 1
        else:
            result.errors.extend(errors)
            # Skip transformation if validation fails
            result.transformation = TestStatus.SKIPPED
            self.statistics["total"] += 1
            self.statistics[f"by_type_{message_type}"] += 1
            return result

        # Step 3: Transform the message using unified Transformation API
        transformed = self.api.transform(message, debug=False)
        if transformed:
            result.transformation = TestStatus.SUCCESS
            self.statistics["transformation_success"] += 1
        else:
            result.errors.append("Transformation failed")
            result.transformation = TestStatus.FAILURE
        
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
            # Only filter by ID
            scenarios = [s for s in scenarios if s.get("id") in specific_scenarios]
            if not scenarios:
                print(f"Warning: None of the specified scenarios found for {message_type}")
                return results
        
        print(f"\nTesting {message_type} with {len(scenarios)} scenario(s), {sample_count} sample(s) each...")
        
        for scenario_info in scenarios:
            for sample_num in range(1, sample_count + 1):
                # Use the full test flow now that sample generation works
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
                    r.transformation.value
                ]
                
                # Add error summary
                if r.errors:
                    error_summary = r.errors[0][:20] if r.errors else "Error"
                    row.append(error_summary)
                else:
                    row.append("")
                
                table_data.append(row)
        
        headers = ["Message Type", "Scenario", "Sample", "Generator", 
                   "Validator", "Transform", "Errors"]
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
                       "transformation_success"]:
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
        
        log_dir = Path("logs")
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
    tester = ScenarioTester(api_client, scenario_manager)
    
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
                print(f"{i:3}. {scenario['id']:<20} {desc}")
            print(f"\nTotal: {len(scenarios)} scenarios")
            print("\nUse the scenario ID with --scenario option")
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