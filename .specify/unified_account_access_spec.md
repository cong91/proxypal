# Feature Specification: Unified Account Access

**Feature Branch**: `unified-account-access`  
**Created**: 2026-02-06  
**Status**: Ready for Review  
**Input**: Feature Brief - Enable `whisks` to reuse `veo3` accounts (Google Labs)

## Overview

### Problem Statement

Hiện tại, `veo3` và `whisks` modules sử dụng platform identifier riêng biệt:
- `veo3`: platform = `"veo3"`
- `whisks`: platform = `"whisks"`

Điều này ngăn cản việc chia sẻ accounts giữa các services cùng sử dụng Google Labs (Google AI Labs platform).

### Solution: Unified Platform Identifier

Chuẩn hóa platform identifier thành `google_labs` cho cả `veo3` và `whisks`, cho phép:
- **Account Reuse**: Một Google account có thể sử dụng cho cả video generation (Veo3) và image generation (Whisks)
- **Simplified Management**: Một record DB cho mỗi Google account thay vì duplicate
- **Cost Efficiency**: Giảm số lượng accounts cần quản lý

## User Scenarios & Testing

### User Story 1 - Account Reuse (Priority: P1)

Operator muốn sử dụng cùng một Google account cho cả Veo3 video generation và Whisks image generation.

**Why this priority**: Core value proposition - đây là mục đích chính của feature.

**Independent Test**: Có thể test bằng cách tạo account với platform=`google_labs`, sau đó verify cả VEO3 SessionManager và Whisks SessionManager đều accept account này.

**Acceptance Scenarios**:

1. **Given** account với platform=`google_labs` trong DB, **When** VEO3 SessionManager.load_session() được gọi, **Then** session được load thành công không throw error.

2. **Given** account với platform=`google_labs` trong DB, **When** Whisks SessionManager.load_session() được gọi, **Then** session được load thành công không throw error.

3. **Given** account với platform=`google_labs` trong DB, **When** Whisks AccountSelector.refresh() được gọi, **Then** account xuất hiện trong danh sách available accounts.

---

### User Story 2 - Backward Compatibility (Priority: P1)

Existing veo3 accounts vẫn hoạt động sau migration.

**Why this priority**: Production stability - không thể break existing functionality.

**Independent Test**: Run VEO3 video generation workflow với existing accounts sau migration.

**Acceptance Scenarios**:

1. **Given** DB với accounts có platform=`veo3`, **When** migration script chạy, **Then** platform được update thành `google_labs`.

2. **Given** accounts đã migrated sang `google_labs`, **When** VEO3 video generation workflow chạy, **Then** workflow hoàn thành successful như trước.

---

### User Story 2b - Dual-Platform Fallback Strategy (Priority: P1)

System hoạt động gracefully với legacy DBs chứa `platform='whisks'` hoặc `platform='veo3'` mà không yêu cầu manual migration trước startup.

**Why this priority**: Zero-downtime transition - hệ thống KHÔNG block usage khi chưa migrate.

**Independent Test**: Deploy code mới với DB cũ (chưa migrate) và verify Whisks hoạt động bình thường.

**Acceptance Scenarios**:

1. **Given** DB với accounts có platform=`whisks` (chưa migrate), **When** Whisks AccountSelector.refresh() được gọi, **Then** accounts được tìm thấy và sử dụng được.

2. **Given** DB với accounts có platform=`google_labs` (đã migrate), **When** Whisks AccountSelector.refresh() được gọi, **Then** accounts với google_labs được ưu tiên sử dụng.

3. **Given** DB với MIX accounts (một số `whisks`, một số `google_labs`), **When** Whisks AccountSelector.refresh() được gọi, **Then** accounts `google_labs` được trả về, fallback sang `whisks` nếu empty.

4. **Given** DB với accounts có platform=`veo3` (legacy VEO3), **When** VEO3 SessionManager.load_session() được gọi, **Then** session được load thành công (backward compat).

---

### User Story 3 - UI Account Selection (Priority: P2)

Whisks UI hiển thị tất cả Google Labs accounts để user chọn.

**Why this priority**: User experience - cho phép selection trong UI.

