# Database, Testing & WebSocket Rules

Combined rules for database (Prisma), testing, and WebSocket communication.

**Scope**: Database files (`prisma/**/*`), Test files (`**/*.test.ts`, `**/*.spec.ts`), WebSocket handlers

---

## Database (Prisma)

### Schema Design

```prisma
// prisma/schema.prisma
datasource db {
  provider = "postgresql"
  url      = env("DATABASE_URL")
}

generator client {
  provider = "prisma-client-js"
}

model User {
  id        String   @id @default(cuid())
  email     String   @unique
  name      String?
  createdAt DateTime @default(now())
  updatedAt DateTime @updatedAt
  
  sessions  Session[]
  
  @@index([email])
}

model Session {
  id          String   @id @default(cuid())
  userId      String
  worldFile   String
  status      String   @default("pending")
  recordingUrl String?
  createdAt   DateTime @default(now())
  completedAt DateTime?
  
  user        User     @relation(fields: [userId], references: [id], onDelete: Cascade)
  
  @@index([userId])
  @@index([status])
}
```

### Best Practices

#### Use Relations
```prisma
model Session {
  user User @relation(fields: [userId], references: [id], onDelete: Cascade)
}
```

#### Add Indexes
```prisma
@@index([email])
@@index([userId, status])
```

#### Use Enums
```prisma
enum SessionStatus {
  PENDING
  RUNNING
  COMPLETED
  FAILED
}

model Session {
  status SessionStatus @default(PENDING)
}
```

### Client Usage

```typescript
// lib/db.ts
import { PrismaClient } from '@prisma/client';

const globalForPrisma = global as unknown as { prisma: PrismaClient };

export const prisma = globalForPrisma.prisma || new PrismaClient();

if (process.env.NODE_ENV !== 'production') {
  globalForPrisma.prisma = prisma;
}

// Usage
import { prisma } from '@/lib/db';

async function getSessions(userId: string) {
  return await prisma.session.findMany({
    where: { userId },
    include: { user: true },
    orderBy: { createdAt: 'desc' }
  });
}
```

### Migrations

```bash
# Create migration
npx prisma migrate dev --name add_session_table

# Run migrations in production
npx prisma migrate deploy

# Generate Prisma Client
npx prisma generate
```

### Error Handling

```typescript
import { Prisma } from '@prisma/client';

try {
  await prisma.user.create({ data: { email: 'test@example.com' } });
} catch (error) {
  if (error instanceof Prisma.PrismaClientKnownRequestError) {
    if (error.code === 'P2002') {
      throw new Error('Email already exists');
    }
  }
  throw error;
}
```

---

## Testing

### Unit Tests (Vitest/Jest)

#### Setup
```typescript
// vitest.config.ts
import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./tests/setup.ts']
  }
});
```

#### Component Tests
```typescript
// __tests__/SimulationCard.test.tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { SimulationCard } from '@/components/SimulationCard';

describe('SimulationCard', () => {
  it('renders simulation title', () => {
    render(<SimulationCard title="Test Sim" status="stopped" />);
    expect(screen.getByText('Test Sim')).toBeInTheDocument();
  });
  
  it('calls onStart when start button clicked', () => {
    const handleStart = vi.fn();
    render(<SimulationCard title="Test" status="stopped" onStart={handleStart} />);
    
    fireEvent.click(screen.getByText('Start'));
    expect(handleStart).toHaveBeenCalledTimes(1);
  });
});
```

#### API Route Tests
```typescript
// __tests__/api/sessions.test.ts
import { GET, POST } from '@/app/api/sessions/route';
import { NextRequest } from 'next/server';

describe('Sessions API', () => {
  it('returns sessions list', async () => {
    const request = new NextRequest('http://localhost:3000/api/sessions');
    const response = await GET(request);
    const data = await response.json();
    
    expect(response.status).toBe(200);
    expect(Array.isArray(data)).toBe(true);
  });
});
```

### E2E Tests (Playwright)

#### Configuration
```typescript
// playwright.config.ts
import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './e2e',
  use: {
    baseURL: 'http://localhost:3000',
    trace: 'on-first-retry',
  },
  webServer: {
    command: 'npm run dev',
    port: 3000,
    reuseExistingServer: !process.env.CI,
  },
});
```

#### Test Examples
```typescript
// e2e/simulation-flow.spec.ts
import { test, expect } from '@playwright/test';

test.describe('Simulation Flow', () => {
  test('create and start simulation', async ({ page }) => {
    await page.goto('/');
    
    // Click "New Simulation"
    await page.click('button:has-text("New Simulation")');
    
    // Fill form
    await page.fill('input[name="world"]', 'sumo_arena.wbt');
    await page.click('button:has-text("Create")');
    
    // Verify simulation created
    await expect(page.locator('.simulation-card')).toBeVisible();
    
    // Start simulation
    await page.click('.simulation-card button:has-text("Start")');
    await expect(page.locator('.status')).toHaveText('Running');
  });
});
```

### Python Tests (unittest/pytest)

```python
# tests/test_simulation_server.py
import unittest
from unittest.mock import Mock, patch
from simulation_server import SimulationServer

class TestSimulationServer(unittest.TestCase):
    def setUp(self):
        self.server = SimulationServer()
    
    def test_start_simulation(self):
        """Test simulation startup."""
        result = self.server.start_simulation("test.wbt")
        self.assertTrue(result)
    
    @patch('subprocess.run')
    def test_docker_container_start(self, mock_run):
        """Test Docker container startup with mock."""
        mock_run.return_value = Mock(returncode=0)
        
        result = self.server.start_container()
        self.assertTrue(result)
        mock_run.assert_called_once()
```

