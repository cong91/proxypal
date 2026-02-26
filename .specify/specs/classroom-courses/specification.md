# Feature Specification: Classroom/Courses Module

**Feature Branch**: `classroom-courses`  
**Created**: 2026-02-10  
**Status**: Draft  
**Input**: User description: Create comprehensive technical plan and Spec Kit artifacts for Classroom/Courses based on Feature Brief

---

## 1. Executive Summary

Classroom/Courses là module tổ chức lớp học cho S4VN-Webots, cho phép:
- Admin quản lý tài khoản và quyền Teacher
- Teacher tạo lớp học, quản lý học sinh, tạo assignment từ template project
- Student tham gia lớp, làm bài trong editor, submit theo luồng assignment

Phạm vi MVP tập trung vào:
- Quản lý quyền và vòng đời người dùng theo vai trò Admin/Teacher/Student
- Quản lý Course + Enrollment + Assignment + AssignmentSubmission
- Luồng Teacher thêm học sinh có sẵn hoặc tạo học sinh mới rồi ghi danh
- Luồng Student join/start/submit với policy một submission record duy nhất cho mỗi assignment
- Trạng thái submission ở mức `DRAFT`, `PENDING`, `QUEUED`

Ngoài phạm vi MVP:
- Chấm điểm tự động thật qua simulation supervisor
- Lịch sử nhiều lần nộp cho cùng assignment
- Cơ chế role hierarchy nâng cao ngoài 3 vai trò cơ bản

---

## 2. User Scenarios and Testing

### User Story 1 - Admin và Teacher thiết lập lớp học (Priority: P1)

Admin tạo tài khoản Teacher mới hoặc gán quyền Teacher cho tài khoản có sẵn. Teacher sau đó tạo lớp, tìm kiếm học sinh, thêm học sinh có sẵn hoặc tạo học sinh mới rồi ghi danh vào lớp.

**Why this priority**: Đây là luồng bootstrap vận hành hệ thống classroom; nếu không có role + lớp + danh sách học sinh thì các luồng assignment không thể chạy.

**Independent Test**:
1. Admin tạo tài khoản teacher mới thành công
2. Admin gán quyền teacher cho user hiện có thành công
3. Teacher tạo course thành công
4. Teacher thêm học sinh có sẵn vào course thành công
5. Teacher tạo học sinh mới và ghi danh ngay vào course thành công

**Acceptance Scenarios**:
1. **Given** admin đã đăng nhập, **When** gọi API tạo teacher mới với email hợp lệ chưa tồn tại, **Then** hệ thống tạo user mới role `TEACHER`
2. **Given** admin đã đăng nhập, **When** gọi API gán role teacher cho user có sẵn, **Then** user được cập nhật role và có thể tạo course
3. **Given** teacher đã đăng nhập, **When** tạo course với title hợp lệ, **Then** course được tạo với `teacherId` là teacher hiện tại và `joinCode` unique
4. **Given** teacher là owner course, **When** tìm kiếm học sinh theo email hoặc tên hiển thị, **Then** hệ thống trả danh sách user role `STUDENT` phù hợp điều kiện tìm kiếm
5. **Given** teacher là owner course, **When** ghi danh student vào course, **Then** tạo enrollment active duy nhất theo `(courseId, studentId)`
6. **Given** teacher là owner course, **When** tạo student mới trực tiếp từ màn hình lớp học, **Then** hệ thống tạo user role `STUDENT` và enrollment active trong cùng luồng nghiệp vụ

---

### User Story 2 - Student join và bắt đầu làm assignment (Priority: P1)

Student join course bằng join code hoặc được teacher ghi danh trước, sau đó bấm Start Assignment để clone template project thành project cá nhân và mở editor.

**Why this priority**: Đây là hành trình học tập chính mang lại giá trị trực tiếp cho người học.

**Independent Test**:
1. Student join course bằng join code thành công
2. Start assignment tạo hoặc trả về project cá nhân duy nhất
3. Student được điều hướng vào editor với `projectId` của bản clone

