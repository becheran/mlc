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

### Added

* Virtual root dir for easier local testing

## [0.9.3] - 2020-06-24

## [0.9.2] - 2020-05-24

## [0.9.1] - 2020-01-29

### Fixed

* Mailto URI path accepted without double slashes

## [0.9.0] - 2020-01-20

### Changed

* Faster execution with async tasks

### Fixed

* Wildcard parser for excluded links

## [0.8.0] - 2020-01-11

### Added

* HTML support

### Fixed

* No panic for not UTF-8 encoded files

## [0.7.0] - 2020-01-02

### Added

* Reference readme file
* Ignore links option
* No web link option for faster checks without following weblinks

## [0.6.4] - 2019-12-30

### Changed

* Retry with Get for status code 405 Method Not Allowed instead of error
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
