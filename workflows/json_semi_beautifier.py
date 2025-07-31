#!/usr/bin/env python3
"""
JSON Semi-Beautifier

A smart JSON formatter that keeps simple structures compact while breaking down
complex ones for readability. Specifically designed for JSONLogic workflows.
"""

import json
import sys
import argparse
from pathlib import Path
from typing import Any, Union


class SemiBeautifier:
    def __init__(self, max_inline_length=80, max_inline_items=3, indent=4):
        self.max_inline_length = max_inline_length
        self.max_inline_items = max_inline_items
        self.indent_str = ' ' * indent
        
    def should_inline(self, obj: Any) -> bool:
        """Determine if an object should be kept inline (compact)"""
        if isinstance(obj, (str, int, float, bool, type(None))):
            return True
            
        if isinstance(obj, dict):
            # Check if it's a simple JSONLogic operation
            if len(obj) == 1:
                key = next(iter(obj))
                value = obj[key]
                # Common JSONLogic operators
                if key in ['var', '==', '!=', '>', '<', '>=', '<=', 'in', 'cat', 'substr']:
                    return self.should_inline(value)
            
            # Check size constraints
            if len(obj) > self.max_inline_items:
                return False
            
            # Check if all values are simple
            for v in obj.values():
                if not self.should_inline(v):
                    return False
                    
            # Check estimated length
            estimated_length = len(json.dumps(obj, separators=(',', ':')))
            return estimated_length <= self.max_inline_length
            
        if isinstance(obj, list):
            if len(obj) > self.max_inline_items:
                return False
            
            # Check if all items are simple
            for item in obj:
                if not self.should_inline(item):
                    return False
                    
            # Check estimated length
            estimated_length = len(json.dumps(obj, separators=(',', ':')))
            return estimated_length <= self.max_inline_length
            
        return False
    
    def format_value(self, obj: Any, depth: int = 0) -> str:
        """Format a value with smart beautification"""
        indent = self.indent_str * depth
        next_indent = self.indent_str * (depth + 1)
        
        if isinstance(obj, (str, int, float, bool, type(None))):
            return json.dumps(obj)
            
        if self.should_inline(obj):
            # Keep it compact with proper spacing after commas
            return json.dumps(obj, separators=(', ', ': '))
            
        if isinstance(obj, dict):
            if not obj:
                return '{}'
                
            # Check for special JSONLogic patterns
            if len(obj) == 1:
                key = next(iter(obj))
                value = obj[key]
                
                # Special handling for 'if' statements
                if key == 'if' and isinstance(value, list) and len(value) >= 2:
                    result = '{ "if": [\n'
                    for i, item in enumerate(value):
                        result += next_indent + self.format_value(item, depth + 1)
                        if i < len(value) - 1:
                            result += ','
                        result += '\n'
                    result += indent + ']}'
                    return result
                
                # Special handling for 'and'/'or' with multiple conditions
                if key in ['and', 'or'] and isinstance(value, list) and len(value) > 2:
                    result = f'{{ "{key}": [\n'
                    for i, item in enumerate(value):
                        result += next_indent + self.format_value(item, depth + 1)
                        if i < len(value) - 1:
                            result += ','
                        result += '\n'
                    result += indent + ']}'
                    return result
            
            # Regular object formatting
            items = []
            for k, v in obj.items():
                formatted_value = self.format_value(v, depth + 1)
                items.append(f'{next_indent}"{k}": {formatted_value}')
            
            return '{\n' + ',\n'.join(items) + '\n' + indent + '}'
            
        if isinstance(obj, list):
            if not obj:
                return '[]'
                
            # All items are simple - keep inline
            if all(isinstance(item, (str, int, float, bool, type(None))) for item in obj):
                simple_format = json.dumps(obj, separators=(', ', ': '))
                if len(simple_format) <= self.max_inline_length:
                    return simple_format
            
            # Format as multiline
            items = []
            for item in obj:
                items.append(next_indent + self.format_value(item, depth + 1))
            
            return '[\n' + ',\n'.join(items) + '\n' + indent + ']'
            
        return json.dumps(obj)
    
    def beautify(self, data: Any) -> str:
        """Main beautification method"""
        return self.format_value(data, 0)


def main():
    parser = argparse.ArgumentParser(
        description='Semi-beautify JSON files with smart formatting for JSONLogic'
    )
    parser.add_argument('files', nargs='+', help='JSON files to format')
    parser.add_argument('--in-place', '-i', action='store_true', 
                        help='Format files in place')
    parser.add_argument('--max-inline-length', type=int, default=80,
                        help='Maximum length for inline objects/arrays (default: 80)')
    parser.add_argument('--max-inline-items', type=int, default=3,
                        help='Maximum number of items for inline objects/arrays (default: 3)')
    parser.add_argument('--indent', type=int, default=4,
                        help='Number of spaces for indentation (default: 4)')
    
    args = parser.parse_args()
    
    beautifier = SemiBeautifier(
        max_inline_length=args.max_inline_length,
        max_inline_items=args.max_inline_items,
        indent=args.indent
    )
    
    for file_path in args.files:
        path = Path(file_path)
        
        if not path.exists():
            print(f"Error: File not found: {file_path}", file=sys.stderr)
            continue
            
        try:
            with open(path, 'r') as f:
                data = json.load(f)
            
            formatted = beautifier.beautify(data)
            
            if args.in_place:
                with open(path, 'w') as f:
                    f.write(formatted)
                    f.write('\n')  # Add final newline
                print(f"Formatted: {file_path}")
            else:
                print(f"# {file_path}")
                print(formatted)
                print()
                
        except json.JSONDecodeError as e:
            print(f"Error: Invalid JSON in {file_path}: {e}", file=sys.stderr)
        except Exception as e:
            print(f"Error processing {file_path}: {e}", file=sys.stderr)


if __name__ == '__main__':
    main()