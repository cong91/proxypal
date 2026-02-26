# Project-Specific Conventions

Webots-specific conventions for the S4VN Robotics project.

**Scope**: All files (`**/*`)

---

## Project Structure

### S4VN-Simulation (Python Backend)
```
S4VN-Simulation/
├── src/                          # Main Python source code
│   ├── simulation_server.py      # Main server
│   ├── websocket_handler.py      # WebSocket handling
│   ├── docker_manager.py         # Docker operations
│   └── utils/                    # Utility modules
├── config/                       # Configuration files
│   ├── docker/                   # Docker configs
│   │   ├── Dockerfile.default
│   │   ├── docker-compose-default.yml
│   │   └── docker-compose-theia.yml
│   ├── nginx/                    # Nginx configs
│   └── simulation/               # Simulation configs
├── scripts/                      # Utility scripts
├── worlds/                       # Webots world files
├── requirements.txt              # Python dependencies
└── deploy.sh                     # Main deployment script
```

### S4VN-Simulation-Frontend (Next.js)
```
S4VN-Simulation-Frontend/
├── app/                          # Next.js App Router
│   ├── layout.tsx
│   ├── page.tsx
│   ├── simulation/
│   └── api/
├── lib/                          # Shared utilities
│   ├── db.ts                     # Prisma client
│   ├── auth.ts                   # Authentication
│   └── utils.ts
├── components/                   # React components
│   ├── ui/                       # UI components
│   └── simulation/               # Simulation-specific
├── prisma/                       # Database schema
│   ├── schema.prisma
│   └── migrations/
├── __tests__/                    # Unit tests
├── e2e/                          # End-to-end tests
└── public/                       # Static assets
```

### Robotics_Simulation (Webots Worlds & Controllers)
```
Robotics_Simulation/
├── ROBO 001/
│   └── Sumo_Challenge/
│       ├── worlds/
│       │   └── sumo_arena.wbt
│       └── controllers/
│           └── sumo_arena/
│               ├── sumo_arena.py
│               └── Makefile
└── UI/
    └── controllers/
        └── my_controller/
            ├── my_controller.py
            └── Makefile
```

---

## Naming Conventions

### Files

#### Python
```
snake_case.py
```
Examples: `simulation_server.py`, `gpu_checker.py`, `docker_manager.py`

#### TypeScript/React
```
camelCase.ts
PascalCase.tsx  (for components)
kebab-case.tsx  (alternative for components)
```
Examples: `utils.ts`, `SimulationCard.tsx`, `api-client.ts`

#### Configuration
```
kebab-case.yml
kebab-case.json
SCREAMING_SNAKE_CASE.env
```
Examples: `docker-compose.yml`, `tsconfig.json`, `.env.local`

### Code

#### Python
- Functions/variables: `snake_case`
- Classes: `PascalCase`
- Constants: `SCREAMING_SNAKE_CASE`

```python
MAX_SIMULATION_TIME = 300

class SimulationServer:
    def start_simulation(self):
        pass

def calculate_robot_position(x, y):
    return (x * 2, y * 2)
```

#### TypeScript
- Functions/variables: `camelCase`
- Types/Interfaces/Components: `PascalCase`
- Constants: `SCREAMING_SNAKE_CASE` or `camelCase`

```typescript
const MAX_RETRIES = 3;

interface UserSession {
  userId: string;
}

function calculateDistance(x: number, y: number): number {
  return Math.sqrt(x * x + y * y);
}

function SimulationCard({ title }: { title: string }) {
  return <div>{title}</div>;
}
```

---

## Import Conventions

### Python
Group imports: **stdlib → third-party → local**

```python
# Standard library
import os
import sys
from typing import Optional

# Third-party
import tornado
import psutil

# Local
from simulation_server import SimulationServer
from utils.gpu_checker import check_gpu_available
```

### TypeScript
Group imports: **external → internal (@/) → relative**

```typescript
// External libraries
import { NextRequest } from 'next/server';
import { z } from 'zod';

// Internal libraries (from @/)
import { prisma } from '@/lib/db';
import { getSession } from '@/lib/auth';

// Components
import { Button } from '@/components/ui/Button';

// Relative imports
import { SessionType } from './types';
```

---

## Error Handling (CRITICAL)

### No Fallback Policy
**NEVER implement fallback logic.** See [no-fallback-policy.md](no-fallback-policy.md)

### Error Messages Must Include:
1. **What** failed
2. **Why** it failed
3. **How** to fix

```python
# ✅ GOOD
if not gpu_available:
    error = """
    GPU not available. NVIDIA GPU is required.
    
    Fix Steps:
    1. Install NVIDIA drivers
    2. Install nvidia-container-toolkit: sudo apt install nvidia-container-toolkit
    3. Verify with: nvidia-smi
    4. Restart Docker: sudo systemctl restart docker
    """
    logging.error(error)
    raise RuntimeError("GPU not available")
```

### Log Errors with Context
```python
try:
    start_simulation(session_id)
except Exception as e:
    logging.error(f"Failed to start simulation {session_id}: {e}")
    raise
```

---

## WebSocket Communication

### Message Format
Use structured JSON messages with:
- `type`: Message type
- `session_id` or `client_id`: Identifier
- `data`: Payload

