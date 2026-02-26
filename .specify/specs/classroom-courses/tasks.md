# Tasks: Classroom/Courses Module

**Input**: Design documents from `.specify/specs/classroom-courses/`
**Prerequisites**: [planning.md](./planning.md), [specification.md](./specification.md)
**Branch**: `classroom-courses`
**Created**: 2026-02-10

**Tests**: Có bao gồm test tasks cho API, integration, RBAC matrix, idempotency, migration verification.

**Organization**: Task được nhóm theo phase và user story để có thể triển khai tăng dần theo giá trị.

---

## Phase 1: Setup

**Purpose**: Chuẩn bị schema, migration, RBAC nền, khung routes/pages cho Classroom.

### Backend and data setup

- [ ] T001 Cập nhật schema `User` thêm field `role` và enum `UserRole` trong `S4VN-Simulation-Frontend/prisma/schema.prisma`
- [ ] T002 Cập nhật schema với enums và models Classroom trong `S4VN-Simulation-Frontend/prisma/schema.prisma`
- [ ] T003 Bổ sung relations vào `User` và `Project` trong `S4VN-Simulation-Frontend/prisma/schema.prisma`
- [ ] T004 Tạo migration Prisma cho role + classroom models trong `S4VN-Simulation-Frontend/prisma/migrations/`
- [ ] T005 Viết migration data script map role mặc định và bootstrap admin trong `S4VN-Simulation-Frontend/scripts/`
- [ ] T006 Generate Prisma client sau migration trong `S4VN-Simulation-Frontend/`

### Shared API and frontend scaffolding

- [ ] T007 [P] Tạo kiểu dữ liệu Classroom API envelope trong `S4VN-Simulation-Frontend/types/classroom.ts`
- [ ] T008 [P] Tạo helper RBAC guard dùng chung trong `S4VN-Simulation-Frontend/lib/classroom/rbac.ts`
- [ ] T009 [P] Tạo helper error code map trong `S4VN-Simulation-Frontend/lib/classroom/errors.ts`
- [ ] T010 [P] Tạo khung routes admin user-role trong `S4VN-Simulation-Frontend/app/api/admin/users/`
- [ ] T011 [P] Tạo khung routes `app/api/courses/...` trong `S4VN-Simulation-Frontend/app/api/courses/`
- [ ] T012 [P] Tạo khung routes `app/api/assignments/...` trong `S4VN-Simulation-Frontend/app/api/assignments/`
- [ ] T013 [P] Tạo khung routes `app/api/submissions/...` trong `S4VN-Simulation-Frontend/app/api/submissions/`
- [ ] T014 [P] Tạo skeleton pages `app/courses/...` trong `S4VN-Simulation-Frontend/app/courses/`

**Checkpoint**: Schema, migration, RBAC helper và skeleton code sẵn sàng cho luồng Admin/Teacher/Student.

---

## Phase 2: Core - User Story 1 Teacher quản lý Course/Assignment (Priority P1)

**Goal**: Hoàn thành luồng Admin bootstrap teacher và luồng Teacher quản lý lớp, học sinh, assignment.

**Independent Test**: Admin tạo/gán teacher thành công; Teacher tạo và quản trị lớp học của mình; Teacher quản trị student enrollment đúng RBAC.

### 2.1 Admin flows backend

- [ ] T015 [US1] Implement `POST /api/admin/users/teachers` trong `S4VN-Simulation-Frontend/app/api/admin/users/teachers/route.ts`
- [ ] T016 [US1] Implement `PATCH /api/admin/users/[userId]/role` trong `S4VN-Simulation-Frontend/app/api/admin/users/[userId]/role/route.ts`
- [ ] T017 [US1] Implement service `createTeacherAccount` trong `S4VN-Simulation-Frontend/lib/classroom/admin-user-service.ts`
- [ ] T018 [US1] Implement service `assignTeacherRole` trong `S4VN-Simulation-Frontend/lib/classroom/admin-user-service.ts`

### 2.2 Teacher course and student management backend

