#!/usr/bin/env python3
"""
Simple memory leak detector for Reframe.
Sends repeated requests and analyzes memory growth patterns.
"""

import requests
import time
import psutil
import numpy as np
from scipy import stats
import matplotlib.pyplot as plt
import json
import argparse

class LeakDetector:
    def __init__(self, base_url="http://localhost:3000"):
        self.base_url = base_url
        self.memory_samples = []
        self.request_counts = []
        self.mt_sample = None
        self.mx_sample = None
        
    def find_process(self):
        """Find the Reframe process."""
        for proc in psutil.process_iter(['pid', 'name', 'cmdline']):
            try:
                if 'Reframe' in proc.info['name'] or \
                   any('Reframe' in str(arg) for arg in (proc.info['cmdline'] or [])):
                    return psutil.Process(proc.info['pid'])
            except (psutil.NoSuchProcess, psutil.AccessDenied):
                continue
        return None
    
    def get_sample_messages(self):
        """Get sample messages from the generator API."""
        try:
            # Get MT103 sample
            response = requests.post(f"{self.base_url}/generate/sample",
                                   json={"message_type": "MT103", "config": {"scenario": "standard"}},
                                   timeout=5)
            if response.status_code == 200:
                result = response.json()
                self.mt_sample = result.get("result", result.get("message", ""))
                print("✅ Got MT103 sample from generator")
            else:
                print("⚠️  Failed to get MT103 sample, using fallback")
                
            # Get pacs.008 sample  
            response = requests.post(f"{self.base_url}/generate/sample",
                                   json={"message_type": "pacs.008", "config": {"scenario": "high_value"}},
                                   timeout=5)
            if response.status_code == 200:
                result = response.json()
                self.mx_sample = result.get("result", result.get("message", ""))
                print("✅ Got pacs.008 sample from generator")
            else:
                print("⚠️  Failed to get pacs.008 sample, using fallback")
        except Exception as e:
            print(f"⚠️  Error getting samples: {e}, using fallbacks")
        
        # Use fallbacks if needed
        if not self.mt_sample:
            self.mt_sample = self._get_fallback_mt()
        if not self.mx_sample:
            self.mx_sample = self._get_fallback_mx()
    
    def _get_fallback_mt(self):
        """Get fallback MT103 message."""
        return """{1:F01BANKBEBBAXXX0000000000}{2:I103BANKDEFFXXXXN}{3:{108:ILOVESEPA}}{4:
:20:B4E08MS9D00A0009
:23B:CRED
:32A:141031EUR1875,75
:33B:EUR1875,75
:50K:/GB74HLFX11008081265013
JOHN DOE
:52A:BANKFRPPXXX
:59:/FR7630006000011234567890189
BEN BENEFICIARY
:71A:SHA
:72:/BNF/BENEFITS
-}"""

    def _get_fallback_mx(self):
        """Get fallback pacs.008 message."""
        return """<?xml version="1.0" encoding="UTF-8"?>
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:pacs.008.001.08">
    <FIToFICstmrCdtTrf>
        <GrpHdr>
            <MsgId>BANKMSG20231124001</MsgId>
            <CreDtTm>2023-11-24T10:30:00</CreDtTm>
            <NbOfTxs>1</NbOfTxs>
            <SttlmInf>
                <SttlmMtd>CLRG</SttlmMtd>
            </SttlmInf>
        </GrpHdr>
        <CdtTrfTxInf>
            <PmtId>
                <InstrId>INSTRID001</InstrId>
                <EndToEndId>E2E001</EndToEndId>
                <TxId>TXID001</TxId>
            </PmtId>
            <IntrBkSttlmAmt Ccy="EUR">1000.00</IntrBkSttlmAmt>
            <ChrgBr>SLEV</ChrgBr>
            <Dbtr>
                <Nm>John Doe</Nm>
            </Dbtr>
            <DbtrAcct>
                <Id>
                    <IBAN>GB74HLFX11008081265013</IBAN>
                </Id>
            </DbtrAcct>
            <DbtrAgt>
                <FinInstnId>
                    <BICFI>BANKBEBBXXX</BICFI>
                </FinInstnId>
            </DbtrAgt>
            <CdtrAgt>
                <FinInstnId>
                    <BICFI>BANKDEFFXXX</BICFI>
                </FinInstnId>
            </CdtrAgt>
            <Cdtr>
                <Nm>Ben Beneficiary</Nm>
            </Cdtr>
            <CdtrAcct>
                <Id>
                    <IBAN>FR7630006000011234567890189</IBAN>
                </Id>
            </CdtrAcct>
        </CdtTrfTxInf>
    </FIToFICstmrCdtTrf>
</Document>"""
    
    def send_request(self, request_type="mt-to-mx"):
        """Send a transformation request."""
        # Make sure we have samples
        if not self.mt_sample or not self.mx_sample:
            self.get_sample_messages()
            
        if request_type == "mt-to-mx":
            url = f"{self.base_url}/transform/mt-to-mx"
            payload = {
                "message": self.mt_sample,
                "options": {"debug": False}
            }
        else:
            url = f"{self.base_url}/transform/mx-to-mt"
            payload = {
                "message": self.mx_sample,
                "options": {"debug": False}
            }
        
        try:
            response = requests.post(url, json=payload, timeout=5)
            if response.status_code != 200:
                # Uncomment for debugging
                # print(f"Request failed: {response.status_code} - {response.text[:200]}")
                pass
            return response.status_code == 200
        except Exception as e:
            # Uncomment for debugging
            # print(f"Request error: {e}")
            return False
    
    def collect_samples(self, num_samples=100, requests_per_sample=10):
        """Collect memory samples."""
        process = self.find_process()
        if not process:
            print("❌ Reframe process not found")
            return False
        
        print(f"📊 Collecting {num_samples} samples...")
        print(f"   Each sample: {requests_per_sample} requests")
        
        # Get sample messages first
        print("📝 Getting sample messages...")
        self.get_sample_messages()
        
        # Initial stabilization
        print("⏳ Waiting for process to stabilize...")
        time.sleep(5)
        
        total_requests = 0
        
        for i in range(num_samples):
            # Send requests
            for _ in range(requests_per_sample):
                self.send_request("mt-to-mx" if total_requests % 2 == 0 else "mx-to-mt")
                total_requests += 1
            
            # Measure memory
            try:
                memory_mb = process.memory_info().rss / 1024 / 1024
                self.memory_samples.append(memory_mb)
                self.request_counts.append(total_requests)
                
                if (i + 1) % 10 == 0:
                    print(f"   Sample {i+1}/{num_samples}: {memory_mb:.2f} MB after {total_requests} requests")
            except:
                print(f"   ⚠️ Failed to get memory at sample {i+1}")
                continue
            
            # Small delay between samples
            time.sleep(0.5)
        
        return True
    
    def analyze_leak(self):
        """Analyze memory growth pattern for leaks."""
        if len(self.memory_samples) < 10:
            print("❌ Not enough samples for analysis")
            return None
        
        # Calculate linear regression
        slope, intercept, r_value, p_value, std_err = stats.linregress(
            self.request_counts, self.memory_samples
        )
        
        # Calculate memory growth rate
        memory_per_request_kb = slope * 1024  # Convert MB to KB (regression slope)
        r_squared = r_value ** 2
        
        # Also calculate simple average
        total_growth_mb = self.memory_samples[-1] - self.memory_samples[0]
        total_requests = self.request_counts[-1] if self.request_counts else 1
        simple_per_request_kb = (total_growth_mb / total_requests) * 1024 if total_requests > 0 else 0
        
        # Use the higher value for leak detection (more conservative)
        effective_leak_kb = max(abs(memory_per_request_kb), abs(simple_per_request_kb))
        
        # Determine if there's a leak
        has_leak = False
        severity = "None"
        
        if p_value < 0.05 and slope > 0:  # Statistically significant positive slope
            if effective_leak_kb > 100:
                has_leak = True
                severity = "CRITICAL"
            elif effective_leak_kb > 10:
                has_leak = True
                severity = "HIGH"
            elif effective_leak_kb > 1:
                has_leak = True
                severity = "MEDIUM"
            elif effective_leak_kb > 0.1:
                has_leak = True
                severity = "LOW"
        
        results = {
            'has_leak': has_leak,
            'severity': severity,
            'memory_per_request_kb': memory_per_request_kb,
            'memory_per_request_kb_simple': simple_per_request_kb,
            'effective_leak_kb': effective_leak_kb,
            'total_growth_mb': total_growth_mb,
            'r_squared': r_squared,
            'p_value': p_value,
            'samples': len(self.memory_samples),
            'total_requests': total_requests
        }
        
        return results
    
    def plot_results(self, filename="memory_leak_analysis.png"):
        """Plot memory usage over requests."""
        if len(self.memory_samples) < 2:
            return
        
        plt.figure(figsize=(12, 6))
        
        # Plot 1: Memory over requests
        plt.subplot(1, 2, 1)
        plt.plot(self.request_counts, self.memory_samples, 'b-', alpha=0.5, label='Actual')
        
        # Add trend line
        z = np.polyfit(self.request_counts, self.memory_samples, 1)
        p = np.poly1d(z)
        plt.plot(self.request_counts, p(self.request_counts), 'r--', 
                label=f'Trend (slope={z[0]:.4f} MB/req)')
        
        plt.xlabel('Number of Requests')
        plt.ylabel('Memory Usage (MB)')
        plt.title('Memory Usage vs Requests')
        plt.legend()
        plt.grid(True, alpha=0.3)
        
        # Plot 2: Memory growth rate
        plt.subplot(1, 2, 2)
        if len(self.memory_samples) > 1:
            growth_rates = np.diff(self.memory_samples)
            plt.plot(self.request_counts[1:], growth_rates, 'g-', alpha=0.5)
            plt.axhline(y=0, color='k', linestyle='-', linewidth=0.5)
            plt.xlabel('Number of Requests')
            plt.ylabel('Memory Growth (MB)')
            plt.title('Memory Growth Rate')
            plt.grid(True, alpha=0.3)
        
        plt.tight_layout()
        plt.savefig(filename)
        print(f"📈 Plot saved to: {filename}")
        plt.show()
    
    def print_report(self, results):
        """Print analysis report."""
        print("\n" + "="*60)
        print("MEMORY LEAK ANALYSIS REPORT")
        print("="*60)
        
        if results['has_leak']:
            print(f"⚠️  MEMORY LEAK DETECTED - Severity: {results['severity']}")
        else:
            print("✅ No significant memory leak detected")
        
        print(f"\n📊 Statistics:")
        print(f"   • Total requests:         {results['total_requests']}")
        print(f"   • Total memory growth:    {results['total_growth_mb']:.2f} MB")
        print(f"   • Memory per request (regression): {results['memory_per_request_kb']:.3f} KB")
        print(f"   • Memory per request (simple):     {results.get('memory_per_request_kb_simple', 0):.3f} KB")
        print(f"   • Effective leak rate:    {results.get('effective_leak_kb', 0):.3f} KB/request")
        print(f"   • R² (correlation):       {results['r_squared']:.4f}")
        print(f"   • P-value:               {results['p_value']:.6f}")
        
        if results['has_leak']:
            print(f"\n⚠️  Leak Analysis:")
            leak_rate = results.get('effective_leak_kb', results['memory_per_request_kb'])
            if leak_rate > 0:
                print(f"   • At this rate, the application will consume:")
                print(f"     - 1 GB after ~{int(1024 * 1024 / leak_rate):,} requests")
                print(f"     - 10 GB after ~{int(10 * 1024 * 1024 / leak_rate):,} requests")
            
            print(f"\n💡 Recommendations:")
            if results['severity'] == "CRITICAL":
                print("   • IMMEDIATE ACTION REQUIRED")
                print("   • Application will quickly exhaust memory")
                print("   • Review recent changes for memory allocation issues")
            elif results['severity'] == "HIGH":
                print("   • High priority fix needed")
                print("   • Application will run out of memory under load")
                print("   • Profile with heaptrack or valgrind for details")
            elif results['severity'] == "MEDIUM":
                print("   • Should be addressed soon")
                print("   • Will cause issues in long-running deployments")
            else:
                print("   • Minor leak - monitor in production")
                print("   • May accumulate over time")


