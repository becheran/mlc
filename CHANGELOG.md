<!-- The changelog shall follow the recommendations described here: https://keepachangelog.com/en/1.0.0/ 
Types for Changes:
- Added
- Changed
- Deprecated
- Removed
- Fixed
- Security
-->

# Changelog

<!-- next-header -->

## [Unreleased] - ReleaseDate

### Changed

* Column number now points to the link directly instead of the markdown link beginning

### Fixed

* Nested link support (Issue #1)

## [0.6.3] - 2019-12-29

### Changed

* Release binaries on GitHub releases instead of GitLab

## [0.6.2] - 2019-12-28

### Removed

* Remove pipeline badge from crates io

## [0.6.1] - 2019-12-28

### Changed

* Speedup for http links. Do create client only once
* Move from GitLab to GitHub

## [0.6.0] - 2019-12-26

### Added

* Mail check support

## [0.5.1] - 2019-12-25

### Fixed

* Inline html link at start of line

## [0.5.0] - 2019-12-25

### Added

* Markup reference link support

## [0.4.2] - 2019-12-24

### Changed

* Description in readme

### Fixed

* Typo

## [0.4.1] - 2019-12-23

### Changed

* Result output formatting

## [0.4.0] - 2019-12-23

### Added

* Change Log
* Code block support in markdown files
* More file markdown endings support (markdown, mkdn,...)

### Fixed

* File extension separator (previously "somefilemd" was also taken as markdown file)

## [0.3.1] - 2019-12-21

### Fixed

* Code cleanup
* Readme update

## [0.3.0] - 2019-12-19

### Added

* First version of markup link checker (previously mlc was another rust lib project)
