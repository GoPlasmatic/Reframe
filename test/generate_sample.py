#!/usr/bin/env python3

import argparse
import json
import requests
import sys

def generate_sample(message_type, scenario=None, host="http://localhost:3000", debug=False, validation=False):
    """
    Generate a sample message using the Reframe API
    
    Args:
        message_type: The message type to generate (e.g., MT103, pacs.008)
        scenario: The scenario to use for generation
        host: The API host URL
        debug: Enable debug output
        validation: Enable validation
    
    Returns:
        The generated message as a string
    """
    url = f"{host}/generate/sample"
    
    payload = {
        "message_type": message_type,
        "options": {
            "debug": debug,
            "validation": validation
        }
    }
    
    if scenario:
        payload["config"] = {"scenario": scenario}
    
    try:
        response = requests.post(url, json=payload)
        response.raise_for_status()
        
        result = response.json()
        
        if "error" in result:
            print(f"Error: {result['error']}", file=sys.stderr)
            return None
        
        # Check for message in different possible fields
        if "message" in result:
            return result["message"]
        elif "result" in result:
            return result["result"]
        else:
            print(f"Unexpected response format: {json.dumps(result, indent=2)}", file=sys.stderr)
            return None
            
    except requests.exceptions.ConnectionError:
        print(f"Error: Could not connect to {host}. Is the server running?", file=sys.stderr)
        return None
    except requests.exceptions.HTTPError as e:
        print(f"HTTP Error: {e}", file=sys.stderr)
        try:
            error_detail = response.json()
            print(f"Error details: {json.dumps(error_detail, indent=2)}", file=sys.stderr)
        except:
            pass
        return None
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return None

def main():
    parser = argparse.ArgumentParser(description='Generate sample SWIFT MT or ISO 20022 messages')
    parser.add_argument('message_type', help='Message type to generate (e.g., MT103, pacs.008, camt.052)')
    parser.add_argument('-s', '--scenario', help='Scenario to use for generation (e.g., single_payment, bulk_payment)')
    parser.add_argument('-H', '--host', default='http://localhost:3000', help='API host URL (default: http://localhost:3000)')
    parser.add_argument('-d', '--debug', action='store_true', help='Enable debug output')
    parser.add_argument('-v', '--validation', action='store_true', help='Enable validation')
    parser.add_argument('-o', '--output', help='Output file (default: stdout)')
    parser.add_argument('-p', '--pretty', action='store_true', help='Pretty print XML output')
    
    args = parser.parse_args()
    
    message = generate_sample(args.message_type, args.scenario, args.host, args.debug, args.validation)
    
    if message:
        # Pretty print XML if requested
        if args.pretty and message.strip().startswith('<?xml'):
            try:
                import xml.dom.minidom
                dom = xml.dom.minidom.parseString(message)
                message = dom.toprettyxml(indent="  ")
            except:
                pass  # If pretty printing fails, use original
        
        if args.output:
            with open(args.output, 'w') as f:
                f.write(message)
            print(f"Generated {args.message_type} message saved to {args.output}")
        else:
            print(message)
        
        return 0
    else:
        return 1

if __name__ == '__main__':
    sys.exit(main())