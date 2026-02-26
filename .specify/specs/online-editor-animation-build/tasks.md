# Tasks: Online Editor & Animation Build System

**Input**: Design documents from `/specs/online-editor-animation-build/`
**Prerequisites**: [planning.md](./planning.md), [specification.md](./specification.md)
**Branch**: `online-editor-animation-build`
**Created**: 2026-02-06

**Tests**: Included based on specification requirements (Jest + Playwright).

**Organization**: Tasks are grouped by user story and phase. **Phase A (MVP Editor)** is the immediate priority.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization, dependencies, and basic structure

- [ ] T001 [P] Install Monaco Editor dependency `@monaco-editor/react` in `S4VN-Simulation-Frontend/`
- [ ] T002 [P] Install GitHub API client `@octokit/rest` in `S4VN-Simulation-Frontend/`
- [ ] T003 [P] Add S3 folder constant `EDITOR` to `S4VN-Simulation-Frontend/lib/s3.ts`
- [ ] T004 [P] Create editor types in `S4VN-Simulation-Frontend/types/editor.ts`
- [ ] T005 [P] Create animation build types in `S4VN-Simulation-Frontend/types/build.ts`

---

## Phase 2: Database Schema (Foundational)

**Purpose**: Core data models that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T006 Add `EditorSession` model to `S4VN-Simulation-Frontend/prisma/schema.prisma`
- [ ] T007 Add `BuildJob` model with `BuildStatus` enum to `S4VN-Simulation-Frontend/prisma/schema.prisma`
- [ ] T008 Run Prisma migration: `npx prisma migrate dev --name add-editor-build-models`
- [ ] T009 Generate Prisma client: `npx prisma generate`

**Checkpoint**: Database schema ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Edit Controller Code Online (Priority: P1) 🎯 MVP

**Goal**: User can load code from S3, edit in Monaco Editor, and save back to S3

**Independent Test**: Load a project, edit code, save to S3, verify file changed

**This is Phase A (MVP Editor) - Immediate Priority**

### 3.1 Monaco Editor Component

- [ ] T010 [US1] Create `MonacoEditor.tsx` component in `S4VN-Simulation-Frontend/components/editor/`
  - Lazy loading with dynamic import
  - Dark/light theme support
  - Ctrl+S save shortcut
  - Language detection from file extension
- [ ] T011 [US1] Create `EditorToolbar.tsx` component in `S4VN-Simulation-Frontend/components/editor/`
  - Save button with loading state
  - File path display
  - Dirty indicator (unsaved changes)
- [ ] T012 [US1] Create `FileTree.tsx` component in `S4VN-Simulation-Frontend/components/editor/`
  - Display project file structure
  - File selection handler
  - Folder expand/collapse

### 3.2 Editor API Routes

- [ ] T013 [US1] Create `GET /api/editor/load` route in `S4VN-Simulation-Frontend/app/api/editor/load/route.ts`
  - Params: projectId, filePath
  - Get file content from S3 `editor/{projectId}/{filePath}`
  - Return content, lastModified, path
- [ ] T014 [US1] Create `PUT /api/editor/save` route in `S4VN-Simulation-Frontend/app/api/editor/save/route.ts`
  - Body: projectId, filePath, content
  - Save file to S3 `editor/{projectId}/{filePath}`
  - Update EditorSession lastSaved timestamp
- [ ] T015 [US1] Create `GET /api/editor/tree` route in `S4VN-Simulation-Frontend/app/api/editor/tree/route.ts`
  - Params: projectId
  - List files in S3 prefix `editor/{projectId}/`
  - Return hierarchical file tree structure

### 3.3 S3 Editor Utilities

- [ ] T016 [US1] Create `s3-editor.ts` in `S4VN-Simulation-Frontend/lib/editor/`
  - `getFileContent(projectId, filePath)` - Get file content from S3
  - `saveFileContent(projectId, filePath, content)` - Save file to S3
  - `listProjectFiles(projectId)` - List all files in project
  - `getFileMetadata(projectId, filePath)` - Get file lastModified

### 3.4 Editor State Management

- [ ] T017 [US1] Create `useEditorSession` hook in `S4VN-Simulation-Frontend/hooks/useEditorSession.ts`
  - Track current file, dirty state
  - Auto-save functionality (optional)
  - Session persistence

