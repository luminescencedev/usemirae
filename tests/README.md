# tests/

Cross-cutting tests only. Subsystem-local unit and component tests stay next to
their implementation.

```text
integration/    multi-process engine/shell behavior
e2e/            full application flows
compatibility/  project round-trip and version compatibility fixtures
performance/    benchmarks and budgets
fault/          crash, interruption, and recovery scenarios
```

Created empty by `MIR-0001`. First harness arrives with `MIR-0015`.

Canonical documentation: `docs/08-development/809-testing-and-validation-workflow.md`.
