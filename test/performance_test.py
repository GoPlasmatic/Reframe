#!/usr/bin/env python3
"""
Performance Testing Suite for Reframe
=====================================

This script performs comprehensive performance testing to measure metrics before
and after vertical scaling optimizations as documented in scaling.md.

Key Metrics Measured:
- Throughput (requests per second)
- Latency (P50, P95, P99)
- CPU utilization
- Memory usage
- Concurrency capabilities
- Error rates under load

Usage:
    # Run baseline test
    python3 test/performance_test.py --baseline
    
    # Run specific test
    python3 test/performance_test.py --test load --concurrency 100
    
    # Compare before/after results
    python3 test/performance_test.py --compare baseline.json optimized.json
"""

import json
import time
import requests
import argparse
import sys
import os
import threading
import queue
import statistics
import subprocess
from datetime import datetime
from pathlib import Path
from typing import List, Dict, Any, Optional, Tuple
from dataclasses import dataclass, asdict, field
from concurrent.futures import ThreadPoolExecutor, as_completed
from collections import defaultdict
import signal

# Optional imports with fallbacks
try:
    import psutil
    HAS_PSUTIL = True
except ImportError:
    HAS_PSUTIL = False
    print("Warning: psutil not installed. System metrics will be limited.")
    print("Install with: pip3 install psutil")

try:
    import numpy as np
    HAS_NUMPY = True
except ImportError:
    HAS_NUMPY = False
    print("Warning: numpy not installed. Using basic percentile calculation.")
    print("Install with: pip3 install numpy")

try:
    from tabulate import tabulate
    HAS_TABULATE = True
except ImportError:
    HAS_TABULATE = False
    print("Warning: tabulate not installed. Output formatting will be basic.")
    print("Install with: pip3 install tabulate")

# Import shared components from test_scenarios.py if available
sys.path.append(os.path.dirname(os.path.abspath(__file__)))
try:
    from test_scenarios import APIEndpoints, ReframeAPIClient
except ImportError:
    @dataclass
    class APIEndpoints:
        """API endpoint configuration"""
        base_url: str
        generate_sample: str = "/generate/sample"
        validate_mt: str = "/validate/mt"
        validate_mx: str = "/validate/mx"
        transform_mt_to_mx: str = "/transform/mt-to-mx"
        transform_mx_to_mt: str = "/transform/mx-to-mt"


# ==================== Performance Metrics ====================

@dataclass
class PerformanceMetrics:
    """Performance metrics collected during testing"""
    test_name: str
    timestamp: str
    duration_seconds: float
    total_requests: int
    successful_requests: int
    failed_requests: int
    throughput_rps: float
    latency_p50_ms: float
    latency_p95_ms: float
    latency_p99_ms: float
    latency_min_ms: float
    latency_max_ms: float
    latency_mean_ms: float
    latency_stdev_ms: float
    cpu_usage_percent: float
    memory_usage_mb: float
    thread_count: int
    concurrent_connections: int
    error_rate_percent: float
    reframe_version: str = "unknown"
    system_info: Dict[str, Any] = field(default_factory=dict)
    
    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)
    
    def print_summary(self):
        """Print formatted summary of metrics"""
        print("\n" + "="*60)
        print(f"Performance Test Results: {self.test_name}")
        print(f"Reframe Version: {self.reframe_version}")
        print("="*60)
        
        summary = [
            ["Metric", "Value"],
            ["Test Timestamp", self.timestamp],
            ["Duration", f"{self.duration_seconds:.2f} seconds"],
            ["Total Requests", f"{self.total_requests:,}"],
            ["Successful", f"{self.successful_requests:,}"],
            ["Failed", f"{self.failed_requests:,}"],
            ["Throughput", f"{self.throughput_rps:.2f} req/s"],
            ["Error Rate", f"{self.error_rate_percent:.2f}%"],
            ["", ""],
            ["Latency P50", f"{self.latency_p50_ms:.2f} ms"],
            ["Latency P95", f"{self.latency_p95_ms:.2f} ms"],
            ["Latency P99", f"{self.latency_p99_ms:.2f} ms"],
            ["Latency Min", f"{self.latency_min_ms:.2f} ms"],
            ["Latency Max", f"{self.latency_max_ms:.2f} ms"],
            ["Latency Mean", f"{self.latency_mean_ms:.2f} ms"],
            ["Latency StdDev", f"{self.latency_stdev_ms:.2f} ms"],
            ["", ""],
            ["Reframe CPU Usage", f"{self.cpu_usage_percent:.1f}%"],
            ["Reframe Memory (RSS)", f"{self.memory_usage_mb:.2f} MB"],
            ["Reframe Threads", f"{self.thread_count}"],
            ["Concurrent Connections", f"{self.concurrent_connections}"],
        ]
        
        # Add system info if available
        if self.system_info:
            summary.append(["", ""])
            summary.append(["System CPU Cores", str(self.system_info.get("cpu_count", "N/A"))])
            if "os_version" in self.system_info:
                summary.append(["Operating System", self.system_info["os_version"]])
            if "cpu_model" in self.system_info:
                # Truncate long CPU model names
                cpu_model = self.system_info["cpu_model"]
                if len(cpu_model) > 40:
                    cpu_model = cpu_model[:37] + "..."
                summary.append(["CPU Model", cpu_model])
        
        if HAS_TABULATE:
            print(tabulate(summary, headers="firstrow", tablefmt="grid"))
        else:
            # Basic formatting without tabulate
            for row in summary:
                if len(row) == 2:
                    print(f"{row[0]:25s} {row[1]}")


@dataclass
class SystemMetrics:
    """System resource metrics"""
    timestamp: float
    cpu_percent: float
    memory_mb: float
    thread_count: int
    open_files: int


# ==================== Sample Message Templates ====================

