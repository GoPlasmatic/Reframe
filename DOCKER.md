# 🐳 Docker Setup for Reframe v3.1.1

Complete guide for running Reframe in Docker with the package-based architecture.

## Quick Start

```bash
# Build and run with docker-compose
docker-compose up -d

# View logs
docker-compose logs -f reframe-app

# Check health
curl http://localhost:3000/health

# Stop services
docker-compose down
```

## Architecture Overview

The Docker setup for Reframe v3.1.1 uses a **package-based architecture** where:
- The Reframe application binary is built into the container
- Workflow packages are **mounted as volumes** at runtime
- Configuration is **mounted as a volume** for flexibility
- Logs are persisted to a mounted volume

This approach allows you to:
- Update workflows without rebuilding the container
- Switch between different package versions
- Customize configuration per environment
- Persist logs for debugging

## Files

### Dockerfile

Multi-stage build that creates a minimal runtime image:
- **Builder stage**: Compiles Reframe with Rust 1.89
- **Runtime stage**: Debian slim with only necessary dependencies
- **No embedded workflows**: Expects packages to be mounted

### docker-compose.yml

Orchestrates the Reframe service with proper volume mounts:
- Mounts the external workflow package (read-only)
- Mounts optional configuration file (read-only)
- Mounts logs directory (read-write)
- Includes benchmark runner service (optional profile)

### .dockerignore

Excludes unnecessary files from the Docker build context:
- Documentation and markdown files
- Build artifacts (target/)
- IDE and editor files
- Old workflows/ and scenarios/ directories (now external)
- Configuration files (mounted at runtime)

### Dockerfile.benchmark

Separate container for running benchmark tests:
- Python 3.11 with aiohttp and requests
- Waits for Reframe to be healthy before running
- Can be enabled with `--profile benchmark`

## Prerequisites

Before running with Docker, ensure you have:

1. **Docker** installed (20.10+)
2. **Docker Compose** installed (v2.0+)
3. **Workflow Package** cloned:
   ```bash
   cd ..
   git clone https://github.com/GoPlasmatic/reframe-package-swift-cbpr.git
   cd Reframe
   ```

## Directory Structure

Your directory structure should look like:

```
parent-directory/
├── Reframe/                          # This repository
│   ├── Dockerfile
│   ├── docker-compose.yml
│   ├── reframe.config.json           # Optional: Custom configuration
│   ├── logs/                         # Created automatically
│   └── src/
└── reframe-package-swift-cbpr/       # External workflow package
    ├── reframe-package.json
    ├── transform/
    ├── generate/
    ├── validate/
    └── scenarios/
```

## Usage

### Basic Usage

Start Reframe with default configuration:

```bash
# Build and start
docker-compose up -d

# View logs
docker-compose logs -f reframe-app

# Check status
docker-compose ps

# Stop
docker-compose down
```

### Custom Configuration

Create a `reframe.config.json` file for custom settings:

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
      "tokio_worker_threads": "8"
    }
  },
  "logging": {
    "format": "json",
    "level": "info"
  }
}
```

The docker-compose.yml will automatically mount this file.

### Environment Variables

Override settings with environment variables:

```bash
# Edit docker-compose.yml or use docker-compose.override.yml
services:
  reframe-app:
    environment:
      - RUST_LOG=debug
      - TOKIO_WORKER_THREADS=16
      - API_SERVER_URL=https://api.production.com
```

Or use command-line:

```bash
RUST_LOG=debug docker-compose up
```

### Running with Benchmarks

To run with the benchmark service:

```bash
# Start with benchmark profile
docker-compose --profile benchmark up -d

# View benchmark logs
docker-compose logs -f benchmark-runner

# The benchmark will run automatically after reframe-app is healthy
```

## Volume Mounts

### Workflow Package Mount

```yaml
volumes:
  - ../reframe-package-swift-cbpr:/packages/swift-cbpr:ro
```

- **Source**: `../reframe-package-swift-cbpr` (relative path)
- **Target**: `/packages/swift-cbpr` (container path)
- **Mode**: `ro` (read-only for safety)

### Configuration Mount

```yaml
volumes:
  - ./reframe.config.json:/app/reframe.config.json:ro
```

- **Source**: `./reframe.config.json` (optional)
- **Target**: `/app/reframe.config.json`
- **Mode**: `ro` (read-only)

### Logs Mount

```yaml
volumes:
  - ./logs:/var/log/reframe
```

- **Source**: `./logs` (created automatically)
- **Target**: `/var/log/reframe`
- **Mode**: `rw` (read-write)

## Building

### Build the Image

```bash
# Build with docker-compose
docker-compose build

# Build manually
docker build -t reframe:3.1.1 .

# Build with custom tag
docker build -t myorg/reframe:latest .
```

### Build Arguments

The Dockerfile uses Rust 1.89 by default. To use a different version:

```bash
docker build --build-arg RUST_VERSION=1.85 -t reframe:3.1.1 .
```

## Running

### Run with docker-compose (Recommended)

```bash
# Start in background
docker-compose up -d

# Start with logs
docker-compose up

# Scale (if needed)
docker-compose up -d --scale reframe-app=3
```

### Run Standalone

```bash
# Run with mounted package
docker run -d \
  --name reframe \
  -p 3000:3000 \
  -v $(pwd)/../reframe-package-swift-cbpr:/packages/swift-cbpr:ro \
  -v $(pwd)/logs:/var/log/reframe \
  -e RUST_LOG=info \
  -e TOKIO_WORKER_THREADS=8 \
  -e REFRAME_PACKAGE_PATH=/packages/swift-cbpr \
  reframe:3.1.1

