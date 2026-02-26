# Implementation Plan: Online Editor & Animation Build System

**Branch**: `online-editor-animation-build` | **Date**: 2026-02-06 | **Spec**: [specification.md](./specification.md)
**Input**: Feature specification from `/specs/online-editor-animation-build/specification.md`

---

## Summary

Triển khai hệ thống "Online Editor & Animation Build" cho S4VN-Webots platform với mục tiêu:
1. **Monaco Editor** tích hợp vào frontend để edit code online
2. **GitHub Actions workflow** để build animation từ code
3. **S3 storage** cho animation HTML và playback trên WebotsView

---

## Technical Context

**Language/Version**: TypeScript 5.x (Frontend), Python 3.11 (Build scripts)
**Primary Dependencies**: Next.js 14, Monaco Editor, AWS SDK v3, Octokit
**Storage**: AWS S3 (files), PostgreSQL (metadata)
**Testing**: Jest, Playwright
**Target Platform**: Web (Desktop + Mobile browsers)
**Project Type**: Web application (existing monorepo)
**Performance Goals**: Editor load < 2s, Build complete < 10 min
**Constraints**: Free GitHub Actions tier (2000 min/month), S3 cost optimization
**Scale/Scope**: 100+ concurrent users, 1000+ animation builds/month

---

## Project Structure

### Documentation (this feature)

```text
.specify/specs/online-editor-animation-build/
├── specification.md      # Feature specification (created)
├── planning.md           # This file
└── tasks.md              # Implementation tasks (to be created)
```

### Source Code (new files)

```text
S4VN-Simulation-Frontend/
├── app/
│   ├── editor/
│   │   ├── page.tsx              # Editor page
│   │   └── [projectId]/
│   │       └── page.tsx          # Project-specific editor
│   └── api/
│       ├── editor/
│       │   ├── load/route.ts     # Load file from S3
│       │   ├── save/route.ts     # Save file to S3
│       │   └── commit/route.ts   # Commit to GitHub
│       └── animation/
│           ├── build/route.ts    # Trigger build
│           ├── status/route.ts   # Check build status
│           └── callback/route.ts # Build completion callback
├── components/
│   ├── editor/
│   │   ├── MonacoEditor.tsx      # Monaco wrapper
│   │   ├── FileTree.tsx          # Project file tree
│   │   └── EditorToolbar.tsx     # Save/Commit/Build buttons
│   └── animation/
│       └── AnimationPlayer.tsx   # Animation playback
├── lib/
│   ├── editor/
│   │   ├── s3-editor.ts          # S3 operations for editor
│   │   └── github-commit.ts      # GitHub API integration
│   └── animation/
│       └── build-trigger.ts      # GitHub Actions trigger
└── hooks/
    ├── useEditorSession.ts       # Editor state management
    └── useBuildStatus.ts         # Build status polling

.github/workflows/
└── build-animation.yml           # Animation build workflow
```

---

## Phase 0: Research & Validation

### 0.1 Monaco Editor Integration

**Status**: ✅ Validated

Monaco Editor có thể được tích hợp qua `@monaco-editor/react`:

```bash
npm install @monaco-editor/react
```

**Key considerations:**
- Lazy loading để giảm initial bundle
- Worker setup cho syntax highlighting
- Theme customization (dark/light)

### 0.2 GitHub Actions API

**Status**: ✅ Validated

Trigger workflow qua Repository Dispatch:

```typescript
// Using Octokit
await octokit.repos.createDispatchEvent({
  owner: 'pal-admin',
  repo: 'S4VN-Webots',
  event_type: 'build-animation',
  client_payload: { projectId, duration }
});
```

### 0.3 S3 Presigned URLs

**Status**: ✅ Validated (existing implementation)

Đã có trong [`lib/s3.ts`](../../S4VN-Simulation-Frontend/lib/s3.ts:80):
- `getPresignedUploadUrl()`
- `getPresignedDownloadUrl()`

---

## Phase 1: Core Implementation

### 1.1 Monaco Editor Component

**Priority**: P1 - Critical path

```tsx
// components/editor/MonacoEditor.tsx
'use client';
import Editor from '@monaco-editor/react';
import { useState, useCallback } from 'react';

interface Props {
  value: string;
  language: string;
  onChange: (value: string) => void;
  onSave?: () => void;
}

export function MonacoEditor({ value, language, onChange, onSave }: Props) {
  const handleEditorChange = useCallback((val: string | undefined) => {
    onChange(val ?? '');
  }, [onChange]);

  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    if ((e.ctrlKey || e.metaKey) && e.key === 's') {
      e.preventDefault();
      onSave?.();
    }
  }, [onSave]);

  return (
    <Editor
      height="100%"
      language={language}
      value={value}
      onChange={handleEditorChange}
      theme="vs-dark"
      options={{
        minimap: { enabled: true },
        fontSize: 14,
        wordWrap: 'on',
        automaticLayout: true,
      }}
    />
  );
}
```