### Testing Best Practices

- **Test critical paths** - Focus on important user flows
- **Mock external dependencies** - Don't hit real APIs/DB in unit tests
- **Use descriptive test names** - `test_user_can_create_session`
- **Clean up test data** - Ensure tests don't leave artifacts
- **Test edge cases** - Empty inputs, errors, boundary conditions

---

## WebSocket

### Message Protocol

#### Structure
```typescript
interface WebSocketMessage {
  type: string;
  sessionId?: string;
  clientId?: string;
  data: unknown;
  timestamp?: number;
}
```

#### Message Types
```typescript
// Client → Server
type ClientMessage =
  | { type: 'start_simulation'; sessionId: string; world: string }
  | { type: 'stop_simulation'; sessionId: string }
  | { type: 'control_command'; sessionId: string; command: string; params: object };

// Server → Client
type ServerMessage =
  | { type: 'simulation_started'; sessionId: string; data: object }
  | { type: 'simulation_stopped'; sessionId: string; reason: string }
  | { type: 'sensor_data'; sessionId: string; data: number[] }
  | { type: 'error'; error: string; details?: string };
```

### Server Implementation (Tornado)

```python
import json
import tornado.websocket
import logging

class SimulationWebSocket(tornado.websocket.WebSocketHandler):
    clients = set()
    
    def open(self):
        """Handle new WebSocket connection."""
        self.clients.add(self)
        self.session_id = None
        logging.info(f"WebSocket opened: {self.request.remote_ip}")
    
    async def on_message(self, message: str):
        """Handle incoming message."""
        try:
            data = json.loads(message)
            msg_type = data.get('type')
            
            if msg_type == 'start_simulation':
                await self.handle_start_simulation(data)
            elif msg_type == 'stop_simulation':
                await self.handle_stop_simulation(data)
            else:
                await self.send_error(f"Unknown message type: {msg_type}")
        
        except json.JSONDecodeError:
            await self.send_error("Invalid JSON")
        except Exception as e:
            logging.error(f"Error processing message: {e}")
            await self.send_error(str(e))
    
    async def handle_start_simulation(self, data):
        """Start simulation."""
        self.session_id = data.get('sessionId')
        world = data.get('world')
        
        # Start simulation logic...
        
        await self.write_message(json.dumps({
            'type': 'simulation_started',
            'sessionId': self.session_id,
            'data': {'world': world, 'timestamp': time.time()}
        }))
    
    async def send_error(self, error: str):
        """Send error message to client."""
        await self.write_message(json.dumps({
            'type': 'error',
            'error': error
        }))
    
    def on_close(self):
        """Handle connection close."""
        self.clients.discard(self)
        logging.info(f"WebSocket closed: {self.session_id}")
        self.cleanup_resources()
    
    def cleanup_resources(self):
        """Clean up resources on disconnect."""
        if self.session_id:
            # Stop simulation, release resources, etc.
            pass
    
    @classmethod
    def broadcast(cls, message: dict):
        """Broadcast message to all connected clients."""
        msg = json.dumps(message)
        for client in cls.clients:
            client.write_message(msg)
```

### Client Implementation (TypeScript)

```typescript
class SimulationWebSocket {
  private ws: WebSocket | null = null;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  
  connect(url: string) {
    this.ws = new WebSocket(url);
    
    this.ws.onopen = () => {
      console.log('WebSocket connected');
      this.reconnectAttempts = 0;
    };
    
    this.ws.onmessage = (event) => {
      const message = JSON.parse(event.data);
      this.handleMessage(message);
    };
    
    this.ws.onerror = (error) => {
      console.error('WebSocket error:', error);
    };
    
    this.ws.onclose = () => {
      console.log('WebSocket closed');
      this.reconnect(url);
    };
  }
  
  private handleMessage(message: ServerMessage) {
    switch (message.type) {
      case 'simulation_started':
        console.log('Simulation started:', message.data);
        break;
      case 'sensor_data':
        this.updateSensorDisplay(message.data);
        break;
      case 'error':
        console.error('Server error:', message.error);
        break;
    }
  }
  
  send(message: ClientMessage) {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message));
    } else {
      console.error('WebSocket not connected');
    }
  }
  
  private reconnect(url: string) {
    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      this.reconnectAttempts++;
      setTimeout(() => this.connect(url), 1000 * this.reconnectAttempts);
    }
  }
  
  disconnect() {
    this.ws?.close();
  }
}
```

### Connection Health Monitoring

```python
# Server-side ping/pong
class SimulationWebSocket(tornado.websocket.WebSocketHandler):
    def open(self):
        self.ping_interval = tornado.ioloop.PeriodicCallback(
            self.send_ping, 30000  # 30 seconds
        )
        self.ping_interval.start()
    
    def send_ping(self):
        try:
            self.ping(b'')
        except Exception:
            self.close()
    
    def on_pong(self, data):
        logging.debug("Pong received")
    
    def on_close(self):
        if hasattr(self, 'ping_interval'):
            self.ping_interval.stop()
```

---

## Code Review Checklist

### Database
- [ ] Proper indexes defined
- [ ] Relations configured correctly
- [ ] Enums used where appropriate
- [ ] Migrations tested
- [ ] Error handling for unique constraints

### Testing
- [ ] Critical paths covered
- [ ] External dependencies mocked
- [ ] Test names descriptive
- [ ] Test data cleaned up
- [ ] Edge cases tested

### WebSocket
- [ ] Message format consistent
- [ ] Error handling implemented
- [ ] Resources cleaned on disconnect
- [ ] Connection lifecycle logged
- [ ] Reconnection logic implemented (client)
