# Feature Specification: Database Constraint Update for google_labs Platform

**Feature Branch**: `db-constraint-update-google-labs`  
**Created**: 2026-02-06  
**Status**: Draft  
**Input**: Feature Brief from Ask mode - "Add 'google_labs' to account_management.platform CHECK constraint"

## Context

The unified `account_management` table currently supports platforms: `'google_labs'`, `'sora'`, `'grok'`, `'facebook'`, `'tiktok'`, `'youtube'`, `'instagram'`. However, the requirement is to ensure `'google_labs'` is properly recognized as a valid platform value in existing databases that may have older constraints.

**Current Schema** (from [`schema.py:140`](pyBot_modules/shared/db/schema.py:140)):
```sql
platform TEXT NOT NULL CHECK(platform IN ('google_labs', 'sora', 'grok', 'facebook', 'tiktok', 'youtube', 'instagram'))
```

## User Scenarios & Testing

### User Story 1 - Migrate Existing Database (Priority: P1)

Developer runs migration to ensure existing databases support `google_labs` platform value.

**Why this priority**: Critical for backward compatibility - existing databases may have older CHECK constraints that don't include `google_labs`.

**Independent Test**: Run migration script on existing database and verify INSERT with `platform='google_labs'` succeeds.

**Acceptance Scenarios**:

1. **Given** an existing database with old platform constraint, **When** migration runs, **Then** new CHECK constraint includes `'google_labs'`
2. **Given** a fresh database, **When** migration runs, **Then** migration is skipped (no-op) since schema.py already includes `'google_labs'`
3. **Given** migration already applied, **When** migration runs again, **Then** it is idempotent and does not fail

---

### User Story 2 - Application Startup Safety (Priority: P2)

Application startup succeeds without DB lock issues.

**Why this priority**: Production safety - app should handle migration gracefully.

**Independent Test**: Start application, verify no database lock errors in logs.

**Acceptance Scenarios**:

1. **Given** app is running, **When** migration attempts to run, **Then** migration should use retry logic or require app stop
2. **Given** app is stopped, **When** migration runs, **Then** migration completes successfully

---

### Edge Cases

- What happens when database file is locked by another process? → Retry with exponential backoff or fail gracefully with clear error message
- What happens when table has foreign key references? → Preserve all FK constraints during recreate
- What happens when table has indexes? → Recreate all indexes after table rebuild

## Requirements

### Functional Requirements

- **FR-001**: Migration MUST add `'google_labs'` to platform CHECK constraint using "Recreate Table" strategy
- **FR-002**: Migration MUST be idempotent - safe to run multiple times
- **FR-003**: Migration MUST preserve all existing data during table recreation
- **FR-004**: Migration MUST recreate all indexes after table rebuild
- **FR-005**: Migration MUST be tracked in `schema_migrations` table
- **FR-006**: Migration MUST log all operations for debugging

### Non-Functional Requirements

- **NFR-001**: Migration should complete within reasonable time for tables with < 10K rows
- **NFR-002**: Migration should use transaction to ensure atomicity (rollback on error)

### Key Entities

- **account_management**: Unified table for all platform accounts (veo3, sora, grok, sns)
- **schema_migrations**: Tracking table for migration history

## Technical Notes

### Why "Recreate Table" Strategy?

SQLite does NOT support `ALTER TABLE ... MODIFY CONSTRAINT`. The only way to change CHECK constraints is to:
1. Create new table with updated constraint
2. Copy all data from old table
3. Drop old table
4. Rename new table

### Reference Implementations

- [`migrate_sns_accounts_add_instagram`](pyBot_modules/shared/db/migrations.py:233) - Pattern for manual table recreation with explicit column list
- [`migrate_account_management_remove_profile_type_and_refresh_schema`](pyBot_modules/shared/db/migrations.py:3198) - Pattern for rebuild using imported schema functions

### Risk: Database Lock

If the application is running and holding a connection, the migration may fail with `SQLITE_BUSY` error.

**Mitigation Options**:
1. Ensure app is stopped before running migration
2. Add retry logic with exponential backoff
3. Use WAL mode for better concurrency (already enabled in project)
