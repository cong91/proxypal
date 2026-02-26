# Python Rules

Python best practices for the S4VN Webots project.

**Scope**: `**/*.py` (all Python files)

---

## Code Style

- Follow **PEP 8** style guide
- Use `snake_case` for functions and variables
- Use `PascalCase` for classes
- Use `SCREAMING_SNAKE_CASE` for constants
- **Maximum line length: 100 characters** (prefer 80)
- Use **4 spaces for indentation** (no tabs)

### Example
```python
# Constants
MAX_SIMULATION_TIME = 300
WEBOTS_PORT = 1234

# Class
class SimulationServer:
    def __init__(self):
        self.is_running = False
    
    def start_simulation(self):
        pass

# Function
def calculate_robot_position(x: float, y: float) -> tuple[float, float]:
    return (x * 2, y * 2)
```

---

## Type Hints

- **Use type hints** for function parameters and return values
- Use `typing` module for complex types
- Prefer `Optional[Type]` over `Type | None` for compatibility
- Use `Union` for multiple types
- Document complex types with docstrings

### Example
```python
from typing import Optional, Union, List, Dict

def process_sensor_data(
    data: List[float],
    threshold: Optional[float] = None
) -> Dict[str, Union[float, bool]]:
    """Process sensor data and return analysis."""
    result = {
        "average": sum(data) / len(data),
        "exceeds_threshold": False
    }
    if threshold:
        result["exceeds_threshold"] = max(data) > threshold
    return result
```

---

## Error Handling

- Use **specific exceptions**, not bare `except:`
- **Log errors with context** before raising
- **Fail fast** with clear error messages
- **Never use fallback logic** (see [no-fallback-policy.md](no-fallback-policy.md))
- Provide **actionable error messages**

### Example
```python
import logging

# ❌ BAD
try:
    start_simulation()
except:
    pass  # Silent failure!

# ✅ GOOD
try:
    start_simulation()
except SimulationError as e:
    logging.error(f"Simulation failed to start: {e}")
    raise RuntimeError(
        f"Cannot start simulation: {e}\n"
        f"Fix: Check Webots installation and GPU configuration"
    )
```

---

## Async/Await

- Use `async def` for async functions
- Use `await` for async operations
- Handle asyncio properly (use `asyncio.run()` or event loop)
- **Don't mix sync and async** without proper handling
- Use `AsyncProcess` for subprocess management (Tornado)

### Example
```python
import asyncio

async def run_simulation_async(duration: int) -> dict:
    """Run simulation asynchronously."""
    await asyncio.sleep(duration)
    return {"status": "completed", "duration": duration}

# Tornado WebSocket handler
class SimulationWebSocket(tornado.websocket.WebSocketHandler):
    async def on_message(self, message):
        result = await self.process_command(message)
        await self.write_message(result)
```

---

## Imports

- **Group imports**: stdlib, third-party, local
- Use **absolute imports** when possible
- **Avoid** `from module import *`
- Sort imports **alphabetically** within groups

### Example
```python
# Standard library
import os
import sys
from typing import Optional

# Third-party
import tornado
import psutil
from tornado.web import RequestHandler

# Local
from simulation_server import SimulationServer
from utils.gpu_checker import check_gpu_available
```

---

## Documentation

- Write **docstrings** for all public functions/classes
- Use **Google-style docstrings**
- Document parameters and return values
- Include examples for complex functions

### Example
```python
def calculate_trajectory(
    start_pos: tuple[float, float],
    velocity: float,
    time: float
) -> tuple[float, float]:
    """Calculate robot trajectory based on velocity and time.
    
    Args:
        start_pos: Initial position (x, y) in meters
        velocity: Robot velocity in m/s
        time: Simulation time in seconds
    
    Returns:
        Final position (x, y) in meters
    
    Example:
        >>> calculate_trajectory((0, 0), 1.0, 5.0)
        (5.0, 0.0)
    """
    x, y = start_pos
    return (x + velocity * time, y)
```

