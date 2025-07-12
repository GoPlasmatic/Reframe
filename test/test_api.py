#!/usr/bin/env python3
"""
Reframe API Test Script
Tests all sample files in test_data/ against the Reframe API endpoints
and logs results to test_data/logs/
"""

import os
import sys
import json
import datetime
import requests
from pathlib import Path
from typing import Dict, List, Tuple
import argparse
from concurrent.futures import ThreadPoolExecutor, as_completed
import time
from colorama import init, Fore, Style

# Initialize colorama for cross-platform colored output
init(autoreset=True)

class ReframeAPITester:
    def __init__(self, base_url: str = "http://localhost:3000"):
        self.base_url = base_url
        self.test_data_dir = Path("data")
        self.logs_dir = self.test_data_dir / "logs"
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
    
    def save_debug_info(self, file_name: str, direction: str, debug_info: Dict):
        """Save debug info for a specific file"""
        # Create filename without extension and add direction
        base_name = Path(file_name).stem
        debug_file = self.debug_dir / f"{direction}_{base_name}.json"
        
        with open(debug_file, 'w') as f:
            json.dump({
                "file": file_name,
                "direction": direction,
                "debug_info": debug_info
            }, f, indent=2)
        
        self.log_detail(f"Debug info saved to {debug_file.name}")
    
    def test_forward_transformation(self, file_path: Path) -> Tuple[bool, Dict]:
        """Test MT to MX transformation"""
        endpoint = f"{self.base_url}/transform/mt-to-mx"
        
        try:
            # Read the MT message
            with open(file_path, 'r') as f:
                mt_content = f.read()
            
            # Prepare request with debug option
            payload = {
                "message": mt_content,
                "options": {
                    "include_debug": True,
                    "validation": True,
                    "format_output": False
                }
            }
            
            self.log_detail(f"Testing forward transformation for: {file_path.name}")
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
                "file": file_path.name,
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
                        self.save_debug_info(file_path.name, "forward", response_data["debug_info"])
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
                "file": file_path.name,
                "error": error_msg,
                "success": False
            }
    
    def test_reverse_transformation(self, file_path: Path) -> Tuple[bool, Dict]:
        """Test MX to MT transformation"""
        endpoint = f"{self.base_url}/transform/mx-to-mt"
        
        try:
            # Read the XML message
            with open(file_path, 'r') as f:
                mx_content = f.read()
            
            # Prepare request with debug option
            payload = {
                "message": mx_content,
                "options": {
                    "include_debug": True,
                    "validation": True,
                    "format_output": False
                }
            }
            
            self.log_detail(f"Testing reverse transformation for: {file_path.name}")
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
                "file": file_path.name,
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
                        self.save_debug_info(file_path.name, "reverse", response_data["debug_info"])
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
                "file": file_path.name,
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
    
    def print_progress(self, direction: str, file_name: str, success: bool, elapsed_ms: float, error: str = None):
        """Print colored progress to console"""
        direction_color = Fore.CYAN if direction == "forward" else Fore.MAGENTA
        status_icon = f"{Fore.GREEN}✓{Style.RESET_ALL}" if success else f"{Fore.RED}✗{Style.RESET_ALL}"
        
        message = f"{direction_color}[{direction.upper():7}]{Style.RESET_ALL} {status_icon} {file_name:30} [{elapsed_ms:7.2f}ms]"
        
        if error:
            message += f" {Fore.RED}{error[:60]}...{Style.RESET_ALL}" if len(error) > 60 else f" {Fore.RED}{error}{Style.RESET_ALL}"
        
        print(message)
    
    def run_tests(self, parallel: bool = False):
        """Run all tests"""
        print(f"\n{Fore.YELLOW}Reframe API Test Runner{Style.RESET_ALL}")
        print(f"{Fore.YELLOW}{'='*60}{Style.RESET_ALL}\n")
        
        # Check server health
        print(f"Checking server at {self.base_url}...")
        if not self.check_server_health():
            print(f"{Fore.RED}Error: Server is not responding. Make sure Reframe is running.{Style.RESET_ALL}")
            sys.exit(1)
        print(f"{Fore.GREEN}Server is healthy!{Style.RESET_ALL}\n")
        
        # Collect test files
        mt_files = list(self.test_data_dir.glob("*.txt"))
        mx_files = list(self.test_data_dir.glob("*.xml"))
        
        print(f"Found {len(mt_files)} MT files for forward transformation")
        print(f"Found {len(mx_files)} MX files for reverse transformation\n")
        
        # Test forward transformations
        print(f"{Fore.CYAN}Testing Forward Transformations (MT → MX){Style.RESET_ALL}")
        print(f"{Fore.CYAN}{'-'*60}{Style.RESET_ALL}")
        
        forward_results = []
        for mt_file in sorted(mt_files):
            success, result = self.test_forward_transformation(mt_file)
            forward_results.append(result)
            if success:
                self.results["forward"]["success"] += 1
            else:
                self.results["forward"]["failed"] += 1
                self.results["forward"]["errors"].append({
                    "file": mt_file.name,
                    "error": result.get("error", "Unknown error")
                })
            
            self.print_progress(
                "forward", 
                mt_file.name, 
                success, 
                result.get("elapsed_ms", 0),
                result.get("error")
            )
        
        print()
        
        # Test reverse transformations
        print(f"{Fore.MAGENTA}Testing Reverse Transformations (MX → MT){Style.RESET_ALL}")
        print(f"{Fore.MAGENTA}{'-'*60}{Style.RESET_ALL}")
        
        reverse_results = []
        for mx_file in sorted(mx_files):
            success, result = self.test_reverse_transformation(mx_file)
            reverse_results.append(result)
            if success:
                self.results["reverse"]["success"] += 1
            else:
                self.results["reverse"]["failed"] += 1
                self.results["reverse"]["errors"].append({
                    "file": mx_file.name,
                    "error": result.get("error", "Unknown error")
                })
            
            self.print_progress(
                "reverse",
                mx_file.name,
                success,
                result.get("elapsed_ms", 0),
                result.get("error")
            )
        
        # Save results
        summary = {
            "timestamp": self.timestamp,
            "server": self.base_url,
            "results": self.results,
            "forward_details": forward_results,
            "reverse_details": reverse_results
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
        
        total_tests = len(mt_files) + len(mx_files)
        total_success = self.results["forward"]["success"] + self.results["reverse"]["success"]
        total_failed = self.results["forward"]["failed"] + self.results["reverse"]["failed"]
        
        print(f"\nForward (MT→MX): {Fore.GREEN}{self.results['forward']['success']} passed{Style.RESET_ALL}, "
              f"{Fore.RED}{self.results['forward']['failed']} failed{Style.RESET_ALL}")
        print(f"Reverse (MX→MT): {Fore.GREEN}{self.results['reverse']['success']} passed{Style.RESET_ALL}, "
              f"{Fore.RED}{self.results['reverse']['failed']} failed{Style.RESET_ALL}")
        print(f"\nTotal: {total_tests} tests, "
              f"{Fore.GREEN}{total_success} passed{Style.RESET_ALL}, "
              f"{Fore.RED}{total_failed} failed{Style.RESET_ALL}")
        
        # Print failed tests details
        if total_failed > 0:
            print(f"\n{Fore.RED}Failed Tests:{Style.RESET_ALL}")
            for error in self.results["forward"]["errors"]:
                print(f"  - {error['file']}: {error['error']}")
            for error in self.results["reverse"]["errors"]:
                print(f"  - {error['file']}: {error['error']}")
        
        print(f"\nLogs saved to:")
        print(f"  - Summary: {self.summary_log}")
        print(f"  - Details: {self.details_log}")
        if debug_files_created:
            print(f"  - Debug Files: {self.debug_dir}/ ({len(debug_files_created)} files)")

def main():
    parser = argparse.ArgumentParser(description="Test Reframe API with sample files")
    parser.add_argument(
        "--url", 
        default="http://localhost:3000",
        help="Base URL of the Reframe server (default: http://localhost:3000)"
    )
    parser.add_argument(
        "--parallel",
        action="store_true",
        help="Run tests in parallel (experimental)"
    )
    
    args = parser.parse_args()
    
    tester = ReframeAPITester(base_url=args.url)
    tester.run_tests(parallel=args.parallel)

if __name__ == "__main__":
    main()