def main():
    parser = argparse.ArgumentParser(description='Memory leak detector for Reframe')
    parser.add_argument('--url', default='http://localhost:3000', help='Reframe base URL')
    parser.add_argument('--samples', type=int, default=50, help='Number of samples to collect')
    parser.add_argument('--requests-per-sample', type=int, default=20, 
                       help='Requests to send per sample')
    parser.add_argument('--plot', action='store_true', help='Generate plot of results')
    
    args = parser.parse_args()
    
    print("🔍 Reframe Memory Leak Detector")
    print("="*60)
    
    # Check service
    try:
        response = requests.get(f"{args.url}/health", timeout=5)
        if response.status_code != 200:
            print(f"❌ Service not healthy at {args.url}")
            return
    except Exception as e:
        print(f"❌ Cannot connect to service: {e}")
        print("   Run: cargo run --release")
        return
    
    detector = LeakDetector(args.url)
    
    # Collect samples
    if not detector.collect_samples(args.samples, args.requests_per_sample):
        return
    
    # Analyze
    results = detector.analyze_leak()
    if not results:
        return
    
    # Report
    detector.print_report(results)
    
    # Save results
    with open('leak_analysis.json', 'w') as f:
        json.dump(results, f, indent=2)
    print(f"\n💾 Results saved to: leak_analysis.json")
    
    # Plot if requested
    if args.plot:
        detector.plot_results()


if __name__ == '__main__':
    main()