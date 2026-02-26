# TypeScript, Next.js & React Rules

Combined rules for TypeScript, Next.js, and React development in the S4VN Webots project.

**Scope**: `**/*.ts`, `**/*.tsx` (TypeScript and React files)

---

## TypeScript Code Style

### Naming Conventions
- `camelCase` for functions and variables
- `PascalCase` for types, interfaces, and components
- `SCREAMING_SNAKE_CASE` for constants
- Prefix interfaces with `I` (optional, project preference)

```typescript
// Constants
const MAX_RETRIES = 3;
const API_BASE_URL = 'https://api.example.com';

// Interface
interface UserSession {
  userId: string;
  email: string;
  createdAt: Date;
}

// Function
function calculateDistance(x1: number, y1: number): number {
  return Math.sqrt(x1 * x1 + y1 * y1);
}

// Component
function SimulationViewer({ sessionId }: { sessionId: string }) {
  return <div>...</div>;
}
```

### Type Safety
- **Always use strict mode** (`"strict": true` in tsconfig.json)
- **No `any`** - use `unknown` if type is truly unknown
- Use union types instead of `any`
- Prefer type inference when obvious

```typescript
// ❌ BAD
function processData(data: any) {
  return data.map((item: any) => item.value);
}

// ✅ GOOD
interface DataItem {
  value: number;
  label: string;
}

function processData(data: DataItem[]): number[] {
  return data.map(item => item.value);
}
```

---

## Next.js Best Practices

### Server Components (Default)
- **Use Server Components by default** (Next.js 13+ App Router)
- Only use Client Components when needed (interactivity, hooks)
- Mark Client Components with `'use client'`

```tsx
// app/simulation/page.tsx - Server Component (default)
export default async function SimulationPage() {
  const sessions = await fetchSessions(); // Can use async/await
  return <SessionList sessions={sessions} />;
}

// components/InteractiveButton.tsx - Client Component
'use client';

import { useState } from 'react';

export function InteractiveButton() {
  const [count, setCount] = useState(0);
  return <button onClick={() => setCount(c => c + 1)}>{count}</button>;
}
```

### File Structure (App Router)
```
app/
├── layout.tsx           # Root layout
├── page.tsx             # Home page
├── simulation/
│   ├── page.tsx         # /simulation
│   ├── [id]/
│   │   └── page.tsx     # /simulation/:id
│   └── loading.tsx      # Loading UI
├── api/
│   ├── sessions/
│   │   └── route.ts     # API route
│   └── webhook/
│       └── route.ts
└── components/          # Shared components (optional)
```

### API Routes
- Use `route.ts` files in `app/api/`
- Export named functions: `GET`, `POST`, `PUT`, `DELETE`
- Return `Response` or `NextResponse`

```typescript
// app/api/sessions/route.ts
import { NextRequest, NextResponse } from 'next/server';
import { prisma } from '@/lib/db';

export async function GET(request: NextRequest) {
  try {
    const sessions = await prisma.session.findMany();
    return NextResponse.json(sessions);
  } catch (error) {
    console.error('Failed to fetch sessions:', error);
    return NextResponse.json(
      { error: 'Failed to fetch sessions' },
      { status: 500 }
    );
  }
}

export async function POST(request: NextRequest) {
  const body = await request.json();
  // ... create session
  return NextResponse.json({ id: newSession.id }, { status: 201 });
}
```

### Data Fetching
- Fetch data in Server Components when possible
- Use `fetch` with caching options
- Use SWR or React Query for client-side fetching

```tsx
// Server Component - Fetch at build/request time
async function SessionList() {
  const res = await fetch('http://localhost:3000/api/sessions', {
    cache: 'no-store' // or 'force-cache', next: { revalidate: 60 }
  });
  const sessions = await res.json();
  return <ul>{sessions.map(s => <li key={s.id}>{s.name}</li>)}</ul>;
}
```

---

## React Best Practices

### Component Structure
- Functional components only (no class components)
- Keep components small and focused
- Extract logic into custom hooks

```tsx
// ❌ BAD - Too much logic in component
function SimulationDashboard() {
  const [sessions, setSessions] = useState([]);
  const [loading, setLoading] = useState(false);
  
  useEffect(() => {
    setLoading(true);
    fetch('/api/sessions')
      .then(res => res.json())
      .then(data => setSessions(data))
      .finally(() => setLoading(false));
  }, []);
  
  return <div>...</div>;
}

// ✅ GOOD - Logic extracted to hook
function useSessionsAPI() {
  const [sessions, setSessions] = useState([]);
  const [loading, setLoading] = useState(false);
  
  useEffect(() => {
    setLoading(true);
    fetch('/api/sessions')
      .then(res => res.json())
      .then(data => setSessions(data))
      .finally(() => setLoading(false));
  }, []);
  
  return { sessions, loading };
}

function SimulationDashboard() {
  const { sessions, loading } = useSessionsAPI();
  return <div>...</div>;
}
```