### 1.2 Editor API Routes

**Priority**: P1 - Critical path

```typescript
// app/api/editor/load/route.ts
import { getFromS3 } from '@/lib/s3';
import { NextRequest, NextResponse } from 'next/server';

export async function GET(req: NextRequest) {
  const projectId = req.nextUrl.searchParams.get('projectId');
  const filePath = req.nextUrl.searchParams.get('path');
  
  if (!projectId || !filePath) {
    return NextResponse.json({ error: 'Missing params' }, { status: 400 });
  }
  
  const key = `editor/${projectId}/${filePath}`;
  const result = await getFromS3(S3_BUCKET, key);
  
  if (!result.success) {
    return NextResponse.json({ error: 'File not found' }, { status: 404 });
  }
  
  return NextResponse.json({
    content: result.content,
    path: filePath,
    lastModified: result.lastModified,
  });
}
```

### 1.3 Build Trigger

**Priority**: P2

```typescript
// lib/animation/build-trigger.ts
import { Octokit } from '@octokit/rest';

const octokit = new Octokit({ auth: process.env.GITHUB_TOKEN });

export async function triggerBuild(
  projectId: string,
  duration: number = 10
): Promise<{ success: boolean; runId?: number }> {
  try {
    await octokit.repos.createDispatchEvent({
      owner: process.env.GITHUB_OWNER!,
      repo: process.env.GITHUB_REPO!,
      event_type: 'build-animation',
      client_payload: {
        project_id: projectId,
        duration,
        callback_url: `${process.env.API_BASE}/api/animation/callback`,
      },
    });
    
    return { success: true };
  } catch (error) {
    console.error('Build trigger failed:', error);
    return { success: false };
  }
}
```

### 1.4 GitHub Actions Workflow

**Priority**: P2

```yaml
# .github/workflows/build-animation.yml
name: Build Animation

on:
  repository_dispatch:
    types: [build-animation]

jobs:
  build:
    runs-on: ubuntu-latest
    timeout-minutes: 30
    
    steps:
      - name: Checkout animation action
        uses: actions/checkout@v4
        with:
          repository: pal-admin/webots-animation-action
          token: ${{ secrets.PAT_TOKEN }}
          
      - name: Configure AWS credentials
        uses: aws-actions/configure-aws-credentials@v4
        with:
          aws-access-key-id: ${{ secrets.AWS_ACCESS_KEY_ID }}
          aws-secret-access-key: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
          aws-region: ${{ secrets.AWS_REGION }}
          
      - name: Download project from S3
        run: |
          aws s3 cp s3://${{ secrets.S3_BUCKET }}/editor/${{ github.event.client_payload.project_id }}/ \
            ./project --recursive
            
      - name: Setup Webots
        run: |
          sudo apt-get update
          sudo apt-get install -y xvfb
          
      - name: Record animation
        run: |
          cd ./project
          python3 -m wb_animation_action \
            --duration=${{ github.event.client_payload.duration }} \
            --output=/tmp/animation
            
      - name: Upload animation to S3
        run: |
          aws s3 cp /tmp/animation/index.html \
            s3://${{ secrets.S3_BUCKET }}/animations/${{ github.event.client_payload.project_id }}/animation.html \
            --content-type "text/html"
            
      - name: Notify completion
        run: |
          curl -X POST "${{ github.event.client_payload.callback_url }}" \
            -H "Content-Type: application/json" \
            -H "Authorization: Bearer ${{ secrets.CALLBACK_SECRET }}" \
            -d '{
              "project_id": "${{ github.event.client_payload.project_id }}",
              "status": "completed",
              "animation_url": "s3://${{ secrets.S3_BUCKET }}/animations/${{ github.event.client_payload.project_id }}/animation.html"
            }'
```

### 1.5 Animation Player Component

**Priority**: P1

```tsx
// components/animation/AnimationPlayer.tsx
'use client';
import { useEffect, useState } from 'react';

interface Props {
  animationId: string;
}

export function AnimationPlayer({ animationId }: Props) {
  const [url, setUrl] = useState<string>();
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string>();

  useEffect(() => {
    async function loadAnimation() {
      try {
        const res = await fetch(`/api/animation/${animationId}`);
        if (!res.ok) throw new Error('Animation not found');
        
        const data = await res.json();
        setUrl(data.presignedUrl);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load');
      } finally {
        setLoading(false);
      }
    }
    
    loadAnimation();
  }, [animationId]);

  if (loading) {
    return (
      <div className="flex items-center justify-center h-[600px] bg-gray-100">
        <div className="animate-spin h-8 w-8 border-4 border-blue-500 rounded-full border-t-transparent" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-[600px] bg-red-50">
        <p className="text-red-600">{error}</p>
      </div>
    );
  }

  return (
    <iframe
      src={url}
      className="w-full h-[600px] border-0 rounded-lg"
      sandbox="allow-scripts allow-same-origin"
      title="Animation Player"
    />
  );
}
```

