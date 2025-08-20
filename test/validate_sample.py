#!/usr/bin/env python3

import argparse
import json
import requests
import sys

def generate_sample(message_type, scenario=None, host="http://localhost:3000", debug=False):
    """
    Generate a sample message using the Reframe API
    
    Args:
        message_type: The message type to generate (e.g., MT103, pacs.008)
        scenario: The scenario to use for generation
        host: The API host URL
        debug: Enable debug output
    
    Returns:
        The generated message as a string
    """
    url = f"{host}/generate/sample"
    
    payload = {
        "message_type": message_type,
        "options": {
            "debug": debug,
            "validation": False
        }
    }
    
    if scenario:
        payload["config"] = {"scenario": scenario}
    
    try:
        response = requests.post(url, json=payload)
        response.raise_for_status()
        
        result = response.json()
        
        if "error" in result:
            print(f"Error generating sample: {result['error']}", file=sys.stderr)
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
        print(f"HTTP Error during generation: {e}", file=sys.stderr)
        return None
    except Exception as e:
        print(f"Error during generation: {e}", file=sys.stderr)
        return None

def detect_message_type(message):
    """
    Detect if a message is MT or MX based on its content
    
    Args:
        message: The message content as a string
    
    Returns:
        'mt' for SWIFT MT messages, 'mx' for ISO 20022 XML messages
    """
    message = message.strip()
    
    # Check for XML declaration or ISO 20022 namespaces
    if message.startswith('<?xml') or 'urn:iso:std:iso:20022' in message:
        return 'mx'
    # Check for SWIFT MT message blocks
    elif '{1:' in message or '{2:' in message or message.startswith(':'):
        return 'mt'
    else:
        # Default to MT if unclear
        return 'mt'

def validate_message(message, message_type=None, host="http://localhost:3000", 
                    business_validation=False, canonical=True, fail_fast=False):
    """
    Validate a SWIFT MT or ISO 20022 message using the Reframe API
    
    Args:
        message: The message content to validate
        message_type: 'mt' or 'mx', auto-detected if not provided
        host: The API host URL
        business_validation: Enable business rule validation
        canonical: Use canonical format for validation
        fail_fast: Stop validation on first error
    
    Returns:
        The validation response as a dictionary
    """
    # Auto-detect message type if not provided
    if not message_type:
        message_type = detect_message_type(message)
        print(f"Auto-detected message type: {message_type.upper()}", file=sys.stderr)
    
    url = f"{host}/validate/{message_type}"
    
    payload = {
        "message": message,
        "options": {
            "business_validation": business_validation,
            "canonical": canonical,
            "fail_fast": fail_fast
        }
    }
    
    try:
        response = requests.post(url, json=payload)
        response.raise_for_status()
        
        return response.json()
            
    except requests.exceptions.ConnectionError:
        print(f"Error: Could not connect to {host}. Is the server running?", file=sys.stderr)
        return None
    except requests.exceptions.HTTPError as e:
        print(f"HTTP Error: {e}", file=sys.stderr)
        try:
            error_detail = response.json()
            return error_detail
        except:
            return {"error": str(e)}
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return {"error": str(e)}

def format_validation_result(result):
    """
    Format the validation result for display
    
    Args:
        result: The validation response dictionary
    
    Returns:
        Formatted string for display
    """
    if not result:
        return "No validation result"
    
    output = []
    
    # Check if validation passed or failed
    if "valid" in result:
        if result["valid"]:
            output.append("✅ VALIDATION PASSED")
        else:
            output.append("❌ VALIDATION FAILED")
        output.append("")
    
    # Display validation errors if present
    if "errors" in result and result["errors"]:
        output.append("Validation Errors:")
        output.append("-" * 40)
        for i, error in enumerate(result["errors"], 1):
            output.append(f"{i}. {error}")
        output.append("")
    
    # Display validation warnings if present
    if "warnings" in result and result["warnings"]:
        output.append("Warnings:")
        output.append("-" * 40)
        for i, warning in enumerate(result["warnings"], 1):
            output.append(f"{i}. {warning}")
        output.append("")
    
    # Display parsed message if present
    if "parsed_message" in result:
        output.append("Parsed Message:")
        output.append("-" * 40)
        output.append(json.dumps(result["parsed_message"], indent=2))
        output.append("")
    
    # Display any error message
    if "error" in result:
        output.append(f"Error: {result['error']}")
    
    # Display any additional info
    if "message_type" in result:
        output.append(f"Message Type: {result['message_type']}")
    
    if "details" in result:
        output.append("Additional Details:")
        output.append("-" * 40)
        output.append(json.dumps(result["details"], indent=2))
    
    return "\n".join(output)

def main():
    parser = argparse.ArgumentParser(description='Generate and validate SWIFT MT or ISO 20022 messages')
    parser.add_argument('message_type', help='Message type to generate and validate (e.g., MT103, pacs.008, camt.052)')
    parser.add_argument('-s', '--scenario', help='Scenario to use for generation (e.g., single_payment, bulk_payment)')
    parser.add_argument('-H', '--host', default='http://localhost:3000', help='API host URL (default: http://localhost:3000)')
    parser.add_argument('-d', '--debug', action='store_true', help='Enable debug output for generation')
    parser.add_argument('-b', '--business-validation', action='store_true', help='Enable business rule validation')
    parser.add_argument('-nc', '--no-canonical', action='store_false', dest='canonical', help='Disable canonical format')
    parser.add_argument('-f', '--fail-fast', action='store_true', help='Stop validation on first error')
    parser.add_argument('-j', '--json', action='store_true', help='Output raw JSON response')
    parser.add_argument('-v', '--verbose', action='store_true', help='Show generated message before validation')
    
    args = parser.parse_args()
    
    # Step 1: Generate the sample message
    print(f"Generating {args.message_type} sample...", file=sys.stderr)
    message = generate_sample(args.message_type, args.scenario, args.host, args.debug)
    
    if not message:
        print("Failed to generate sample message", file=sys.stderr)
        return 1
    
    if args.verbose:
        print("\n" + "="*60)
        print("GENERATED MESSAGE:")
        print("="*60)
        print(message)
        print("="*60 + "\n")
    
    # Step 2: Auto-detect message type for validation
    message_type = detect_message_type(message)
    print(f"Validating as {message_type.upper()} message...", file=sys.stderr)
    
    # Step 3: Validate the generated message
    result = validate_message(
        message=message,
        message_type=message_type,
        host=args.host,
        business_validation=args.business_validation,
        canonical=args.canonical,
        fail_fast=args.fail_fast
    )
    
    if result:
        if args.json:
            # Output raw JSON
            print(json.dumps(result, indent=2))
        else:
            # Output formatted result
            print(format_validation_result(result))
        
        # Return non-zero exit code if validation failed
        if "valid" in result and not result["valid"]:
            return 1
        return 0
    else:
        return 1

if __name__ == '__main__':
    sys.exit(main())