**Acceptance Scenarios**:
1. **Given** student có join code hợp lệ, **When** gửi request join, **Then** hệ thống tạo enrollment active nếu chưa có
2. **Given** student chưa có submission cho assignment, **When** bấm start, **Then** hệ thống deep copy dữ liệu template sang project mới và tạo submission `DRAFT`
3. **Given** student đã có submission cho assignment, **When** bấm start lại, **Then** hệ thống trả về project hiện có thay vì tạo clone mới

---

### User Story 3 - Student submit và teacher theo dõi trạng thái (Priority: P2)

Student submit bài từ editor, hệ thống cập nhật bản submission duy nhất theo policy overwrite; teacher theo dõi danh sách submissions để quản lý tiến độ.

**Why this priority**: Hoàn thiện vòng đời assignment ở mức MVP, tạo nền cho auto-grading phase sau.

**Independent Test**:
1. Student submit lần đầu đặt trạng thái `PENDING`
2. Submit lại cập nhật cùng record submission, không tạo bản ghi mới
3. Teacher xem được danh sách submissions trong assignment của lớp mình

**Acceptance Scenarios**:
1. **Given** student thuộc enrollment active của course, **When** submit assignment, **Then** hệ thống upsert submission duy nhất theo cặp assignment-student
2. **Given** submission đã tồn tại, **When** student submit lại, **Then** hệ thống overwrite `status`, `score`, `feedback`, `submittedAt`
3. **Given** teacher mở danh sách submissions của assignment, **When** tải trang, **Then** hệ thống trả về danh sách gồm student, project, status, submittedAt

---

## 3. Edge Cases

- Admin cố gán role `TEACHER` cho tài khoản đã bị vô hiệu hóa
- Admin tạo teacher với email đã tồn tại
- Teacher không phải owner course nhưng cố cập nhật course hoặc enrollment
- Teacher thêm một student đã enrollment active vào cùng course
- Teacher ghi danh user role không phải `STUDENT`
- Teacher tạo student mới với email không hợp lệ hoặc email trùng
- Student dùng join code của course đã archive
- Student submit assignment không thuộc course mà mình đang active enrollment
- Deep copy S3 bị lỗi một phần gây clone không đầy đủ
- Race condition khi student click start nhiều lần liên tiếp
- Race condition khi submit đồng thời từ nhiều tab editor

---

## 4. Requirements

### 4.1 Functional Requirements

- **FR-001**: Hệ thống MUST hỗ trợ role người dùng tối thiểu gồm `ADMIN`, `TEACHER`, `STUDENT`
- **FR-002**: Hệ thống MUST cho phép admin tạo tài khoản teacher mới
- **FR-003**: Hệ thống MUST cho phép admin gán quyền teacher cho tài khoản có sẵn
- **FR-004**: Hệ thống MUST hỗ trợ model `Course` có `teacherId`, `joinCode`, trạng thái archive và metadata thời gian
- **FR-005**: Hệ thống MUST hỗ trợ model `Enrollment` chỉ dành cho student, unique theo `(courseId, studentId)`
- **FR-006**: Hệ thống MUST hỗ trợ model `Assignment` liên kết `Course` và `templateProjectId`
- **FR-007**: Hệ thống MUST hỗ trợ model `AssignmentSubmission` unique theo `(assignmentId, studentId)` để đảm bảo một submission record duy nhất
- **FR-008**: Hệ thống MUST cung cấp API CRUD cho course và assignment với phân quyền teacher phù hợp
- **FR-009**: Hệ thống MUST cung cấp API tìm kiếm học sinh có sẵn để teacher thêm vào lớp
- **FR-010**: Hệ thống MUST cung cấp API tạo học sinh mới từ ngữ cảnh lớp học
- **FR-011**: Hệ thống MUST cung cấp API ghi danh và huỷ ghi danh học sinh theo course
- **FR-012**: Hệ thống MUST cung cấp API join course bằng `joinCode` cho student
- **FR-013**: Hệ thống MUST cung cấp API start assignment thực hiện deep copy template project sang student project
- **FR-014**: Hệ thống MUST tạo `Project` và `EditorSession` khi start assignment lần đầu thành công
- **FR-015**: Hệ thống MUST cung cấp API submit assignment theo cơ chế upsert overwrite
- **FR-016**: Hệ thống MUST đặt submission status trong MVP theo tập trạng thái tối thiểu gồm `DRAFT`, `PENDING`, `QUEUED`
- **FR-017**: Hệ thống MUST cho teacher xem danh sách submissions của assignment trong course do mình quản lý
- **FR-018**: Hệ thống MUST tích hợp điều hướng từ Classroom sang editor hiện có bằng route `/editor/[projectId]`
- **FR-019**: Hệ thống MUST hiển thị nút submit trong editor khi ngữ cảnh assignment hiện diện
- **FR-020**: Hệ thống MUST đảm bảo idempotency cho start assignment để tránh tạo nhiều clone cho cùng student-assignment
- **FR-021**: Hệ thống MUST có chiến lược rollback/cleanup khi deep copy S3 thất bại giữa chừng

