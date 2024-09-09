# applications-rs

[![Crates.io Version](https://img.shields.io/crates/v/applications)](https://crates.io/crates/applications)

> This crate is created for project [kunkun](https://github.com/kunkunsh/kunkun) to
>
> - get a list of installed applications on the system
> - get the frontmost application
> - get a list of running applications

## Platforms

- [x] Mac
- [x] Linux
- [x] Windows

> [!WARNING]
> Linux and Windows support don't have full support yet. e.g. `get_running_apps()` only works on Mac.
> This crate will be maintained and improved as development of [kunkun](https://github.com/kunkunsh/kunkun) progress



## Usage

```rust
use applications::{AppInfoContext, AppInfo};

fn main() {
    let mut ctx = AppInfoContext::new();
    ctx.refresh_apps().unwrap(); // must refresh apps before getting them

    let apps = ctx.get_all_apps();
    println!("Apps: {:#?}", apps);

    let frontmost_app = ctx.get_frontmost_application().unwrap();
    println!("Frontmost App: {:#?}", frontmost_app);

    let running_apps = ctx.get_running_apps();
    println!("Running Apps: {:#?}", running_apps);
}
```

## How?

> How and where to search for available desktop applications on each platform?

### Linux

Desktop applications are specified in files that ends with `.desktop`. `echo $XDG_DATA_DIRS` to see a list of paths where these desktop files could reside in.

The `.desktop` files are in toml format. Parse them with [toml](https://crates.io/crates/toml) crate.

The `Exec` can be used to launch the app, and `Icon` field contains the app icon.

### MacOS

The simplest way is to search in `/Applications` folder. The app icon is in `.icns` format.
Apple silicon macs can now run iOS apps. iOS app icons are in `.png` format.

`system_profiler` command can be used to get installed applications.

`system_profiler SPApplicationsDataType` is the command to use to get a full list of applications.

### Windows

https://crates.io/crates/winreg could be useful.

## Libraries

- https://crates.io/crates/icns: Read and write Mac icns files, convert into PNG format.
