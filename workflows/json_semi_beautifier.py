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
    def __init__(self, max_inline_length=80, max_inline_items=3, indent=4, operator_config=None):
        self.max_inline_length = max_inline_length
        self.max_inline_items = max_inline_items
        self.indent_str = ' ' * indent
        
        # Default operator configuration
        # Levels: 'inline', 'compact', 'expanded'
        self.operator_config = operator_config or {
            # Always inline (single line)
            'inline': ['var', 'exists', '==', '!=', '>', '<', '>=', '<='],
            
            # Compact (may break into multiple lines but kept simple)
            'compact': ['cat', 'substr', 'in', 'some', 'all', 'starts_with', 'ends_with'],
            
            # Always expanded (multi-line with proper indentation)
            'expanded': ['if', 'and', 'or'],
            
            # Special handling for specific patterns
            'special': {
                'map': {'min_items': 2},  # Expand map if more than 2 items
                'filter': {'min_items': 2},
                'reduce': {'always_expand': True}
            }
        }
        
    def get_operator_level(self, operator: str) -> str:
        """Get the beautification level for an operator"""
        for level, ops in self.operator_config.items():
            if level == 'special':
                continue
            if operator in ops:
                return level
        return 'auto'  # Default to auto-detection
    
    def should_inline(self, obj: Any) -> bool:
        """Determine if an object should be kept inline (compact)"""
        if isinstance(obj, (str, int, float, bool, type(None))):
            return True
            
        if isinstance(obj, dict):
            # Check if it's a simple JSONLogic operation
            if len(obj) == 1:
                key = next(iter(obj))
                value = obj[key]
                
                # Check operator configuration
                level = self.get_operator_level(key)
                if level == 'inline':
                    return True
                elif level == 'expanded':
                    return False
                elif level == 'compact':
                    # Use size-based logic for compact operators
                    if isinstance(value, list) and len(value) > 2:
                        return False
                    return self.should_inline(value)
                
                # Check special configurations
                if key in self.operator_config.get('special', {}):
                    special_config = self.operator_config['special'][key]
                    if special_config.get('always_expand'):
                        return False
                    if 'min_items' in special_config:
                        if isinstance(value, list) and len(value) > special_config['min_items']:
                            return False
                
                # Default behavior for unknown operators
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
                
                # Get operator level
                level = self.get_operator_level(key)
                
                # Force expanded formatting for configured operators
                if level == 'expanded' and isinstance(value, list):
                    result = f'{{ "{key}": [\n'
                    for i, item in enumerate(value):
                        result += next_indent + self.format_value(item, depth + 1)
                        if i < len(value) - 1:
                            result += ','
                        result += '\n'
                    result += indent + ']}'
                    return result
                
                # Check special configurations
                if key in self.operator_config.get('special', {}):
                    special_config = self.operator_config['special'][key]
                    if special_config.get('always_expand') and isinstance(value, list):
                        result = f'{{ "{key}": [\n'
                        for i, item in enumerate(value):
                            result += next_indent + self.format_value(item, depth + 1)
                            if i < len(value) - 1:
                                result += ','
                            result += '\n'
                        result += indent + ']}'
                        return result
                    
                    if 'min_items' in special_config:
                        if isinstance(value, list) and len(value) > special_config['min_items']:
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


def load_config(config_file):
    """Load operator configuration from a JSON file"""
    if not config_file:
        return None
    
    try:
        with open(config_file, 'r') as f:
            return json.load(f)
    except Exception as e:
        print(f"Warning: Could not load config file {config_file}: {e}", file=sys.stderr)
        return None


def main():
    parser = argparse.ArgumentParser(
        description='Semi-beautify JSON files with smart formatting for JSONLogic',
        epilog='''
Operator Levels:
  inline:   Always keep on a single line
  compact:  May break into lines based on size
  expanded: Always use multi-line formatting
  
Example config file:
{
  "inline": ["var", "==", "!="],
  "compact": ["cat", "substr"],
  "expanded": ["if", "and", "or"],
  "special": {
    "map": {"min_items": 2},
    "reduce": {"always_expand": true}
  }
}
        ''',
        formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument('files', nargs='*', help='JSON files to format')
    parser.add_argument('--in-place', '-i', action='store_true', 
                        help='Format files in place')
    parser.add_argument('--max-inline-length', type=int, default=80,
                        help='Maximum length for inline objects/arrays (default: 80)')
    parser.add_argument('--max-inline-items', type=int, default=3,
                        help='Maximum number of items for inline objects/arrays (default: 3)')
    parser.add_argument('--indent', type=int, default=4,
                        help='Number of spaces for indentation (default: 4)')
    parser.add_argument('--config', '-c', type=str,
                        help='JSON file with operator configuration')
    parser.add_argument('--inline', action='append', default=[],
                        help='Operators to always inline (can be used multiple times)')
    parser.add_argument('--compact', action='append', default=[],
                        help='Operators to format compactly (can be used multiple times)')
    parser.add_argument('--expanded', action='append', default=[],
                        help='Operators to always expand (can be used multiple times)')
    parser.add_argument('--save-default-config', type=str,
                        help='Save the default configuration to a file and exit')
    
    args = parser.parse_args()
    
    # Handle saving default config
    if args.save_default_config:
        default_config = {
            'inline': ['var', 'exists', '==', '!=', '>', '<', '>=', '<='],
            'compact': ['cat', 'substr', 'in', 'some', 'all'],
            'expanded': ['if', 'and', 'or'],
            'special': {
                'map': {'min_items': 2},
                'filter': {'min_items': 2},
                'reduce': {'always_expand': True}
            }
        }
        with open(args.save_default_config, 'w') as f:
            json.dump(default_config, f, indent=2)
        print(f"Default configuration saved to {args.save_default_config}")
        sys.exit(0)
    
    # Check if files were provided
    if not args.files:
        parser.error("No files specified. Use --save-default-config to save configuration or provide files to format.")
    
    # Build operator configuration
    operator_config = None
    
    # Load from config file if provided
    if args.config:
        operator_config = load_config(args.config)
    
    # Override or create config from command line arguments
    if args.inline or args.compact or args.expanded:
        if not operator_config:
            # Start with default config if no file was loaded
            operator_config = {
                'inline': ['var', 'exists', '==', '!=', '>', '<', '>=', '<='],
                'compact': ['cat', 'substr', 'in', 'some', 'all'],
                'expanded': ['if', 'and', 'or'],
                'special': {}
            }
        
        # Apply command line overrides
        if args.inline:
            operator_config['inline'] = list(set(operator_config.get('inline', []) + args.inline))
        if args.compact:
            operator_config['compact'] = list(set(operator_config.get('compact', []) + args.compact))
        if args.expanded:
            operator_config['expanded'] = list(set(operator_config.get('expanded', []) + args.expanded))
    
    beautifier = SemiBeautifier(
        max_inline_length=args.max_inline_length,
        max_inline_items=args.max_inline_items,
        indent=args.indent,
        operator_config=operator_config
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