### 4.2 Non-Functional Requirements

- **NFR-001**: API Classroom tuân theo response envelope nhất quán: `{ success: boolean, data?: object, error?: { code: string, message: string, details?: object } }`
- **NFR-002**: Thiết kế schema và API không phá vỡ các flow hiện có của editor và projects
- **NFR-003**: Mọi endpoint Classroom phải xác thực session trước khi xử lý nghiệp vụ
- **NFR-004**: MVP không yêu cầu gọi simulation supervisor để chấm điểm thật
- **NFR-005**: Tất cả hành động thay đổi role hoặc enrollment phải có audit log cơ bản

### 4.3 RBAC Policy and Access Denial Rules

#### Role permissions

- **ADMIN**:
  - Tạo tài khoản teacher mới
  - Gán role teacher cho user có sẵn
  - Xem danh sách user để quản trị
- **TEACHER**:
  - Tạo/cập nhật/đóng course do mình sở hữu
  - Tạo/cập nhật/xóa assignment trong course của mình
  - Tìm kiếm học sinh, tạo học sinh mới, ghi danh/huỷ ghi danh học sinh trong course của mình
  - Xem submissions của assignment trong course của mình
- **STUDENT**:
  - Join course bằng join code
  - Start assignment nếu có enrollment active
  - Submit assignment của course mình tham gia
  - Xem submission cá nhân

#### Preconditions per action

- Role phải hợp lệ theo endpoint
- Course phải tồn tại và không archive cho các luồng ghi danh/join/start
- Teacher phải là owner course để thao tác enrollment/assignment
- Student phải có enrollment active để start/submit assignment

#### Access denial cases

- Thiếu session: `401 UNAUTHORIZED`
- Sai role: `403 FORBIDDEN_ROLE`
- Không phải owner course: `403 FORBIDDEN_OWNER_MISMATCH`
- Tài nguyên không tồn tại: `404 NOT_FOUND`
- Dữ liệu sai ràng buộc: `422 VALIDATION_ERROR`
- Xung đột unique enrollment/submission: `409 CONFLICT`

---

## 5. Key Entities

- **User**: Người dùng hệ thống với role `ADMIN` hoặc `TEACHER` hoặc `STUDENT`
- **Course**: Lớp học do teacher sở hữu, chứa join code, danh sách assignment, danh sách enrollment
- **Enrollment**: Quan hệ student thuộc course, dùng để xác thực quyền truy cập assignment
- **Assignment**: Bài tập thuộc một course, tham chiếu template project làm nguồn clone
- **AssignmentSubmission**: Bản nộp duy nhất theo cặp assignment-student, liên kết với student project và trạng thái nộp
- **Project**: Entity sẵn có, được tái sử dụng cho template project và student cloned project
- **EditorSession**: Entity sẵn có, được tạo khi student bắt đầu assignment để mở editor liền mạch

---

## 6. API Surface

### 6.1 User management và phân quyền teacher

#### Endpoint U1 - Admin tạo teacher mới
- **Method**: `POST`
- **Path**: `/api/admin/users/teachers`
- **Auth requirement**: Session hợp lệ, role `ADMIN`
- **Request schema**:
  - `email: string` bắt buộc
  - `password: string` bắt buộc, min 8
  - `displayName: string` tùy chọn
