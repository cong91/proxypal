# Coding Principles

Core software engineering principles for 10X development quality.

**Scope**: All files (`**/*`)

---

## DRY (Don't Repeat Yourself)

- Extract common logic into reusable functions
- Use shared utilities instead of copy-paste
- Create abstractions for repeated patterns
- **If you write something twice, refactor immediately**
- Share code between projects when appropriate

### Examples
```python
# ❌ BAD - Duplication
def get_user_email(user_id):
    user = db.query(f"SELECT * FROM users WHERE id={user_id}")
    return user.email

def get_user_name(user_id):
    user = db.query(f"SELECT * FROM users WHERE id={user_id}")
    return user.name

# ✅ GOOD - Extract common logic
def get_user(user_id):
    return db.query(f"SELECT * FROM users WHERE id={user_id}")

def get_user_email(user_id):
    return get_user(user_id).email

def get_user_name(user_id):
    return get_user(user_id).name
```

---

## KISS (Keep It Simple, Stupid)

- Prefer simple solutions over clever ones
- Avoid over-engineering
- Write code that's easy to understand
- When in doubt, choose the simpler approach
- **Don't add features "just in case"**

### Examples
```python
# ❌ BAD - Over-engineered
class DataProcessorFactoryBuilder:
    def create_processor(self, type, config, options):
        # 100 lines of complex factory pattern
        pass

# ✅ GOOD - Simple and clear
def process_data(data):
    return [item.upper() for item in data]
```

---

## SOLID Principles

### Single Responsibility
One reason to change per class/function.

```python
# ❌ BAD - Multiple responsibilities
class User:
    def save_to_db(self): pass
    def send_email(self): pass
    def generate_report(self): pass

# ✅ GOOD - Single responsibility
class User:
    def save(self): pass

class EmailService:
    def send_user_email(self, user): pass

class ReportGenerator:
    def generate_user_report(self, user): pass
```

### Open/Closed
Open for extension, closed for modification.

### Liskov Substitution
Subtypes must be substitutable for their base types.

### Interface Segregation
Many specific interfaces > one general interface.

### Dependency Inversion
Depend on abstractions, not concretions.

---

## Code Quality

### Naming
- Use meaningful names (variables, functions, classes)
- Avoid abbreviations unless universally understood
- Be specific: `user_email` not `ue`, `calculate_total_price` not `calc`

### Function Size
- Keep functions small (**< 20 lines ideal, < 50 max**)
- One function = one task
- If function is too long, split it

### Function Parameters
- Limit parameters (**< 4 ideal**)
- Use objects/dicts for many related parameters
- Use default values when appropriate

### Error Handling
- Handle errors explicitly (**no silent failures**)
- Log errors with context
- Provide actionable error messages
- See [no-fallback-policy.md](no-fallback-policy.md)

### Testing
- Write tests for critical paths
- Test edge cases
- Mock external dependencies
- Keep tests fast

### Comments
- Comment **"why"**, not **"what"**
- Code should be self-documenting
- Update comments when code changes

```python
# ❌ BAD - Obvious comment
x = x + 1  # increment x

# ✅ GOOD - Explains why
x = x + 1  # Account for zero-based indexing
```

---

## Performance Mindset

- **Don't optimize prematurely** - make it work first
- **Measure before optimizing** - use profilers
- Consider memory and CPU implications
- Use appropriate data structures
- Profile in production when needed

---

## Security First

- **Validate all inputs** - never trust user data
- **Sanitize before display** - prevent XSS
- **Use parameterized queries** - prevent SQL injection
- Never expose sensitive data in logs/errors
- Follow principle of least privilege

```python
# ❌ BAD - SQL injection vulnerability
query = f"SELECT * FROM users WHERE id={user_input}"

# ✅ GOOD - Parameterized query
query = "SELECT * FROM users WHERE id=?"
db.execute(query, (user_input,))
```

---

## Code Review Checklist

Before submitting code, verify:
- [ ] No code duplication (DRY)
- [ ] Simple and understandable (KISS)
- [ ] Functions have single responsibility
- [ ] Meaningful variable/function names
- [ ] Functions < 50 lines
- [ ] Error handling is explicit
- [ ] Security best practices followed
- [ ] Tests written for critical paths

---

**Remember**: Good code is code that other developers (including future you) can understand and maintain easily.
