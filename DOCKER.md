# 🐳 Docker Setup for Reframe v3.1.1

Complete guide for running Reframe in Docker with the package-based architecture.

## Quick Start

```bash
# Build and run with docker-compose (downloads package automatically)
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
- Workflow packages are **downloaded during build** from a configurable URL
- Configuration is **baked into the image** with Docker-appropriate paths
- Logs are persisted to a mounted volume

This approach allows you to:
- No need to clone packages locally - they're downloaded automatically
- Configure package versions via URL (e.g., GitHub releases)
- Switch between different package versions by changing the URL
- Deploy to environments without local package dependencies
- Persist logs for debugging

## Files

### Dockerfile

Multi-stage build that creates a minimal runtime image:
- **Builder stage**: Compiles Reframe with Rust 1.89
- **Runtime stage**: Debian slim with only necessary dependencies
- **Package Download**: Downloads and extracts workflow package from URL during build
- **Configuration**: Bakes reframe.config.json into the image

### docker-compose.yml

Orchestrates the Reframe service:
- **Build Args**: Configures PACKAGE_URL for downloading workflow packages
- **Environment Variables**: Allows .env file configuration
- **Logs Mount**: Mounts logs directory (read-write)
- **Benchmark Service**: Includes benchmark runner service (optional profile)

### .dockerignore

Excludes unnecessary files from the Docker build context:
- Documentation and markdown files
- Build artifacts (target/)
- IDE and editor files
- Old workflows/ and scenarios/ directories (now downloaded during build)
- Environment files (.env)

### .env.example

Example environment configuration for docker-compose:
- **PACKAGE_URL**: Configurable URL for downloading workflow packages
- Supports GitHub releases, custom servers, or any accessible URL
- Allows easy switching between package versions

### Dockerfile.benchmark

Separate container for running benchmark tests:
- Python 3.11 with aiohttp and requests
- Waits for Reframe to be healthy before running
- Can be enabled with `--profile benchmark`

## Prerequisites

Before running with Docker, ensure you have:

1. **Docker** installed (20.10+)
2. **Docker Compose** installed (v2.0+)
3. **Internet connection** for downloading workflow packages during build

**Note**: No need to clone workflow packages locally - they are downloaded automatically during the Docker build process.

## Directory Structure

Your directory structure is simple:

```
Reframe/                              # This repository
├── Dockerfile
├── docker-compose.yml
├── .env.example                      # Example configuration
├── .env                              # Optional: Your configuration
├── reframe.config.json               # Configuration (baked into image)
├── logs/                             # Created automatically for log output
└── src/
```

**Workflow packages are downloaded during build and stored in `/packages/swift-cbpr` inside the container.**

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

### Custom Package URL

Configure which workflow package to download by creating a `.env` file:

```bash
# Copy the example
cp .env.example .env

# Edit to set your package URL
PACKAGE_URL=https://github.com/GoPlasmatic/reframe-package-swift-cbpr/releases/download/v2.1.1/reframe-swift-cbpr-v2.1.1.zip
```

**Examples**:
```bash
# Use a specific version
PACKAGE_URL=https://github.com/GoPlasmatic/reframe-package-swift-cbpr/releases/download/v2.1.1/reframe-swift-cbpr-v2.1.1.zip

# Use a custom server
PACKAGE_URL=https://your-server.com/packages/custom-package.zip
```

After changing the URL, rebuild:
```bash
docker-compose build
docker-compose up -d
```

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

### Logs Mount (Only Volume)

```yaml
volumes:
  - ./logs:/var/log/reframe
```

- **Source**: `./logs` (created automatically)
- **Target**: `/var/log/reframe`
- **Mode**: `rw` (read-write)

**Note**: Workflow packages and configuration are no longer mounted as volumes - they are baked into the Docker image during build.

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

The Dockerfile supports the following build arguments:

```bash
# Specify package URL
docker build --build-arg PACKAGE_URL=https://example.com/package.zip -t reframe:3.1.1 .

# Use different Rust version (default: 1.89)
docker build --build-arg RUST_VERSION=1.85 -t reframe:3.1.1 .

# Combine both
docker build \
  --build-arg PACKAGE_URL=https://example.com/package.zip \
  --build-arg RUST_VERSION=1.89 \
  -t reframe:3.1.1 .
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
# Run the container (package already baked in during build)
docker run -d \
  --name reframe \
  -p 3000:3000 \
  -v $(pwd)/logs:/var/log/reframe \
  -e RUST_LOG=info \
  -e TOKIO_WORKER_THREADS=8 \
  reframe:3.1.1

# View logs
docker logs -f reframe

# Stop
docker stop reframe
docker rm reframe
```

### Run with Different Package Version

To use a different package version, rebuild the image with a new PACKAGE_URL:

```bash
# Build with specific package version
docker build \
  --build-arg PACKAGE_URL=https://github.com/GoPlasmatic/reframe-package-swift-cbpr/releases/download/v2.1.1/reframe-swift-cbpr-v2.1.1.zip \
  -t reframe:3.1.1-v2.1 .

# Run the new image
docker run -d \
  --name reframe \
  -p 3000:3000 \
  -v $(pwd)/logs:/var/log/reframe \
  reframe:3.1.1-v2.1
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

### Package Download Issues

**Check if package was downloaded correctly**:
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

**If package is missing**:
1. Check build logs for download errors: `docker-compose build`
2. Verify PACKAGE_URL is accessible: `curl -I $PACKAGE_URL`
3. Try rebuilding with `--no-cache`: `docker-compose build --no-cache`

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

### Workflow Updates

To update workflows to a new version:

```bash
# 1. Update PACKAGE_URL in .env to point to new version
echo "PACKAGE_URL=https://github.com/GoPlasmatic/reframe-package-swift-cbpr/releases/download/v2.1.1/reframe-swift-cbpr-v2.1.1.zip" > .env

# 2. Rebuild the image
docker-compose build

# 3. Restart with new image
docker-compose down
docker-compose up -d

# 4. Verify new version
curl http://localhost:3000/health
```

**Note**: Workflows are baked into the image, so updates require a rebuild. However, rebuilds are fast since only the package download layer changes.

## Production Deployment

### Production docker-compose.yml

```yaml
version: '3.8'

services:
  reframe:
    build:
      context: .
      args:
        PACKAGE_URL: https://github.com/GoPlasmatic/reframe-package-swift-cbpr/releases/download/v2.1.1/reframe-swift-cbpr-v2.1.1.zip
    image: myorg/reframe:3.1.1
    container_name: reframe-prod
    ports:
      - "3000:3000"
    volumes:
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

**Note**: Package is downloaded during build, not mounted at runtime.

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

1. **Use explicit package versions** in production:
   ```yaml
   build:
     args:
       PACKAGE_URL: https://github.com/.../releases/download/v2.1.0/package-v2.1.0.zip
   ```

2. **Use explicit image versions** in production:
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
