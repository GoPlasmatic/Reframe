#!/usr/bin/env python3
"""
Simple benchmark to test Reframe performance with different configurations
"""

import asyncio
import aiohttp
import time
import sys
import json

async def make_request(session, url, data):
    """Make a single request and return latency"""
    start = time.perf_counter()
    try:
        async with session.post(url, json=data) as resp:
            await resp.text()
            return time.perf_counter() - start, resp.status == 200
    except:
        return time.perf_counter() - start, False

async def get_sample_message(session):
    """Get a sample MT103 message from the generator API"""
    try:
        async with session.post("http://localhost:3000/generate/sample",
                                json={"message_type": "MT103", "config": {"scenario": "standard"}}) as resp:
            if resp.status == 200:
                result = await resp.json()
                return {
                    "message": result.get("result", result.get("message", "")),
                    "options": {"validation": False}
                }
    except:
        pass
    
    # Fallback message
    return {
        "message": "{1:F01BANKBEBBAXXX0237205215}{2:O103080907BANKFRPPAXXX02372052150809070917N}{3:{108:ILOVESEPA}}{4:\n:20:REF12345678901234\n:23B:CRED\n:32A:240101EUR1000,00\n:50K:/12345678901234567890\nJOHN DOE\n123 MAIN STREET\nANYTOWN\n:59:/98765432109876543210\nJANE SMITH\n456 PARK AVENUE\nOTHERCITY\n:71A:SHA\n-}",
        "options": {"validation": False}
    }

async def run_test(num_requests=100, concurrent=8):
    """Run a simple performance test"""
    url = "http://localhost:3000/transform/mt-to-mx"
    
    async with aiohttp.ClientSession() as session:
        # Get sample message
        print("Getting sample message...")
        data = await get_sample_message(session)
        
        # Warmup
        print(f"Warming up with 10 requests...")
        for _ in range(10):
            await make_request(session, url, data)
        
        print(f"Running {num_requests} requests with {concurrent} concurrent tasks...")
        start_time = time.perf_counter()
        
        tasks = []
        latencies = []
        successes = 0
        
        # Process in batches
        for i in range(0, num_requests, concurrent):
            batch_size = min(concurrent, num_requests - i)
            batch = [make_request(session, url, data) for _ in range(batch_size)]
            results = await asyncio.gather(*batch)
            
            for latency, success in results:
                latencies.append(latency)
                if success:
                    successes += 1
        
        total_time = time.perf_counter() - start_time
        
        # Calculate statistics
        latencies.sort()
        avg_latency = sum(latencies) / len(latencies) if latencies else 0
        min_latency = latencies[0] if latencies else 0
        max_latency = latencies[-1] if latencies else 0
        
        # Calculate percentiles
        def get_percentile(data, percentile):
            if not data:
                return 0
            index = int(len(data) * percentile / 100)
            if index >= len(data):
                index = len(data) - 1
            return data[index]
        
        p95_latency = get_percentile(latencies, 95)
        p99_latency = get_percentile(latencies, 99)
        
        throughput = num_requests / total_time if total_time > 0 else 0
        
        print(f"\nResults:")
        print(f"  Total time:     {total_time:.2f} seconds")
        print(f"  Throughput:     {throughput:.1f} req/s")
        print(f"  Success rate:   {(successes/num_requests)*100:.1f}%")
        print(f"  Total requests: {num_requests}")
        print(f"  Successful:     {successes}")
        print(f"\nLatency Statistics (ms):")
        print(f"  Min:            {min_latency*1000:.1f} ms")
        print(f"  Avg:            {avg_latency*1000:.1f} ms")
        print(f"  P95:            {p95_latency*1000:.1f} ms")
        print(f"  P99:            {p99_latency*1000:.1f} ms")
        print(f"  Max:            {max_latency*1000:.1f} ms")
        
        return {
            'throughput': throughput,
            'min_latency': min_latency * 1000,
            'avg_latency': avg_latency * 1000,
            'p95_latency': p95_latency * 1000,
            'p99_latency': p99_latency * 1000,
            'max_latency': max_latency * 1000
        }

async def main():
    """Test different concurrency levels"""
    print("Simple Reframe Performance Test")
    print("================================\n")
    
    # Check if server is running
    try:
        async with aiohttp.ClientSession() as session:
            async with session.get("http://localhost:3000/health") as resp:
                if resp.status != 200:
                    print("Server is not running!")
                    return
                health = await resp.json()
                print(f"Server is running: {health['engines']}\n")
    except:
        print("Cannot connect to server on port 3000!")
        return
    
    # Test different concurrency levels
    configs = [
        (100000, 8, "8 concurrent"),
        (100000, 32, "32 concurrent"),
        (100000, 128, "128 concurrent"),
        (100000, 256, "256 concurrent"),
    ]
    
    results = []
    for requests, concurrent, desc in configs:
        print(f"\n--- Testing: {desc} ---")
        stats = await run_test(requests, concurrent)
        results.append((desc, stats))
    
    # Summary
    print("\n\n=== SUMMARY ===")
    print(f"{'Configuration':<20} {'Throughput':<12} {'Min':<8} {'Avg':<8} {'P95':<8} {'P99':<8} {'Max':<8}")
    print(f"{'                   ':<20} {'(req/s)':<12} {'(ms)':<8} {'(ms)':<8} {'(ms)':<8} {'(ms)':<8} {'(ms)':<8}")
    print("-" * 92)
    for desc, stats in results:
        print(f"{desc:<20} {stats['throughput']:<12.1f} {stats['min_latency']:<8.1f} {stats['avg_latency']:<8.1f} {stats['p95_latency']:<8.1f} {stats['p99_latency']:<8.1f} {stats['max_latency']:<8.1f}")
    
    best = max(results, key=lambda x: x[1]['throughput'])
    print(f"\nBest throughput: {best[0]} with {best[1]['throughput']:.1f} req/s")
    
    best_latency = min(results, key=lambda x: x[1]['p99_latency'])
    print(f"Best P99 latency: {best_latency[0]} with {best_latency[1]['p99_latency']:.1f} ms")

if __name__ == "__main__":
    asyncio.run(main())