# Analysis: Online Editor & Animation Build System

**Generated**: 2026-02-06
**Spec**: [specification.md](./specification.md) | [planning.md](./planning.md) | [tasks.md](./tasks.md)

---

## Executive Summary

Phân tích trạng thái hiện tại của codebase so với các tasks đã đề xuất cho feature "Online Editor & Animation Build System".

### Overall Status: 🟡 New Feature - Minimal Existing Code

| Category | Existing | New/Modified |
|----------|----------|--------------|
| Dependencies | 0 | 2 packages |
| Database Models | 0 | 2 models |
| API Routes | 0 | 8 routes |
| Components | 0 | 6 components |
| Lib/Utils | 0 | 4 files |
| Hooks | 0 | 2 hooks |
| Pages | 0 | 2 pages |
| Tests | 0 | ~8 test files |

---

## Detailed Analysis by Phase

### Phase 1: Setup ✅ Ready to Start

| Task | Status | Notes |
|------|--------|-------|
| T001: Install `@monaco-editor/react` | 🆕 Not installed | `package.json` không có dependency này |
| T002: Install `@octokit/rest` | 🆕 Not installed | Cần cho GitHub API integration |
| T003: Add S3 folder `EDITOR` | 🆕 | [`lib/s3.ts:17-22`](../../../S4VN-Simulation-Frontend/lib/s3.ts:17) đã có ANIMATION, SCENE, PROTO, ASSETS |
| T004: Create `types/editor.ts` | 🆕 | Folder `types/` đã tồn tại, chỉ thêm file mới |
| T005: Create `types/build.ts` | 🆕 | Có thể tái sử dụng `CompetitionSubmissionStatus` pattern |

**Prerequisites Met**: ✅

---

### Phase 2: Database Schema ✅ Ready to Start

| Task | Status | Notes |
|------|--------|-------|
| T006: Add `EditorSession` model | 🆕 | Schema hiện có `User`, `Project` - có thể tạo relation |
| T007: Add `BuildJob` model | 🆕 | Có thể tham khảo `CompetitionSubmission` status pattern |
| T008: Run Prisma migration | 🆕 | Workflow đã được thiết lập |
| T009: Generate Prisma client | 🆕 | Workflow đã được thiết lập |

**Existing Resources**:
- [`prisma/schema.prisma`](../../../S4VN-Simulation-Frontend/prisma/schema.prisma) - Schema chuẩn, có enum pattern
- `CompetitionSubmissionStatus` enum (line 91-98) - mẫu tốt cho `BuildStatus`

**Prerequisites Met**: ✅

---

### Phase 3: User Story 1 - Editor (P1 MVP) 🟡 Mostly New

#### 3.1 Monaco Editor Component

| Task | Status | Notes |
|------|--------|-------|
| T010: `MonacoEditor.tsx` | 🆕 | Không có component editor nào tồn tại |
| T011: `EditorToolbar.tsx` | 🆕 | |
| T012: `FileTree.tsx` | 🆕 | |

**Existing Resources**:
- [`components/IDEEmbed.tsx`](../../../S4VN-Simulation-Frontend/components/IDEEmbed.tsx) - iframe embed, KHÔNG phải Monaco
- Pattern component có thể tham khảo từ các component hiện có

#### 3.2 Editor API Routes

| Task | Status | Notes |
|------|--------|-------|
| T013: `GET /api/editor/load` | 🆕 | Folder `app/api/editor/` chưa tồn tại |
| T014: `PUT /api/editor/save` | 🆕 | |
| T015: `GET /api/editor/tree` | 🆕 | |

**Existing Resources**:
- [`app/api/s3/presigned-url/route.ts`](../../../S4VN-Simulation-Frontend/app/api/s3/presigned-url/route.ts) - pattern S3 API
- [`app/api/animations/[id]/route.ts`](../../../S4VN-Simulation-Frontend/app/api/animations/[id]/route.ts) - pattern dynamic route

#### 3.3 S3 Editor Utilities

| Task | Status | Notes |
|------|--------|-------|
| T016: `lib/editor/s3-editor.ts` | 🆕 | Folder `lib/editor/` chưa tồn tại |

**Existing Resources - CÓ THỂ TÁI SỬ DỤNG**:
- [`lib/s3.ts`](../../../S4VN-Simulation-Frontend/lib/s3.ts) ✅ Có đầy đủ:
  - `uploadToS3()` (line 50-75)
  - `getPresignedUploadUrl()` (line 80-102)
  - `getPresignedDownloadUrl()` (line 107-134)
  - `deleteFromS3()` (line 139-150)
  - `S3_FOLDERS` constant (line 17-22) - cần thêm EDITOR

**Action**: Tái sử dụng lib/s3.ts, chỉ cần wrapper functions cho editor-specific operations

#### 3.4 Editor State Management

| Task | Status | Notes |
|------|--------|-------|
| T017: `useEditorSession` hook | 🆕 | Folder `hooks/` chưa tồn tại |

**Existing Resources**:
- [`lib/webots/use-simulation.ts`](../../../S4VN-Simulation-Frontend/lib/webots/use-simulation.ts) - hook pattern reference

#### 3.5 Editor Pages

