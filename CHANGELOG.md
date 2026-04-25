# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.8.1] - 2026-04-25

### Fixed

- StarCraft was not detected in release builds, leaving the indicator stuck on "Offline"
- Version is now shown in the title bar

## [0.8.0] - 2026-04-25

### Fixed

- Saved player profiles no longer get overwritten with an opponent's data after navigating between profiles
- StarCraft online/offline indicator no longer flickers; transient detection misses are now tolerated and a reconnecting UI immediately learns the current state
- Eliminated the white/gray flash that appeared on app launch before the dark theme was applied

## [0.7.0] - 2025-12-30

### Fixed

- Fixed issues with alias updates

### Changed

- Hiding short replays now blurs instead to prevent continuous scroll/api requests for lag hacker profiles

## [0.6.0] - 2025-12-19

### Added

- Page for saving players

### Fixed

- Various UI issues

## [0.5.0] - 2025-12-18

### Added

- Initial release.