---

## Dependencies

- **Pin versions** in `requirements.txt`
- Separate optional dependencies (e.g., `boto3` for S3)
- **Document why** each dependency is needed
- Keep dependencies minimal

### Example requirements.txt
```txt
# Web framework
tornado==6.3.3

# System monitoring
psutil==5.9.5

# Optional: S3 storage
boto3==1.28.25  # For simulation recording upload
```

---

## File Organization

- **One class per file** (when possible)
- Group related functions in modules
- Use `__init__.py` for package structure
- Keep modules focused and cohesive

### Project Structure
```
S4VN-Simulation/
├── src/
│   ├── simulation_server.py      # Main server class
│   ├── websocket_handler.py      # WebSocket handlers
│   ├── docker_manager.py         # Docker operations
│   └── utils/
│       ├── __init__.py
│       ├── gpu_checker.py        # GPU utilities
│       └── logger.py             # Logging utilities
```

---

## WebSocket/Tornado

- Use **Tornado's async patterns**
- Handle WebSocket connections properly
- **Clean up resources on disconnect**
- Log connection lifecycle events
- Handle errors gracefully without disconnecting unnecessarily

### Example
```python
class SimulationWebSocket(tornado.websocket.WebSocketHandler):
    def open(self):
        logging.info(f"WebSocket opened: {self.request.remote_ip}")
        self.session_id = str(uuid.uuid4())
    
    async def on_message(self, message):
        try:
            data = json.loads(message)
            result = await self.process_command(data)
            await self.write_message(json.dumps(result))
        except Exception as e:
            logging.error(f"Message processing error: {e}")
            await self.write_message(json.dumps({"error": str(e)}))
    
    def on_close(self):
        logging.info(f"WebSocket closed: {self.session_id}")
        self.cleanup_resources()
```

---

## Docker Integration

- Use **environment variables** for configuration
- Handle Docker errors explicitly
- Log Docker operations clearly
- **Never assume Docker is available** without checking

### Example
```python
import os

DOCKER_IMAGE = os.getenv("WEBOTS_IMAGE", "cyberbotics/webots:latest")
NVIDIA_VISIBLE_DEVICES = os.getenv("NVIDIA_VISIBLE_DEVICES", "all")

def start_container():
    if not check_docker_available():
        raise RuntimeError(
            "Docker not available.\n"
            "Fix: Install Docker and ensure daemon is running"
        )
    
    logging.info(f"Starting container with image: {DOCKER_IMAGE}")
    # ... container startup logic
```

---

## GPU/NVIDIA

- **Check GPU availability** before use
- Handle NVIDIA errors explicitly
- Log GPU status for debugging
- **Fail if GPU required but not available** (NO FALLBACK)

### Example
```python
def check_gpu_available() -> bool:
    """Check if NVIDIA GPU is available."""
    try:
        result = subprocess.run(
            ["nvidia-smi"],
            capture_output=True,
            text=True,
            timeout=5
        )
        return result.returncode == 0
    except (FileNotFoundError, subprocess.TimeoutExpired):
        return False

def initialize_gpu():
    """Initialize GPU for rendering."""
    if not check_gpu_available():
        error = """
        GPU not available. NVIDIA GPU is required for Webots simulation.
        
        Fix Steps:
        1. Install NVIDIA drivers
        2. Install nvidia-container-toolkit
        3. Verify with: nvidia-smi
        4. Restart Docker daemon
        """
        logging.error(error)
        raise RuntimeError("GPU not available")
    
    logging.info("GPU available and initialized")
```

---

## Code Review Checklist

- [ ] Follows PEP 8 style guide
- [ ] Type hints for all functions
- [ ] Proper error handling (no bare except)
- [ ] Docstrings for public functions
- [ ] Imports properly organized
- [ ] No fallback logic
- [ ] Async/await used correctly
- [ ] Resources cleaned up properly
