# VEO3 → Google Labs Replacement Plan

> **Goal**: Replace all 'veo3' platform identifiers with 'google_labs' across codebase

---

## Tasks

### Phase 1: Backend Refactor

- [ ] **1.1 Update collector_factory.py** → Change `"veo3"` to `"google_labs"` in Platform type, create_collector(), create()
  - File: `pyBot_modules/shared/browsers/collector_factory.py`
  - Lines: 16, 29, 34, 44, 47, 63, 70, 75
  - Verify: `CollectorFactory.create_collector("google_labs")` returns SharedSessionCollector

- [ ] **1.2 Update session_manager_factory.py** → Change `"veo3"` to `"google_labs"` in create_session_manager()
  - File: `pyBot_modules/shared/browsers/session_manager_factory.py`  
  - Lines: 36, 47, 50, 68
  - Verify: `SessionManagerFactory.create_session_manager("google_labs", db_path)` works

- [ ] **1.3 Update VEO3VideoProvider registry** → Change `@register("veo3")` to `@register("google_labs")`
  - File: `pyBot_modules/shared/video/providers/veo3.py`
  - Lines: 27, 74-76
  - Verify: `VideoProviderRegistry.get("google_labs")` returns VEO3VideoProvider

- [ ] **1.4 Update ProfileRepository default** → Change `platform: str = "veo3"` to `"google_labs"`
  - File: `pyBot_modules/shared/db/repositories/profile_repository.py`
  - Lines: 132, 148
  - Verify: Default platform in method signature is "google_labs"

- [ ] **1.5 Update Account model example** → Change example platform from "veo3" to "google_labs"
  - File: `pyBot_modules/shared/db/models/account.py`
  - Line: 75
  - Verify: JSON schema example shows "google_labs"

### Phase 2: Frontend Refactor

- [ ] **2.1 Update AccountPlatform type** → Replace "veo3" with "google_labs"
  - File: `pyBot_web/lib/api/accountManagement.ts`
  - Line: 3
  - Verify: TypeScript type includes "google_labs"

- [ ] **2.2 Update scenarios page dropdowns** → Change value="veo3" to value="google_labs"
  - File: `pyBot_web/app/[locale]/scenarios/page.tsx`
  - Lines: 409, 415, 425, 431
  - Verify: Video provider dropdown shows "Google Labs" option

- [ ] **2.3 Update account form dropdown** → Change value="veo3" to value="google_labs"
  - File: `pyBot_web/components/accounts/account-form.tsx`
  - Line: 353
  - Verify: Platform dropdown shows "google_labs" option

### Phase 3: DB Migration (if needed)

- [ ] **3.1 Check existing veo3 records** → Run SQL to verify no 'veo3' platform exists
  - Query: `SELECT COUNT(*) FROM account_management WHERE platform = 'veo3'`
  - Expected: 0 (already migrated by previous migration)
  
- [ ] **3.2 Update DB migration to REMOVE 'veo3' from CHECK** (optional - if not already done)
  - File: `pyBot_modules/shared/db/migrations.py`
  - This should already be handled by schema.py which excludes 'veo3'

### Phase 4: Test & Verification

- [ ] **4.1 Run existing tests** → Ensure no regressions
  - Command: `pytest tests/unit tests/integration -v`
  - Verify: All tests pass (update fixtures if needed)

- [ ] **4.2 Manual verification** → Create account, generate video
  - Create account with platform="google_labs"
  - Verify video generation works with google_labs account

---

## Files Changed Summary

| Layer | File | Changes |
|-------|------|---------|
| Backend | `collector_factory.py` | Replace "veo3" → "google_labs" in 5+ locations |
| Backend | `session_manager_factory.py` | Replace "veo3" → "google_labs" in 4 locations |
| Backend | `providers/veo3.py` | Update decorator and platform_name property |
| Backend | `profile_repository.py` | Update default parameter |
| Backend | `models/account.py` | Update example in docstring |
| Frontend | `accountManagement.ts` | Update AccountPlatform type |
| Frontend | `scenarios/page.tsx` | Update 4 SelectItem components |
| Frontend | `account-form.tsx` | Update 1 SelectItem component |

---

## Done When

- [ ] All `"veo3"` strings replaced with `"google_labs"` in factories and registries
- [ ] Frontend dropdowns show "Google Labs" instead of "VEO3"
- [ ] No 'veo3' platform values exist in database
- [ ] All tests pass
- [ ] Video generation works with google_labs platform accounts

---

## Notes

1. **VEO3 module name unchanged**: The `pyBot_modules/veo3/` directory stays as-is (internal implementation)
2. **PLATFORM_GOOGLE_LABS constant**: Already exists in `shared/consts.py`, use it where appropriate
3. **Backward compatibility**: Not needed - this is a complete replacement
4. **Display labels**: Frontend should show "Google Labs" (not "VEO3 (Google)")
