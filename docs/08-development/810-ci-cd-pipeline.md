# 810 — CI/CD Pipeline

**Status:** Proposed  
**Audience:** Build, release, all contributors  
**Canonical:** Yes  
**Required context:** `06-quality/616-release-quality-gates.md`, `806-build-system-and-toolchain.md`

---

## 1. Purpose

CI protects contracts and produces traceable artifacts.

---

## 2. Pull Request Pipeline

Jobs:

1. repository policy and secret scan;
2. schema/codegen drift;
3. Rust format/lint/unit tests;
4. TypeScript format/lint/typecheck/tests;
5. documentation links and structure;
6. integration tests;
7. architecture dependency checks;
8. security parser/fuzz smoke;
9. affected platform build;
10. benchmark smoke for sensitive paths.

---

## 3. Main Branch Pipeline

Adds:

- all supported platform builds;
- packaging smoke;
- compatibility corpus;
- broader integration suite;
- artifact signing test with non-production keys;
- benchmark recording;
- SBOM/dependency reports;
- nightly artifact creation if configured.

---

## 4. Release Pipeline

Adds:

- clean tagged source;
- locked/pinned inputs;
- production signing;
- notarization/platform verification;
- package manifest;
- symbol upload/storage;
- update metadata signing;
- install/update/rollback test;
- release-quality gate report;
- artifact retention.

---

## 5. Change Detection

Affected jobs may use path/dependency graph filtering, but:

- schema changes run all contract consumers;
- foundation changes run broad tests;
- CI configuration changes run full validation;
- dependency-rule changes run full architecture checks.

---

## 6. Artifacts

CI artifacts may include:

- test reports;
- coverage;
- benchmark reports;
- generated-contract diff;
- packages;
- symbols;
- SBOM;
- support diagnostics;
- screenshots for UI tests;
- release gate report.

Retention is bounded and privacy-safe.

---

## 7. Invariants

1. PRs cannot merge with mandatory failures.
2. Generated drift fails CI.
3. Release artifacts are traceable.
4. Secrets are not printed.
5. Signing uses protected environments.
6. Release input is immutable.
7. Platform builds are verified.
8. Benchmark regressions receive disposition.
9. Artifacts have retention policy.
10. CI commands match local commands.