- **Response schema**:
  - `success: true`
  - `data.user: { id, email, role, displayName, createdAt }`
- **Error codes**:
  - `401 UNAUTHORIZED`
  - `403 FORBIDDEN_ROLE`
  - `409 EMAIL_ALREADY_EXISTS`
  - `422 VALIDATION_ERROR`
- **Validation rules**:
  - Email đúng định dạng
  - Password đủ độ dài và complexity cơ bản
  - Email unique trong bảng user
- **Example request**:
```json
{
  "email": "teacher.math01@s4vn.edu.vn",
  "password": "T3acher!Pass",
  "displayName": "Nguyen Van A"
}
```
- **Example response**:
```json
{
  "success": true,
  "data": {
    "user": {
      "id": "cma_teacher_001",
      "email": "teacher.math01@s4vn.edu.vn",
      "role": "TEACHER",
      "displayName": "Nguyen Van A",
      "createdAt": "2026-02-10T04:00:00.000Z"
    }
  }
}
```

#### Endpoint U2 - Admin gán quyền teacher cho user có sẵn
- **Method**: `PATCH`
- **Path**: `/api/admin/users/{userId}/role`
- **Auth requirement**: Session hợp lệ, role `ADMIN`
- **Request schema**:
  - `role: "TEACHER"`
  - `reason: string` tùy chọn
- **Response schema**:
  - `success: true`
  - `data.user: { id, email, role, updatedAt }`
- **Error codes**:
  - `401 UNAUTHORIZED`
  - `403 FORBIDDEN_ROLE`
  - `404 USER_NOT_FOUND`
  - `409 INVALID_ROLE_TRANSITION`
- **Validation rules**:
  - Không cho phép hạ quyền chính admin hiện tại qua endpoint này
  - Không xử lý role ngoài tập `ADMIN|TEACHER|STUDENT`
- **Example request**:
```json
{
  "role": "TEACHER",
  "reason": "Assigned by school administrator"
}
```
- **Example response**:
```json
{
  "success": true,
  "data": {
    "user": {
      "id": "cma_user_002",
      "email": "existing.user@s4vn.edu.vn",
      "role": "TEACHER",
      "updatedAt": "2026-02-10T04:05:00.000Z"
    }
  }
}
```

### 6.2 Tạo và cập nhật khóa học-lớp học

#### Endpoint C1 - Teacher tạo course
- **Method**: `POST`
- **Path**: `/api/courses`
- **Auth requirement**: Session hợp lệ, role `TEACHER`
- **Request schema**:
  - `title: string` bắt buộc, max 256
  - `description: string` tùy chọn, max 2048
- **Response schema**:
  - `success: true`
  - `data.course: { id, title, description, joinCode, teacherId, isArchived }`
- **Error codes**:
  - `401 UNAUTHORIZED`
  - `403 FORBIDDEN_ROLE`
  - `422 VALIDATION_ERROR`
- **Validation rules**:
  - `title` không rỗng sau trim
  - `joinCode` sinh tự động unique
- **Example request**:
```json
{
  "title": "Webots Robotics 101 - Class A",
  "description": "Intro class for autonomous navigation"
}
```
- **Example response**:
```json
{
  "success": true,
  "data": {
    "course": {
      "id": "cma_course_001",
      "title": "Webots Robotics 101 - Class A",
      "description": "Intro class for autonomous navigation",
      "joinCode": "A7K3Q9",
      "teacherId": "cma_teacher_001",
      "isArchived": false
    }
  }
}
```

#### Endpoint C2 - Teacher cập nhật course
- **Method**: `PATCH`
- **Path**: `/api/courses/{courseId}`
- **Auth requirement**: Session hợp lệ, role `TEACHER`, owner course
- **Request schema**:
  - `title?: string`
  - `description?: string`
  - `isArchived?: boolean`
- **Response schema**:
  - `success: true`
  - `data.course: { id, title, description, isArchived, updatedAt }`
- **Error codes**:
  - `401 UNAUTHORIZED`
  - `403 FORBIDDEN_ROLE`
  - `403 FORBIDDEN_OWNER_MISMATCH`
  - `404 COURSE_NOT_FOUND`
  - `422 VALIDATION_ERROR`