- [ ] T019 [US1] Implement `POST /api/courses` trong `S4VN-Simulation-Frontend/app/api/courses/route.ts`
- [ ] T020 [US1] Implement `GET /api/courses` theo role trong `S4VN-Simulation-Frontend/app/api/courses/route.ts`
- [ ] T021 [US1] Implement `GET /api/courses/[courseId]` trong `S4VN-Simulation-Frontend/app/api/courses/[courseId]/route.ts`
- [ ] T022 [US1] Implement `PATCH /api/courses/[courseId]` với owner guard trong `S4VN-Simulation-Frontend/app/api/courses/[courseId]/route.ts`
- [ ] T023 [US1] Implement `DELETE /api/courses/[courseId]` soft delete trong `S4VN-Simulation-Frontend/app/api/courses/[courseId]/route.ts`
- [ ] T024 [US1] Implement `GET /api/courses/[courseId]/students/search` trong `S4VN-Simulation-Frontend/app/api/courses/[courseId]/students/search/route.ts`
- [ ] T025 [US1] Implement `POST /api/courses/[courseId]/students` trong `S4VN-Simulation-Frontend/app/api/courses/[courseId]/students/route.ts`
- [ ] T026 [US1] Implement `POST /api/courses/[courseId]/enrollments` trong `S4VN-Simulation-Frontend/app/api/courses/[courseId]/enrollments/route.ts`
- [ ] T027 [US1] Implement `DELETE /api/courses/[courseId]/enrollments/[studentId]` trong `S4VN-Simulation-Frontend/app/api/courses/[courseId]/enrollments/[studentId]/route.ts`

### 2.3 Teacher assignment management backend

- [ ] T028 [US1] Implement `POST /api/courses/[courseId]/assignments` trong `S4VN-Simulation-Frontend/app/api/courses/[courseId]/assignments/route.ts`
- [ ] T029 [US1] Implement `GET /api/courses/[courseId]/assignments` trong `S4VN-Simulation-Frontend/app/api/courses/[courseId]/assignments/route.ts`
- [ ] T030 [US1] Implement `GET /api/assignments/[assignmentId]` trong `S4VN-Simulation-Frontend/app/api/assignments/[assignmentId]/route.ts`
- [ ] T031 [US1] Implement `PATCH /api/assignments/[assignmentId]` trong `S4VN-Simulation-Frontend/app/api/assignments/[assignmentId]/route.ts`
- [ ] T032 [US1] Implement `DELETE /api/assignments/[assignmentId]` trong `S4VN-Simulation-Frontend/app/api/assignments/[assignmentId]/route.ts`

### 2.4 Frontend for Admin and Teacher flows

- [ ] T033 [US1] Tạo UI admin tạo teacher và gán teacher role tại `S4VN-Simulation-Frontend/app/settings/page.tsx` hoặc route admin tương ứng
- [ ] T034 [US1] Tạo trang teacher dashboard tại `S4VN-Simulation-Frontend/app/courses/page.tsx`
- [ ] T035 [US1] Tạo trang tạo course tại `S4VN-Simulation-Frontend/app/courses/new/page.tsx`
- [ ] T036 [US1] Tạo trang chi tiết course tại `S4VN-Simulation-Frontend/app/courses/[courseId]/page.tsx`
- [ ] T037 [US1] Tạo panel search student tại `S4VN-Simulation-Frontend/components/classroom/StudentSearchPanel.tsx`
- [ ] T038 [US1] Tạo modal create student tại `S4VN-Simulation-Frontend/components/classroom/CreateStudentModal.tsx`
- [ ] T039 [US1] Tạo bảng enrollment và action unenroll tại `S4VN-Simulation-Frontend/components/classroom/EnrollmentTable.tsx`
- [ ] T040 [US1] Tạo trang tạo assignment tại `S4VN-Simulation-Frontend/app/courses/[courseId]/assignments/new/page.tsx`

### Acceptance criteria - Admin flow

- [ ] AC-A1 Admin có thể tạo teacher mới với email chưa tồn tại
- [ ] AC-A2 Admin có thể gán role teacher cho tài khoản có sẵn
- [ ] AC-A3 User không phải admin gọi API admin nhận `403 FORBIDDEN_ROLE`

### Acceptance criteria - Teacher flow