```python
# Server → Client
{
    "type": "simulation_started",
    "session_id": "abc123",
    "data": {
        "world": "sumo_arena.wbt",
        "timestamp": 1234567890
    }
}

# Client → Server
{
    "type": "control_command",
    "session_id": "abc123",
    "data": {
        "command": "move_forward",
        "speed": 1.5
    }
}
```

### Connection Lifecycle
```python
class SimulationWebSocket(tornado.websocket.WebSocketHandler):
    def open(self):
        logging.info(f"WebSocket opened: {self.request.remote_ip}")
        self.session_id = str(uuid.uuid4())
    
    async def on_message(self, message):
        # Process message
        pass
    
    def on_close(self):
        logging.info(f"WebSocket closed: {self.session_id}")
        self.cleanup_resources()  # IMPORTANT
```

---

## Docker Configuration

### Use Dockerfile.default for Webots
```bash
# Build
docker-compose -f config/docker/docker-compose-default.yml build

# Run
docker-compose -f config/docker/docker-compose-default.yml up
```

### GPU Requirements
- **NVIDIA GPU on host** (required)
- **NVIDIA Container Toolkit** installed
- **Proper docker-compose GPU configuration**
- **DRI extensions enabled in Xvfb**

### Error Detection
```python
def check_gpu_rendering():
    """Detect Mesa/software rendering and FAIL."""
    gl_renderer = os.getenv("GL_RENDERER", "")
    
    if "Mesa" in gl_renderer or "llvmpipe" in gl_renderer:
        raise RuntimeError(
            "Software rendering detected. GPU acceleration required.\n"
            "See deployment guide for GPU setup instructions."
        )
```

---

## S3 Integration

### Upload (Presigned URLs)
```python
import boto3

s3_client = boto3.client('s3')

def generate_upload_url(filename: str) -> str:
    """Generate presigned URL for client upload."""
    return s3_client.generate_presigned_url(
        'put_object',
        Params={
            'Bucket': S3_BUCKET,
            'Key': f'simulations/{filename}',
            'ContentType': 'video/mp4'
        },
        ExpiresIn=3600  # 1 hour
    )
```

### Download (Presigned URLs)
```typescript
async function getRecordingUrl(key: string): Promise<string> {
  const response = await fetch(`/api/recordings/${key}/url`);
  const { url } = await response.json();
  return url;
}
```

---

## Authentication (NextAuth)

### Protect API Routes
```typescript
// app/api/sessions/route.ts
import { requireAuth } from '@/lib/auth';

export async function GET() {
  const session = await requireAuth();
  // ... protected logic
}
```

### Protect Pages
```tsx
// app/admin/page.tsx
import { redirect } from 'next/navigation';
import { getSession } from '@/lib/auth';

export default async function AdminPage() {
  const session = await getSession();
  
  if (!session) {
    redirect('/auth/signin');
  }
  
  return <AdminDashboard />;
}
```

---

## Logging

### Python
```python
import logging

# Configure at startup
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)

logger = logging.getLogger(__name__)

# Usage
logger.info("Simulation started")
logger.error("GPU not available", exc_info=True)
```

### TypeScript
```typescript
// lib/logger.ts
export const logger = {
  info: (message: string, meta?: object) => {
    console.log(JSON.stringify({ level: 'info', message, ...meta }));
  },
  error: (message: string, error?: Error, meta?: object) => {
    console.error(JSON.stringify({ 
      level: 'error', 
      message, 
      error: error?.message,
      stack: error?.stack,
      ...meta 
    }));
  }
};
```

---

## Testing

### Python Unit Tests
```python
# tests/test_simulation.py
import unittest
from simulation_server import SimulationServer

class TestSimulation(unittest.TestCase):
    def setUp(self):
        self.server = SimulationServer()
    
    def test_start_simulation(self):
        result = self.server.start("test_world.wbt")
        self.assertTrue(result)
```

### TypeScript E2E Tests (Playwright)
```typescript
// e2e/simulation.spec.ts
import { test, expect } from '@playwright/test';

test('start simulation', async ({ page }) => {
  await page.goto('/simulation');
  await page.click('button:has-text("Start")');
  await expect(page.locator('.status')).toHaveText('Running');
});
```

---

## Deployment

### S4VN-Simulation
```bash
# Run deployment script
cd S4VN-Simulation
./deploy.sh

# What it does:
# 1. Fix CRLF issues
# 2. Install NVIDIA Container Toolkit (if needed)
# 3. Build Docker containers
# 4. Set up systemd service
# 5. Verify GPU access
```

### S4VN-Simulation-Frontend
```bash
# Build for production
npm run build

# Run with PM2 (optional)
pm2 start ecosystem.config.js

# Or use Next.js standalone
node .next/standalone/server.js
```

---

## Code Review Checklist

- [ ] Follows project naming conventions
- [ ] Imports properly organized
- [ ] Error handling with clear messages
- [ ] No fallback logic
- [ ] Logging with appropriate levels
- [ ] WebSocket messages properly structured
- [ ] GPU requirements verified
- [ ] Tests written for critical paths
- [ ] Documentation updated (if needed)