- **Validation rules**:
  - Chỉ owner mới update
  - Course archived không cho phép cập nhật assignment mới

### 6.3 Tìm kiếm học sinh có sẵn để thêm vào lớp

#### Endpoint S1 - Teacher tìm student theo từ khoá
- **Method**: `GET`
- **Path**: `/api/courses/{courseId}/students/search?query={keyword}&limit={n}`
- **Auth requirement**: Session hợp lệ, role `TEACHER`, owner course
- **Request schema**:
  - Query params: `query` bắt buộc, `limit` tùy chọn default 20 max 50
- **Response schema**:
  - `success: true`
  - `data.items[]: { id, email, displayName, alreadyEnrolled }`
- **Error codes**:
  - `401 UNAUTHORIZED`
  - `403 FORBIDDEN_ROLE`
  - `403 FORBIDDEN_OWNER_MISMATCH`
  - `404 COURSE_NOT_FOUND`
  - `422 VALIDATION_ERROR`
- **Validation rules**:
  - Chỉ trả user role `STUDENT`
  - `query` tối thiểu 2 ký tự sau trim

### 6.4 Tạo học sinh mới

#### Endpoint S2 - Teacher tạo student mới từ context course
- **Method**: `POST`
- **Path**: `/api/courses/{courseId}/students`
- **Auth requirement**: Session hợp lệ, role `TEACHER`, owner course
- **Request schema**:
  - `email: string` bắt buộc
  - `password: string` bắt buộc
  - `displayName?: string`
  - `autoEnroll: boolean` mặc định `true`
- **Response schema**:
  - `success: true`
  - `data.student: { id, email, role }`
  - `data.enrollment?: { id, courseId, studentId, status }`
- **Error codes**:
  - `401 UNAUTHORIZED`
  - `403 FORBIDDEN_ROLE`
  - `403 FORBIDDEN_OWNER_MISMATCH`
  - `404 COURSE_NOT_FOUND`
  - `409 EMAIL_ALREADY_EXISTS`
  - `422 VALIDATION_ERROR`
- **Validation rules**:
  - Email unique
  - Role tạo mới bắt buộc `STUDENT`
  - Nếu `autoEnroll=true` thì phải tạo enrollment trong cùng transaction
- **Example request**:
```json
{
  "email": "student.new01@s4vn.edu.vn",
  "password": "Stud3nt!Pass",
  "displayName": "Tran Thi B",
  "autoEnroll": true
}
```
- **Example response**:
```json
{
  "success": true,
  "data": {
    "student": {
      "id": "cma_student_001",
      "email": "student.new01@s4vn.edu.vn",
      "role": "STUDENT"
    },
    "enrollment": {
      "id": "cma_enroll_001",
      "courseId": "cma_course_001",
      "studentId": "cma_student_001",
      "status": "ACTIVE"
    }
  }
}
```

### 6.5 Ghi danh và huỷ ghi danh học sinh

#### Endpoint E1 - Teacher ghi danh học sinh có sẵn
- **Method**: `POST`
- **Path**: `/api/courses/{courseId}/enrollments`
- **Auth requirement**: Session hợp lệ, role `TEACHER`, owner course
- **Request schema**:
  - `studentId: string` bắt buộc
- **Response schema**:
  - `success: true`
  - `data.enrollment: { id, courseId, studentId, status, joinedAt }`
- **Error codes**:
  - `401 UNAUTHORIZED`
  - `403 FORBIDDEN_ROLE`
  - `403 FORBIDDEN_OWNER_MISMATCH`
  - `404 COURSE_NOT_FOUND`
  - `404 STUDENT_NOT_FOUND`
  - `409 ENROLLMENT_ALREADY_EXISTS`
  - `422 VALIDATION_ERROR`
- **Validation rules**:
  - User phải có role `STUDENT`
  - Không cho duplicate enrollment active

#### Endpoint E2 - Teacher huỷ ghi danh học sinh
- **Method**: `DELETE`
- **Path**: `/api/courses/{courseId}/enrollments/{studentId}`
- **Auth requirement**: Session hợp lệ, role `TEACHER`, owner course
- **Request schema**: Không body
- **Response schema**:
  - `success: true`
  - `data: { courseId, studentId, status: "REMOVED", removedAt }`
