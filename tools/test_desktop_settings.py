"""Check settings persistence and real AppKit actions without opening a window.

Run after GEOTHITE_DESKTOP_PROFILE=wasm-dev just desktop-build.
Reuse development dependencies: release libraries use panic=abort and cannot link
into the standalone unit-test harness.
"""
from pathlib import Path
import os
import platform
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / "crates/crystal-bevy/src/bevy_shell/desktop_settings"
TARGET = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target/desktop-build"))
PROFILE = os.environ.get("GEOTHITE_DESKTOP_PROFILE", "wasm-dev")
DEPS = TARGET / ("debug" if PROFILE == "dev" else PROFILE) / "deps"


def library(name, version=None):
    for dependency in sorted(DEPS.glob(f"{name}-*.d"), key=lambda path: path.stat().st_mtime, reverse=True):
        if version and f"/{name.replace('_', '-')}-{version}/" not in dependency.read_text():
            continue
        library = DEPS / f"lib{dependency.stem}.rlib"
        if library.exists():
            return library
    raise RuntimeError(f"Missing {name} dependency in {DEPS}; run just desktop-build first.")


def compile_and_run(path, binary, libraries, test=False):
    command = ["rustc", "--edition=2024", "-A", "dead_code", str(path), "-L", f"dependency={DEPS}", "-o", str(binary)]
    if test:
        command.append("--test")
    for name, version in libraries:
        command.extend(["--extern", f"{name}={library(name, version)}"])
    subprocess.run(command, check=True)
    subprocess.run([str(binary)], check=True)


