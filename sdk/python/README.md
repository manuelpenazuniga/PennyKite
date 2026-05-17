# PennyKite Python SDK

Local convenience wrapper for agents that call APIs through a running PennyKite proxy.

```python
from pennykite import PennyKiteClient, PennyKiteLoopDetected

with PennyKiteClient(session_id="demo-session") as client:
    try:
        response = client.get("/matches/runaway-loop")
    except PennyKiteLoopDetected as exc:
        print(exc.reason)
```
