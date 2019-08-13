# Changelog

This project adheres to [Semantic Versioning]. All notable changes to it are documented in this file, which is auto-generated using [Conventional Commits], and whose format is based on [Keep a Changelog].

[keep a changelog]: https://keepachangelog.com/en/1.0.0/
[semantic versioning]: https://semver.org/spec/v2.0.0.html
[conventional commits]: https://www.conventionalcommits.org/en/v1.0.0-beta.4/

## Overview

- [Unreleased](#unreleased)

## _[Unreleased]_

Changes that have been made since the last release.

### Miscellaneous

- deps: Update cargo dependencies ([`62a9a3f`])
- deps: Rust 1.35 + Gtk-rs 0.7 ([`eb494e0`])
- deps: system76-firmware ([`5f1bfcf`])
- deps: system76-firmware ([`326615a`])
- deps: fwupd-dbus ([`2714b3d`])
- gtk: Remove last-remaining usage of Rc in the lib ([`5530069`])
- gtk: Remove TODO comment that's no longer valid ([`ea8d024`])

### Bug Fixes

- build: Builds without Makefile now build with fwupd + system76 features by default ([`ec3eb63`])
- build: Fix the bionic build ([`cfcaac7`])
- core: Activate fwupd with dbus ping ([`9ec6efd`])
- core: Handle network connection errors ([`64230ff`])
- core: Collapse Thelio I/O devices into one ([`d503c98`])
- core: Fix latest version # for Thelio I/O boards ([`71f86e0`])
- core: Fix parsing of changelog for Thelio Major R1 ([`884b363`])
- core: Thelio I/O boards will number from 1, not 0 ([`825693d`])
- fwupd: Only update enabled remotes ([`ba81fc4`])
- fwupd: Fix fwupd system detection ([`cc04be5`])
- fwupd: Further improvements to fwupd support ([`e3f4c77`])
- gtk: Add a waiting label widget to device widget stack ([`29c2412`])
- gtk: Improve the layout of revealer changelogs ([`7a1c66f`])
- gtk: Fix battery detection and battery message text ([`340ffd2`])
- gtk: Fix misalignment of firmware dialog ([`85d9686`])
- gtk: Fix the waiting progress label ([`776253d`])
- gtk: Remove button margin causing button to shrink ([`1f73db6`])
- gtk: Ensure homogenous device entity box heights ([`cc8859c`])
- gtk: Fix display of progress bar and its label ([`dbaa010`])
- gtk: Fix alignment of device labels ([`dd5d257`])
- gtk: Fix buttons to allow shadow without clipping ([`8c041d1`])
- gtk: Show 100% before hiding the progress bar ([`3bfd71f`])
- gtk: Default progress bars to "Waiting" after click ([`9912ef4`])
- gtk: Only show battery message when a battery is detected ([`3beb737`])
- gtk: Set margins for changelogs in dropdown ([`794c741`])
- gtk: Show the widget by default ([`15370fc`])
- gtk: Fix the dynamic width of the firmware content container ([`2d716de`])
- gtk: System76 changelog places most-recent version first ([`a79e5a7`])
- gtk: Display the "not found" view only when firmware was not found ([`14ac000`])
- gtk: Progress bar event loop now handles the `Disconnected` signal ([`b488a41`])
- gtk: Put all requires reboot devices into System Firmware ([`44ad330`])
- gtk: Allow the background thread to gracefully exit on drop ([`4794f83`])
- gtk: Fix GTK application window size ([`12a7695`])
- gtk: Display fwupd devices even if no updates are available ([`8f6fcd4`])
- notify: Fix update boolean expression ([`01a616e`])
- notify: Actually check if discovered firmware is updateable ([`5fb0b53`])
- notify: Make the notification non-resident, so it closes on click ([`861f90d`])
- notify: Also update fwupd remote cache in the notify binary ([`5dbd1c9`])
- packaging: Update debian packaging ([`3a9ebaf`])
- packaging: Fix the CI ([`6e8f5e8`])
- packaging: Generate the Cargo manifests w/ Makefile to fix conditional compilation ([`905f8c2`])
- packaging: Add postinst script to start fwupd service if it is inactive ([`bada25a`])
- packaging: Have libfirmware-manager depend on -notify too ([`7d7a383`])
- packaging: Remove System76 from application & .desktop files ([`7e52066`])
- packaging: Fix conflict in debian packaging ([`e066028`])
- packaging: Ensure fwupd is started at init ([`0b6cecf`])
- packaging: Fix the pkconfig generation ([`a7cea63`])
- packaging: Fix the pkgconfig generation ([`883c492`])
- packaging: Ensure that build tools aren't building in release mode ([`533a0a3`])
- packaging: Fix the debian packaging ([`2285e77`])

### Features

- core: Integrate fwupd-dbus client for fwupd support ([`e2d8e2e`])
- core: Use slotmap EC arch to manage device widgets ([`c0c9b16`])
- fwupd: Update the fwupd remote cache every 2 weeks ([`08f7102`])
- gtk: Refresh GTK widget after error dismissal ([`f74733a`])
- gtk: Add indicator for dropdown state ([`52e0817`])
- gtk: Hide device and system firmware sections until they are found ([`3681390`])
- gtk: Use a GTK revealer, instead of dialogs, for changelogs ([`317080d`])
- gtk: Handle Ctrl + Q in the standalone application ([`5749234`])
- gtk: Seamless navigation between system and device list boxes ([`ad20678`])
- gtk: Improved keyboard navigation ([`4a56807`])
- gtk: Accurate progress bars on devices ([`ab72570`])
- gtk: Display dialog with changelogs when clicking devices ([`527ccfc`])
- gtk: Allow multiple system firmware to be displayed ([`f1c137b`])
- gtk: Add refresh button to the application ([`c7ec88e`])
- gtk: Implemented the GTK Firmware Widget ([`e19f325`])
- notify: Create firmware-manager-notify package ([`79348e1`])
- packaging: Add firmware-manager-notify as dependency ([`46940bf`])
- packaging: systemd support for periodic update checks ([`8c32296`])
- packaging: Conditional compilation; Makefile improvements; Updated README.md ([`a2b5a00`])
- packaging: Use freedesktop-desktop-entry to generate desktop entry ([`34be9b3`])
- packaging: Create the desktop entry ([`c89c709`])


[unreleased]: https://github.com/pop-os/firmware-manager/commits


[`5530069`]: https://github.com/pop-os/firmware-manager/commit/553006989ec0a2f97d972fbca5ca916f1a4b8013
[`9ec6efd`]: https://github.com/pop-os/firmware-manager/commit/9ec6efdc9823a65587bd8b16663fdf897d26a5f4
[`3a9ebaf`]: https://github.com/pop-os/firmware-manager/commit/3a9ebafabe35b3cf22264223e5590de9e25cc163
[`ea8d024`]: https://github.com/pop-os/firmware-manager/commit/ea8d0249739160123cdaa84e2870f81a1808ac30
[`62a9a3f`]: https://github.com/pop-os/firmware-manager/commit/62a9a3f0d5dc115baaa18a814e88da49a8f05895
[`01a616e`]: https://github.com/pop-os/firmware-manager/commit/01a616e1d3c460501279456981cdd9ead9056de5
[`f74733a`]: https://github.com/pop-os/firmware-manager/commit/f74733a45fb2bdcba89a36339487835372bbfc17
[`6e8f5e8`]: https://github.com/pop-os/firmware-manager/commit/6e8f5e87d9a9e01784a0aca62c0a1e6eb72e9367
[`905f8c2`]: https://github.com/pop-os/firmware-manager/commit/905f8c205cef9b39d16c00fedfc84065bfcd49d6
[`64230ff`]: https://github.com/pop-os/firmware-manager/commit/64230ffa65b7609ed2c5a7618319adba337e0e1c
[`29c2412`]: https://github.com/pop-os/firmware-manager/commit/29c2412bcd875118d7145e32caf7b4df8748238a
[`bada25a`]: https://github.com/pop-os/firmware-manager/commit/bada25a517df74b5198ae1ae49cddd27f58a0001
[`7d7a383`]: https://github.com/pop-os/firmware-manager/commit/7d7a383a3ce2ea383e2498e4650891bca5589345
[`7a1c66f`]: https://github.com/pop-os/firmware-manager/commit/7a1c66feea217481aa8ec3f0e2934408b68e6dcf
[`340ffd2`]: https://github.com/pop-os/firmware-manager/commit/340ffd2a74c702a7cab71ee7b544aa871a49946e
[`46940bf`]: https://github.com/pop-os/firmware-manager/commit/46940bf714762ddeee276a147469cbc4cb5d6df2
[`85d9686`]: https://github.com/pop-os/firmware-manager/commit/85d968681ae34467ec445045ce58f67f57589c1e
[`ec3eb63`]: https://github.com/pop-os/firmware-manager/commit/ec3eb6367181950ebe8160b98492fbe8136b07e3
[`776253d`]: https://github.com/pop-os/firmware-manager/commit/776253da61049e59ebc190fe2d773f447d4d09bc
[`7e52066`]: https://github.com/pop-os/firmware-manager/commit/7e52066f1bbdd4b697c8b1ed993776ebffef4492
[`1f73db6`]: https://github.com/pop-os/firmware-manager/commit/1f73db67b0d6bf3316c2b493c292b478366e5a0b
[`e066028`]: https://github.com/pop-os/firmware-manager/commit/e066028ae043cd7b6e36a1fd620b430895f6943e
[`cc8859c`]: https://github.com/pop-os/firmware-manager/commit/cc8859c2a9d2058eb4eca0f3fd2d91cdbd3b88f1
[`dbaa010`]: https://github.com/pop-os/firmware-manager/commit/dbaa010a776840290dd42b61c303ca45d88adf7f
[`dd5d257`]: https://github.com/pop-os/firmware-manager/commit/dd5d2573ca4f9a377801735805c2b07d4cc9a5c9
[`8c041d1`]: https://github.com/pop-os/firmware-manager/commit/8c041d13b5331f8a7782032e740141e22b52b134
[`3bfd71f`]: https://github.com/pop-os/firmware-manager/commit/3bfd71f260c7a37f31409fda530dd2ae2774b818
[`d503c98`]: https://github.com/pop-os/firmware-manager/commit/d503c983de50b429b349e336cdd5517466c7939c
[`9912ef4`]: https://github.com/pop-os/firmware-manager/commit/9912ef48b9abdc9ef3977374434b1a0dc678a85e
[`3beb737`]: https://github.com/pop-os/firmware-manager/commit/3beb7375f88301f695df4b16aabb6ce1caf6be9f
[`794c741`]: https://github.com/pop-os/firmware-manager/commit/794c741872e400b86ce1ec2ae7f63cbc5acf962c
[`52e0817`]: https://github.com/pop-os/firmware-manager/commit/52e08177b5788f36fc8d95958974708d9c456be2
[`0b6cecf`]: https://github.com/pop-os/firmware-manager/commit/0b6cecf270f7252f0a0b83d164611f95b0a38ca4
[`8c32296`]: https://github.com/pop-os/firmware-manager/commit/8c32296e5283a446eecf303df980e720cd2ede56
[`15370fc`]: https://github.com/pop-os/firmware-manager/commit/15370fc21a7c9e27816f02ab274523bb1699ccf0
[`2d716de`]: https://github.com/pop-os/firmware-manager/commit/2d716decb30fdbc3609b00acf69782525c2d5dd2
[`5fb0b53`]: https://github.com/pop-os/firmware-manager/commit/5fb0b53fd2b918b9af7ae62b31a855d356b6b4e4
[`3681390`]: https://github.com/pop-os/firmware-manager/commit/3681390fb9acb5dc65f38445aac1a930518d28b7
[`317080d`]: https://github.com/pop-os/firmware-manager/commit/317080d2e6c955ef1a4a0903fe7dcf9c637ade87
[`861f90d`]: https://github.com/pop-os/firmware-manager/commit/861f90d1669b08ed4f202532d8f4811e4f94cee5
[`5749234`]: https://github.com/pop-os/firmware-manager/commit/57492343f5a0aeddc124079b646a6e33937253fa
[`a79e5a7`]: https://github.com/pop-os/firmware-manager/commit/a79e5a7569405db4d22adf94247fa54805410e0b
[`71f86e0`]: https://github.com/pop-os/firmware-manager/commit/71f86e0f2169b2e3eb49dd475e13aefaf055c0dd
[`ad20678`]: https://github.com/pop-os/firmware-manager/commit/ad20678f348732c3543fffecbeca85ce6d8406ba
[`4a56807`]: https://github.com/pop-os/firmware-manager/commit/4a56807687257cf2357a695463979c184c7c3cd9
[`ba81fc4`]: https://github.com/pop-os/firmware-manager/commit/ba81fc46c1a9650df1ca05208510da95a8b32569
[`a7cea63`]: https://github.com/pop-os/firmware-manager/commit/a7cea6310c2289caddef974f2c570325b64ededf
[`ab72570`]: https://github.com/pop-os/firmware-manager/commit/ab72570390fbb2016b3348c59718a4b5bde27688
[`883c492`]: https://github.com/pop-os/firmware-manager/commit/883c492e0dbf6cc36807386ac58e8fff35a11feb
[`533a0a3`]: https://github.com/pop-os/firmware-manager/commit/533a0a3c66181a558dca87569460b9e9f17077d4
[`884b363`]: https://github.com/pop-os/firmware-manager/commit/884b36386149bfb2d5a98b7da0c06f5fd124474a
[`527ccfc`]: https://github.com/pop-os/firmware-manager/commit/527ccfc297a8870bccc1ee075e93ccee307cda26
[`825693d`]: https://github.com/pop-os/firmware-manager/commit/825693df3919804ad1ec8751a93998220ee2656a
[`14ac000`]: https://github.com/pop-os/firmware-manager/commit/14ac000cf917659cf714fe752c413c0b99b411a7
[`b488a41`]: https://github.com/pop-os/firmware-manager/commit/b488a419e0c112349b4c32aea7fc9910e0425cc1
[`5dbd1c9`]: https://github.com/pop-os/firmware-manager/commit/5dbd1c9faa6697a45c18f08a7342529791522735
[`08f7102`]: https://github.com/pop-os/firmware-manager/commit/08f7102760b6c9995d194d69c1b957987a430e3d
[`f1c137b`]: https://github.com/pop-os/firmware-manager/commit/f1c137b3ccfabf4627e9a5252132e9dd50e67eb6
[`44ad330`]: https://github.com/pop-os/firmware-manager/commit/44ad33056f68128204a8e9ed2a79d36ec18373db
[`cc04be5`]: https://github.com/pop-os/firmware-manager/commit/cc04be54db16aacc7c8b2c569254875be76cb893
[`2285e77`]: https://github.com/pop-os/firmware-manager/commit/2285e770c130802ea8fa966a66d19d5e1206bd32
[`4794f83`]: https://github.com/pop-os/firmware-manager/commit/4794f83dd7afd4e197c6f26eac0c6b676c691e6a
[`eb494e0`]: https://github.com/pop-os/firmware-manager/commit/eb494e0d73211b675de8b9119b800b883dfe0095
[`79348e1`]: https://github.com/pop-os/firmware-manager/commit/79348e14456a176b3d2063246566d0f0718d6bdf
[`a2b5a00`]: https://github.com/pop-os/firmware-manager/commit/a2b5a00cc193d5306881c93d6a74a67b103ec20a
[`34be9b3`]: https://github.com/pop-os/firmware-manager/commit/34be9b3479885ff6fbcf04480b41a53750dc80d1
[`c89c709`]: https://github.com/pop-os/firmware-manager/commit/c89c70966820887e9948ec94ef4edacc2270e9fd
[`5f1bfcf`]: https://github.com/pop-os/firmware-manager/commit/5f1bfcf8f31fb3b9f703fc7c38780a6e10ca1fc9
[`c7ec88e`]: https://github.com/pop-os/firmware-manager/commit/c7ec88e13569ebeed80968222b1dcf92d4b976e1
[`326615a`]: https://github.com/pop-os/firmware-manager/commit/326615a2d87c8e5dde5ffaff3758b03eec4ef895
[`e3f4c77`]: https://github.com/pop-os/firmware-manager/commit/e3f4c77c8292288ec5407b16346ac420e06d2f36
[`12a7695`]: https://github.com/pop-os/firmware-manager/commit/12a769516a1b93ec894ad02f518ceedc99e884fc
[`2714b3d`]: https://github.com/pop-os/firmware-manager/commit/2714b3d762295a915db2f37ee7fa2221f1ba85e9
[`8f6fcd4`]: https://github.com/pop-os/firmware-manager/commit/8f6fcd476d4b8303ecc219184dfec56c5dd71391
[`e2d8e2e`]: https://github.com/pop-os/firmware-manager/commit/e2d8e2e1e08b86cb43bf944ef353f2a3949db2b6
[`c0c9b16`]: https://github.com/pop-os/firmware-manager/commit/c0c9b16a097c3204c2d7b50c7ecbd2b48a1b82a2
[`cfcaac7`]: https://github.com/pop-os/firmware-manager/commit/cfcaac7c736e432035d6abe185e599d4842024f8
[`e19f325`]: https://github.com/pop-os/firmware-manager/commit/e19f325abe0a903243a99116477d92938ff0799f
<!--
Config(
  github: ( repo: "pop-os/firmware-manager" ),
  accept_types: ["chore", "feat", "fix", "perf"],
  type_headers: {
    "chore": "Miscellaneous",
    "feat": "Features",
    "fix": "Bug Fixes",
    "perf": "Perf. Improvements",
  }
)

Template(
# Changelog

This project adheres to [Semantic Versioning]. All notable changes to it are documented in this file, which is auto-generated using [Conventional Commits], and whose format is based on [Keep a Changelog].

[keep a changelog]: https://keepachangelog.com/en/1.0.0/
[semantic versioning]: https://semver.org/spec/v2.0.0.html
[conventional commits]: https://www.conventionalcommits.org/en/v1.0.0-beta.4/

## Overview

- [Unreleased](#unreleased)

{%- for release in releases %}
- [`{{ release.version }}`](#{{ release.version | replace(from=".", to="") }}) – _{{ release.date | date(format="%Y.%m.%d")}}_
{%- endfor %}

## _[Unreleased]_

Changes that have been made since the last release.

{% if unreleased.changes -%}
  {%- for type, changes in unreleased.changes | group_by(attribute="type") -%}

### {{ type | typeheader }}

{% for change in changes | sort(attribute="scope") -%}
- {% if change.scope %}{{ change.scope }}: {% endif %}{{ change.description }} ([`{{ change.commit.short_id }}`])
{% endfor %}
{% endfor %}
{% else -%}
_nothing new to show for… yet!_

{% endif -%}
{%- for release in releases -%}

## [{{ release.version }}] – _{{ release.title }}_

_{{ release.date | date(format="%Y.%m.%d") }}_

{{ release.notes }}
{%- if release.changeset.contributors %}

### Contributions

This release is made possible by the following people (in alphabetical order).
Thank you all for your contributions. Your work – no matter how significant – is
greatly appreciated by the community.

{% for contributor in release.changeset.contributors -%}

- {{ contributor.name }} (<{{ contributor.email }}>)

{%- endfor %}
{%- endif %}

### Changes

{% for type, changes in release.changeset.changes | group_by(attribute="type") -%}

#### {{ type | typeheader }}

{% for change in changes -%}
- **{{ change.description }}** ([`{{ change.commit.short_id }}`])

{% if change.body -%}
{{ change.body | indent(n=2) }}

{% endif -%}
{%- endfor -%}

{% endfor %}
{%- endfor -%}

{% if config.github.repo -%}
  {%- set url = "https://github.com/" ~ config.github.repo -%}
{%- else -%}
  {%- set url = "#" -%}
{%- endif -%}
{% if releases -%}
[unreleased]: {{ url }}/compare/v{{ releases | first | get(key="version") }}...HEAD
{%- else -%}
[unreleased]: {{ url }}/commits
{%- endif -%}
{%- for release in releases %}
[{{ release.version }}]: {{ url }}/releases/tag/v{{ release.version }}
{%- endfor %}

{% for change in unreleased.changes %}
[`{{ change.commit.short_id }}`]: {{ url }}/commit/{{ change.commit.id }}
{%- endfor -%}
{%- for release in releases %}
{%- for change in release.changeset.changes %}
[`{{ change.commit.short_id }}`]: {{ url }}/commit/{{ change.commit.id }}
{%- endfor -%}
{%- endfor %}
)
-->