### 3.5 Editor Page

- [ ] T018 [US1] Create `/editor` page in `S4VN-Simulation-Frontend/app/editor/page.tsx`
  - Project selector or redirect to specific project
- [ ] T019 [US1] Create `/editor/[projectId]` page in `S4VN-Simulation-Frontend/app/editor/[projectId]/page.tsx`
  - Layout: FileTree (left) + MonacoEditor (main) + Toolbar (top)
  - Load default file (e.g., `controllers/robot_controller.py`)

### 3.6 Tests for User Story 1

- [ ] T020 [P] [US1] Unit test for MonacoEditor component in `S4VN-Simulation-Frontend/__tests__/components/editor/MonacoEditor.test.tsx`
- [ ] T021 [P] [US1] Unit test for editor API routes in `S4VN-Simulation-Frontend/__tests__/api/editor/`
- [ ] T022 [P] [US1] Integration test for S3 editor utilities in `S4VN-Simulation-Frontend/__tests__/integration/editor-s3.test.ts`

**Checkpoint**: User Story 1 should be fully functional - user can edit and save code online

---

## Phase 4: User Story 3 - View Pre-recorded Animation (Priority: P1) 🎯 MVP

**Goal**: User can view animation that has been pre-recorded and stored in S3

**Independent Test**: Load animation URL, verify 3D rendering works

**This is also Phase A priority - enables viewing before build system is ready**

### 4.1 Animation Player Component

- [ ] T023 [US3] Create `AnimationPlayer.tsx` component in `S4VN-Simulation-Frontend/components/animation/`
  - Fetch animation presigned URL from API
  - Render in iframe with sandbox
  - Loading and error states

### 4.2 Animation API Routes

- [ ] T024 [US3] Create `GET /api/animation/[id]` route in `S4VN-Simulation-Frontend/app/api/animation/[id]/route.ts`
  - Get animation metadata from DB
  - Generate presigned S3 URL for animation.html
  - Return URL, metadata

### 4.3 Animation Page Enhancement

- [ ] T025 [US3] Update `/animation/[id]` page to use new AnimationPlayer component
  - Replace existing implementation if needed
  - Add playback controls integration

### 4.4 Tests for User Story 3

- [ ] T026 [P] [US3] Unit test for AnimationPlayer component in `S4VN-Simulation-Frontend/__tests__/components/animation/AnimationPlayer.test.tsx`
- [ ] T027 [P] [US3] Integration test for animation playback flow

**Checkpoint**: User Story 3 should be functional - user can view pre-recorded animations

---

## Phase 5: User Story 2 - Build Animation from Code (Priority: P2)

**Goal**: User can trigger animation build from code, track status, view result

**Independent Test**: Trigger build, wait for completion, verify animation playback

**This is Phase B (Build System) - After MVP Editor is complete**

### 5.1 GitHub Integration

- [ ] T028 [US2] Create `github-commit.ts` in `S4VN-Simulation-Frontend/lib/editor/`
  - `commitFile(projectId, filePath, content, message)` - Commit via GitHub Contents API
  - `getFileFromGitHub(owner, repo, path)` - Get file from GitHub
- [ ] T029 [US2] Create `POST /api/editor/commit` route in `S4VN-Simulation-Frontend/app/api/editor/commit/route.ts`
  - Body: projectId, filePath, content, commitMessage
  - Commit file to GitHub repository

### 5.2 Build Trigger

- [ ] T030 [US2] Create `build-trigger.ts` in `S4VN-Simulation-Frontend/lib/animation/`
  - `triggerBuild(projectId, duration)` - Trigger GitHub Actions via repository dispatch
  - Handle rate limiting and errors
- [ ] T031 [US2] Create `POST /api/animation/build` route in `S4VN-Simulation-Frontend/app/api/animation/build/route.ts`
  - Body: projectId, duration
  - Create BuildJob record in DB
  - Trigger GitHub Actions
  - Return jobId

### 5.3 Build Status Tracking

- [ ] T032 [US2] Create `GET /api/animation/status/[jobId]` route in `S4VN-Simulation-Frontend/app/api/animation/status/[jobId]/route.ts`
  - Get BuildJob from DB
  - Return status, progress, outputUrl if completed
