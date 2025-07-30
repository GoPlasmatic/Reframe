#!/usr/bin/env python3
"""
Reframe API Test Script
Tests transformations using dynamically generated samples with scenarios
from the swift-mt-message library and logs results to logs/
"""

import os
import sys
import json
import datetime
import requests
from pathlib import Path
from typing import Dict, List, Tuple, Optional
import argparse
from concurrent.futures import ThreadPoolExecutor, as_completed
import time
from colorama import init, Fore, Style

# Initialize colorama for cross-platform colored output
init(autoreset=True)

class ReframeAPITester:
    def __init__(self, base_url: str = "http://localhost:3000"):
        self.base_url = base_url
        self.logs_dir = Path("logs")
        self.timestamp = datetime.datetime.now().strftime("%Y%m%d_%H%M%S")
        
        # Create logs directory if it doesn't exist
        self.logs_dir.mkdir(exist_ok=True)
        
        # Create debug subdirectory for individual debug files
        self.debug_dir = self.logs_dir / "debug"
        self.debug_dir.mkdir(exist_ok=True)
        
        # Log files - overwrite on each run
        self.summary_log = self.logs_dir / "test_summary_latest.json"
        self.details_log = self.logs_dir / "test_details_latest.log"
        
        # Clear previous debug files
        for debug_file in self.debug_dir.glob("*.json"):
            debug_file.unlink()
        
        # Results tracking
        self.results = {
            "forward": {"success": 0, "failed": 0, "errors": []},
            "reverse": {"success": 0, "failed": 0, "errors": []}
        }
        
    def log_detail(self, message: str):
        """Write detailed log message"""
        with open(self.details_log, 'a') as f:
            timestamp = datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S.%f")[:-3]
            f.write(f"[{timestamp}] {message}\n")
    
    def save_debug_info(self, test_name: str, direction: str, debug_info: Dict):
        """Save debug info for a specific test"""
        debug_file = self.debug_dir / f"{direction}_{test_name}.json"
        
        with open(debug_file, 'w') as f:
            json.dump({
                "test": test_name,
                "direction": direction,
                "debug_info": debug_info
            }, f, indent=2)
        
        self.log_detail(f"Debug info saved to {debug_file.name}")
    
    def generate_mt_sample(self, message_type: str, scenario: str = None) -> Tuple[bool, Optional[str], Dict]:
        """Generate MT sample using the API"""
        endpoint = f"{self.base_url}/generate/mt-sample"
        
        try:
            # Prepare request
            config = {}
            if scenario:
                config["scenario"] = scenario
                
            payload = {
                "message_type": message_type,
                "config": config,
                "options": {
                    "validation": True,
                    "include_debug": True
                }
            }
            
            self.log_detail(f"Generating {message_type} sample with scenario: {scenario or 'default'}")
            
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
                    self.log_detail(f"Successfully generated {message_type} ({len(mt_message)} characters)")
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
    
    def test_forward_transformation(self, mt_content: str, test_name: str) -> Tuple[bool, Dict]:
        """Test MT to MX transformation"""
        endpoint = f"{self.base_url}/transform/mt-to-mx"
        
        try:
            # Prepare request with debug option
            payload = {
                "message": mt_content,
                "options": {
                    "include_debug": True,
                    "validation": True,
                    "format_output": False
                }
            }
            
            self.log_detail(f"Testing forward transformation for: {test_name}")
            self.log_detail(f"Request payload size: {len(mt_content)} bytes")
            
            # Make API call
            start_time = time.time()
            response = requests.post(
                endpoint,
                json=payload,
                headers={"Content-Type": "application/json"},
                timeout=30
            )
            elapsed_time = (time.time() - start_time) * 1000  # Convert to milliseconds
            
            self.log_detail(f"Response status: {response.status_code}")
            self.log_detail(f"Response time: {elapsed_time:.2f}ms")
            
            result = {
                "test_name": test_name,
                "status_code": response.status_code,
                "elapsed_ms": elapsed_time,
                "success": response.status_code == 200
            }
            
            if response.status_code == 200:
                try:
                    response_data = response.json()
                    result["response"] = response_data
                    
                    # Extract message type from debug info if available
                    if "debug_info" in response_data and "intermediate_data" in response_data["debug_info"]:
                        intermediate = response_data["debug_info"]["intermediate_data"]
                        if "mx_message_type" in intermediate:
                            result["mx_message_type"] = intermediate["mx_message_type"]
                        else:
                            result["mx_message_type"] = "Unknown"
                    else:
                        result["mx_message_type"] = "Unknown"
                    
                    # Store debug_info for further analysis
                    if "debug_info" in response_data:
                        result["debug_info"] = response_data["debug_info"]
                        self.log_detail(f"Debug info captured: {len(str(response_data['debug_info']))} bytes")
                        # Save debug info to individual file
                        self.save_debug_info(test_name, "forward", response_data["debug_info"])
                    else:
                        self.log_detail(f"No debug_info in response. Keys: {list(response_data.keys())}")
                    
                    # Store errors and warnings if present
                    if "errors" in response_data and response_data["errors"]:
                        result["errors"] = response_data["errors"]
                        self.log_detail(f"Errors found: {response_data['errors']}")
                    
                    if "warnings" in response_data and response_data["warnings"]:
                        result["warnings"] = response_data["warnings"]
                        self.log_detail(f"Warnings found: {response_data['warnings']}")
                    
                    # Check if transformation was successful based on success field and transformed_message
                    if not response_data.get("success", False):
                        # Use specific error details from API if available
                        if "errors" in response_data and response_data["errors"]:
                            result["error"] = "; ".join(response_data["errors"])
                        else:
                            result["error"] = "Transformation failed (success=false)"
                        result["success"] = False
                        self.log_detail(f"Error: {result['error']}")
                    else:
                        transformed_message = response_data.get("transformed_message")
                        if transformed_message is None or transformed_message == []:
                            result["error"] = "Transformation returned empty or null result"
                            result["success"] = False
                            self.log_detail(f"Error: Empty or null transformed_message")
                        else:
                            self.log_detail(f"Success: Transformed to {result['mx_message_type']}")
                except json.JSONDecodeError:
                    result["error"] = "Invalid JSON response"
                    result["success"] = False
                    self.log_detail(f"Error: Invalid JSON response")
            else:
                result["error"] = response.text
                self.log_detail(f"Error response: {response.text}")
            
            return result["success"], result
            
        except Exception as e:
            error_msg = f"Exception during forward transformation: {str(e)}"
            self.log_detail(error_msg)
            return False, {
                "test_name": test_name,
                "error": error_msg,
                "success": False
            }
    
    def test_reverse_transformation(self, mx_content: str, test_name: str) -> Tuple[bool, Dict]:
        """Test MX to MT transformation"""
        endpoint = f"{self.base_url}/transform/mx-to-mt"
        
        try:
            # Prepare request with debug option
            payload = {
                "message": mx_content,
                "options": {
                    "include_debug": True,
                    "validation": True,
                    "format_output": False
                }
            }
            
            self.log_detail(f"Testing reverse transformation for: {test_name}")
            self.log_detail(f"Request payload size: {len(mx_content)} bytes")
            
            # Make API call
            start_time = time.time()
            response = requests.post(
                endpoint,
                json=payload,
                headers={"Content-Type": "application/json"},
                timeout=30
            )
            elapsed_time = (time.time() - start_time) * 1000  # Convert to milliseconds
            
            self.log_detail(f"Response status: {response.status_code}")
            self.log_detail(f"Response time: {elapsed_time:.2f}ms")
            
            result = {
                "test_name": test_name,
                "status_code": response.status_code,
                "elapsed_ms": elapsed_time,
                "success": response.status_code == 200
            }
            
            if response.status_code == 200:
                try:
                    response_data = response.json()
                    result["response"] = response_data
                    result["mt_message_type"] = response_data.get("mt_message_type", "Unknown")
                    
                    # Store debug_info for further analysis
                    if "debug_info" in response_data:
                        result["debug_info"] = response_data["debug_info"]
                        self.log_detail(f"Debug info captured: {len(str(response_data['debug_info']))} bytes")
                        # Save debug info to individual file
                        self.save_debug_info(test_name, "reverse", response_data["debug_info"])
                    else:
                        self.log_detail(f"No debug_info in response. Keys: {list(response_data.keys())}")
                    
                    # Store errors and warnings if present
                    if "errors" in response_data and response_data["errors"]:
                        result["errors"] = response_data["errors"]
                        self.log_detail(f"Errors found: {response_data['errors']}")
                    
                    if "warnings" in response_data and response_data["warnings"]:
                        result["warnings"] = response_data["warnings"]
                        self.log_detail(f"Warnings found: {response_data['warnings']}")
                    
                    # Check if transformation was successful based on success field and transformed_message
                    if not response_data.get("success", False):
                        # Use specific error details from API if available
                        if "errors" in response_data and response_data["errors"]:
                            result["error"] = "; ".join(response_data["errors"])
                        else:
                            result["error"] = "Transformation failed (success=false)"
                        result["success"] = False
                        self.log_detail(f"Error: {result['error']}")
                    else:
                        # Check if transformed_message is null or empty array
                        transformed_message = response_data.get("transformed_message")
                        if transformed_message is None or transformed_message == []:
                            result["error"] = "Transformation returned empty or null result"
                            result["success"] = False
                            self.log_detail(f"Error: Empty or null transformed_message")
                        else:
                            self.log_detail(f"Success: Transformed to {result['mt_message_type']}")
                except json.JSONDecodeError:
                    result["error"] = "Invalid JSON response"
                    result["success"] = False
                    self.log_detail(f"Error: Invalid JSON response")
            else:
                result["error"] = response.text
                self.log_detail(f"Error response: {response.text}")
            
            return result["success"], result
            
        except Exception as e:
            error_msg = f"Exception during reverse transformation: {str(e)}"
            self.log_detail(error_msg)
            return False, {
                "test_name": test_name,
                "error": error_msg,
                "success": False
            }
    
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
    
    def print_progress(self, direction: str, test_name: str, success: bool, elapsed_ms: float, error: str = None):
        """Print colored progress to console"""
        direction_color = Fore.CYAN if direction == "forward" else Fore.MAGENTA
        status_icon = f"{Fore.GREEN}✓{Style.RESET_ALL}" if success else f"{Fore.RED}✗{Style.RESET_ALL}"
        
        message = f"{direction_color}[{direction.upper():7}]{Style.RESET_ALL} {status_icon} {test_name:30} [{elapsed_ms:7.2f}ms]"
        
        if error:
            message += f" {Fore.RED}{error[:60]}...{Style.RESET_ALL}" if len(error) > 60 else f" {Fore.RED}{error}{Style.RESET_ALL}"
        
        print(message)
    
    def get_test_scenarios(self) -> List[Dict]:
        """Get test scenarios for different message types"""
        scenarios = []
        
        # MT103 scenarios
        for scenario in ["standard", "high_value", "remittance_enhanced", "cbpr_stp_compliant"]:
            scenarios.append({
                "message_type": "MT103",
                "scenario": scenario,
                "test_name": f"MT103_{scenario}"
            })
        
        # MT202 scenarios  
        for scenario in ["standard", "cbpr_cov_standard", "fi_to_fi_transparency"]:
            scenarios.append({
                "message_type": "MT202",
                "scenario": scenario,
                "test_name": f"MT202_{scenario}"
            })
            
        # MT900/910 scenarios
        for mt_type in ["MT900", "MT910"]:
            scenarios.append({
                "message_type": mt_type,
                "scenario": "standard",
                "test_name": f"{mt_type}_standard"
            })
            
        # Add a few more message types with standard scenario
        for mt_type in ["MT192", "MT196", "MT292", "MT296", "MT940", "MT950"]:
            scenarios.append({
                "message_type": mt_type,
                "scenario": "standard",
                "test_name": f"{mt_type}_standard"
            })
            
        return scenarios
    
    def run_tests(self, test_scenarios: List[Dict] = None):
        """Run all tests"""
        print(f"\n{Fore.YELLOW}Reframe API Test Runner{Style.RESET_ALL}")
        print(f"{Fore.YELLOW}{'='*60}{Style.RESET_ALL}\n")
        
        # Check server health
        print(f"Checking server at {self.base_url}...")
        if not self.check_server_health():
            print(f"{Fore.RED}Error: Server is not responding. Make sure Reframe is running.{Style.RESET_ALL}")
            sys.exit(1)
        print(f"{Fore.GREEN}Server is healthy!{Style.RESET_ALL}\n")
        
        # Use provided scenarios or get defaults
        if not test_scenarios:
            test_scenarios = self.get_test_scenarios()
        
        print(f"Running tests for {len(test_scenarios)} scenarios\n")
        
        # Test each scenario
        print(f"{Fore.CYAN}Testing Message Generation and Transformation{Style.RESET_ALL}")
        print(f"{Fore.CYAN}{'-'*60}{Style.RESET_ALL}")
        
        all_results = []
        
        for scenario in test_scenarios:
            test_name = scenario["test_name"]
            message_type = scenario["message_type"]
            scenario_name = scenario.get("scenario")
            
            # Generate MT sample
            gen_success, mt_content, gen_metrics = self.generate_mt_sample(message_type, scenario_name)
            
            if not gen_success:
                result = {
                    "test_name": test_name,
                    "message_type": message_type,
                    "scenario": scenario_name,
                    "stage": "generation",
                    "success": False,
                    "error": gen_metrics.get("error", "Generation failed")
                }
                all_results.append(result)
                self.results["forward"]["failed"] += 1
                self.results["forward"]["errors"].append({
                    "test": test_name,
                    "error": result["error"]
                })
                
                self.print_progress(
                    "generate",
                    test_name,
                    False,
                    gen_metrics.get("elapsed_ms", 0),
                    result["error"]
                )
                continue
            
            # Test forward transformation
            fwd_success, fwd_result = self.test_forward_transformation(mt_content, test_name)
            fwd_result["message_type"] = message_type
            fwd_result["scenario"] = scenario_name
            fwd_result["stage"] = "forward"
            all_results.append(fwd_result)
            
            if fwd_success:
                self.results["forward"]["success"] += 1
            else:
                self.results["forward"]["failed"] += 1
                self.results["forward"]["errors"].append({
                    "test": test_name,
                    "error": fwd_result.get("error", "Unknown error")
                })
            
            self.print_progress(
                "forward",
                test_name,
                fwd_success,
                fwd_result.get("elapsed_ms", 0),
                fwd_result.get("error")
            )
            
            # If forward transformation succeeded, we could test reverse too
            # (but that's covered by round_trip tests)
        
        # Save results
        summary = {
            "timestamp": self.timestamp,
            "server": self.base_url,
            "results": self.results,
            "test_details": all_results
        }
        
        with open(self.summary_log, 'w') as f:
            json.dump(summary, f, indent=2)
        
        # Debug info is now saved in individual files during processing
        debug_files_created = list(self.debug_dir.glob("*.json"))
        if debug_files_created:
            self.log_detail(f"Created {len(debug_files_created)} individual debug files in {self.debug_dir}")
        
        # Print summary
        print(f"\n{Fore.YELLOW}Test Summary{Style.RESET_ALL}")
        print(f"{Fore.YELLOW}{'='*60}{Style.RESET_ALL}")
        
        total_tests = len(test_scenarios)
        total_success = self.results["forward"]["success"]
        total_failed = self.results["forward"]["failed"]
        
        print(f"\nResults: {Fore.GREEN}{self.results['forward']['success']} passed{Style.RESET_ALL}, "
              f"{Fore.RED}{self.results['forward']['failed']} failed{Style.RESET_ALL}")
        print(f"\nTotal: {total_tests} tests, "
              f"{Fore.GREEN}{total_success} passed{Style.RESET_ALL}, "
              f"{Fore.RED}{total_failed} failed{Style.RESET_ALL}")
        
        # Print failed tests details
        if total_failed > 0:
            print(f"\n{Fore.RED}Failed Tests:{Style.RESET_ALL}")
            for error in self.results["forward"]["errors"]:
                print(f"  - {error['test']}: {error['error']}")
        
        print(f"\nLogs saved to:")
        print(f"  - Summary: {self.summary_log}")
        print(f"  - Details: {self.details_log}")
        if debug_files_created:
            print(f"  - Debug Files: {self.debug_dir}/ ({len(debug_files_created)} files)")