- [ ] AC-T1 Teacher tạo/cập nhật/archive course do mình sở hữu thành công
- [ ] AC-T2 Teacher chỉ tìm kiếm và thêm user role `STUDENT`
- [ ] AC-T3 Teacher có thể tạo student mới và auto-enroll vào lớp
- [ ] AC-T4 Teacher không sở hữu course không thể sửa dữ liệu lớp đó

---

## Phase 3: Core - User Story 2 Student join và start assignment (Priority P1)

**Goal**: Student join bằng join code và clone assignment thành project cá nhân để mở editor.

**Independent Test**: Student join course và start assignment thành công, start lại không tạo clone mới.

### Backend

- [ ] T041 [US2] Implement `POST /api/courses/join` trong `S4VN-Simulation-Frontend/app/api/courses/join/route.ts`
- [ ] T042 [US2] Tạo service kiểm tra membership và quyền truy cập trong `S4VN-Simulation-Frontend/lib/classroom/access.ts`
- [ ] T043 [US2] Tạo service deep copy S3 key-prefix trong `S4VN-Simulation-Frontend/lib/classroom/deep-copy.ts`
- [ ] T044 [US2] Implement orchestration `startAssignment` trong `S4VN-Simulation-Frontend/lib/classroom/start-assignment.ts`
- [ ] T045 [US2] Implement `POST /api/assignments/[assignmentId]/start` với idempotency trong `S4VN-Simulation-Frontend/app/api/assignments/[assignmentId]/start/route.ts`

### Frontend

- [ ] T046 [US2] Thêm trang join code tại `S4VN-Simulation-Frontend/app/courses/join/page.tsx`
- [ ] T047 [US2] Thêm trang assignment detail student tại `S4VN-Simulation-Frontend/app/courses/[courseId]/assignments/[assignmentId]/page.tsx`
- [ ] T048 [US2] Tích hợp redirect sang editor `/editor/[projectId]` có query assignment context trong `S4VN-Simulation-Frontend/app/courses/[courseId]/assignments/[assignmentId]/page.tsx`

### Acceptance criteria - Student start flow

- [ ] AC-S1 Student có enrollment active start assignment thành công
- [ ] AC-S2 Start assignment lần 2 trả về project cũ với `reusedExisting=true`
- [ ] AC-S3 Student ngoài lớp nhận `403 ENROLLMENT_REQUIRED`

---

## Phase 4: Core - User Story 3 Submit và theo dõi trạng thái (Priority P2)

**Goal**: Student submit theo policy one-record-overwrite, teacher xem submissions.

**Independent Test**: Submit nhiều lần chỉ update một record theo `(assignmentId, studentId)`.

### Backend

- [ ] T049 [US3] Implement `POST /api/assignments/[assignmentId]/submit` với upsert overwrite trong `S4VN-Simulation-Frontend/app/api/assignments/[assignmentId]/submit/route.ts`
- [ ] T050 [US3] Implement `GET /api/assignments/[assignmentId]/submission/me` trong `S4VN-Simulation-Frontend/app/api/assignments/[assignmentId]/submission/me/route.ts`
- [ ] T051 [US3] Implement `GET /api/assignments/[assignmentId]/submissions` teacher-only trong `S4VN-Simulation-Frontend/app/api/assignments/[assignmentId]/submissions/route.ts`

### Frontend

- [ ] T052 [US3] Tạo component submit action trong `S4VN-Simulation-Frontend/components/classroom/AssignmentSubmitButton.tsx`
- [ ] T053 [US3] Tích hợp nút submit vào editor context tại `S4VN-Simulation-Frontend/app/editor/[projectId]/page.tsx`
- [ ] T054 [US3] Hiển thị submission status `DRAFT`, `PENDING`, `QUEUED` tại trang assignment detail

### Acceptance criteria - Submit flow

- [ ] AC-SUB1 Submit lần đầu tạo hoặc cập nhật submission với status `PENDING`
- [ ] AC-SUB2 Submit lặp lại không tạo record thứ hai
- [ ] AC-SUB3 Teacher chỉ xem submissions của assignment thuộc course mình sở hữu

---

