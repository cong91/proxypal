---
name: automated-code-review
description: A workflow to automatically execute static analysis, type checking, and test validation based on project type. Use this skill when starting a code review to verify code quality before manual inspection.
allowed-tools: execute_command, read_file, search_files, list_files
---

# Automated Code Review Workflow

## Overview

This skill provides a systematic approach to automated code verification. Execute these checks BEFORE performing manual code review.

---

## 1. Environment Detection

First, identify the project type by checking configuration files:

```bash
# Check for Python project
ls pyproject.toml requirements.txt setup.py 2>/dev/null

# Check for Node.js project
ls package.json tsconfig.json 2>/dev/null
```

### Detection Matrix

| File Found                       | Project Type    | Primary Tools            |
| -------------------------------- | --------------- | ------------------------ |
| `pyproject.toml`                 | Python (Modern) | ruff, mypy, pytest       |
| `requirements.txt`               | Python (Legacy) | flake8, mypy, pytest     |
| `package.json` + `tsconfig.json` | TypeScript      | tsc, eslint, jest/vitest |
| `package.json` only              | JavaScript      | eslint, jest             |
| Both Python + Node files         | Hybrid          | All tools                |

---

## 2. Python Verification Strategy

If Python files are modified, execute in order:

### Step A: Static Analysis (Linter & Formatter)

```bash
# Option 1: Ruff (Preferred - fast, comprehensive)
ruff check {file_path} --output-format=concise

# Option 2: Ruff with auto-fix preview
ruff check {file_path} --diff

# Option 3: Format check
ruff format --check {file_path}
```

### Step B: Type Checking

```bash
# Basic type check
mypy {file_path}

# Strict mode (recommended for new code)
mypy {file_path} --strict

# Check pyproject.toml for project-specific mypy config
```

### Step C: Security Scan

```bash
# Bandit security linter
bandit -r {directory} -f json

# Check for known vulnerabilities in dependencies
pip-audit
```

### Step D: Testing

```bash
# Discover related tests
pytest --collect-only -q | grep {module_name}

# Run specific tests with coverage
pytest {test_file_path} -v --tb=short

# Run with coverage report
pytest {test_file_path} --cov={source_path} --cov-report=term-missing
```

---

## 3. TypeScript/JavaScript Verification Strategy

If TS/JS files are modified:

### Step A: Dependency Verification

```bash
# Check if node_modules exists
ls node_modules 2>/dev/null || echo "Run: npm install"

# Verify no outdated/vulnerable packages
npm audit --audit-level=moderate
```

### Step B: Type Checking (Critical for TypeScript)

```bash
# Project-wide type check
npx tsc --noEmit

# Specific file check (with common flags)
npx tsc {file_path} --noEmit --esModuleInterop --skipLibCheck --resolveJsonModule
```

### Step C: Linting

```bash
# If lint script exists
npm run lint

# Direct ESLint
npx eslint {file_path} --format=stylish

# With auto-fix preview
npx eslint {file_path} --fix-dry-run
```

### Step D: Testing

```bash
# Jest
npm test -- --testPathPattern={file_pattern} --verbose

# Vitest
npx vitest run {file_pattern} --reporter=verbose

# With coverage
npm test -- --coverage --collectCoverageFrom="{source_pattern}"
```

---

## 4. Code Quality Metrics

### Complexity Analysis

```bash
# Python - radon for cyclomatic complexity
radon cc {file_path} -a -s

# Python - radon for maintainability index
radon mi {file_path} -s
```

### Duplicate Code Detection

```bash
# Python - pylint duplicate check
pylint {file_path} --disable=all --enable=duplicate-code

# Search for similar patterns
grep -rn "pattern_to_find" {directory}
```

---

## 5. Report Generation Template

After running all checks, generate a summary:

```markdown
## 🔍 Automated Code Review Report

### Environment

- **Project Type**: Python/TypeScript/Hybrid
- **Files Reviewed**: {list of files}
- **Review Date**: {timestamp}

### Quality Gates

| Check       | Tool                 | Status | Details               |
| ----------- | -------------------- | ------ | --------------------- |
| Linting     | ruff/eslint          | ✅/❌  | {error count}         |
| Type Safety | mypy/tsc             | ✅/❌  | {error count}         |
| Formatting  | ruff format/prettier | ✅/❌  | {diff count}          |
| Security    | bandit/npm audit     | ✅/❌  | {vulnerability count} |
| Tests       | pytest/jest          | ✅/❌  | {pass}/{total}        |
| Coverage    | coverage.py/istanbul | ⚠️     | {percentage}%         |

### Blocking Issues

{List any errors that must be fixed}

### Warnings

{List non-blocking concerns}

### Passed Checks

{Confirmation of what succeeded}

---

**Recommendation**: ✅ APPROVE / ⚠️ APPROVE WITH NOTES / ❌ REQUEST CHANGES
```

---

## 6. Quick Commands Reference

### Python Project

```bash
# Full check suite
ruff check . && mypy . && pytest

# Fast check (lint only)
ruff check . --fix --unsafe-fixes
```

### TypeScript Project

```bash
# Full check suite
npm run lint && npm run type-check && npm test

# Fast check (type only)
npx tsc --noEmit
```

### Pre-commit Style

```bash
# Run all formatters and linters
ruff format . && ruff check . --fix
npx prettier --write . && npx eslint . --fix
```

---

## 7. Common Issues and Solutions

| Issue                     | Cause         | Solution                        |
| ------------------------- | ------------- | ------------------------------- |
| `mypy: Module not found`  | Missing stubs | `pip install types-{package}`   |
| `tsc: Cannot find module` | Missing types | `npm i -D @types/{package}`     |
| `eslint: Parsing error`   | Wrong parser  | Check `.eslintrc` parser config |
| `pytest: No tests found`  | Wrong path    | Use `-v` to see collection      |
| `ruff: Unknown rule`      | Old version   | `pip install -U ruff`           |

---

## Integration Notes

- This skill should be invoked at the START of every review
- If any BLOCKING issues found, stop review and report immediately
- Use results to inform the manual review phase
- Always include command outputs in the review report