def main():
    parser = argparse.ArgumentParser(description="Test Reframe API with dynamically generated samples")
    parser.add_argument(
        "--url", 
        default="http://localhost:3000",
        help="Base URL of the Reframe server (default: http://localhost:3000)"
    )
    parser.add_argument(
        "--scenario",
        help="Test a specific scenario (e.g., 'standard', 'high_value')"
    )
    parser.add_argument(
        "--message-type",
        help="Test a specific message type (e.g., 'MT103', 'MT202')"
    )
    
    args = parser.parse_args()
    
    tester = ReframeAPITester(base_url=args.url)
    
    # Build custom test scenarios if filters provided
    test_scenarios = None
    if args.message_type or args.scenario:
        test_scenarios = []
        if args.message_type:
            test_scenarios.append({
                "message_type": args.message_type,
                "scenario": args.scenario or "standard",
                "test_name": f"{args.message_type}_{args.scenario or 'standard'}"
            })
        else:
            # Use all message types with specified scenario
            for mt_type in ["MT103", "MT202", "MT900", "MT910"]:
                test_scenarios.append({
                    "message_type": mt_type,
                    "scenario": args.scenario,
                    "test_name": f"{mt_type}_{args.scenario}"
                })
    
    tester.run_tests(test_scenarios)

if __name__ == "__main__":
    main()