---

## Phase 2: Database Schema

### New Tables/Models

```prisma
// prisma/schema.prisma additions

model EditorSession {
  id          String   @id @default(cuid())
  userId      String
  projectId   String
  currentFile String?
  lastSaved   DateTime @default(now())
  createdAt   DateTime @default(now())
  
  user        User     @relation(fields: [userId], references: [id])
  project     Project  @relation(fields: [projectId], references: [id])
  
  @@index([userId])
  @@index([projectId])
}

model BuildJob {
  id          String   @id @default(cuid())
  projectId   String
  status      BuildStatus @default(QUEUED)
  duration    Int      @default(10)
  runId       String?  // GitHub Actions run ID
  outputUrl   String?
  errorMessage String?
  triggeredAt DateTime @default(now())
  startedAt   DateTime?
  completedAt DateTime?
  
  project     Project  @relation(fields: [projectId], references: [id])
  
  @@index([projectId])
  @@index([status])
}

enum BuildStatus {
  QUEUED
  RUNNING
  COMPLETED
  FAILED
  CANCELLED
}
```

---

## Phase 3: Security Implementation

### 3.1 API Authentication

```typescript
// middleware.ts additions
export function middleware(request: NextRequest) {
  const editorPaths = ['/api/editor', '/editor'];
  const isEditorPath = editorPaths.some(p => 
    request.nextUrl.pathname.startsWith(p)
  );
  
  if (isEditorPath) {
    const session = await getServerSession(authOptions);
    if (!session) {
      return NextResponse.redirect(new URL('/auth/signin', request.url));
    }
  }
  
  return NextResponse.next();
}
```

### 3.2 Build Callback Validation

```typescript
// app/api/animation/callback/route.ts
export async function POST(req: NextRequest) {
  const authHeader = req.headers.get('authorization');
  const expectedToken = `Bearer ${process.env.CALLBACK_SECRET}`;
  
  if (authHeader !== expectedToken) {
    return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
  }
  
  const body = await req.json();
  // Process callback...
}
```

---

## Phase 4: Testing Strategy

### Unit Tests

```typescript
// __tests__/lib/animation/build-trigger.test.ts
describe('triggerBuild', () => {
  it('should trigger GitHub Actions workflow', async () => {
    const result = await triggerBuild('project-123', 10);
    expect(result.success).toBe(true);
  });
  
  it('should handle API errors gracefully', async () => {
    // Mock API failure
    const result = await triggerBuild('invalid', 10);
    expect(result.success).toBe(false);
  });
});
```

### Integration Tests

```typescript
// e2e/editor.spec.ts
test('should save file to S3', async ({ page }) => {
  await page.goto('/editor/test-project');
  
  // Type in editor
  await page.locator('.monaco-editor textarea').fill('print("Hello")');
  
  // Save
  await page.keyboard.press('Control+S');
  
  // Verify save confirmation
  await expect(page.getByText('Saved')).toBeVisible();
});
```

---

## Implementation Phases & Timeline

### Phase A: MVP Editor (Week 1-2)

- [ ] Monaco Editor integration
- [ ] Load file from S3 API
- [ ] Save file to S3 API
- [ ] Basic editor page UI

### Phase B: Build System (Week 3-4)

- [ ] GitHub Actions workflow
- [ ] Build trigger API
- [ ] Build status tracking
- [ ] Callback handling

### Phase C: Playback & Polish (Week 5-6)

- [ ] Animation player component
- [ ] Build history UI
- [ ] Error handling
- [ ] Performance optimization

### Phase D: GitHub Sync (Week 7-8)

- [ ] Commit to GitHub API
- [ ] Two-way sync
- [ ] Conflict resolution
- [ ] Webhook integration

---

## Risks & Mitigations

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| GitHub Actions rate limit | High | Medium | Implement queue, use self-hosted runner |
| S3 cost overrun | Medium | Low | Set lifecycle policies, compression |
| Build failures | Medium | Medium | Retry logic, detailed error messages |
| Security vulnerabilities | High | Low | Sandboxing, input validation, code review |

---

## Success Metrics

1. **Editor adoption**: >50% of users use online editor
2. **Build success rate**: >95%
3. **Build time**: <5 minutes average
4. **Animation load time**: <3 seconds
5. **User satisfaction**: >4.0/5.0 rating

---

## Dependencies

- **External**: GitHub Actions, AWS S3, Cyberbotics CDN
- **Internal**: Auth system, S3 utilities, Database
- **Packages**: @monaco-editor/react, @octokit/rest, @aws-sdk/client-s3

---

## Rollout Plan

1. **Alpha**: Internal testing with 5 projects
2. **Beta**: Selected users (10-20) with feedback form
3. **GA**: Full rollout with feature flag
4. **Optimization**: Based on metrics and feedback
