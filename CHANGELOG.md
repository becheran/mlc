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

## [0.16.3] - 2023-11-20

* Fixes issue with throttle parameter

## [0.16.2] - 2023-06-15

## [0.16.1] - 2022-12-19

* Fixed Installation via `cargo install` failed #67

## [0.16.0] - 2022-12-06

* Added config file
* Added workflow command output for github actions #63
* Added format links in vs code console so that ctrl + left click opens the file at right location #60
* Changed report redirects as warnings, unless their destination errors #55
* Fixed wrong first line separator on windows #61
* Fixed set accept encoding headers #52

## [0.15.4] - 2022-08-23

* Fix #54 column line index for files with CR + LF endings
* Update external dependencies

## [0.15.3] - 2022-08-18

* Fix #53 broken docker container

## [0.15.2] - 2022-07-11

## [0.15.1] - 2022-07-07

## [0.15.0] - 2022-07-07

* Changed markdown parser to be CommonMark compatible
* Changed column of detected link to start of tag instead of actual link
* Fixed issue #35 detect link in headlines

## [0.14.3] - 2021-05-15

* Changed throttle for increased performance
* Security upgraded external dependencies
* Fixed #33 link not found near code block

## [0.14.2] - 2021-03-13

* Fixed broken path check if ../ were included on windows file systems

## [0.14.1] - 2021-03-07

## [0.14.0] - 2020-11-11

* Fallback to GET requests if HEAD fails. See <https://github.com/becheran/mlc/issues/28>

## [0.13.12] - 2020-10-26

* Added GitHub action to README.md

## [0.13.11] - 2020-10-26

## [0.13.9] - 2020-10-25

* Fix wrong count output for skipped links

## [0.13.8] - 2020-10-25

* Fix ignore-links
* Add -i short for ignore-links argument
* #23 - Add github action
* Upgrade external dependencies

## [0.13.7] - 2020-09-16

* Fixed #24 - thanks to Alex Melville (Melvillian) for fixing the issue
* Fixed #26 - add user-agent to requests

## [0.13.6] - 2020-08-31

* OSX builds

## [0.13.5] - 2020-08-30

* Fixed http requests to crates.io. Added header fields (#20)

## [0.13.4] - 2020-08-04

## [0.13.3] - 2020-08-04

* Fixed https requests in docker container (#17)

## [0.13.2] - 2020-07-21

## [0.13.1] - 2020-07-21

## [0.13.0] - 2020-07-17

* Added `--throttle` command

## [0.12.0] - 2020-07-15

* Added `--match-file-extension` switch
* Changed check links only once for speed improvement  

## [0.11.2] - 2020-07-13

### Changed

* Improve fs checkup speed

## [0.11.1] - 2020-07-10

### Fixed

* Ignore path parameter

## [0.11.0] - 2020-07-08

### Added

* Ignore files and directories

## [0.10.5] - 2020-07-07

### Fixed

* Allow email address with special chars
* Add html comment support to markdown files

## [0.10.4] - 2020-07-03

### Changed

* No error for unknown URL schemes

### Fixed

* Ref links with hashtag are not classified as error
* Case insensitive mail addresses

## [0.10.3] - 2020-07-02

### Fixed

* Link refs only allowed at beginning of line
* Path separator for os

### Changed

* Allow mails without mailto schema

## [0.10.2] - 2020-07-02

## [0.10.1] - 2020-07-02

## [0.10.0] - 2020-07-01

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