### Props and State
- Destructure props in function signature
- Use TypeScript interfaces for props
- Keep state minimal and derived

```tsx
interface SimulationCardProps {
  sessionId: string;
  title: string;
  status: 'running' | 'stopped' | 'error';
  onStart?: () => void;
}

function SimulationCard({ 
  sessionId, 
  title, 
  status, 
  onStart 
}: SimulationCardProps) {
  return (
    <div className="card">
      <h3>{title}</h3>
      <StatusBadge status={status} />
      {status === 'stopped' && onStart && (
        <button onClick={onStart}>Start</button>
      )}
    </div>
  );
}
```

---

## Error Handling

### API Error Handling
```typescript
async function fetchSessions(): Promise<Session[]> {
  try {
    const res = await fetch('/api/sessions');
    
    if (!res.ok) {
      throw new Error(`HTTP ${res.status}: ${res.statusText}`);
    }
    
    return await res.json();
  } catch (error) {
    console.error('Failed to fetch sessions:', error);
    throw new Error(
      'Failed to load sessions. Please check your connection and try again.'
    );
  }
}
```

### Error Boundaries (Client Components)
```tsx
'use client';

import { Component, ReactNode } from 'react';

interface ErrorBoundaryProps {
  children: ReactNode;
  fallback?: ReactNode;
}

interface ErrorBoundaryState {
  hasError: boolean;
  error?: Error;
}

export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError(error: Error) {
    return { hasError: true, error };
  }

  render() {
    if (this.state.hasError) {
      return this.props.fallback || <div>Something went wrong</div>;
    }
    return this.props.children;
  }
}
```

---

## Performance

### Image Optimization
```tsx
import Image from 'next/image';

function RobotAvatar({ src, name }: { src: string; name: string }) {
  return (
    <Image
      src={src}
      alt={name}
      width={200}
      height={200}
      priority={false}
      placeholder="blur"
    />
  );
}
```

### Code Splitting
```tsx
import dynamic from 'next/dynamic';

// Lazy load heavy component
const HeavySimulationViewer = dynamic(
  () => import('@/components/SimulationViewer'),
  { loading: () => <p>Loading viewer...</p> }
);

function SimulationPage() {
  return (
    <div>
      <h1>Simulation</h1>
      <HeavySimulationViewer />
    </div>
  );
}
```

### Memoization
```tsx
import { useMemo, useCallback } from 'react';

function ExpensiveComponent({ data }: { data: number[] }) {
  const processedData = useMemo(() => {
    return data.map(n => n * 2).filter(n => n > 10);
  }, [data]);
  
  const handleClick = useCallback(() => {
    console.log('Clicked');
  }, []);
  
  return <div onClick={handleClick}>{processedData.length}</div>;
}
```

---

## Authentication (NextAuth)

```typescript
// lib/auth.ts
import { getServerSession } from 'next-auth';
import { authOptions } from '@/app/api/auth/[...nextauth]/route';

export async function getSession() {
  return await getServerSession(authOptions);
}

export async function requireAuth() {
  const session = await getSession();
  if (!session) {
    throw new Error('Unauthorized');
  }
  return session;
}

// app/api/protected/route.ts
import { requireAuth } from '@/lib/auth';

export async function GET() {
  const session = await requireAuth();
  // ... protected logic
}
```

---

## Imports Organization

```typescript
// 1. External libraries
import { NextRequest, NextResponse } from 'next/server';
import { prisma } from '@prisma/client';

// 2. Internal libraries (from @/)
import { getSession } from '@/lib/auth';
import { logger } from '@/lib/logger';

// 3. Components
import { Button } from '@/components/ui/Button';
import { Card } from '@/components/ui/Card';

// 4. Relative imports
import { SessionType } from './types';
import { formatDate } from './utils';
```

---

## Code Review Checklist

- [ ] TypeScript strict mode enabled
- [ ] No `any` types used
- [ ] Server Components used by default
- [ ] Client Components marked with `'use client'`
- [ ] Proper error handling
- [ ] Images optimized with `next/image`
- [ ] Props typed with interfaces
- [ ] Custom hooks for reusable logic
- [ ] Imports properly organized
- [ ] Performance optimizations applied
