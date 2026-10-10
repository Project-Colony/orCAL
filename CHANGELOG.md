# Changelog

## [0.3.1](https://github.com/Project-Colony/orCAL/compare/v0.3.0...v0.3.1) (2026-10-09)


### Bug Fixes

* evaluate scientific expressions in the Rust engine and lock the webview down with a content security policy ([#36](https://github.com/Project-Colony/orCAL/issues/36)) ([faf1048](https://github.com/Project-Colony/orCAL/commit/faf1048a6505532d50d1d7de5bc2b248160ea10e))
* **ui:** show results at full precision and keep percent exact ([#39](https://github.com/Project-Colony/orCAL/issues/39)) ([92c2af5](https://github.com/Project-Colony/orCAL/commit/92c2af5b344106e80608c1d1144a70ac1d89b3ec))

## [0.3.0](https://github.com/Project-Colony/orCAL/compare/v0.2.2...v0.3.0) (2026-10-08)


### Features

* migrate to Tauri 2 ([#32](https://github.com/Project-Colony/orCAL/issues/32)) ([98e1fd2](https://github.com/Project-Colony/orCAL/commit/98e1fd2787b56111783fc2657025e21e8bececb0))

## [0.2.2](https://github.com/Project-Colony/orCAL/compare/v0.2.1...v0.2.2) (2026-10-08)


### Bug Fixes

* **release:** sign and publish through the shared Colony workflow ([#30](https://github.com/Project-Colony/orCAL/issues/30)) ([9db6b1a](https://github.com/Project-Colony/orCAL/commit/9db6b1ae5d2c9d1e170e9a9388633e5c0306cdfc))

## [0.2.1](https://github.com/Project-Colony/orCAL/compare/v0.2.0...v0.2.1) (2026-08-01)


### Bug Fixes

* **ci:** tell gh which repository to upload the signatures to ([#20](https://github.com/Project-Colony/orCAL/issues/20)) ([5a675a5](https://github.com/Project-Colony/orCAL/commit/5a675a563f5dd27cbbf4a891773e83088e76ba69))

## [0.2.0](https://github.com/Project-Colony/orCAL/compare/v0.1.3...v0.2.0) (2026-08-01)


### Features

* **ci:** sign release assets with the Project-Colony org key ([b16621d](https://github.com/Project-Colony/orCAL/commit/b16621d0946322f242f56db913aef6a00c4dc909))
* declare signed releases in the manifest ([47a90a1](https://github.com/Project-Colony/orCAL/commit/47a90a16bc8c20585b6def47f867ae63db71bb2e))

## [0.1.3](https://github.com/Project-Colony/orCAL/compare/v0.1.2...v0.1.3) (2026-07-13)


### Bug Fixes

* **linux:** pin webkit2gtk to 0.18.0, the version wry compiles against ([#15](https://github.com/Project-Colony/orCAL/issues/15)) ([43cfa7c](https://github.com/Project-Colony/orCAL/commit/43cfa7c8d0adbc7ad925acab95915065818d1c40))
* **linux:** update wry to 0.24.12 so the webkitgtk backend compiles ([#17](https://github.com/Project-Colony/orCAL/issues/17)) ([8515194](https://github.com/Project-Colony/orCAL/commit/8515194b2a524db4bd54fca39653169887e432c9))

## [0.1.2](https://github.com/Project-Colony/orCAL/compare/v0.1.1...v0.1.2) (2026-07-13)


### Bug Fixes

* **security:** compile only the window APIs instead of api-all ([#11](https://github.com/Project-Colony/orCAL/issues/11)) ([f5554dc](https://github.com/Project-Colony/orCAL/commit/f5554dcfad7c6f5763deb503a9005859d28e7d9b))

## [0.1.1](https://github.com/Project-Colony/orCAL/compare/v0.1.0...v0.1.1) (2026-07-13)


### Bug Fixes

* **security:** restrict the Tauri API allowlist to what the UI uses ([#5](https://github.com/Project-Colony/orCAL/issues/5)) ([4b7baa4](https://github.com/Project-Colony/orCAL/commit/4b7baa4c037d6214e18602c9a3287c011e95d442))

Future entries are generated automatically by
[release-please](https://github.com/googleapis/release-please) from
[Conventional Commits](https://www.conventionalcommits.org/).

Pre-automation binaries were published manually under the `Linux` and
`Windows` tags (February 2026).