# Pre-defined sample messages for different scenarios
# These match the scenarios defined in scenarios/index.json
SAMPLE_MESSAGES = {
    "mt103_simple": {
        "message_type": "MT103",
        "config": {"scenario": "standard"},  # Maps to mt103_to_pacs008_cbpr_standard
        "description": "Standard Cross-Border Payment"
    },
    "mt103_high_value": {
        "message_type": "MT103",
        "config": {"scenario": "high_value"},  # Maps to mt103_to_pacs008_cbpr_high_value
        "description": "High Value Payment with Priority"
    },
    "mt101_single": {
        "message_type": "MT101",
        "config": {"scenario": "single_payment"},  # Maps to mt101_to_pain001_cbpr_single
        "description": "Single Payment Initiation"
    },
    "pacs008_standard": {
        "message_type": "pacs.008",
        "config": {"scenario": "cbpr_standard"},  # Reverse scenario
        "description": "Standard pacs.008 credit transfer"
    },
    "camt052_cbpr": {
        "message_type": "camt.052",
        "config": {"scenario": "cbpr"},  # Reverse scenario
        "description": "Account Report with transactions"
    }
}


# ==================== Performance Test Runner ====================

class PerformanceTestRunner:
    """Main performance test execution engine"""
    
    def __init__(self, base_url: str = "http://localhost:3000", debug: bool = False):
        self.base_url = base_url
        self.debug = debug
        self.endpoints = APIEndpoints(base_url=base_url)
        self.session = requests.Session()
        self.results_queue = queue.Queue()
        self.system_metrics = []
        self.stop_monitoring = threading.Event()
        self.reframe_version = self._get_reframe_version()
        self.system_info = self._get_system_info()
        
    def _log(self, message: str):
        """Log message with timestamp"""
        timestamp = datetime.now().strftime("%H:%M:%S.%f")[:-3]
        print(f"[{timestamp}] {message}")
    
    def _get_reframe_version(self) -> str:
        """Get Reframe version from health endpoint or Cargo.toml"""
        try:
            # First try to get from health endpoint
            url = f"{self.base_url}/health"
            response = self.session.get(url, timeout=2)
            if response.status_code == 200:
                data = response.json()
                # Check if version is in health response (might be added in future)
                if 'version' in data:
                    return data['version']
            
            # Fallback: try to read from Cargo.toml
            # Try different possible locations
            possible_paths = [
                Path(__file__).parent.parent / "Cargo.toml",  # test/../Cargo.toml
                Path.cwd() / "Cargo.toml",  # Current directory
                Path(__file__).parent.parent.parent / "Cargo.toml",  # In case of deeper nesting
            ]
            
            cargo_path = None
            for path in possible_paths:
                if path.exists():
                    cargo_path = path
                    break
            
            if cargo_path and cargo_path.exists():
                with open(cargo_path, 'r') as f:
                    for line in f:
                        if line.strip().startswith('version'):
                            # Parse: version = "3.0.6"
                            parts = line.split('=')
                            if len(parts) >= 2:
                                version = parts[1].strip().strip('"').strip("'")
                                if version:  # Make sure we got something
                                    return version
        except Exception as e:
            if self.debug:
                self._log(f"Could not get Reframe version: {e}")
        return "unknown"
    
    def _get_system_info(self) -> Dict[str, Any]:
        """Get system information"""
        info = {
            "platform": sys.platform,
            "python_version": sys.version.split()[0],
            "cpu_count": os.cpu_count() or 0,
        }
        
        try:
            if sys.platform == "darwin":  # macOS
                # Get macOS version
                result = subprocess.run(["sw_vers", "-productVersion"], 
                                      capture_output=True, text=True, timeout=2)
                if result.returncode == 0:
                    info["os_version"] = f"macOS {result.stdout.strip()}"
                
                # Get CPU model
                result = subprocess.run(["sysctl", "-n", "machdep.cpu.brand_string"], 
                                      capture_output=True, text=True, timeout=2)
                if result.returncode == 0:
                    info["cpu_model"] = result.stdout.strip()
                    
            elif sys.platform.startswith("linux"):
                # Get Linux distribution
                try:
                    with open("/etc/os-release") as f:
                        for line in f:
                            if line.startswith("PRETTY_NAME"):
                                info["os_version"] = line.split("=")[1].strip().strip('"')
                                break
                except:
                    info["os_version"] = "Linux"
                
                # Get CPU model
                try:
                    with open("/proc/cpuinfo") as f:
                        for line in f:
                            if "model name" in line:
                                info["cpu_model"] = line.split(":")[1].strip()
                                break
                except:
                    pass
        except Exception as e:
            if self.debug:
                self._log(f"Could not get full system info: {e}")
        
        return info
    
    def _generate_sample_message(self, message_config: Dict) -> Optional[str]:
        """Generate a sample message using the API"""
        try:
            url = f"{self.base_url}{self.endpoints.generate_sample}"
            # Build proper request format
            request_data = {
                "message_type": message_config.get("message_type"),
                "config": message_config.get("config", {"scenario": "standard"})
            }
            response = self.session.post(url, json=request_data, timeout=5)
            if response.status_code == 200:
                data = response.json()
                return data.get("result", data.get("sample", data.get("message")))
            else:
                self._log(f"Failed to generate sample: {response.status_code}")
                if self.debug and response.text:
                    self._log(f"Response: {response.text}")
                return None
        except Exception as e:
            self._log(f"Error generating sample: {e}")
            return None
    
    def _make_transformation_request(self, message: str, direction: str = "mt_to_mx") -> Tuple[bool, float]:
        """Make a single transformation request and measure latency"""
        start_time = time.perf_counter()
        success = False
        
        try:
            if direction == "mt_to_mx":
                url = f"{self.base_url}{self.endpoints.transform_mt_to_mx}"
                payload = {"message": message}
            else:
                url = f"{self.base_url}{self.endpoints.transform_mx_to_mt}"
                payload = {"message": message}
            
            response = self.session.post(url, json=payload, timeout=30)
            success = response.status_code == 200
        except requests.exceptions.RequestException:
            success = False
        
        latency_ms = (time.perf_counter() - start_time) * 1000
        return success, latency_ms
    
    def _worker_thread(self, message: str, num_requests: int, direction: str = "mt_to_mx"):
        """Worker thread for concurrent request execution"""
        for _ in range(num_requests):
            success, latency_ms = self._make_transformation_request(message, direction)
            self.results_queue.put((success, latency_ms))
    
    def _get_cpu_usage_fallback(self) -> float:
        """Get CPU usage without psutil (macOS/Linux fallback)"""
        try:
            if sys.platform == "darwin":  # macOS
                # First try to get Reframe process CPU specifically
                cmd = "ps aux | grep -E '(cargo.*run|reframe)' | grep -v grep | awk '{print $3}' | head -1"
                result = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=1)
                if result.returncode == 0 and result.stdout.strip():
                    try:
                        return float(result.stdout.strip())
                    except ValueError:
                        pass
                
                # Fall back to overall CPU usage
                cmd = "top -l 2 -n 0 -s 1 | grep 'CPU usage' | tail -1 | awk '{print $3}' | sed 's/%//'"
                result = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=3)
                if result.returncode == 0 and result.stdout.strip():
                    return float(result.stdout.strip())
            elif sys.platform.startswith("linux"):  # Linux
                # Try to get Reframe process CPU
                cmd = "ps aux | grep -E '(cargo.*run|reframe)' | grep -v grep | awk '{print $3}' | head -1"
                result = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=1)
                if result.returncode == 0 and result.stdout.strip():
                    try:
                        return float(result.stdout.strip())
                    except ValueError:
                        pass
                
                # Fall back to overall CPU
                cmd = "top -bn1 | grep 'Cpu(s)' | awk '{print $2 + $4}'"
                result = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=2)
                if result.returncode == 0 and result.stdout.strip():
                    return float(result.stdout.strip())
        except Exception as e:
            if self.debug:
                self._log(f"CPU monitoring error: {e}")
        return 0.0
    
    def _get_memory_usage_fallback(self) -> float:
        """Get memory usage without psutil (macOS/Linux fallback)"""
        try:
            if sys.platform == "darwin":  # macOS
                # Try to get Reframe process memory first
                cmd = "ps aux | grep -E '(cargo.*run|reframe)' | grep -v grep | awk '{print $6}' | head -1"
                result = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=1)
                if result.returncode == 0 and result.stdout.strip():
                    try:
                        # RSS is in KB, convert to MB
                        return float(result.stdout.strip()) / 1024
                    except ValueError:
                        pass
                
                # Fall back to overall memory usage
                cmd = "vm_stat | grep -E '(Pages active|Pages wired)' | awk '{sum+=$3} END {print sum}' | sed 's/\\.//'"
                result = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=2)
                if result.returncode == 0 and result.stdout.strip():
                    pages = int(result.stdout.strip())
                    # Each page is 4096 bytes on macOS
                    return (pages * 4096) / (1024 * 1024)
            elif sys.platform.startswith("linux"):  # Linux
                # Try to get Reframe process memory
                cmd = "ps aux | grep -E '(cargo.*run|reframe)' | grep -v grep | awk '{print $6}' | head -1"
                result = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=1)
                if result.returncode == 0 and result.stdout.strip():
                    try:
                        # RSS is in KB, convert to MB
                        return float(result.stdout.strip()) / 1024
                    except ValueError:
                        pass
                
                # Fall back to overall memory
                cmd = "free -m | grep Mem | awk '{print $3}'"
                result = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=2)
                if result.returncode == 0 and result.stdout.strip():
                    return float(result.stdout.strip())
        except Exception as e:
            if self.debug:
                self._log(f"Memory monitoring error: {e}")
        return 0.0
    
    def _find_reframe_process_stats(self) -> Dict[str, int]:
        """Find Reframe process and get its stats"""
        if not HAS_PSUTIL:
            # Try to get thread count via ps command
            try:
                if sys.platform == "darwin":  # macOS
                    cmd = "ps -M -p $(pgrep -f 'cargo.*run|reframe' | head -1) 2>/dev/null | wc -l | awk '{print $1-1}'"
                elif sys.platform.startswith("linux"):
                    cmd = "ps -T -p $(pgrep -f 'cargo.*run|reframe' | head -1) 2>/dev/null | wc -l | awk '{print $1-1}'"
                else:
                    return {'threads': 0, 'open_files': 0}
                    
                result = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=1)
                if result.returncode == 0 and result.stdout.strip():
                    try:
                        threads = int(result.stdout.strip())
                        return {'threads': max(0, threads), 'open_files': 0}
                    except ValueError:
                        pass
            except:
                pass
            return {'threads': 0, 'open_files': 0}
        
        try:
            # Look for cargo or reframe process
            for proc in psutil.process_iter(['pid', 'name', 'cmdline']):
                try:
                    cmdline = ' '.join(proc.info.get('cmdline', []))
                    if 'reframe' in cmdline.lower() or ('cargo' in cmdline.lower() and 'run' in cmdline):
                        # Found the Reframe process
                        process = psutil.Process(proc.info['pid'])
                        return {
                            'threads': process.num_threads(),
                            'open_files': len(process.open_files()) if sys.platform != "win32" else 0
                        }
                except (psutil.NoSuchProcess, psutil.AccessDenied):
                    continue
        except:
            pass
        return {'threads': 0, 'open_files': 0}
    
    def _monitor_system_resources(self, interval: float = 0.5):
        """Monitor system resources in background thread"""
        if not HAS_PSUTIL:
            # Try to get CPU usage via system commands as fallback
            while not self.stop_monitoring.is_set():
                cpu_percent = self._get_cpu_usage_fallback()
                memory_mb = self._get_memory_usage_fallback()
                reframe_stats = self._find_reframe_process_stats()
                self.system_metrics.append(SystemMetrics(
                    timestamp=time.time(),
                    cpu_percent=cpu_percent,
                    memory_mb=memory_mb,
                    thread_count=reframe_stats.get('threads', 0),
                    open_files=reframe_stats.get('open_files', 0)
                ))
                time.sleep(interval)
            return
            
        # Monitor Reframe process resources specifically
        while not self.stop_monitoring.is_set():
            try:
                reframe_process = None
                reframe_cpu = 0.0
                reframe_memory = 0.0
                
                # Find the Reframe/cargo process
                for proc in psutil.process_iter(['pid', 'name', 'cmdline']):
                    try:
                        cmdline = ' '.join(proc.info.get('cmdline', []))
                        if 'reframe' in cmdline.lower() or ('cargo' in cmdline.lower() and 'run' in cmdline):
                            reframe_process = psutil.Process(proc.info['pid'])
                            # Get CPU percentage for this process (0-100 per core, so can exceed 100 on multi-core)
                            reframe_cpu = reframe_process.cpu_percent(interval=None)
                            # Get memory in MB
                            reframe_memory = reframe_process.memory_info().rss / (1024 * 1024)
                            break
                    except (psutil.NoSuchProcess, psutil.AccessDenied):
                        continue
                
                if reframe_process:
                    metrics = SystemMetrics(
                        timestamp=time.time(),
                        cpu_percent=reframe_cpu,
                        memory_mb=reframe_memory,
                        thread_count=reframe_process.num_threads(),
                        open_files=len(reframe_process.open_files()) if sys.platform != "win32" else 0
                    )
                else:
                    # Fallback if process not found
                    metrics = SystemMetrics(
                        timestamp=time.time(),
                        cpu_percent=0.0,
                        memory_mb=0.0,
                        thread_count=0,
                        open_files=0
                    )
                
                self.system_metrics.append(metrics)
            except Exception as e:
                if self.debug:
                    self._log(f"Monitoring error: {e}")
            time.sleep(interval)
    
    def run_baseline_test(self) -> PerformanceMetrics:
        """Run baseline single-threaded performance test"""
        self._log("Starting baseline performance test (single-threaded)...")
        
        # Generate sample message
        sample_config = SAMPLE_MESSAGES["mt103_simple"]
        message = self._generate_sample_message(sample_config)
        if not message:
            raise Exception("Failed to generate sample message")
        
        # Start monitoring
        self.system_metrics = []
        monitor_thread = threading.Thread(target=self._monitor_system_resources)
        monitor_thread.start()
        
        # Run sequential requests
        num_requests = 100
        latencies = []
        successful = 0
        
        start_time = time.perf_counter()
        
        for i in range(num_requests):
            success, latency_ms = self._make_transformation_request(message)
            latencies.append(latency_ms)
            if success:
                successful += 1
            
            if (i + 1) % 20 == 0:
                self._log(f"Progress: {i + 1}/{num_requests} requests")
        
        duration = time.perf_counter() - start_time
        
        # Stop monitoring
        self.stop_monitoring.set()
        monitor_thread.join()
        
        # Calculate metrics
        return self._calculate_metrics(
            test_name="Baseline (Single-threaded)",
            duration=duration,
            total_requests=num_requests,
            successful_requests=successful,
            latencies=latencies,
            concurrent_connections=1
        )
    
    def run_load_test(self, concurrency: int = 10, total_requests: int = 1000, 
                      message_type: str = "mt103_simple") -> PerformanceMetrics:
        """Run load test with specified concurrency"""
        self._log(f"Starting load test (concurrency={concurrency}, requests={total_requests})...")
        
        # Generate sample message
        sample_config = SAMPLE_MESSAGES.get(message_type, SAMPLE_MESSAGES["mt103_simple"])
        message = self._generate_sample_message(sample_config)
        if not message:
            raise Exception("Failed to generate sample message")
        
        # Clear results queue
        while not self.results_queue.empty():
            self.results_queue.get()
        
        # Start monitoring
        self.system_metrics = []
        self.stop_monitoring.clear()
        monitor_thread = threading.Thread(target=self._monitor_system_resources)
        monitor_thread.start()
        
        # Calculate requests per worker
        requests_per_worker = total_requests // concurrency
        remaining = total_requests % concurrency
        
        # Start workers
        start_time = time.perf_counter()
        workers = []
        
        for i in range(concurrency):
            worker_requests = requests_per_worker + (1 if i < remaining else 0)
            worker = threading.Thread(target=self._worker_thread, 
                                     args=(message, worker_requests))
            worker.start()
            workers.append(worker)
        
        # Wait for completion
        for worker in workers:
            worker.join()
        
        duration = time.perf_counter() - start_time
        
        # Stop monitoring
        self.stop_monitoring.set()
        monitor_thread.join()
        
        # Collect results
        latencies = []
        successful = 0
        
        while not self.results_queue.empty():
            success, latency_ms = self.results_queue.get()
            latencies.append(latency_ms)
            if success:
                successful += 1
        
        # Calculate metrics
        return self._calculate_metrics(
            test_name=f"Load Test (c={concurrency})",
            duration=duration,
            total_requests=len(latencies),
            successful_requests=successful,
            latencies=latencies,
            concurrent_connections=concurrency
        )
    
    def run_stress_test(self, max_concurrency: int = 200, 
                       step: int = 20) -> List[PerformanceMetrics]:
        """Run gradual stress test with increasing load"""
        self._log("Starting stress test with gradual load increase...")
        
        results = []
        concurrency_levels = list(range(step, max_concurrency + 1, step))
        
        for concurrency in concurrency_levels:
            self._log(f"\nTesting with {concurrency} concurrent connections...")
            
            try:
                metrics = self.run_load_test(
                    concurrency=concurrency,
                    total_requests=concurrency * 10  # 10 requests per connection
                )
                results.append(metrics)
                
                # Print intermediate results
                print(f"  Throughput: {metrics.throughput_rps:.2f} req/s")
                print(f"  P99 Latency: {metrics.latency_p99_ms:.2f} ms")
                print(f"  Error Rate: {metrics.error_rate_percent:.2f}%")
                
                # Stop if error rate is too high
                if metrics.error_rate_percent > 50:
                    self._log("Error rate exceeded 50%, stopping stress test")
                    break
                    
            except Exception as e:
                self._log(f"Error at concurrency {concurrency}: {e}")
                break
            
            # Brief pause between tests
            time.sleep(2)
        
        return results
    
    def run_spike_test(self, spike_concurrency: int = 500, 
                       duration_seconds: int = 10) -> PerformanceMetrics:
        """Run spike test with sudden load increase"""
        self._log(f"Starting spike test ({spike_concurrency} connections for {duration_seconds}s)...")
        
        # Generate sample message
        message = self._generate_sample_message(SAMPLE_MESSAGES["mt103_simple"])
        if not message:
            raise Exception("Failed to generate sample message")
        
        # Clear results queue
        while not self.results_queue.empty():
            self.results_queue.get()
        
        # Start monitoring
        self.system_metrics = []
        self.stop_monitoring.clear()
        monitor_thread = threading.Thread(target=self._monitor_system_resources)
        monitor_thread.start()
        
        # Create spike load
        start_time = time.perf_counter()
        stop_time = start_time + duration_seconds
        workers = []
        
        def spike_worker():
            while time.perf_counter() < stop_time:
                success, latency_ms = self._make_transformation_request(message)
                self.results_queue.put((success, latency_ms))
        
        # Launch all workers simultaneously
        for _ in range(spike_concurrency):
            worker = threading.Thread(target=spike_worker)
            worker.start()
            workers.append(worker)
        
        # Wait for duration
        time.sleep(duration_seconds)
        
        # Wait for workers to finish
        for worker in workers:
            worker.join(timeout=5)
        
        actual_duration = time.perf_counter() - start_time
        
        # Stop monitoring
        self.stop_monitoring.set()
        monitor_thread.join()
        
        # Collect results
        latencies = []
        successful = 0
        
        while not self.results_queue.empty():
            success, latency_ms = self.results_queue.get()
            latencies.append(latency_ms)
            if success:
                successful += 1
        
        # Calculate metrics
        return self._calculate_metrics(
            test_name=f"Spike Test (c={spike_concurrency})",
            duration=actual_duration,
            total_requests=len(latencies),
            successful_requests=successful,
            latencies=latencies,
            concurrent_connections=spike_concurrency
        )
    
    def run_endurance_test(self, concurrency: int = 50, 
                          duration_minutes: int = 5) -> PerformanceMetrics:
        """Run endurance test for extended period"""
        self._log(f"Starting endurance test ({concurrency} connections for {duration_minutes} minutes)...")
        
        # Generate sample message
        message = self._generate_sample_message(SAMPLE_MESSAGES["mt103_simple"])
        if not message:
            raise Exception("Failed to generate sample message")
        
        # Clear results queue
        while not self.results_queue.empty():
            self.results_queue.get()
        
        # Start monitoring
        self.system_metrics = []
        self.stop_monitoring.clear()
        monitor_thread = threading.Thread(target=self._monitor_system_resources)
        monitor_thread.start()
        
        # Run for specified duration
        start_time = time.perf_counter()
        stop_time = start_time + (duration_minutes * 60)
        workers = []
        
        def endurance_worker():
            while time.perf_counter() < stop_time:
                success, latency_ms = self._make_transformation_request(message)
                self.results_queue.put((success, latency_ms))
        
        # Launch workers
        for _ in range(concurrency):
            worker = threading.Thread(target=endurance_worker)
            worker.start()
            workers.append(worker)
        
        # Monitor progress
        last_report = start_time
        while time.perf_counter() < stop_time:
            time.sleep(10)
            current_time = time.perf_counter()
            if current_time - last_report >= 30:  # Report every 30 seconds
                elapsed = (current_time - start_time) / 60
                remaining = duration_minutes - elapsed
                self._log(f"Progress: {elapsed:.1f}/{duration_minutes} minutes, "
                         f"{remaining:.1f} minutes remaining")
                last_report = current_time
        
        # Wait for workers to finish
        for worker in workers:
            worker.join(timeout=5)
        
        actual_duration = time.perf_counter() - start_time
        
        # Stop monitoring
        self.stop_monitoring.set()
        monitor_thread.join()
        
        # Collect results
        latencies = []
        successful = 0
        
        while not self.results_queue.empty():
            success, latency_ms = self.results_queue.get()
            latencies.append(latency_ms)
            if success:
                successful += 1
        
        # Calculate metrics
        return self._calculate_metrics(
            test_name=f"Endurance Test (c={concurrency}, {duration_minutes}min)",
            duration=actual_duration,
            total_requests=len(latencies),
            successful_requests=successful,
            latencies=latencies,
            concurrent_connections=concurrency
        )
    
    def _calculate_metrics(self, test_name: str, duration: float, 
                          total_requests: int, successful_requests: int,
                          latencies: List[float], 
                          concurrent_connections: int) -> PerformanceMetrics:
        """Calculate performance metrics from test results"""
        
        # Calculate latency percentiles
        if latencies:
            sorted_latencies = sorted(latencies)
            
            # Calculate percentiles
            if HAS_NUMPY:
                p50 = np.percentile(sorted_latencies, 50)
                p95 = np.percentile(sorted_latencies, 95)
                p99 = np.percentile(sorted_latencies, 99)
            else:
                # Basic percentile calculation without numpy
                def percentile(data, percent):
                    n = len(data)
                    idx = (n - 1) * percent / 100
                    lower = int(idx)
                    upper = lower + 1
                    if upper >= n:
                        return data[lower]
                    weight = idx - lower
                    return data[lower] * (1 - weight) + data[upper] * weight
                
                p50 = percentile(sorted_latencies, 50)
                p95 = percentile(sorted_latencies, 95)
                p99 = percentile(sorted_latencies, 99)
            
            min_latency = min(sorted_latencies)
            max_latency = max(sorted_latencies)
            mean_latency = statistics.mean(sorted_latencies)
            stdev_latency = statistics.stdev(sorted_latencies) if len(sorted_latencies) > 1 else 0
        else:
            p50 = p95 = p99 = min_latency = max_latency = mean_latency = stdev_latency = 0
        
        # Calculate system metrics averages
        if self.system_metrics:
            avg_cpu = statistics.mean([m.cpu_percent for m in self.system_metrics])
            avg_memory = statistics.mean([m.memory_mb for m in self.system_metrics])
            avg_threads = statistics.mean([m.thread_count for m in self.system_metrics])
        else:
            avg_cpu = avg_memory = avg_threads = 0
        
        # Calculate derived metrics
        throughput = total_requests / duration if duration > 0 else 0
        error_rate = ((total_requests - successful_requests) / total_requests * 100) if total_requests > 0 else 0
        
        return PerformanceMetrics(
            test_name=test_name,
            timestamp=datetime.now().isoformat(),
            duration_seconds=duration,
            total_requests=total_requests,
            successful_requests=successful_requests,
            failed_requests=total_requests - successful_requests,
            throughput_rps=throughput,
            latency_p50_ms=p50,
            latency_p95_ms=p95,
            latency_p99_ms=p99,
            latency_min_ms=min_latency,
            latency_max_ms=max_latency,
            latency_mean_ms=mean_latency,
            latency_stdev_ms=stdev_latency,
            cpu_usage_percent=avg_cpu,
            memory_usage_mb=avg_memory,
            thread_count=int(avg_threads),
            concurrent_connections=concurrent_connections,
            error_rate_percent=error_rate,
            reframe_version=self.reframe_version,
            system_info=self.system_info
        )


