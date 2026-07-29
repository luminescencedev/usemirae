# 006 — Terminology

**Status:** Proposed  
**Audience:** All contributors and coding agents  
**Canonical:** Yes

---

## 1. Purpose

This document defines canonical Mirae terminology.

Code, schemas, UI copy, diagnostics, and documentation SHOULD use these terms consistently.

---

## 2. Core Terms

### Project

The persisted root configuration for a production.

### Project Library

The local index of known projects, templates, recent items, recovery items, and library metadata.

### Scene

A semantic composition containing ordered scene items.

### Source Definition

Persisted configuration describing a reusable source.

### Source Runtime

Active runtime resources that acquire or generate media for a source definition.

### Scene Item

An instance of a source or group placed inside a scene.

### Group

A scene item containing ordered child scene items.

### Preview

The operator-visible composition prepared before it becomes live.

### Program

The composition currently feeding one or more live production outputs.

### Transition

A bounded runtime operation that changes program composition from one state to another.

### Canvas

The logical composition coordinate space of a scene or output.

### Surface

A render destination such as preview, program, encoder input, virtual camera, or screenshot.

---

## 3. Runtime Terms

### Engine

The native authoritative runtime coordinating state, media, rendering, outputs, and persistence.

### Engine Session

One lifetime of an engine process, identified by a session identifier.

### Command

A validated request to mutate authoritative state or invoke a controlled operation.

### Acknowledgement

The result of command submission: accepted, rejected, failed, or pending through a defined asynchronous lifecycle.

### Event

A committed domain change or runtime observation published to subscribers.

### Transaction

An atomic group of state changes that either commits or has no visible effect.

### State Generation

A monotonically increasing version identifying a committed authoritative state revision.

### Snapshot

A complete representation of a state projection at one generation.

### Patch

An ordered change transforming a known state generation into a later generation.

---

## 4. Rendering Terms

### Scene Graph

The semantic hierarchy of scenes, source instances, groups, transforms, and effects.

### Frame Compiler

The component that resolves semantic scene state and runtime source availability into frame-specific render requirements.

### Render Graph

A directed acyclic description of render passes, resources, dependencies, and execution ordering for a frame or surface.

### Compositor

The rendering component that combines source images, masks, transforms, effects, color operations, and overlays into a surface.

### Render Pass

A unit of GPU work with declared inputs, outputs, and dependencies.

### GPU Resource

A texture, buffer, sampler, pipeline, bind group, or related graphics object owned through the renderer abstraction.

### Resource Generation

A version identifying a valid lifetime of a replaceable GPU or runtime resource.

---

## 5. Media Terms

### Media Frame

A timestamped video frame, audio block, subtitle unit, or metadata unit in the media pipeline.

### Master Clock

The selected timing authority used to schedule synchronized media presentation.

### Timebase

The unit and rational scale used to interpret timestamps.

### Presentation Timestamp

The intended media presentation time.

### Capture Timestamp

The time assigned when media is acquired from an external source.

### Frame Queue

A bounded queue between media stages with an explicit overflow policy.

### Discontinuity

A non-contiguous jump or reset in media timing requiring explicit handling.

### Source Health

Structured runtime status describing availability, timing, errors, and recovery state.

---

## 6. Audio Terms

### Audio Graph

The real-time processing graph for audio sources, buses, effects, meters, monitoring, and outputs.

### Bus

A named audio routing and mixing destination.

### Monitor

Audio rendered for local operator listening.

### Program Mix

The audio mix associated with live program output.

### Audio Block

A fixed or bounded set of audio samples processed as one real-time unit.

### XRUN

An audio underrun or overrun caused by data not being available or consumed in time.

---

## 7. Output Terms

### Output Profile

Persisted configuration for a stream, recording, replay, or virtual output.

### Output Runtime

The active encoder, muxer, sink, transport, retry, and diagnostics state for one output.

### Output Router

The service that creates, coordinates, and isolates output pipelines.

### Encoder Session

One active hardware or software encoder lifetime.

### Muxer

The component combining encoded streams into a container or protocol format.

### Sink

The final destination for output data, such as file, network transport, or virtual device.

### Backpressure

A controlled response when a downstream stage cannot accept work at the current rate.

---

## 8. Persistence Terms

### Project Schema

The stable serialized representation and semantic rules for a project version.

### Migration

A deterministic transformation from an older supported schema to a newer schema.

### Autosave

A non-destructive periodic or event-driven recovery save separate from the user's explicit save operation.

### Recovery Snapshot

A project representation retained to restore work after interruption or corruption.

### Atomic Save

A save whose visible result is either the complete previous version or the complete new version, never a partial file.

### Asset Registry

The project-level mapping of asset identities to locations, hashes, metadata, and availability.

---

## 9. Platform Terms

### Platform Adapter

A platform-specific implementation of a domain-defined interface.

### Capability

A supported feature reported by the current platform, device, backend, or extension permission context.

### Permission

User or operating-system authorization required for an operation.

### Entitlement

A platform packaging or signing declaration granting application access to a protected capability.

### Device Worker

An optional isolated process handling unstable, privileged, or vendor-specific device integration.

---

## 10. Extension Terms

### Extension

A separately distributed component that adds behavior through the supported SDK.

### Extension Host

The isolated process or runtime supervising extensions.

### Manifest

The declarative extension metadata, API version, entry points, and requested capabilities.

### Capability Grant

A user- and policy-approved permission allowing an extension to call a defined API group.

### Sandbox

The technical restrictions limiting extension filesystem, network, process, memory, and host access.

---

## 11. Terms to Avoid

| Avoid | Use |
|---|---|
| Widget for scene content | Scene item or source |
| Layer when referring to persisted composition | Scene item |
| Backend for the whole engine | Engine |
| Frontend as authoritative state | Control UI |
| Plugin running in engine memory | Extension |
| Save cache | Autosave or recovery snapshot |
| Frame buffer when meaning replay | Replay buffer |
| Render tree | Scene graph or render graph, depending on meaning |
| Device ID without scope | Platform device identifier or stable device reference |
| Sync time | Master clock, timestamp, drift, or synchronization state |

---

## 12. Naming Rule

A new foundational term must be added here before it appears in a stable public API or schema.

If two terms appear interchangeable, the owning subsystem specification must distinguish them or select one canonical term.
