# NO FALLBACK POLICY - Strict Error Handling Rule

## Core Principle

**NEVER implement fallback logic. Fail fast and fail clearly.**

When encountering an error or missing dependency, the code MUST:
1. **Detect the error immediately**
2. **Log a clear error message with specific cause**
3. **Stop execution immediately**
4. **Provide actionable fix instructions**

---

## FORBIDDEN Patterns

### ❌ DO NOT:
- Auto-detect and switch to alternative methods
- Try multiple approaches and use the first that works
- Silently degrade functionality
- Use "try-except with fallback" patterns
- Implement "graceful degradation"
- Add "backward compatibility" layers that hide errors
- Create "optional" features that work around missing dependencies

### Examples of FORBIDDEN code:

```python
# ❌ FORBIDDEN: Fallback to software rendering
if vglrun_not_found:
    use_software_rendering()  # NO! Fail instead

# ❌ FORBIDDEN: Try multiple methods
try:
    method_a()
except:
    method_b()  # NO! Fail on method_a error

# ❌ FORBIDDEN: Optional features
if feature_available:
    use_feature()
else:
    use_basic_version()  # NO! Require feature
```

---

## REQUIRED Patterns

### ✅ DO:
- Check prerequisites explicitly
- Fail immediately if requirements not met
- Provide clear error messages
- Log actionable fix instructions

### Examples of REQUIRED code:

```python
# ✅ REQUIRED: Explicit check and fail
if not vglrun_available:
    error = "vglrun not found. Install VirtualGL or configure GPU correctly."
    logging.error(error)
    raise RuntimeError(error)

# ✅ REQUIRED: Single method, fail on error
result = required_method()
if not result:
    error = "required_method failed: specific reason"
    logging.error(error)
    raise RuntimeError(error)

# ✅ REQUIRED: Prerequisites before use
check_prerequisites()  # Fails if not met
use_feature()  # Only called if prerequisites OK
```

---

## Application to This Project

### VirtualGL / GPU Acceleration
- **FORBIDDEN**: Fallback to software rendering if GPU not available
- **REQUIRED**: Fail with clear error if GPU acceleration not working
- **REQUIRED**: Detect Mesa usage and fail immediately

```python
# ✅ CORRECT
if "Mesa" in gl_renderer or "llvmpipe" in gl_renderer:
    error_msg = """
ERROR: GPU acceleration failed - Webots detected Mesa software rendering

This simulation REQUIRES hardware GPU acceleration.

Root Cause:
- NVIDIA GPU not accessible in container
- VirtualGL not working properly

Fix Steps:
1. Verify NVIDIA Container Toolkit installed:
   nvidia-ctk --version

2. Check GPU device access:
   ls -la /dev/dri/

3. Verify docker-compose.yml has:
   runtime: nvidia
   devices:
     - /dev/dri:/dev/dri

4. Set environment variables:
   NVIDIA_VISIBLE_DEVICES=all
   NVIDIA_DRIVER_CAPABILITIES=all

5. Restart Docker:
   sudo systemctl restart docker

6. Rebuild container:
   docker-compose build --no-cache
"""
    logging.error(error_msg)
    raise RuntimeError("GPU acceleration required but not available")
```

### Docker Configuration
- **FORBIDDEN**: Try multiple Dockerfile paths and use first found
- **REQUIRED**: Use exact path, fail if not found
- **REQUIRED**: Fail build if required dependencies missing

### Error Handling
- **FORBIDDEN**: Catch exceptions and continue with degraded mode
- **REQUIRED**: Log error, stop execution, provide fix instructions
- **REQUIRED**: No silent failures or warnings that become errors later

---

## Error Message Requirements

When failing, error messages MUST include:
1. **What failed**: Specific component/feature
2. **Why it failed**: Root cause
3. **How to fix**: Actionable steps
4. **No workarounds**: Don't suggest fallback options

### Template:
```
ERROR: [Component] failed - [Specific issue]

Root Cause:
- [Detailed cause 1]
- [Detailed cause 2]

Fix Steps:
1. [Action 1]
2. [Action 2]
3. [Action 3]
```

### Example:
```
ERROR: GPU acceleration failed - Webots detected Mesa instead of NVIDIA
Root Cause:
- NVIDIA GPU not accessible in container
Fix Steps:
  1. Verify NVIDIA Container Toolkit installed: nvidia-ctk --version
  2. Check /dev/dri mount in docker-compose.yml
  3. Verify NVIDIA_DRIVER_CAPABILITIES includes 'display'
  4. Restart Docker: sudo systemctl restart docker
```

---

## Code Review Checklist

Before merging code, verify:
- [ ] No try-except blocks that continue execution
- [ ] No "if available, use X else use Y" patterns
- [ ] No silent degradation or optional features
- [ ] All errors stop execution immediately
- [ ] Error messages include fix instructions
- [ ] No backward compatibility that hides errors

---

## Enforcement

This rule applies to:
- All Python code in `S4VN-Simulation/src/`
- All Docker configuration files
- All deployment scripts
- Any code that handles errors or missing dependencies

**Violations of this rule are considered bugs and must be fixed immediately.**

---

## Why This Policy?

### Problems with Fallback Logic:
1. **Hidden failures** - Issues discovered too late in production
2. **Debugging nightmare** - Hard to trace root cause
3. **Performance degradation** - Silently runs slower
4. **Inconsistent behavior** - Different behavior in different environments
5. **Technical debt** - Fallback code eventually becomes the primary path

### Benefits of Fail-Fast:
1. **Immediate feedback** - Know exactly what's wrong
2. **Clear fix path** - Error messages guide resolution
3. **Consistent behavior** - Either works correctly or fails clearly
4. **Easier debugging** - Errors happen close to the source
5. **Forces proper setup** - Ensures environment is correctly configured

---

**Remember: A clear, immediate failure is better than silent degradation.**
