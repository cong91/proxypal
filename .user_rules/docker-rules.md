# Docker Rules

Docker and containerization best practices for the S4VN Webots project.

**Scope**: `**/Dockerfile*`, `**/docker-compose*.yml`, `**/docker-compose*.yaml`

---

## Dockerfile Best Practices

### Base Images
- Use **specific base image tags** (not `latest`)
- Document why each base image is chosen
- Keep base images minimal

```dockerfile
# ✅ GOOD - Specific version
FROM cyberbotics/webots:R2023b-ubuntu22.04

# ❌ BAD - Latest tag
FROM cyberbotics/webots:latest
```

### Layer Optimization
- **Order instructions** from least to most frequently changing
- **Combine RUN commands** to reduce layers
- Clean up in the same layer as installation

```dockerfile
# ✅ GOOD - Combined and cleaned up
RUN apt-get update && \
    apt-get install -y \
        python3 \
        python3-pip \
        virtualgl \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# ❌ BAD - Multiple layers, no cleanup
RUN apt-get update
RUN apt-get install -y python3
RUN apt-get install -y virtualgl
```

### .dockerignore
Always use `.dockerignore` to exclude unnecessary files:

```
# .dockerignore
.git
.gitignore
node_modules
__pycache__
*.pyc
*.log
.env
.vscode
.idea
*.md
!README.md
```

---

## Dockerfile.default (Webots Project)

This project uses `Dockerfile.default` for Webots containers:

```dockerfile
FROM cyberbotics/webots:R2023b-ubuntu22.04

# Install NVIDIA/VirtualGL dependencies
RUN apt-get update && apt-get install -y \
    virtualgl \
    libglu1-mesa \
    libxv1 \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# Install Python dependencies
COPY requirements.txt /tmp/
RUN pip3 install --no-cache-dir -r /tmp/requirements.txt

# Copy application code
WORKDIR /app
COPY src/ /app/src/
COPY config/ /app/config/

# Environment variables
ENV NVIDIA_VISIBLE_DEVICES=all
ENV NVIDIA_DRIVER_CAPABILITIES=all
ENV DISPLAY=:0

# Verify installations
RUN which vglrun || (echo "ERROR: VirtualGL not installed" && exit 1)
RUN python3 --version

CMD ["python3", "src/simulation_server.py"]
```

---

## docker-compose.yml

### Structure
```yaml
version: '3.8'

services:
  simulation:
    build:
      context: .
      dockerfile: config/docker/Dockerfile.default
    
    runtime: nvidia  # CRITICAL for GPU access
    
    environment:
      - NVIDIA_VISIBLE_DEVICES=all
      - NVIDIA_DRIVER_CAPABILITIES=all
      - DISPLAY=:0
      - WEBOTS_HOME=/usr/local/webots
    
    devices:
      - /dev/dri:/dev/dri  # GPU device access
    
    volumes:
      - ./src:/app/src:ro
      - ./worlds:/app/worlds:ro
      - simulation_data:/app/data
    
    ports:
      - "1234:1234"  # WebSocket port
      - "1235:1235"  # Streaming port
    
    networks:
      - webots_net
    
    restart: unless-stopped
    
    healthcheck:
      test: ["CMD", "python3", "-c", "import requests; requests.get('http://localhost:1234/health')"]
      interval: 30s
      timeout: 10s
      retries: 3

volumes:
  simulation_data:

networks:
  webots_net:
    driver: bridge
```

---

## NVIDIA GPU Support (CRITICAL)

### Requirements
1. NVIDIA GPU on host
2. NVIDIA drivers installed
3. nvidia-container-toolkit installed
4. Proper docker-compose configuration

### Configuration
```yaml
services:
  simulation:
    runtime: nvidia  # Enable NVIDIA runtime
    
    environment:
      # Make all GPUs visible
      - NVIDIA_VISIBLE_DEVICES=all
      
      # Enable all NVIDIA capabilities
      - NVIDIA_DRIVER_CAPABILITIES=all
      
      # Or specific capabilities:
      # - NVIDIA_DRIVER_CAPABILITIES=graphics,utility,compute,display
    
    devices:
      - /dev/dri:/dev/dri  # Direct Rendering Infrastructure
```