# ==================== Apache Bench Integration ====================

class ApacheBenchRunner:
    """Run performance tests using Apache Bench (ab)"""
    
    def __init__(self, base_url: str = "http://localhost:3000"):
        self.base_url = base_url
        
    def check_ab_installed(self) -> bool:
        """Check if Apache Bench is installed"""
        try:
            result = subprocess.run(["which", "ab"], capture_output=True, text=True)
            return result.returncode == 0
        except:
            return False
    
    def generate_sample_file(self, message_type: str = "mt103_simple") -> str:
        """Generate sample message file for ab testing"""
        runner = PerformanceTestRunner(self.base_url)
        sample_config = SAMPLE_MESSAGES.get(message_type, SAMPLE_MESSAGES["mt103_simple"])
        message = runner._generate_sample_message(sample_config)
        
        if not message:
            raise Exception("Failed to generate sample message")
        
        # Save to temporary file
        filename = f"/tmp/reframe_test_{message_type}.json"
        with open(filename, 'w') as f:
            json.dump({"message": message}, f)
        
        return filename
    
    def run_ab_test(self, requests: int = 1000, concurrency: int = 100, 
                    endpoint: str = "/transform/mt-to-mx") -> Dict[str, Any]:
        """Run Apache Bench test"""
        
        # Generate sample file
        sample_file = self.generate_sample_file()
        
        # Construct ab command
        url = f"{self.base_url}{endpoint}"
        cmd = [
            "ab",
            "-n", str(requests),
            "-c", str(concurrency),
            "-p", sample_file,
            "-T", "application/json",
            "-H", "Content-Type: application/json",
            url
        ]
        
        print(f"Running: {' '.join(cmd)}")
        
        # Run ab
        result = subprocess.run(cmd, capture_output=True, text=True)
        
        # Parse output
        output = result.stdout
        metrics = self._parse_ab_output(output)
        
        # Clean up
        os.remove(sample_file)
        
        return metrics
    
    def _parse_ab_output(self, output: str) -> Dict[str, Any]:
        """Parse Apache Bench output"""
        metrics = {}
        
        lines = output.split('\n')
        for line in lines:
            if 'Requests per second:' in line:
                metrics['throughput_rps'] = float(line.split(':')[1].split('[')[0].strip())
            elif 'Time per request:' in line and '(mean)' in line:
                metrics['mean_latency_ms'] = float(line.split(':')[1].split('[')[0].strip())
            elif 'Failed requests:' in line:
                metrics['failed_requests'] = int(line.split(':')[1].strip())
            elif 'Complete requests:' in line:
                metrics['complete_requests'] = int(line.split(':')[1].strip())
            elif 'Percentage of the requests served within a certain time' in line:
                # Parse percentile table
                percentiles = {}
                i = lines.index(line) + 1
                while i < len(lines) and lines[i].strip():
                    parts = lines[i].strip().split()
                    if len(parts) >= 2:
                        percentile = parts[0].rstrip('%')
                        time_ms = parts[1]
                        if percentile in ['50', '95', '99']:
                            percentiles[f'p{percentile}'] = float(time_ms)
                    i += 1
                metrics['percentiles'] = percentiles
        
        return metrics


