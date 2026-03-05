# Changelog

## [v1.1.0] - 2026-03-05

### New Features

#### Null Fallback for Unresolved Mapper Fields

The mapper processor now returns `null` when a template reference points to a field that does not exist in the event, instead of falling through to string evaluation.

**Before:**
```json
{ "missingField": "{{ issue.nonexistent }}" }
```
Previously this would produce an empty or unexpected string value.

**After:**
```json
{ "missingField": null }
```

This makes downstream field handling predictable: consumers can now reliably check for `null` to detect absent source fields.

### Dependencies

- Bump `bytes` 1.11.0 → 1.11.1
- Bump `time` 0.3.44 → 0.3.47
- Bump `keccak` 0.1.5 → 0.1.6

---

## [v1.0.0] - 2026-01-14

### New Features

#### Type Casting Support in Mapper

Added explicit type casting functionality to the mapper processor, allowing you to convert values between string and number types using the `castTo` property:

```json
{
  "issueId": {
    "value": "{{ issue.id }}",
    "castTo": "number"
  },
  "priorityLabel": {
    "value": "{{ issue.fields.priority }}",
    "castTo": "string"
  }
}
```

**Supported cast types:**
- `string` — Converts numbers and booleans to strings
- `number` — Parses strings as integers or floats (e.g., `"123"` → `123`, `"45.67"` → `45.67`)

### Bug Fixes & Improvements

- Fixed color output in logs by detecting terminal vs non-terminal environments
- Logs now automatically disable ANSI colors in non-terminal environments (e.g., Docker, CI/CD)

### CI/CD

- ARM64 Docker builds now only run for tagged releases (`refs/tags/v*`)
- AMD64 builds run for all commits, significantly speeding up CI for non-release branches

### Dependencies

- Added `atty` v0.2 for TTY detection
