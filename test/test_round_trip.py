#!/usr/bin/env python3
"""
Reframe Round-Trip Test Script
Tests the complete flow: Generate MT → Transform to MX → Transform back to MT → Compare
"""

import sys
import json
import datetime
import requests
from pathlib import Path
from typing import Dict, List, Tuple, Optional
import argparse
import time
from colorama import init, Fore, Style
import difflib

# Initialize colorama for cross-platform colored output
init(autoreset=True)

class RoundTripTester:
    def __init__(self, base_url: str = "http://localhost:3000"):
        self.base_url = base_url
        self.logs_dir = Path("logs")
        self.timestamp = datetime.datetime.now().strftime("%Y%m%d_%H%M%S")
        
        # Create logs directories if they don't exist
        self.logs_dir.mkdir(parents=True, exist_ok=True)
        
        # Create debug subdirectory for individual debug files
        self.debug_dir = self.logs_dir / "debug"
        self.debug_dir.mkdir(exist_ok=True)
        
        # Log files - overwrite on each run
        self.summary_log = self.logs_dir / "summary_latest.json"
        self.details_log = self.logs_dir / "details_latest.log"
        
        # Clear previous debug files
        for debug_file in self.debug_dir.glob("*.json"):
            debug_file.unlink()
        
        # Results tracking
        self.results = {
            "total_tests": 0,
            "successful": 0,
            "failed": 0,
            "errors": [],
            "test_details": []
        }
        
        # Supported message types
        self.supported_types = [
            "MT101", "MT103", "MT104", "MT107", "MT110", "MT111", "MT112",
            "MT192", "MT196", "MT199", "MT202", "MT205", "MT210",
            "MT292", "MT296", "MT299", "MT900", "MT910", "MT920",
            "MT935", "MT940", "MT941", "MT942", "MT950"
        ]
        
    def log_detail(self, message: str):
        """Write detailed log message"""
        with open(self.details_log, 'a') as f:
            timestamp = datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S.%f")[:-3]
            f.write(f"[{timestamp}] {message}\n")
    
    def save_debug_info(self, test_name: str, stage: str, debug_info: Dict):
        """Save debug info for a specific test stage"""
        debug_file = self.debug_dir / f"{test_name}_{stage}.json"
        
        with open(debug_file, 'w') as f:
            json.dump({
                "test_name": test_name,
                "stage": stage,
                "timestamp": datetime.datetime.now().isoformat(),
                "debug_info": debug_info
            }, f, indent=2)
        
        self.log_detail(f"Debug info saved to {debug_file.name}")
    
    def check_server_health(self) -> bool:
        """Check if the server is running and healthy"""
        try:
            response = requests.get(f"{self.base_url}/health", timeout=5)
            if response.status_code == 200:
                health_data = response.json()
                self.log_detail(f"Server health check passed: {json.dumps(health_data)}")
                return True
        except Exception as e:
            self.log_detail(f"Server health check failed: {str(e)}")
        return False
    
    def generate_mt_sample(self, message_type: str, config: Dict, test_name: str) -> Tuple[bool, Optional[str], Dict]:
        """Generate MT sample using the API"""
        endpoint = f"{self.base_url}/generate/mt-sample"
        
        try:
            # Prepare request with proper default config structure
            default_config = {
                "include_optional": False,
                "scenario": "Standard",
                "field_configs": {}
            }
            user_config = config.get("config", {})
            merged_config = {**default_config, **user_config}
            
            payload = {
                "message_type": message_type,
                "config": merged_config,
                "options": config.get("options", {
                    "validation": True,
                    "include_debug": True
                })
            }
            
            self.log_detail(f"Generating MT sample for {message_type}")
            self.log_detail(f"Config: {json.dumps(payload['config'])}")
            
            # Make API call
            start_time = time.time()
            response = requests.post(
                endpoint,
                json=payload,
                headers={"Content-Type": "application/json"},
                timeout=30
            )
            elapsed_time = (time.time() - start_time) * 1000
            
            self.log_detail(f"Response status: {response.status_code}")
            self.log_detail(f"Response time: {elapsed_time:.2f}ms")
            
            if response.status_code == 200:
                response_data = response.json()
                
                if response_data.get("success") and response_data.get("transformed_message"):
                    mt_message = response_data["transformed_message"]
                    
                    # Save debug info if available
                    if response_data.get("debug_info"):
                        self.save_debug_info(test_name, "generate", response_data["debug_info"])
                    
                    self.log_detail(f"Successfully generated MT message ({len(mt_message)} characters)")
                    return True, mt_message, {"elapsed_ms": elapsed_time}
                else:
                    error = response_data.get("errors", ["Unknown error"])[0]
                    self.log_detail(f"Generation failed: {error}")
                    return False, None, {"error": error, "elapsed_ms": elapsed_time}
            else:
                error = f"HTTP {response.status_code}: {response.text}"
                self.log_detail(f"Generation failed: {error}")
                return False, None, {"error": error, "elapsed_ms": elapsed_time}
                
        except Exception as e:
            error = f"Exception during generation: {str(e)}"
            self.log_detail(error)
            return False, None, {"error": error}
    
    def transform_mt_to_mx(self, mt_message: str, test_name: str) -> Tuple[bool, Optional[str], Dict]:
        """Transform MT to MX using the API"""
        endpoint = f"{self.base_url}/transform/mt-to-mx"
        
        try:
            # Prepare request
            payload = {
                "message": mt_message,
                "options": {
                    "include_debug": True,
                    "validation": True
                }
            }
            
            self.log_detail(f"Transforming MT to MX")
            self.log_detail(f"MT message length: {len(mt_message)} characters")
            
            # Make API call
            start_time = time.time()
            response = requests.post(
                endpoint,
                json=payload,
                headers={"Content-Type": "application/json"},
                timeout=30
            )
            elapsed_time = (time.time() - start_time) * 1000
            
            self.log_detail(f"Response status: {response.status_code}")
            self.log_detail(f"Response time: {elapsed_time:.2f}ms")
            
            if response.status_code == 200:
                response_data = response.json()
                
                if response_data.get("success") and response_data.get("transformed_message"):
                    mx_message = response_data["transformed_message"]
                    
                    # Convert to string if it's a list or dict
                    if isinstance(mx_message, (list, dict)):
                        mx_message = json.dumps(mx_message)
                    
                    # Save debug info if available
                    if response_data.get("debug_info"):
                        self.save_debug_info(test_name, "mt_to_mx", response_data["debug_info"])
                    
                    self.log_detail(f"Successfully transformed to MX ({len(mx_message)} characters)")
                    return True, mx_message, {"elapsed_ms": elapsed_time}
                else:
                    error = response_data.get("errors", ["Unknown error"])[0] if response_data.get("errors") else "Transformation failed"
                    self.log_detail(f"MT to MX transformation failed: {error}")
                    return False, None, {"error": error, "elapsed_ms": elapsed_time}
            else:
                error = f"HTTP {response.status_code}: {response.text}"
                self.log_detail(f"MT to MX transformation failed: {error}")
                return False, None, {"error": error, "elapsed_ms": elapsed_time}
                
        except Exception as e:
            error = f"Exception during MT to MX transformation: {str(e)}"
            self.log_detail(error)
            return False, None, {"error": error}
    
    def transform_mx_to_mt(self, mx_message: str, test_name: str) -> Tuple[bool, Optional[str], Dict]:
        """Transform MX back to MT using the API"""
        endpoint = f"{self.base_url}/transform/mx-to-mt"
        
        try:
            # Prepare request
            payload = {
                "message": mx_message,
                "options": {
                    "include_debug": True,
                    "validation": True
                }
            }
            
            self.log_detail(f"Transforming MX to MT")
            self.log_detail(f"MX message length: {len(mx_message)} characters")
            
            # Make API call
            start_time = time.time()
            response = requests.post(
                endpoint,
                json=payload,
                headers={"Content-Type": "application/json"},
                timeout=30
            )
            elapsed_time = (time.time() - start_time) * 1000
            
            self.log_detail(f"Response status: {response.status_code}")
            self.log_detail(f"Response time: {elapsed_time:.2f}ms")
            
            if response.status_code == 200:
                response_data = response.json()
                
                if response_data.get("success") and response_data.get("transformed_message"):
                    mt_message = response_data["transformed_message"]
                    
                    # Save debug info if available
                    if response_data.get("debug_info"):
                        self.save_debug_info(test_name, "mx_to_mt", response_data["debug_info"])
                    
                    self.log_detail(f"Successfully transformed back to MT ({len(mt_message)} characters)")
                    return True, mt_message, {"elapsed_ms": elapsed_time}
                else:
                    # Extract detailed error information
                    errors = response_data.get("errors", [])
                    warnings = response_data.get("warnings", [])
                    success_flag = response_data.get("success", False)
                    
                    # Save debug info even for failed cases
                    if response_data.get("debug_info"):
                        self.save_debug_info(test_name, "mx_to_mt_failed", response_data["debug_info"])
                    
                    # Create detailed error message
                    error_parts = []
                    if not success_flag:
                        error_parts.append("success=false")
                    if errors:
                        error_parts.append(f"errors: {', '.join(errors)}")
                    if warnings:
                        error_parts.append(f"warnings: {', '.join(warnings)}")
                    
                    error = "; ".join(error_parts) if error_parts else "Transformation failed with no specific error"
                    
                    self.log_detail(f"MX to MT transformation failed: {error}")
                    self.log_detail(f"Response data: {json.dumps(response_data)}")
                    return False, None, {"error": error, "elapsed_ms": elapsed_time}
            else:
                error = f"HTTP {response.status_code}: {response.text}"
                self.log_detail(f"MX to MT transformation failed: {error}")
                return False, None, {"error": error, "elapsed_ms": elapsed_time}
                
        except Exception as e:
            error = f"Exception during MX to MT transformation: {str(e)}"
            self.log_detail(error)
            return False, None, {"error": error}
    
    def normalize_mt_message(self, mt_message: str) -> str:
        """Normalize MT message for comparison"""
        # Remove trailing whitespace from each line
        lines = [line.rstrip() for line in mt_message.split('\n')]
        
        # Remove empty lines at the end
        while lines and not lines[-1]:
            lines.pop()
        
        # Join back
        normalized = '\n'.join(lines)
        
        # Normalize line endings
        normalized = normalized.replace('\r\n', '\n').replace('\r', '\n')
        
        return normalized
    
    def compare_messages(self, original: str, final: str) -> Tuple[bool, List[str]]:
        """Compare two MT messages and return differences"""
        # Normalize both messages
        original_norm = self.normalize_mt_message(original)
        final_norm = self.normalize_mt_message(final)
        
        if original_norm == final_norm:
            return True, []
        
        # Get detailed differences
        diff = list(difflib.unified_diff(
            original_norm.splitlines(keepends=True),
            final_norm.splitlines(keepends=True),
            fromfile='Original MT',
            tofile='Final MT',
            lineterm=''
        ))
        
        return False, diff
    
    def run_single_test(self, message_type: str, config: Dict, test_name: str) -> Dict:
        """Run a single round-trip test"""
        test_result = {
            "test_name": test_name,
            "message_type": message_type,
            "config": config,
            "stages": {},
            "success": False,
            "comparison_result": None,
            "errors": []
        }
        
        self.log_detail(f"\n{'='*60}")
        self.log_detail(f"Starting round-trip test: {test_name}")
        self.log_detail(f"Message Type: {message_type}")
        self.log_detail(f"{'='*60}")
        
        # Stage 1: Generate MT sample
        success, original_mt, metrics = self.generate_mt_sample(message_type, config, test_name)
        test_result["stages"]["generate"] = {
            "success": success,
            "metrics": metrics
        }
        
        if not success:
            test_result["errors"].append(f"Generation failed: {metrics.get('error', 'Unknown error')}")
            return test_result
        
        # Stage 2: Transform MT to MX
        success, mx_message, metrics = self.transform_mt_to_mx(original_mt, test_name)
        test_result["stages"]["mt_to_mx"] = {
            "success": success,
            "metrics": metrics
        }
        
        if not success:
            test_result["errors"].append(f"MT to MX failed: {metrics.get('error', 'Unknown error')}")
            return test_result
        
        # Stage 3: Transform MX back to MT
        success, final_mt, metrics = self.transform_mx_to_mt(mx_message, test_name)
        test_result["stages"]["mx_to_mt"] = {
            "success": success,
            "metrics": metrics
        }
        
        if not success:
            test_result["errors"].append(f"MX to MT failed: {metrics.get('error', 'Unknown error')}")
            return test_result
        
        # Stage 4: Compare original and final MT
        messages_match, differences = self.compare_messages(original_mt, final_mt)
        test_result["comparison_result"] = {
            "match": messages_match,
            "differences": differences
        }
        
        if not messages_match:
            self.log_detail(f"Messages differ! Found {len(differences)} difference lines")
            # Save the messages and diff for debugging
            debug_file = self.debug_dir / f"{test_name}_comparison.json"
            with open(debug_file, 'w') as f:
                json.dump({
                    "original_mt": original_mt,
                    "final_mt": final_mt,
                    "differences": differences
                }, f, indent=2)
        else:
            self.log_detail("Messages match perfectly!")
        
        test_result["success"] = messages_match
        
        # Calculate total time
        total_time = sum(
            stage.get("metrics", {}).get("elapsed_ms", 0)
            for stage in test_result["stages"].values()
        )
        test_result["total_elapsed_ms"] = total_time
        
        return test_result
    
    def print_test_result(self, result: Dict):
        """Print colored test result to console"""
        test_name = result["test_name"]
        success = result["success"]
        total_time = result.get("total_elapsed_ms", 0)
        
        # Status icon and color
        if success:
            status = f"{Fore.GREEN}✓ PASS{Style.RESET_ALL}"
        else:
            status = f"{Fore.RED}✗ FAIL{Style.RESET_ALL}"
        
        # Print main result
        print(f"\n{status} {test_name:40} [{total_time:7.0f}ms total]")
        
        # Print stage details
        stages = result.get("stages", {})
        for stage_name, stage_data in stages.items():
            stage_success = stage_data.get("success", False)
            stage_time = stage_data.get("metrics", {}).get("elapsed_ms", 0)
            stage_icon = "✓" if stage_success else "✗"
            stage_color = Fore.GREEN if stage_success else Fore.RED
            
            print(f"  {stage_color}{stage_icon}{Style.RESET_ALL} {stage_name:20} [{stage_time:6.0f}ms]")
        
        # Print comparison result
        if result.get("comparison_result"):
            comp_result = result["comparison_result"]
            if comp_result["match"]:
                print(f"  {Fore.GREEN}✓{Style.RESET_ALL} Message comparison:  MATCH")
            else:
                diff_count = len(comp_result.get("differences", []))
                print(f"  {Fore.RED}✗{Style.RESET_ALL} Message comparison:  DIFFER ({diff_count} lines)")
        
        # Print errors if any
        if result.get("errors"):
            print(f"  {Fore.RED}Errors:{Style.RESET_ALL}")
            for error in result["errors"]:
                print(f"    - {error}")
    
    def get_default_configs(self) -> List[Dict]:
        """Get default test configurations"""
        configs = []
        
        # Basic test for each message type
        for mt_type in self.supported_types:
            configs.append({
                "test_name": f"{mt_type}_default",
                "message_type": mt_type,
                "config": {
                    "include_optional": False,
                    "scenario": "Standard",
                    "field_configs": {}
                },
                "options": {
                    "validation": True,
                    "include_debug": True
                }
            })
        
        # Additional tests with optional fields
        for mt_type in ["MT103", "MT202", "MT900"]:
            configs.append({
                "test_name": f"{mt_type}_with_optional",
                "message_type": mt_type,
                "config": {
                    "include_optional": True,
                    "scenario": "Standard",
                    "field_configs": {}
                },
                "options": {
                    "validation": True,
                    "include_debug": True
                }
            })
        
        return configs
    
    def run_batch_tests(self, test_configs: List[Dict]):
        """Run multiple round-trip tests"""
        print(f"\n{Fore.YELLOW}Reframe Round-Trip Test Runner{Style.RESET_ALL}")
        print(f"{Fore.YELLOW}{'='*70}{Style.RESET_ALL}\n")
        
        # Check server health
        print(f"Checking server at {self.base_url}...")
        if not self.check_server_health():
            print(f"{Fore.RED}Error: Server is not responding. Make sure Reframe is running.{Style.RESET_ALL}")
            sys.exit(1)
        print(f"{Fore.GREEN}Server is healthy!{Style.RESET_ALL}\n")
        
        print(f"Running {len(test_configs)} round-trip tests...")
        print(f"{'-'*70}")
        
        # Run each test
        for config in test_configs:
            test_name = config.get("test_name", f"{config['message_type']}_test")
            message_type = config["message_type"]
            
            result = self.run_single_test(message_type, config, test_name)
            self.results["test_details"].append(result)
            self.results["total_tests"] += 1
            
            if result["success"]:
                self.results["successful"] += 1
            else:
                self.results["failed"] += 1
                self.results["errors"].append({
                    "test_name": test_name,
                    "errors": result.get("errors", ["Unknown error"])
                })
            
            self.print_test_result(result)
        
        # Save results
        self.save_results()
        
        # Print summary
        self.print_summary()
    
    def save_results(self):
        """Save test results to files"""
        summary = {
            "timestamp": self.timestamp,
            "server": self.base_url,
            "results_summary": {
                "total": self.results["total_tests"],
                "successful": self.results["successful"],
                "failed": self.results["failed"]
            },
            "test_details": self.results["test_details"],
            "errors": self.results["errors"]
        }
        
        with open(self.summary_log, 'w') as f:
            json.dump(summary, f, indent=2)
        
        self.log_detail(f"\nTest run completed at {datetime.datetime.now()}")
        self.log_detail(f"Results saved to {self.summary_log}")
    
    def print_summary(self):
        """Print test summary"""
        print(f"\n{Fore.YELLOW}{'='*70}{Style.RESET_ALL}")
        print(f"{Fore.YELLOW}Test Summary{Style.RESET_ALL}")
        print(f"{Fore.YELLOW}{'='*70}{Style.RESET_ALL}\n")
        
        total = self.results["total_tests"]
        successful = self.results["successful"]
        failed = self.results["failed"]
        
        print(f"Total Tests: {total}")
        print(f"Successful:  {Fore.GREEN}{successful}{Style.RESET_ALL}")
        print(f"Failed:      {Fore.RED}{failed}{Style.RESET_ALL}")
        
        if failed > 0:
            success_rate = (successful / total) * 100
            print(f"Success Rate: {success_rate:.1f}%")
            
            print(f"\n{Fore.RED}Failed Tests:{Style.RESET_ALL}")
            for error in self.results["errors"]:
                print(f"  - {error['test_name']}")
                for err_msg in error["errors"]:
                    print(f"    {err_msg}")
        else:
            print(f"\n{Fore.GREEN}All tests passed!{Style.RESET_ALL}")
        
        print(f"\nLogs saved to:")
        print(f"  - Summary: {self.summary_log}")
        print(f"  - Details: {self.details_log}")
        
        debug_files = list(self.debug_dir.glob("*.json"))
        if debug_files:
            print(f"  - Debug Files: {self.debug_dir}/ ({len(debug_files)} files)")