- **Error codes**:
  - `401 UNAUTHORIZED`
  - `403 FORBIDDEN_ROLE`
  - `403 FORBIDDEN_OWNER_MISMATCH`
  - `404 ENROLLMENT_NOT_FOUND`
  - `409 CANNOT_REMOVE_SELF_FROM_OWNED_COURSE`
- **Validation rules**:
  - Chuyển `Enrollment.status` sang `REMOVED` thay vì hard delete trong MVP

### 6.6 Assignment và submission APIs

#### Endpoint A1 - Teacher tạo assignment
- **Method**: `POST`
- **Path**: `/api/courses/{courseId}/assignments`
- **Auth requirement**: Session hợp lệ, role `TEACHER`, owner course
- **Request schema**:
  - `title: string`
  - `description?: string`
  - `templateProjectId: string`
  - `dueAt?: ISO datetime`
- **Response schema**:
  - `success: true`
  - `data.assignment: { id, courseId, templateProjectId, dueAt }`
- **Error codes**:
  - `403 FORBIDDEN_OWNER_MISMATCH`
  - `404 TEMPLATE_PROJECT_NOT_FOUND`
  - `422 VALIDATION_ERROR`

#### Endpoint A2 - Student start assignment
- **Method**: `POST`
- **Path**: `/api/assignments/{assignmentId}/start`
- **Auth requirement**: Session hợp lệ, role `STUDENT`, enrollment active
- **Request schema**: Không body
- **Response schema**:
  - `success: true`
  - `data: { assignmentId, studentProjectId, submissionId, editorUrl, reusedExisting }`
- **Error codes**:
  - `403 ENROLLMENT_REQUIRED`
  - `404 ASSIGNMENT_NOT_FOUND`
  - `409 TEMPLATE_COPY_FAILED`

#### Endpoint A3 - Student submit assignment
- **Method**: `POST`
- **Path**: `/api/assignments/{assignmentId}/submit`
- **Auth requirement**: Session hợp lệ, role `STUDENT`, enrollment active
- **Request schema**:
  - `studentProjectId: string`
  - `comment?: string`
- **Response schema**:
  - `success: true`
  - `data.submission: { id, assignmentId, studentId, status, submittedAt }`
- **Error codes**:
  - `403 ENROLLMENT_REQUIRED`
  - `403 PROJECT_OWNERSHIP_MISMATCH`
  - `404 ASSIGNMENT_NOT_FOUND`
  - `422 VALIDATION_ERROR`
- **Validation rules**:
  - Upsert theo `(assignmentId, studentId)`
  - Submit lại overwrite cùng record

---

## 7. Success Criteria

### Measurable Outcomes

- **SC-001**: Admin tạo teacher mới và gán quyền teacher cho user có sẵn thành công qua API với đầy đủ kiểm soát lỗi
- **SC-002**: Teacher tạo và cập nhật course thành công, chỉ owner course được phép chỉnh sửa
- **SC-003**: Teacher tìm kiếm học sinh có sẵn và ghi danh vào lớp thành công, duplicate enrollment bị chặn đúng mã lỗi
- **SC-004**: Teacher tạo học sinh mới và auto-enroll vào lớp trong một luồng nghiệp vụ nhất quán
- **SC-005**: Student start assignment lần đầu tạo đầy đủ `Project` + `EditorSession` + `AssignmentSubmission`; start lần sau tái sử dụng bản hiện có
- **SC-006**: Student submit nhiều lần luôn cập nhật cùng một submission record theo policy overwrite
- **SC-007**: Tất cả endpoint chính trả đúng hành vi RBAC theo ma trận Admin/Teacher/Student và trả mã lỗi nhất quán khi bị từ chối

---

## 8. Out of Scope for MVP

- Auto-grading thật chạy simulation supervisor và trả score tự động
- Multi-attempt history cho submission
- Cơ chế nhiều role đồng thời trên một user
- Deadline policy nâng cao, late penalty, anti-cheat
