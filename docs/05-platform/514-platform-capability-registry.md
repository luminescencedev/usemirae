# 514 — Platform Capability Registry

**Status:** Proposed  
**Audience:** Platform, runtime, UI, SDK contributors  
**Canonical:** Yes  
**Required context:** `500-platform-overview.md`, `01-runtime/106-state-store.md`  
**Related ADRs:** ADR-0035, ADR-0038

---

## 1. Purpose

The capability registry is the authoritative runtime projection of what the current installation, platform, hardware, permissions, and packaging mode can do.

---

## 2. Capability Domains

- rendering;
- capture;
- audio;
- cameras;
- encoders;
- decoders;
- containers;
- network protocols;
- files;
- credentials;
- notifications;
- deep links;
- updates;
- extensions;
- accessibility;
- localization.

---

## 3. Capability Result

```rust
pub struct CapabilityResult<T> {
    pub status: CapabilityStatus,
    pub value: Option<T>,
    pub limitations: Vec<CapabilityLimitation>,
    pub requirements: Vec<CapabilityRequirement>,
    pub source: CapabilitySource,
    pub generation: CapabilityGeneration,
}
```

---

## 4. Sources

Capabilities may derive from:

- compile-time feature;
- OS version;
- API probe;
- permission;
- entitlement;
- packaging mode;
- device enumeration;
- GPU/driver;
- external service availability;
- active workaround;
- user policy.

The source is observable.

---

## 5. Generation

Capability generation increments when any effective capability changes.

Subsystem-specific generations may be used to avoid excessive global churn, but the snapshot has a coherent root generation.

---

## 6. Refresh

Refresh triggers:

- startup;
- permission change;
- device hotplug;
- display change;
- driver/device reset;
- resume;
- portal/service restart;
- packaging/update change;
- extension install/remove;
- user request.

Probes are bounded and may run asynchronously.

---

## 7. UI Projection

UI uses capabilities to:

- enable/disable controls;
- explain limitations;
- propose compatible options;
- avoid invalid configuration;
- show required permissions;
- surface experimental status.

Disabled state must include reason where actionable.

---

## 8. Project Validation

Project open compares required features against capabilities.

Unavailable runtime capability does not always invalidate project.

The validator classifies:

- missing but recoverable;
- unsupported on platform;
- permission required;
- package limitation;
- incompatible hardware;
- required feature impossible.

---

## 9. SDK Projection

Extensions receive only the capability subset allowed by their grants.

They must not receive sensitive device metadata without need.

---

## 10. Caching

Capability cache includes:

- source identity;
- driver/build;
- probe version;
- expiry;
- invalidation conditions.

Cached failure must not persist indefinitely when environment changes.

---

## 11. Invariants

1. Capability is more than a boolean.
2. Generation is explicit.
3. Limitation and requirement are structured.
4. Packaging and permission are separate causes.
5. Refresh is bounded.
6. UI does not infer from OS name.
7. Project validation distinguishes unavailable from invalid.
8. SDK receives filtered view.
9. Probe cache has invalidation.
10. Active workarounds are represented.

---

## 12. Required Tests

- startup snapshot;
- permission change;
- device hotplug;
- package limitation;
- driver workaround;
- cache invalidation;
- project requirement classification;
- UI reason projection;
- SDK filtering;
- experimental capability;
- probe timeout;
- concurrent refresh coalescing.

---

## 13. AI Implementation Notes

Do not spread feature checks like `if windows` or `if macos` through UI and domain code.

Do not reduce capability to `true/false` when limitations matter.

Keep the source and reason for every unavailable capability.