### Verification
```bash
# Inside container
nvidia-smi  # Should show GPU info
glxinfo | grep "OpenGL renderer"  # Should show NVIDIA, not Mesa
```

### Error Handling
**NO FALLBACK to software rendering!** If GPU fails, the container MUST fail with clear instructions.

See [no-fallback-policy.md](no-fallback-policy.md)

---

## Environment Variables

### .env File (Local Development)
```bash
# .env (DO NOT COMMIT)
NVIDIA_VISIBLE_DEVICES=all
NVIDIA_DRIVER_CAPABILITIES=all
WEBOTS_HOME=/usr/local/webots
S3_BUCKET=my-simulation-recordings
AWS_ACCESS_KEY_ID=xxxxx
AWS_SECRET_ACCESS_KEY=xxxxx
```

### docker-compose.yml
```yaml
services:
  simulation:
    env_file:
      - .env
    environment:
      # Or specify directly
      - WEBOTS_HOME=${WEBOTS_HOME}
      - S3_BUCKET=${S3_BUCKET}
```

### Required Variables Documentation
Document all required environment variables in README:

```markdown
## Required Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| NVIDIA_VISIBLE_DEVICES | Yes | all | GPU devices to use |
| NVIDIA_DRIVER_CAPABILITIES | Yes | all | NVIDIA capabilities |
| WEBOTS_HOME | Yes | /usr/local/webots | Webots installation path |
| S3_BUCKET | No | - | S3 bucket for recordings |
```

---

## Health Checks

### Dockerfile
```dockerfile
HEALTHCHECK --interval=30s --timeout=10s --retries=3 \
  CMD python3 -c "import requests; requests.get('http://localhost:1234/health')"
```

### docker-compose.yml
```yaml
services:
  simulation:
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:1234/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
```

---

## Logging

### Container Logs
```yaml
services:
  simulation:
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"
```

### Application Logging (Python)
```python
import logging
import sys

# Log to stdout for Docker
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    handlers=[logging.StreamHandler(sys.stdout)]
)
```

---

## Security

### Non-Root User (When Possible)
```dockerfile
# Create non-root user
RUN useradd -m -u 1000 webots && \
    chown -R webots:webots /app

USER webots
```

### Secrets Management
```yaml
# docker-compose.yml
services:
  simulation:
    secrets:
      - db_password
      - api_key

secrets:
  db_password:
    file: ./secrets/db_password.txt
  api_key:
    external: true
```

---

## Resource Limits

```yaml
services:
  simulation:
    deploy:
      resources:
        limits:
          cpus: '4.0'
          memory: 8G
        reservations:
          cpus: '2.0'
          memory: 4G
          devices:
            - driver: nvidia
              count: 1
              capabilities: [gpu]
```

---

## Multi-Stage Builds

```dockerfile
# Stage 1: Build dependencies
FROM python:3.10 AS builder
WORKDIR /build
COPY requirements.txt .
RUN pip install --user -r requirements.txt

# Stage 2: Runtime
FROM cyberbotics/webots:R2023b-ubuntu22.04
COPY --from=builder /root/.local /root/.local
ENV PATH=/root/.local/bin:$PATH

COPY src/ /app/src/
WORKDIR /app
CMD ["python3", "src/simulation_server.py"]
```

---

## Networking

```yaml
networks:
  webots_net:
    driver: bridge
    ipam:
      config:
        - subnet: 172.28.0.0/16

services:
  simulation:
    networks:
      webots_net:
        ipv4_address: 172.28.0.10
  
  frontend:
    networks:
      - webots_net
```

---

## Code Review Checklist

- [ ] Specific base image tags (not `latest`)
- [ ] `.dockerignore` file exists
- [ ] RUN commands combined and cleaned up
- [ ] NVIDIA GPU configuration correct
- [ ] Environment variables documented
- [ ] Health checks defined
- [ ] Logging configured
- [ ] Resource limits set
- [ ] No secrets in Dockerfile
- [ ] Multi-stage build used (if applicable)