# ==================== Result Comparison and Reporting ====================

class PerformanceReporter:
    """Generate performance test reports and comparisons"""
    
    @staticmethod
    def save_results(metrics: PerformanceMetrics, filename: str):
        """Save metrics to JSON file"""
        with open(filename, 'w') as f:
            json.dump(metrics.to_dict(), f, indent=2)
        print(f"Results saved to {filename}")
    
    @staticmethod
    def load_results(filename: str) -> PerformanceMetrics:
        """Load metrics from JSON file"""
        with open(filename, 'r') as f:
            data = json.load(f)
        return PerformanceMetrics(**data)
    
    @staticmethod
    def compare_results(baseline: PerformanceMetrics, 
                       optimized: PerformanceMetrics) -> Dict[str, Any]:
        """Compare two sets of performance metrics"""
        
        comparison = {
            "throughput_improvement": (optimized.throughput_rps / baseline.throughput_rps) if baseline.throughput_rps > 0 else 0,
            "latency_p99_improvement": (baseline.latency_p99_ms / optimized.latency_p99_ms) if optimized.latency_p99_ms > 0 else 0,
            "cpu_efficiency": (optimized.throughput_rps / optimized.cpu_usage_percent) / (baseline.throughput_rps / baseline.cpu_usage_percent) if baseline.cpu_usage_percent > 0 and baseline.throughput_rps > 0 else 0,
        }
        
        print("\n" + "="*60)
        print("Performance Comparison Report")
        print("="*60)
        
        # Show version information
        print(f"\n📦 VERSION INFO:")
        print(f"  Baseline Version:  {baseline.reframe_version}")
        print(f"  Optimized Version: {optimized.reframe_version}")
        if baseline.reframe_version != optimized.reframe_version:
            print(f"  ⚠️  Different versions being compared!")
        
        # Throughput comparison
        print(f"\n📊 THROUGHPUT:")
        print(f"  Baseline:  {baseline.throughput_rps:.2f} req/s")
        print(f"  Optimized: {optimized.throughput_rps:.2f} req/s")
        print(f"  Improvement: {comparison['throughput_improvement']:.1f}x")
        
        # Latency comparison
        print(f"\n⏱️ LATENCY (P99):")
        print(f"  Baseline:  {baseline.latency_p99_ms:.2f} ms")
        print(f"  Optimized: {optimized.latency_p99_ms:.2f} ms")
        print(f"  Improvement: {comparison['latency_p99_improvement']:.1f}x faster")
        
        # CPU utilization
        print(f"\n💻 CPU UTILIZATION:")
        print(f"  Baseline:  {baseline.cpu_usage_percent:.1f}%")
        print(f"  Optimized: {optimized.cpu_usage_percent:.1f}%")
        
        # Concurrency
        print(f"\n🔄 CONCURRENCY:")
        print(f"  Baseline:  {baseline.concurrent_connections} connections")
        print(f"  Optimized: {optimized.concurrent_connections} connections")
        
        # Success criteria check (from scaling.md)
        print("\n" + "="*60)
        print("Success Criteria Validation (from scaling.md):")
        print("="*60)
        
        criteria = [
            ("10x throughput improvement", comparison['throughput_improvement'] >= 10),
            (">70% CPU utilization under load", optimized.cpu_usage_percent > 70),
            ("P99 latency <200ms at 80% capacity", optimized.latency_p99_ms < 200),
            ("Zero request drops under normal load", optimized.error_rate_percent < 1),
        ]
        
        for criterion, met in criteria:
            status = "✅" if met else "❌"
            print(f"  {status} {criterion}")
        
        return comparison
    
    @staticmethod
    def generate_html_report(results: List[PerformanceMetrics], output_file: str = "performance_report.html"):
        """Generate HTML performance report with charts"""
        
        # Get version info from first result if available
        version_info = ""
        system_info = ""
        if results and len(results) > 0:
            first_result = results[0]
            version_info = f"<p><strong>Reframe Version:</strong> {first_result.reframe_version}</p>"
            if first_result.system_info:
                sys_details = []
                if "os_version" in first_result.system_info:
                    sys_details.append(f"OS: {first_result.system_info['os_version']}")
                if "cpu_count" in first_result.system_info:
                    sys_details.append(f"CPU Cores: {first_result.system_info['cpu_count']}")
                if sys_details:
                    system_info = f"<p><strong>System:</strong> {', '.join(sys_details)}</p>"
        
        html_template = """
        <!DOCTYPE html>
        <html>
        <head>
            <title>Reframe Performance Test Report</title>
            <script src="https://cdn.plot.ly/plotly-latest.min.js"></script>
            <style>
                body {{ font-family: Arial, sans-serif; margin: 20px; }}
                h1 {{ color: #333; }}
                .metric-card {{ 
                    background: #f5f5f5; 
                    padding: 15px; 
                    margin: 10px 0; 
                    border-radius: 5px;
                }}
                .chart {{ width: 100%; height: 400px; margin: 20px 0; }}
                .version-info {{ 
                    background: #e8f4f8; 
                    padding: 10px; 
                    border-left: 4px solid #2196F3;
                    margin: 10px 0;
                }}
            </style>
        </head>
        <body>
            <h1>Reframe Performance Test Report</h1>
            <p>Generated: {timestamp}</p>
            <div class="version-info">
                {version_info}
                {system_info}
            </div>
            
            <div id="throughput-chart" class="chart"></div>
            <div id="latency-chart" class="chart"></div>
            <div id="cpu-chart" class="chart"></div>
            
            <script>
                {charts_script}
            </script>
        </body>
        </html>
        """
        
        # Prepare data for charts
        concurrency = [r.concurrent_connections for r in results]
        throughput = [r.throughput_rps for r in results]
        latency_p99 = [r.latency_p99_ms for r in results]
        cpu_usage = [r.cpu_usage_percent for r in results]
        
        charts_script = f"""
        // Throughput chart
        var throughputTrace = {{
            x: {concurrency},
            y: {throughput},
            type: 'scatter',
            mode: 'lines+markers',
            name: 'Throughput'
        }};
        
        var throughputLayout = {{
            title: 'Throughput vs Concurrency',
            xaxis: {{ title: 'Concurrent Connections' }},
            yaxis: {{ title: 'Requests per Second' }}
        }};
        
        Plotly.newPlot('throughput-chart', [throughputTrace], throughputLayout);
        
        // Latency chart
        var latencyTrace = {{
            x: {concurrency},
            y: {latency_p99},
            type: 'scatter',
            mode: 'lines+markers',
            name: 'P99 Latency'
        }};
        
        var latencyLayout = {{
            title: 'P99 Latency vs Concurrency',
            xaxis: {{ title: 'Concurrent Connections' }},
            yaxis: {{ title: 'Latency (ms)' }}
        }};
        
        Plotly.newPlot('latency-chart', [latencyTrace], latencyLayout);
        
        // CPU chart
        var cpuTrace = {{
            x: {concurrency},
            y: {cpu_usage},
            type: 'scatter',
            mode: 'lines+markers',
            name: 'CPU Usage'
        }};
        
        var cpuLayout = {{
            title: 'CPU Usage vs Concurrency',
            xaxis: {{ title: 'Concurrent Connections' }},
            yaxis: {{ title: 'CPU Usage (%)' }}
        }};
        
        Plotly.newPlot('cpu-chart', [cpuTrace], cpuLayout);
        """
        
        html_content = html_template.format(
            timestamp=datetime.now().isoformat(),
            version_info=version_info,
            system_info=system_info,
            charts_script=charts_script
        )
        
        with open(output_file, 'w') as f:
            f.write(html_content)
        
        print(f"HTML report generated: {output_file}")