**Independent Test**: Launch Whisks UI, verify account dropdown hiển thị accounts với platform=`google_labs`.

**Acceptance Scenarios**:

1. **Given** accounts với platform=`google_labs` trong DB, **When** user mở Whisks UI account dropdown, **Then** tất cả active google_labs accounts được hiển thị.

---

### Edge Cases

- **No active accounts**: System hiển thị message "(chưa có account google_labs nào ở trạng thái active)"
- **Mixed platforms during migration**: Migration script chỉ update `veo3` → `google_labs`, không ảnh hưởng other platforms
- **Rollback scenario**: Nếu cần rollback, có thể update ngược `google_labs` → `veo3`

## Requirements

### Functional Requirements

- **FR-001**: System MUST accept platform identifier `google_labs` cho cả VEO3 và Whisks services
- **FR-002**: VEO3 SessionManager MUST load sessions với platform = `google_labs`
- **FR-003**: Whisks SessionManager MUST load sessions với platform = `google_labs`
- **FR-004**: Whisks AccountSelector MUST default to platform = `google_labs`
- **FR-005**: Migration script MUST update all existing `veo3` accounts to `google_labs`
- **FR-006**: DB schema constraint MUST include `google_labs` as valid platform value
- **FR-007**: Whisks AccountSelector MUST implement dual-platform fallback: Try `google_labs` first → Fallback to `whisks` if empty
- **FR-008**: VEO3 SessionManager MUST accept legacy `veo3` platform without requiring migration
- **FR-009**: System SHOULD perform runtime migration check và log warning nếu legacy platforms detected
- **FR-010**: Documentation/UI SHOULD advise migration nhưng MUST NOT block usage với legacy DB

### Non-Functional Requirements

- **NFR-001**: Migration MUST be atomic (all-or-nothing transaction)
- **NFR-002**: Backward compatibility MUST be maintained for 1 release cycle

### Key Entities

- **Account**: `account_management` table với field `platform` sẽ accept giá trị `google_labs`
- **Platform Constant**: New constant `PLATFORM_GOOGLE_LABS = "google_labs"` trong shared constants

## Technical Notes

### Current State Analysis

| Component | Current Behavior | File Location |
|-----------|------------------|---------------|
| VEO3 SessionManager | Validates `platform in (None, "veo3")` | `pyBot_modules/veo3/services/core/session.py:434-438` |
| Whisks SessionManager | Validates `platform in (None, "whisks")` | `pyBot_modules/whisks/services/core/session.py:271-274` |
| Whisks AccountSelector | Default `platform="whisks"` | `pyBot_modules/whisks/services/account_selector.py:24` |
| DB Schema | CHECK constraint không có `whisks` hay `google_labs` | `pyBot_modules/shared/db/schema.py:139` |
| SessionManagerFactory | Routes to VEO3/Sora/Grok managers | `pyBot_modules/shared/browsers/session_manager_factory.py:47-72` |

### Schema Constraint (Current)

```sql
platform TEXT NOT NULL CHECK(platform IN ('veo3', 'sora', 'grok', 'facebook', 'tiktok', 'youtube', 'instagram'))
```

### Schema Constraint (After Migration)

```sql
platform TEXT NOT NULL CHECK(platform IN ('google_labs', 'sora', 'grok', 'facebook', 'tiktok', 'youtube', 'instagram'))
```

> **Note**: `veo3` bị removed vì tất cả veo3 accounts được migrate sang `google_labs`.

## Out of Scope

- Grok integration với google_labs (Grok sử dụng X/Twitter accounts)
- Sora integration với google_labs (Sora sử dụng OpenAI accounts)
- UI để tạo google_labs accounts (đã có sẵn trong account management)

## Risks & Mitigations

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| VEO3 breaks sau migration | High | Medium | Test thorough trước khi merge; có rollback script |
| Whisks không recognize google_labs | Medium | Low | Update platform validation trước khi migrate data |
| DB constraint fails | High | Low | Migration script creates new constraint trước khi update data |

## Dependencies

- Không có external dependencies mới
- Internal: `pyBot_modules/shared/db/migrations.py` cho migration pattern