- [ ] T033 [US2] Create `POST /api/animation/callback` route in `S4VN-Simulation-Frontend/app/api/animation/callback/route.ts`
  - Validate callback secret
  - Update BuildJob status in DB
  - Store animation URL

### 5.4 Build Status Hook

- [ ] T034 [US2] Create `useBuildStatus` hook in `S4VN-Simulation-Frontend/hooks/useBuildStatus.ts`
  - Poll status endpoint
  - Handle completion callback
  - Error handling with retry

### 5.5 Build UI Components

- [ ] T035 [US2] Add Build button to `EditorToolbar.tsx`
  - Trigger build with confirmation dialog
  - Show build progress
- [ ] T036 [US2] Create `BuildHistory.tsx` component in `S4VN-Simulation-Frontend/components/animation/`
  - List previous builds for project
  - Status indicators, links to animations

### 5.6 GitHub Actions Workflow

- [ ] T037 [US2] Create `build-animation.yml` workflow in `.github/workflows/`
  - Trigger: repository_dispatch with event_type `build-animation`
  - Download project from S3
  - Run Webots animation recording
  - Upload animation.html to S3
  - Call callback URL with result

### 5.7 Tests for User Story 2

- [ ] T038 [P] [US2] Unit test for build-trigger in `S4VN-Simulation-Frontend/__tests__/lib/animation/build-trigger.test.ts`
- [ ] T039 [P] [US2] Unit test for build API routes
- [ ] T040 [P] [US2] Integration test for full build flow (mock GitHub Actions)

**Checkpoint**: User Story 2 should be functional - user can build and view animations

---

## Phase 6: Security & Authentication

**Purpose**: Secure all editor and build endpoints

- [ ] T041 Add authentication check to all `/api/editor/*` routes
- [ ] T042 Add authentication check to `/api/animation/build` route
- [ ] T043 Implement project ownership validation - user can only edit their projects
- [ ] T044 Validate callback secret in `/api/animation/callback`
- [ ] T045 Add rate limiting to build trigger endpoint

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] T046 [P] Add error boundaries to editor pages
- [ ] T047 [P] Add loading skeletons for editor components
- [ ] T048 Add mobile responsive layout for editor
- [ ] T049 [P] Add keyboard shortcuts documentation
- [ ] T050 Performance optimization: Monaco worker configuration
- [ ] T051 [P] Update documentation in docs/EDITOR.md

---

## Dependencies & Execution Order

### Phase Dependencies

```
Phase 1 (Setup) ─────────────────────────────────┐
                                                 │
Phase 2 (Database) ──────────────────────────────┤
                                                 ▼
                        ┌─────── User Story 1 (P1) ──────┐
                        │                                │
Phase 3-4 (MVP) ────────┤                                ├── Phase A Complete
                        │                                │
                        └─────── User Story 3 (P1) ──────┘
                                                 │
                                                 ▼
Phase 5 (Build) ─────────── User Story 2 (P2) ──────────── Phase B Complete
                                                 │
                                                 ▼
Phase 6-7 (Security/Polish) ────────────────────────────── Phase C Complete
```

### Within Each User Story

- Models before services
- Services before API routes
- API routes before UI components
- Components before pages
- Core implementation before tests

### Parallel Opportunities

Tasks marked [P] can run in parallel within the same phase:
- All setup tasks (T001-T005)
- All test tasks within a user story
- Security tasks (T041-T045)
- Polish tasks (T046-T051)

---

## Implementation Strategy

### MVP First (Phase A Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Database
3. Complete Phase 3: User Story 1 (Editor)
4. Complete Phase 4: User Story 3 (Playback)
5. **STOP and VALIDATE**: Test editor + playback independently
6. Deploy/demo if ready

### Full Feature

1. Complete MVP (Phases 1-4)
2. Add Phase 5: User Story 2 (Build System)
3. Add Phase 6: Security
4. Add Phase 7: Polish
5. Full feature complete

---

## Notes

- [P] tasks = different files, no dependencies within phase
- [US#] label maps task to specific user story
- Existing S3 utilities in [`lib/s3.ts`](../../../S4VN-Simulation-Frontend/lib/s3.ts) should be reused
- Monaco Editor requires dynamic import for SSR compatibility
- GitHub Actions has 2000 min/month free tier limit
- Animation HTML must be self-contained (embedded X3D + JSON)