# View logs
docker logs -f reframe

# Stop
docker stop reframe
docker rm reframe
```

### Run with Custom Package Location

```bash
docker run -d \
  --name reframe \
  -p 3000:3000 \
  -v /opt/packages/swift-cbpr-v2:/packages/swift-cbpr:ro \
  -e REFRAME_PACKAGE_PATH=/packages/swift-cbpr \
  reframe:3.1.1
```

## Health Checks

The container includes automatic health checks:

```yaml
healthcheck:
  test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
  interval: 30s
  timeout: 3s
  start_period: 5s
  retries: 3
```

Check health status:

```bash
# Via docker-compose
docker-compose ps

# Via docker
docker inspect reframe-app | jq '.[0].State.Health'

# Via API
curl http://localhost:3000/health
```

## Logging

### View Logs

```bash
# All logs
docker-compose logs

# Follow logs
docker-compose logs -f reframe-app

# Last 100 lines
docker-compose logs --tail=100 reframe-app

# Logs with timestamps
docker-compose logs -t reframe-app
```

### Log Files

When file logging is enabled (via reframe.config.json):

```bash
# View log files
ls -la logs/

# Tail log file
tail -f logs/reframe-prod.log

# Search logs
grep "error" logs/reframe-prod.log
```

## Troubleshooting

### Container Won't Start

**Check logs**:
```bash
docker-compose logs reframe-app
```

**Common issues**:
1. Package path not found:
   ```
   Error: Package configuration not found: /packages/swift-cbpr/reframe-package.json
   ```
   **Solution**: Ensure the workflow package is cloned and mounted correctly

2. Port already in use:
   ```
   Error: Address already in use
   ```
   **Solution**: Stop other services on port 3000 or change the port:
   ```yaml
   ports:
     - "8080:3000"
   ```

### Package Mount Issues

**Check if package is mounted**:
```bash
docker exec reframe-app ls -la /packages/swift-cbpr/
```

**Expected output**:
```
drwxr-xr-x  reframe-package.json
drwxr-xr-x  transform/
drwxr-xr-x  generate/
drwxr-xr-x  validate/
drwxr-xr-x  scenarios/
```

### Performance Issues

**Increase worker threads**:
```yaml
environment:
  - TOKIO_WORKER_THREADS=16
```

**Check resource usage**:
```bash
docker stats reframe-app
```

**Optimize for production**:
```yaml
environment:
  - RUST_LOG=warn  # Reduce logging
  - TOKIO_WORKER_THREADS=16
deploy:
  resources:
    limits:
      cpus: '4'
      memory: 4G
    reservations:
      cpus: '2'
      memory: 2G
```

### Workflow Reload

Reload workflows without restarting:

```bash
# Update workflows in package directory
cd ../reframe-package-swift-cbpr
git pull

# Reload via API
curl -X POST http://localhost:3000/admin/reload-workflows

# Check logs
docker-compose logs -f reframe-app
```

## Production Deployment

### Production docker-compose.yml

```yaml
version: '3.8'

services:
  reframe:
    image: myorg/reframe:3.1.1
    container_name: reframe-prod
    ports:
      - "3000:3000"
    volumes:
      - /opt/packages/swift-cbpr:/packages/swift-cbpr:ro
      - /opt/reframe/config/reframe.config.json:/app/reframe.config.json:ro
      - /var/log/reframe:/var/log/reframe
    environment:
      - RUST_LOG=info
      - TOKIO_WORKER_THREADS=16
      - API_SERVER_URL=https://api.production.com
    deploy:
      resources:
        limits:
          cpus: '4'
          memory: 4G
        reservations:
          cpus: '2'
          memory: 2G
    restart: always
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 5s
      retries: 3
      start_period: 10s
    networks:
      - production-network

networks:
  production-network:
    driver: bridge
```

### Multi-Node Setup

For high availability, run multiple replicas:

```yaml
version: '3.8'

services:
  reframe:
    image: myorg/reframe:3.1.1
    deploy:
      replicas: 3
      restart_policy:
        condition: on-failure
      resources:
        limits:
          cpus: '2'
          memory: 2G
    # ... rest of configuration
```

### Monitoring

Add monitoring services:

```yaml
services:
  reframe:
    # ... existing config
    labels:
      - "prometheus.scrape=true"
      - "prometheus.port=3000"
      - "prometheus.path=/metrics"

  prometheus:
    image: prom/prometheus:latest
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
    ports:
      - "9090:9090"

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3001:3000"
    depends_on:
      - prometheus
```

## Best Practices

1. **Always use read-only mounts** for packages and config:
   ```yaml
   - ../reframe-package-swift-cbpr:/packages/swift-cbpr:ro
   ```

2. **Use explicit versions** in production:
   ```yaml
   image: myorg/reframe:3.1.1  # Not :latest
   ```

3. **Set resource limits**:
   ```yaml
   deploy:
     resources:
       limits:
         cpus: '4'
         memory: 4G
   ```

4. **Enable health checks**:
   ```yaml
   healthcheck:
     test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
   ```

5. **Use restart policies**:
   ```yaml
   restart: unless-stopped  # Development
   restart: always          # Production
   ```

6. **Persist logs**:
   ```yaml
   volumes:
     - ./logs:/var/log/reframe
   ```

7. **Use networks for isolation**:
   ```yaml
   networks:
     - reframe-network
   ```

## Additional Resources

- [Installation Guide](docs/installation.md)
- [Configuration Guide](docs/configuration.md)
- [Architecture Guide](docs/architecture.md)

---

For more information, see the main [README.md](README.md) or visit the [documentation](docs/).