def main():
    parser = argparse.ArgumentParser(
        description="Test Reframe round-trip transformations: Generate MT → MX → MT → Compare"
    )
    parser.add_argument(
        "--url", 
        default="http://localhost:3000",
        help="Base URL of the Reframe server (default: http://localhost:3000)"
    )
    parser.add_argument(
        "--config",
        help="Path to JSON configuration file with test configs"
    )
    parser.add_argument(
        "--message-type",
        choices=[
            "MT101", "MT103", "MT104", "MT107", "MT110", "MT111", "MT112",
            "MT192", "MT196", "MT199", "MT202", "MT205", "MT210",
            "MT292", "MT296", "MT299", "MT900", "MT910", "MT920",
            "MT935", "MT940", "MT941", "MT942", "MT950"
        ],
        help="Test a specific message type"
    )
    parser.add_argument(
        "--include-optional",
        action="store_true",
        help="Include optional fields in generated messages"
    )
    parser.add_argument(
        "--all",
        action="store_true",
        help="Run tests for all supported message types"
    )
    
    args = parser.parse_args()
    
    tester = RoundTripTester(base_url=args.url)
    
    # Determine which tests to run
    test_configs = []
    
    if args.config:
        # Load from config file
        with open(args.config, 'r') as f:
            config_data = json.load(f)
            if isinstance(config_data, list):
                test_configs = config_data
            else:
                test_configs = [config_data]
    elif args.message_type:
        # Single message type test
        test_configs = [{
            "test_name": f"{args.message_type}_cli_test",
            "message_type": args.message_type,
            "config": {
                "include_optional": args.include_optional,
                "scenario": "Standard",
                "field_configs": {}
            },
            "options": {
                "validation": True,
                "include_debug": True
            }
        }]
    elif args.all:
        # All message types
        test_configs = tester.get_default_configs()
    else:
        # Default: test a few common message types
        for mt_type in ["MT103", "MT202", "MT900"]:
            test_configs.append({
                "test_name": f"{mt_type}_default",
                "message_type": mt_type,
                "config": {
                    "include_optional": False,
                    "scenario": "Standard",
                    "field_configs": {}
                },
                "options": {
                    "validation": True,
                    "include_debug": True
                }
            })
    
    # Run the tests
    tester.run_batch_tests(test_configs)

if __name__ == "__main__":
    main()