# ==================== Main Entry Point ====================

def main():
    parser = argparse.ArgumentParser(
        description="Performance testing suite for Reframe transformation service",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Run baseline test
  python3 test/performance_test.py --baseline
  
  # Run load test with 100 concurrent connections
  python3 test/performance_test.py --test load -c 100 -n 1000
  
  # Run stress test up to 200 connections
  python3 test/performance_test.py --test stress --max-concurrency 200
  
  # Run full test suite
  python3 test/performance_test.py --test all
  
  # Compare results
  python3 test/performance_test.py --compare baseline.json optimized.json
        """
    )
    
    parser.add_argument('--url', default='http://localhost:3000',
                       help='Base URL of Reframe service')
    
    parser.add_argument('--test', choices=['baseline', 'load', 'stress', 'spike', 'endurance', 'ab', 'all'],
                       help='Type of test to run')
    
    parser.add_argument('--baseline', action='store_true',
                       help='Run baseline test and save results')
    
    parser.add_argument('-c', '--concurrency', type=int, default=10,
                       help='Number of concurrent connections')
    
    parser.add_argument('-n', '--requests', type=int, default=1000,
                       help='Total number of requests')
    
    parser.add_argument('--max-concurrency', type=int, default=200,
                       help='Maximum concurrency for stress test')
    
    parser.add_argument('--duration', type=int, default=5,
                       help='Duration in minutes for endurance test')
    
    parser.add_argument('--save', help='Save results to JSON file')
    
    parser.add_argument('--compare', nargs=2, metavar=('BASELINE', 'OPTIMIZED'),
                       help='Compare two result files')
    
    parser.add_argument('--html-report', action='store_true',
                       help='Generate HTML report')
    
    parser.add_argument('--debug', action='store_true',
                       help='Enable debug output')
    
    args = parser.parse_args()
    
    # Handle comparison
    if args.compare:
        reporter = PerformanceReporter()
        baseline = reporter.load_results(args.compare[0])
        optimized = reporter.load_results(args.compare[1])
        reporter.compare_results(baseline, optimized)
        return
    
    # Initialize test runner
    runner = PerformanceTestRunner(base_url=args.url, debug=args.debug)
    results = []
    
    try:
        # Run specified tests
        if args.baseline or args.test == 'baseline':
            print("Running baseline performance test...")
            metrics = runner.run_baseline_test()
            metrics.print_summary()
            results.append(metrics)
            
            if args.save or args.baseline:
                filename = args.save or 'baseline.json'
                PerformanceReporter.save_results(metrics, filename)
        
        elif args.test == 'load':
            print(f"Running load test (c={args.concurrency}, n={args.requests})...")
            metrics = runner.run_load_test(
                concurrency=args.concurrency,
                total_requests=args.requests
            )
            metrics.print_summary()
            results.append(metrics)
            
            if args.save:
                PerformanceReporter.save_results(metrics, args.save)
        
        elif args.test == 'stress':
            print(f"Running stress test (max_c={args.max_concurrency})...")
            stress_results = runner.run_stress_test(max_concurrency=args.max_concurrency)
            for metrics in stress_results:
                metrics.print_summary()
            results.extend(stress_results)
            
            if args.save:
                # Save all stress test results
                all_results = [m.to_dict() for m in stress_results]
                with open(args.save, 'w') as f:
                    json.dump(all_results, f, indent=2)
        
        elif args.test == 'spike':
            print("Running spike test...")
            metrics = runner.run_spike_test(spike_concurrency=args.concurrency)
            metrics.print_summary()
            results.append(metrics)
            
            if args.save:
                PerformanceReporter.save_results(metrics, args.save)
        
        elif args.test == 'endurance':
            print(f"Running endurance test ({args.duration} minutes)...")
            metrics = runner.run_endurance_test(
                concurrency=args.concurrency,
                duration_minutes=args.duration
            )
            metrics.print_summary()
            results.append(metrics)
            
            if args.save:
                PerformanceReporter.save_results(metrics, args.save)
        
        elif args.test == 'ab':
            print("Running Apache Bench test...")
            ab_runner = ApacheBenchRunner(base_url=args.url)
            
            if not ab_runner.check_ab_installed():
                print("Error: Apache Bench (ab) is not installed")
                print("Install with: apt-get install apache2-utils (Ubuntu) or brew install ab (macOS)")
                return
            
            ab_results = ab_runner.run_ab_test(
                requests=args.requests,
                concurrency=args.concurrency
            )
            print("Apache Bench Results:")
            print(json.dumps(ab_results, indent=2))
        
        elif args.test == 'all':
            print("Running complete test suite...")
            
            # Baseline
            print("\n1. BASELINE TEST")
            metrics = runner.run_baseline_test()
            results.append(metrics)
            
            # Load tests
            print("\n2. LOAD TESTS")
            for c in [10, 50, 100]:
                metrics = runner.run_load_test(concurrency=c, total_requests=c * 100)
                results.append(metrics)
            
            # Stress test
            print("\n3. STRESS TEST")
            stress_results = runner.run_stress_test(max_concurrency=200, step=50)
            results.extend(stress_results)
            
            # Save all results
            if args.save:
                all_results = [m.to_dict() for m in results]
                with open(args.save, 'w') as f:
                    json.dump(all_results, f, indent=2)
        
        # Generate HTML report if requested
        if args.html_report and results:
            PerformanceReporter.generate_html_report(results)
    
    except KeyboardInterrupt:
        print("\nTest interrupted by user")
    except Exception as e:
        print(f"Error: {e}")
        if args.debug:
            import traceback
            traceback.print_exc()


if __name__ == "__main__":
    main()