## Phase 5: Tests và Hardening

**Purpose**: Đảm bảo đúng phân quyền, chuẩn API, idempotency, migration an toàn.

### Migration and data verification tests

- [ ] T055 [P] Viết test migration role và dữ liệu bootstrap admin trong `S4VN-Simulation-Frontend/__tests__/integration/classroom-role-migration.test.ts`
- [ ] T056 [P] Viết test rollback migration safety cho classroom schema

### API and RBAC tests

- [ ] T057 [P] Viết API tests cho Admin endpoints trong `S4VN-Simulation-Frontend/__tests__/api/classroom/admin-users.test.ts`
- [ ] T058 [P] Viết API tests cho Course/Assignment teacher ownership trong `S4VN-Simulation-Frontend/__tests__/api/classroom/courses.test.ts`
- [ ] T059 [P] Viết API tests cho student search/create/enroll/unenroll trong `S4VN-Simulation-Frontend/__tests__/api/classroom/students-enrollment.test.ts`
- [ ] T060 [P] Viết API tests cho join/start flow trong `S4VN-Simulation-Frontend/__tests__/api/classroom/start.test.ts`
- [ ] T061 [P] Viết API tests cho submit overwrite policy trong `S4VN-Simulation-Frontend/__tests__/api/classroom/submissions.test.ts`
- [ ] T062 [P] Viết RBAC matrix tests Admin Teacher Student trong `S4VN-Simulation-Frontend/__tests__/api/classroom/rbac-matrix.test.ts`

### Integration and concurrency tests

- [ ] T063 [P] Viết integration test deep copy S3 mock trong `S4VN-Simulation-Frontend/__tests__/integration/classroom-deep-copy.test.ts`
- [ ] T064 Kiểm thử race condition start assignment concurrent request
- [ ] T065 Kiểm thử race condition submit đa tab
- [ ] T066 Bổ sung logging và error mapping nhất quán cho routes Classroom

### Phase 5 acceptance criteria

- [ ] AC-H1 Tất cả endpoint chính trả đúng envelope `{ success, data, error }`
- [ ] AC-H2 Tất cả case trái role trả đúng mã lỗi RBAC theo spec
- [ ] AC-H3 Migration có thể chạy và verify dữ liệu role an toàn trên môi trường staging

---

## Phase 6: Later - Auto-grading Integration

**Purpose**: Triển khai sau MVP, không chặn go-live Classroom core.

- [ ] T067 Thiết kế queue worker cập nhật submission status `RUNNING`, `GRADED`, `FAILED`
- [ ] T068 Implement endpoint nhận callback grade từ simulation supervisor
- [ ] T069 Bổ sung `PATCH /api/submissions/[submissionId]/grade` phục vụ manual hoặc async grading
- [ ] T070 Bổ sung UI hiển thị score và feedback chi tiết

---

## Dependencies and Execution Order

### Phase Dependencies

- Phase 1 là bắt buộc trước mọi phase Core
- Phase 2 phụ thuộc hoàn tất schema role và RBAC helpers
- Phase 3 phụ thuộc enrollment flows từ Phase 2
- Phase 4 phụ thuộc flow start assignment ở Phase 3
- Phase 5 chạy sau khi endpoints core hoàn tất
- Phase 6 là độc lập hậu MVP

### Critical Constraints

- Không được phá behavior hiện có của editor routes trong `app/api/editor/...`
- Mọi route Classroom phải qua auth check trước khi truy cập DB
- Mọi route cần RBAC guard trước business logic
- Start assignment và submit phải tôn trọng unique constraints để tránh trùng bản ghi

---

## Delivery Strategy

### MVP Cut

MVP hoàn thành khi hoàn tất:
- Phase 1
- Phase 2
- Phase 3
- Phase 4
- Bộ test tối thiểu: T057, T058, T059, T060, T061, T062
- Acceptance criteria bắt buộc: AC-A1 đến AC-A3, AC-T1 đến AC-T4, AC-S1 đến AC-S3, AC-SUB1 đến AC-SUB3

### Post-MVP

- Triển khai dần Phase 5 hardening mở rộng và toàn bộ Phase 6 auto-grading