| Task | Status | Notes |
|------|--------|-------|
| T018: `/editor` page | 🆕 | Folder `app/editor/` chưa tồn tại |
| T019: `/editor/[projectId]` page | 🆕 | |

---

### Phase 4: User Story 3 - Playback (P1 MVP) ✅ Partially Exists

| Task | Status | Notes |
|------|--------|-------|
| T023: `AnimationPlayer.tsx` | 🟡 Partial | Animation page đã có playback logic inline |
| T024: `GET /api/animation/[id]` | ✅ Exists | [`app/api/animations/[id]/route.ts`](../../../S4VN-Simulation-Frontend/app/api/animations/[id]/route.ts) |
| T025: Update animation page | 🟡 Refactor | Extract component từ page hiện có |

**Existing Resources**:
- [`app/animation/[id]/page.tsx`](../../../S4VN-Simulation-Frontend/app/animation/[id]/page.tsx) (477 lines) - có đầy đủ logic:
  - WebotsView.js loading
  - Animation fetching
  - Scene loading
  - Error handling
- [`app/api/animations/[id]/route.ts`](../../../S4VN-Simulation-Frontend/app/api/animations/[id]/route.ts) - animation API

**Recommendation**: Refactor animation page thành reusable `AnimationPlayer` component

---

### Phase 5: User Story 2 - Build System (P2) 🆕 All New

| Task | Status | Notes |
|------|--------|-------|
| T028-T029: GitHub Integration | 🆕 | Cần `@octokit/rest` |
| T030-T033: Build API Routes | 🆕 | |
| T034: `useBuildStatus` hook | 🆕 | |
| T035-T036: Build UI Components | 🆕 | |
| T037: GitHub Actions workflow | 🆕 | `.github/workflows/` folder cần được tạo |

---

## Gap Analysis

### What Exists (Can Reuse)

1. **S3 Utilities** ([`lib/s3.ts`](../../../S4VN-Simulation-Frontend/lib/s3.ts)):
   - Full CRUD operations ✅
   - Presigned URL generation ✅
   - Folder-based organization ✅

2. **Database Pattern** ([`prisma/schema.prisma`](../../../S4VN-Simulation-Frontend/prisma/schema.prisma)):
   - User model with relations ✅
   - Project model ✅
   - Status enum pattern (`CompetitionSubmissionStatus`) ✅

3. **Animation Infrastructure**:
   - Animation model in DB ✅
   - Animation API routes ✅
   - WebotsView integration ✅

4. **Auth System**:
   - NextAuth.js setup ✅
   - User authentication ✅

### What's Missing (Must Create)

| Priority | Component | Effort |
|----------|-----------|--------|
| HIGH | Monaco Editor component | Medium |
| HIGH | Editor API routes (load/save/tree) | Low |
| HIGH | Editor page layout | Medium |
| MEDIUM | BuildJob DB model | Low |
| MEDIUM | GitHub Actions workflow | Medium |
| MEDIUM | Build trigger API | Low |
| LOW | GitHub commit integration | Medium |

---

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Monaco bundle size (~2MB) | Page load time | Dynamic import, lazy loading |
| S3 CORS for editor | Save failures | Verify CORS policy includes editor domain |
| GitHub Actions rate limit | Build failures | Implement queue, use self-hosted runner fallback |
| Concurrent edit conflicts | Data loss | Implement file locking or last-write-wins |

---

## Recommendations

### Phase A (MVP Editor) - Start Here

1. **T001-T005**: Setup first - install dependencies, add types
2. **T006-T009**: Database models next - foundation for all features
3. **T010-T019**: Editor components and pages
4. **T023-T025**: Refactor animation playback

**Estimated Tasks for Phase A**: 22 tasks

### Optimization Opportunities

1. **Reuse S3 lib** - Don't duplicate S3 logic, wrap existing functions
2. **Extract AnimationPlayer** - Refactor from existing page (T023-T025)
3. **Copy status enum pattern** - Use `CompetitionSubmissionStatus` as template for `BuildStatus`

---

## Task Prioritization for Phase A (MVP)

### Sprint 1: Foundation
```
T001-T005 (Setup) → T006-T009 (Database)
```

### Sprint 2: Core Editor
```
T013-T016 (API + Utils) → T010-T012 (Components) → T017 (Hook)
```

### Sprint 3: Pages & Playback
```
T018-T019 (Editor Pages) → T023-T025 (Animation Refactor)
```

### Sprint 4: Tests & Polish
```
T020-T022, T026-T027 (Tests)
```

---

## Conclusion

Feature "Online Editor & Animation Build" requires **mostly new code** với một số **reusable foundations**:

- **S3 infrastructure**: ✅ Sẵn sàng sử dụng
- **Database patterns**: ✅ Có thể copy/adapt
- **Animation playback**: 🟡 Cần refactor thành component
- **Editor**: 🆕 Hoàn toàn mới
- **Build system**: 🆕 Hoàn toàn mới (Phase B)

**Recommended Approach**: Start với Phase A (MVP Editor), tập trung vào **51 tasks** được liệt kê trong tasks.md, với **22 tasks** thuộc Phase A cần ưu tiên.
