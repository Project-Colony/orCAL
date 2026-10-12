# Changelog

## [0.4.1](https://github.com/Project-Colony/orCAL/compare/v0.4.0...v0.4.1) (2026-10-12)


### Fixes

* keep the webview profile in the cache folder and the Preferences categories on the left in the phone layout ([#49](https://github.com/Project-Colony/orCAL/issues/49)) ([a92468f](https://github.com/Project-Colony/orCAL/commit/a92468f20753952179c089e6a621b31f8fd91535))

## [0.4.0](https://github.com/Project-Colony/orCAL/compare/v0.3.1...v0.4.0) (2026-10-12)


### Features

* add Preferences with Colony themes, fonts, French and accessibility settings, keeping settings under Colony/orCAL ([#42](https://github.com/Project-Colony/orCAL/issues/42)) ([00dd339](https://github.com/Project-Colony/orCAL/commit/00dd339ea551ace8fe975e110d687f8895c27461))


### Fixes

* never save over a preferences file that could not be read, and put the orCAL button top-left ([#44](https://github.com/Project-Colony/orCAL/issues/44)) ([fdb4c10](https://github.com/Project-Colony/orCAL/commit/fdb4c109113e48a9b0971889e9b4ab1f85094547))
* show the whole calculator on Linux, keep it movable and stop the drag crash ([#46](https://github.com/Project-Colony/orCAL/issues/46)) ([214e503](https://github.com/Project-Colony/orCAL/commit/214e5033f71025124effd5ae27ffd7635d790259))
* show Unavailable on About rows the app could not fill instead of leaving them blank ([#47](https://github.com/Project-Colony/orCAL/issues/47)) ([46e6ab1](https://github.com/Project-Colony/orCAL/commit/46e6ab18ea34285fa755cb0be425f0064ef3546c))
* **ui:** say Unavailable on About rows the desktop app could not fill ([46e6ab1](https://github.com/Project-Colony/orCAL/commit/46e6ab18ea34285fa755cb0be425f0064ef3546c))


### Documentation

* rewrite the README on the Colony program template ([#45](https://github.com/Project-Colony/orCAL/issues/45)) ([eebe98e](https://github.com/Project-Colony/orCAL/commit/eebe98e98865ae90d9e8cbcc256031769c93729d))
* say that an unknown theme falls back to Gruvbox dark ([#48](https://github.com/Project-Colony/orCAL/issues/48)) ([8fba622](https://github.com/Project-Colony/orCAL/commit/8fba6223bab1aed8e83cc5311ad9ebb2fecae6d5))

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
