//! Windows build detection.
//!
//! On Windows 11 24H2+ (build 26100+), Quick Settings is hosted by `ShellHost.exe`.
//! On Windows 10 1903+ (build 18362+) through Windows 11 23H2, the Action Center / Quick
//! Settings panel is hosted by `ShellExperienceHost.exe` with an earlier `IQuickAction`
//! / `IQuickActionsProvider` vtable layout.

use std::sync::OnceLock;

/// First Windows 10 build with the Action Center brightness slider (1903).
pub const MIN_WIN10_ACTION_CENTER_BUILD: u32 = 18362;
/// First Windows 11 build (whose `QuickActionsDataModel.dll` uses the Win11 `IQuickAction` vtable).
pub const MIN_WIN11_BUILD: u32 = 22000;
/// First build whose Quick Settings panel is hosted by `ShellHost.exe` (Windows 11 24H2).
pub const MIN_QUICK_SETTINGS_BUILD: u32 = 26100;

#[repr(C)]
struct OsVersionInfoW {
    size: u32,
    major: u32,
    minor: u32,
    build: u32,
    platform_id: u32,
    csd_version: [u16; 128],
}

#[link(name = "ntdll")]
extern "system" {
    /// Unlike `GetVersionEx`, this is not subject to manifest-based version lies.
    fn RtlGetVersion(info: *mut OsVersionInfoW) -> i32;
}

fn detect() -> u32 {
    // Lets the unsupported-system path be exercised on a supported machine.
    if let Some(fake) = std::env::var("BRIGHTTRAY_FAKE_BUILD")
        .ok()
        .and_then(|v| v.trim().parse().ok())
    {
        return fake;
    }
    let mut info = OsVersionInfoW {
        size: std::mem::size_of::<OsVersionInfoW>() as u32,
        major: 0,
        minor: 0,
        build: 0,
        platform_id: 0,
        csd_version: [0; 128],
    };
    if unsafe { RtlGetVersion(&mut info) } == 0 {
        info.build
    } else {
        0
    }
}

/// The Windows build number (e.g. 19045, 22631, 26100), or 0 if it could not be read.
pub fn build() -> u32 {
    static BUILD: OnceLock<u32> = OnceLock::new();
    *BUILD.get_or_init(detect)
}

/// Whether this build uses `ShellHost.exe` (Windows 11 24H2+) rather than `ShellExperienceHost.exe`.
pub fn is_win11_shell_host() -> bool {
    let build = build();
    build == 0 || build >= MIN_QUICK_SETTINGS_BUILD
}

/// Whether `QuickActionsDataModel.dll` uses the Windows 11 `IQuickActionsProvider`/`IQuickAction`
/// vtable slot layout rather than Windows 10's.
pub fn is_win11_quick_actions() -> bool {
    let build = build();
    build == 0 || build >= MIN_WIN11_BUILD
}

/// Target shell host executable name for the current Windows build.
pub fn target_process_name() -> &'static str {
    if is_win11_shell_host() {
        "ShellHost.exe"
    } else {
        "ShellExperienceHost.exe"
    }
}

/// Whether the Quick Settings / Action Center hook can work on this system. An unreadable build
/// number counts as supported: the hook validates everything it patches and gives up safely on its own.
pub fn quick_settings_supported() -> bool {
    let build = build();
    build == 0 || build >= MIN_WIN10_ACTION_CENTER_BUILD
}