def main():
    with tempfile.TemporaryDirectory(prefix=".settings-check-", dir=TARGET) as temporary:
        temporary = Path(temporary)
        state = f'#[path = r#"{SOURCE / "state.rs"}"#] mod state;\n'
        tests = temporary / "tests.rs"
        tests.write_text(state + f'#[path = r#"{SOURCE / "store.rs"}"#] mod store;\n')
        compile_and_run(tests, temporary / "tests", [("serde_json", None)], test=True)
        if platform.system() != "Darwin":
            return
        smoke = temporary / "appkit.rs"
        native_source = (SOURCE / "macos.rs").read_text().replace("//!", "//")
        smoke.write_text(state + 'mod macos {\n#[link(name = "QuartzCore", kind = "framework")] unsafe extern "C" {}\n' + native_source + r'''
pub fn smoke() {
    unsafe {
        let app: *mut AnyObject = msg_send![class!(NSApplication), sharedApplication];
        // Prohibit Dock activation and focus changes during offscreen checks.
        let _: bool = msg_send![app, setActivationPolicy: 2_isize];
        let target = target();
        let menus = [Menu::new(&target, false), Menu::new(&target, true)];
        let mut prefs = Preferences::default();
        for action in [Action::Volume(30), Action::Mute, Action::ToggleFirstPerson,
            Action::ToggleFirstPerson, Action::ToggleFirstPerson, Action::Toggle3d,
            Action::ToggleFirstPerson, Action::Camera(Preset::Wide), Action::Fullscreen, Action::Reset] {
            let item = &menus[0].items.iter().find(|(value, _)| *value == action).unwrap().1;
            let parent: *mut AnyObject = msg_send![item, menu];
            let index: isize = msg_send![parent, indexOfItem: &**item];
            let _: () = msg_send![parent, performActionForItemAtIndex: index];
            assert_eq!(take_actions(), vec![action], "native target/action must dispatch exactly once");
            prefs.apply(action, [1.0, 0.0]);
            for menu in &menus {
                menu.sync(prefs, true);
                let mute = &menu.items.iter().find(|(action, _)| *action == Action::Mute).unwrap().1;
                let checked: isize = msg_send![mute, state];
                assert_eq!(checked == 1, prefs.muted, "both menu copies mirror the effective preference");
                let first_person = &menu.items.iter().find(|(action, _)| *action == Action::ToggleFirstPerson).unwrap().1;
                let checked: isize = msg_send![first_person, state];
                assert_eq!(checked == 1, prefs.view_3d && prefs.camera.first_person,
                    "both native menus mirror the active first-person toggle");
            }
        }
        menus[0].sync(prefs, false);
        let view = &menus[0].items.iter().find(|(action, _)| *action == Action::Toggle3d).unwrap().1;
        let enabled: bool = msg_send![view, isEnabled];
        assert!(!enabled, "a client without the renderer must not offer an ineffective view toggle");
        let first_person = &menus[0].items.iter().find(|(action, _)| *action == Action::ToggleFirstPerson).unwrap().1;
        let enabled: bool = msg_send![first_person, isEnabled];
        assert!(!enabled, "first person requires the 3D renderer");
        for _ in 0..80 { selected(&target, sel!(geothiteSettings:), view); }
        assert_eq!(take_actions().len(), 64, "nested menu event queues remain bounded");
        let allocated: *mut AnyObject = msg_send![class!(NSWindow), alloc];
        let frame = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(320.0, 288.0));
        let window = Retained::<AnyObject>::from_raw(msg_send![allocated,
            initWithContentRect: frame styleMask: 3_usize backing: 2_usize defer: false]).unwrap();
        let _: () = msg_send![&window, setReleasedWhenClosed: false];
        let _: () = msg_send![&window, setTitle: &*NSString::from_str("Settings offscreen test")];
        let visible: bool = msg_send![&window, isVisible];
        assert!(!visible, "the regression must never order a window onscreen");
        let menubar = menu("Offscreen test");
        let _: () = msg_send![app, setMainMenu: &*menubar];
        let view: *mut AnyObject = msg_send![&window, contentView];
        let _: () = msg_send![view, setWantsLayer: true];
        let layer: *mut AnyObject = msg_send![view, layer];
        let frame_before: NSRect = msg_send![view, frame];
        let drawable = Retained::<AnyObject>::from_raw(msg_send![class!(CAMetalLayer), new]).unwrap();
        let _: () = msg_send![layer, addSublayer: &*drawable];
        let native = NativeMenus::new("Settings offscreen test")
            .expect("settings attach before a window becomes main or visible");
        native.sync(prefs, true);
        let button: *mut AnyObject = msg_send![&native.accessory, view];
        let refuses_focus: bool = msg_send![button, refusesFirstResponder];
        assert!(refuses_focus, "a native popup must not take game keyboard ownership");
        let _: () = msg_send![&window, displayIfNeeded];
        let current_parent: *mut AnyObject = msg_send![&drawable, superlayer];
        assert_eq!(current_parent, layer, "titlebar controls must preserve the Metal sublayer");
        let current_view: *mut AnyObject = msg_send![&window, contentView];
        let current_layer: *mut AnyObject = msg_send![current_view, layer];
        let frame_after: NSRect = msg_send![current_view, frame];
        assert_eq!(current_view, view, "titlebar controls must preserve the game content view");
        assert_eq!(current_layer, layer, "titlebar controls must preserve the renderer layer");
        assert_eq!(frame_before, frame_after, "titlebar controls must preserve the drawable frame");
        let accessories: *mut AnyObject = msg_send![&window, titlebarAccessoryViewControllers];
        let count: usize = msg_send![accessories, count];
        assert_eq!(count, 1);
        drop(native);
        let accessories: *mut AnyObject = msg_send![&window, titlebarAccessoryViewControllers];
        let count: usize = msg_send![accessories, count];
        assert_eq!(count, 0, "shutdown detaches the titlebar accessory before its target drops");
        let visible: bool = msg_send![&window, isVisible];
        assert!(!visible);
        let _: () = msg_send![&window, close];
    }
    println!("AppKit menu callbacks, mirrored checkmarks and bounded actions passed offscreen.");
}
}
fn main() { objc2::rc::autoreleasepool(|_| macos::smoke()); }
''')
        compile_and_run(smoke, temporary / "appkit", [("objc2", "0.5.2"), ("objc2_foundation", "0.2.2")])


if __name__ == "__main__":
    main()
