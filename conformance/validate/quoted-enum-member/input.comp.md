```compmd component=status
params:
  tone: info | warn = info
  state: '"to do" | done = done'
---
{{tone}}, {{state}}
```

:: status[Status] {tone="warn" state="to do"}
