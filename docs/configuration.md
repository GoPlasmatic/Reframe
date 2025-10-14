# ⚙️ Configuration Guide

Complete configuration reference for Reframe v3.1.1.

## Table of Contents

- [Overview](#overview)
- [Configuration Files](#configuration-files)
- [Server Configuration](#server-configuration)
- [Logging Configuration](#logging-configuration)
- [API Documentation Configuration](#api-documentation-configuration)
- [Package Configuration](#package-configuration)
- [Environment Variables](#environment-variables)
- [Configuration Priority](#configuration-priority)
- [Examples](#examples)

---

## Overview

Reframe v3.1.1 uses a flexible configuration system that supports multiple sources with a clear priority hierarchy. Configuration can be provided through:

1. **Environment Variables** (highest priority)
2. **reframe.config.json** file
3. **Built-in Defaults** (lowest priority)

This approach provides flexibility for different deployment environments while maintaining sensible defaults for development.

---

## Configuration Files

### reframe.config.json

The main configuration file located in the project root directory. This file is optional - if not present, Reframe will use built-in defaults.

**Location**: `./reframe.config.json`

**Format**: JSON

**Example**:
```json
{
  "packages": [
    {
      "path": "../reframe-package-swift-cbpr",
      "enabled": true
    }
  ],
  "server": {
    "host": "0.0.0.0",
    "port": 3000,
    "runtime": {
      "tokio_worker_threads": "auto",
      "thread_name_prefix": "reframe-worker"
    }
  },
  "logging": {
    "format": "compact",
    "level": "info"
  },
  "api_docs": {
    "enabled": true,
    "title": "Reframe API"
  },
  "defaults": {
    "package_path": "../reframe-package-swift-cbpr"
  }
}
```

### reframe-package.json

Package configuration file located in each workflow package directory. This defines the package metadata, workflows, and scenarios.

**Location**: `<package-path>/reframe-package.json`

**Format**: JSON

**Example**:
```json
{
  "id": "swift-cbpr-mt-mx",
  "name": "SWIFT CBPR+ MT <-> MX Transformations - SR2025",
  "version": "2.0.0",
  "description": "Official SWIFT CBPR+ transformations",
  "author": "Plasmatic Team",
  "license": "Apache-2.0",
  "engine_version": "3.1.1",
  "required_plugins": ["swift-mt-message", "mx-message"],
  "workflows": {
    "transform": {
      "path": "transform",
      "description": "Unified bidirectional transformation workflows",
      "entry_point": "index.json"
    },
    "generate": {
      "path": "generate",
      "description": "Sample message generation",
      "entry_point": "index.json"
    },
    "validate": {
      "path": "validate",
      "description": "Message validation workflows",
      "entry_point": "index.json"
    }
  },
  "scenarios": {
    "path": "scenarios",
    "description": "Test scenarios",
    "entry_point": "index.json"
  }
}
```

---

## Server Configuration

Controls HTTP server behavior and runtime characteristics.

### server

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `host` | string | `"0.0.0.0"` | Bind address for HTTP server |
| `port` | number | `3000` | Port number for HTTP server |
| `runtime` | object | See below | Tokio runtime configuration |

### server.runtime

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `tokio_worker_threads` | string | `"auto"` | Number of Tokio worker threads ("auto" or specific number) |
| `thread_name_prefix` | string | `"reframe-worker"` | Prefix for worker thread names |

**Example**:
```json
{
  "server": {
    "host": "127.0.0.1",
    "port": 8080,
    "runtime": {
      "tokio_worker_threads": "8",
      "thread_name_prefix": "reframe-prod"
    }
  }
}
```

**Notes**:
- Setting `host` to `"0.0.0.0"` allows connections from any network interface
- Setting `host` to `"127.0.0.1"` restricts connections to localhost only
- `tokio_worker_threads` of `"auto"` uses the number of CPU cores
- Explicit thread count useful for resource-constrained environments

---

## Logging Configuration

Controls application logging behavior including format, level, and output destinations.

### logging

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `format` | string | `"compact"` | Log format: "compact", "full", "pretty", or "json" |
| `level` | string | `"info"` | Log level: "error", "warn", "info", "debug", "trace" |
| `show_target` | boolean | `false` | Show module target in logs |
| `show_thread` | boolean | `false` | Show thread name/ID in logs |
| `show_file_location` | boolean | `false` | Show file and line number |
| `show_span_events` | boolean | `false` | Show span enter/exit events |
| `file_output` | object | See below | File logging configuration |

### logging.file_output

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | boolean | `false` | Enable file logging |
| `directory` | string | `"./logs"` | Directory for log files |
| `file_prefix` | string | `"reframe"` | Prefix for log file names |
| `rotation` | string | `"daily"` | Rotation strategy: "daily", "hourly", "never" |

**Example - Development**:
```json
{
  "logging": {
    "format": "pretty",
    "level": "debug",
    "show_target": true,
    "show_file_location": true
  }
}
```

**Example - Production**:
```json
{
  "logging": {
    "format": "json",
    "level": "info",
    "show_target": false,
    "file_output": {
      "enabled": true,
      "directory": "/var/log/reframe",
      "file_prefix": "reframe-prod",
      "rotation": "daily"
    }
  }
}
```

**Log Levels**:
- `error`: Only errors
- `warn`: Warnings and errors
- `info`: General information (recommended for production)
- `debug`: Detailed debugging information
- `trace`: Extremely verbose, includes dataflow-rs internals

---

## API Documentation Configuration

Controls OpenAPI/Swagger documentation generation and display.

### api_docs

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | boolean | `true` | Enable API documentation endpoints |
| `title` | string | `"Reframe API"` | API title in documentation |
| `version` | string | `"3.1.1"` | API version (from Cargo.toml) |
| `description` | string | See below | API description |
| `contact` | object | See below | Contact information |
| `license` | object | See below | License information |
| `external_docs_url` | string | See below | External documentation URL |
| `server_url` | string | `"http://localhost:3000"` | API server URL |

**Default Description**:
```
"Enterprise-grade bidirectional SWIFT MT ↔ ISO 20022 transformation service"
```

### api_docs.contact

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `name` | string | `"Plasmatic Team"` | Contact name |
| `email` | string | `"enquires@goplasmatic.io"` | Contact email |

### api_docs.license

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `name` | string | `"Apache 2.0"` | License name |
| `identifier` | string | `"Apache-2.0"` | SPDX license identifier |
| `url` | string | See below | License URL |

**Default License URL**:
```
"https://opensource.org/license/apache-2-0"
```

**Example - Custom Branding**:
```json
{
  "api_docs": {
    "enabled": true,
    "title": "ACME Bank Transformation API",
    "version": "1.0.0",
    "description": "Internal SWIFT transformation service",
    "contact": {
      "name": "ACME IT Team",
      "email": "it@acmebank.com"
    },
    "server_url": "https://api.acmebank.com/transform"
  }
}
```

**Notes**:
- `server_url` can be overridden by `API_SERVER_URL` environment variable
- When `enabled` is `false`, `/swagger-ui` and `/api-docs/openapi.json` endpoints return 404
- The OpenAPI spec is generated at runtime based on configuration

---

## Package Configuration

Controls which workflow packages are loaded and their settings.

### packages

Array of package references to load at startup.

**Package Reference Structure**:
```json
{
  "path": "../reframe-package-swift-cbpr",
  "enabled": true
}
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `path` | string | Required | Relative or absolute path to package directory |
| `enabled` | boolean | `true` | Whether to load this package |

### defaults

Default package settings.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `package_id` | string/null | `null` | Default package ID to use (null = first loaded) |
| `package_path` | string | See below | Fallback package path if no packages configured |

**Default Package Path**:
```
"../reframe-package-swift-cbpr"
```

**Example - Multiple Packages**:
```json
{
  "packages": [
    {
      "path": "../reframe-package-swift-cbpr",
      "enabled": true
    },
    {
      "path": "/opt/reframe/packages/custom-transformations",
      "enabled": true
    },
    {
      "path": "../reframe-package-experimental",
      "enabled": false
    }
  ],
  "defaults": {
    "package_id": "swift-cbpr-mt-mx"
  }
}
```

**Notes**:
- If no packages are configured, falls back to `defaults.package_path`
- Packages are loaded in the order specified
- Disabled packages (`enabled: false`) are skipped
- Package validation occurs during loading

---

## Environment Variables

Environment variables override configuration file values, providing flexibility for deployment-specific settings.

### Core Environment Variables

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `REFRAME_PACKAGE_PATH` | string | `"../reframe-package-swift-cbpr"` | Path to default workflow package |
| `TOKIO_WORKER_THREADS` | number | CPU count | Number of Tokio async runtime worker threads |
| `API_SERVER_URL` | string | `"http://localhost:3000"` | Override OpenAPI server URL |
| `RUST_LOG` | string | `"info"` | Logging level and filters |

### RUST_LOG Syntax

The `RUST_LOG` environment variable supports flexible filtering:

**Simple Levels**:
```bash
export RUST_LOG=error  # Only errors
export RUST_LOG=warn   # Warnings and errors
export RUST_LOG=info   # General information
export RUST_LOG=debug  # Debugging information
export RUST_LOG=trace  # Very verbose
```

**Module-Specific Levels**:
```bash
# Different levels for different modules
export RUST_LOG=info,reframe=debug

# Very verbose dataflow-rs, info for everything else
export RUST_LOG=info,dataflow_rs=trace

# Multiple module configurations
export RUST_LOG=warn,reframe::handlers=debug,dataflow_rs=trace
```

**Production Examples**:
```bash
# Minimal logging
export RUST_LOG=error

# Standard production
export RUST_LOG=info

# Debugging specific issues
export RUST_LOG=info,reframe::handlers=debug
```

### Usage Examples

**Development Environment**:
```bash
export REFRAME_PACKAGE_PATH=../reframe-package-swift-cbpr
export RUST_LOG=debug
export TOKIO_WORKER_THREADS=auto
cargo run
```

**Production Environment**:
```bash
export REFRAME_PACKAGE_PATH=/opt/reframe/packages/swift-cbpr
export API_SERVER_URL=https://api.production.com
export RUST_LOG=info
export TOKIO_WORKER_THREADS=16
./target/release/reframe
```

**Docker Environment**:
```bash
docker run -d \
  -e REFRAME_PACKAGE_PATH=/packages/swift-cbpr \
  -e RUST_LOG=info \
  -e TOKIO_WORKER_THREADS=8 \
  -e API_SERVER_URL=https://api.example.com \
  -v ./packages:/packages \
  -p 3000:3000 \
  reframe:latest
```

---

## Configuration Priority

Configuration values are resolved using this priority order (highest to lowest):

1. **Environment Variables** (highest priority)
   - `API_SERVER_URL`
   - `REFRAME_PACKAGE_PATH`
   - `TOKIO_WORKER_THREADS`
   - `RUST_LOG`

2. **reframe.config.json**
   - All configuration sections
   - Loaded from project root
   - Optional file

3. **Built-in Defaults** (lowest priority)
   - Hard-coded defaults
   - Always available

### Example Priority Resolution

Given this configuration:

**reframe.config.json**:
```json
{
  "server": {
    "port": 8080
  },
  "logging": {
    "level": "info"
  }
}
```

**Environment**:
```bash
export RUST_LOG=debug
```

**Resolved Configuration**:
- Port: `8080` (from config file)
- Log level: `debug` (from environment variable, overrides config file)
- Host: `"0.0.0.0"` (built-in default, not specified elsewhere)

---

## Examples

### Development Setup

**reframe.config.json**:
```json
{
  "packages": [
    {
      "path": "../reframe-package-swift-cbpr",
      "enabled": true
    }
  ],
  "server": {
    "host": "127.0.0.1",
    "port": 3000,
    "runtime": {
      "tokio_worker_threads": "auto"
    }
  },
  "logging": {
    "format": "pretty",
    "level": "debug",
    "show_target": true,
    "show_file_location": true
  },
  "api_docs": {
    "enabled": true,
    "server_url": "http://localhost:3000"
  }
}
```

**Run Command**:
```bash
cargo run
```

### Production Setup

**reframe.config.json**:
```json
{
  "packages": [
    {
      "path": "/opt/reframe/packages/swift-cbpr-v2.0",
      "enabled": true
    }
  ],
  "server": {
    "host": "0.0.0.0",
    "port": 3000,
    "runtime": {
      "tokio_worker_threads": "16"
    }
  },
  "logging": {
    "format": "json",
    "level": "info",
    "file_output": {
      "enabled": true,
      "directory": "/var/log/reframe",
      "file_prefix": "reframe-prod",
      "rotation": "daily"
    }
  },
  "api_docs": {
    "enabled": true,
    "title": "Production Transformation API",
    "server_url": "https://api.production.com"
  }
}
```

**Environment Variables**:
```bash
export RUST_LOG=info
export API_SERVER_URL=https://api.production.com
```

**Run Command**:
```bash
./target/release/reframe
```

### Docker Deployment

**reframe.config.json**:
```json
{
  "packages": [
    {
      "path": "/packages/swift-cbpr",
      "enabled": true
    }
  ],
  "server": {
    "host": "0.0.0.0",
    "port": 3000,
    "runtime": {
      "tokio_worker_threads": "auto"
    }
  },
  "logging": {
    "format": "json",
    "level": "info"
  }
}
```

**docker-compose.yml**:
```yaml
version: '3.8'

services:
  reframe:
    image: reframe:3.1.1
    ports:
      - "3000:3000"
    volumes:
      - ./packages:/packages:ro
      - ./config:/app/config:ro
      - ./logs:/var/log/reframe
    environment:
      - RUST_LOG=info
      - API_SERVER_URL=https://api.example.com
      - TOKIO_WORKER_THREADS=8
    restart: unless-stopped
```

### Minimal Configuration

If no `reframe.config.json` is provided, Reframe uses these defaults:

```json
{
  "packages": [],
  "server": {
    "host": "0.0.0.0",
    "port": 3000,
    "runtime": {
      "tokio_worker_threads": "auto",
      "thread_name_prefix": "reframe-worker"
    }
  },
  "logging": {
    "format": "compact",
    "level": "info",
    "show_target": false,
    "show_thread": false,
    "show_file_location": false,
    "show_span_events": false,
    "file_output": {
      "enabled": false
    }
  },
  "api_docs": {
    "enabled": true,
    "title": "Reframe API",
    "version": "3.1.1",
    "description": "Enterprise-grade bidirectional SWIFT MT ↔ ISO 20022 transformation service",
    "server_url": "http://localhost:3000"
  },
  "defaults": {
    "package_id": null,
    "package_path": "../reframe-package-swift-cbpr"
  }
}
```

With these defaults, Reframe will:
- Look for packages at `../reframe-package-swift-cbpr` (or use `REFRAME_PACKAGE_PATH`)
- Listen on `0.0.0.0:3000`
- Use all available CPU cores for Tokio runtime
- Log at INFO level with compact format
- Enable API documentation at `/swagger-ui`

---

## Validation and Errors

### Configuration Validation

Reframe validates configuration at startup and reports errors clearly:

**Invalid Port**:
```
Error: Invalid server port: 0 (must be 1-65535)
```

**Missing Package**:
```
Error: Package not found: /opt/reframe/packages/nonexistent
```

**Invalid Package Configuration**:
```
Error: Failed to load package from ../reframe-package-swift-cbpr:
  Package configuration not found: ../reframe-package-swift-cbpr/reframe-package.json
```

### Hot Reload

Configuration changes to workflow files can be reloaded without restart:

```bash
curl -X POST http://localhost:3000/admin/reload-workflows
```

This reloads:
- All workflow definitions from packages
- Scenario configurations
- Package metadata

**Does NOT reload**:
- Server configuration (host, port, runtime)
- Logging configuration
- API documentation settings

To apply server/logging changes, restart the service.

---

## Next Steps

- [Architecture Guide](architecture.md) - Understanding the technical architecture
- [Installation Guide](installation.md) - Setup and deployment instructions
- [Message Formats](message-formats.md) - Supported message types
