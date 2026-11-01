//! AppKit owns menu tracking; actions enter a bounded queue for the Bevy thread.
use super::state::{Action, Preferences, Preset};
use objc2::{
    ClassType, class, msg_send,
    rc::Retained,
    runtime::{AnyClass, AnyObject, ClassBuilder, NSObject, Sel},
    sel,
};
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};
use std::{
    collections::VecDeque,
    sync::{Mutex, OnceLock},
};

#[link(name = "AppKit", kind = "framework")]
unsafe extern "C" {}

static PENDING: Mutex<VecDeque<Action>> = Mutex::new(VecDeque::new());

pub(super) fn take_actions() -> Vec<Action> {
    PENDING
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .drain(..)
        .collect()
}

fn action_tag(action: Action) -> isize {
    match action {
        Action::Mute => 1,
        Action::Toggle3d => 2,
        Action::ResetCamera => 3,
        Action::Fullscreen => 4,
        Action::Reset => 5,
        Action::ToggleFirstPerson => 6,
        Action::Camera(preset) => {
            10 + match preset {
                Preset::Default => 0,
                Preset::Close => 1,
                Preset::Wide => 2,
            }
        }
        Action::Volume(volume) => 1000 + isize::from(volume),
    }
}

fn tag_action(tag: isize) -> Option<Action> {
    Some(match tag {
        1 => Action::Mute,
        2 => Action::Toggle3d,
        3 => Action::ResetCamera,
        4 => Action::Fullscreen,
        5 => Action::Reset,
        6 => Action::ToggleFirstPerson,
        10 => Action::Camera(Preset::Default),
        11 => Action::Camera(Preset::Close),
        12 => Action::Camera(Preset::Wide),
        1000..=1100 => Action::Volume((tag - 1000) as u8),
        _ => return None,
    })
}

extern "C" fn selected(_: &AnyObject, _: Sel, sender: &AnyObject) {
    // AppKit invokes this only for the NSMenuItems created below. Never borrow
    // the ECS world from a nested menu-tracking run loop.
    let tag: isize = unsafe { msg_send![sender, tag] };
    if let Some(action) = tag_action(tag) {
        let mut queue = PENDING.lock().unwrap_or_else(|error| error.into_inner());
        if queue.len() == 64 {
            queue.pop_front();
        }
        queue.push_back(action);
    }
}

fn target() -> Retained<AnyObject> {
    static CLASS: OnceLock<&'static AnyClass> = OnceLock::new();
    let class = CLASS.get_or_init(|| {
        let mut builder = ClassBuilder::new("GeothiteSettingsTarget", NSObject::class())
            .expect("unique settings action class");
        // The selector has one object argument, matching the registered ABI.
        unsafe {
            builder.add_method(sel!(geothiteSettings:), selected as extern "C" fn(_, _, _));
        }
        builder.register()
    });
    unsafe {
        let allocated: *mut AnyObject = msg_send![*class, alloc];
        Retained::from_raw(msg_send![allocated, init]).expect("allocate settings target")
    }
}

unsafe fn menu(title: &str) -> Retained<AnyObject> {
    unsafe {
        let allocated: *mut AnyObject = msg_send![class!(NSMenu), alloc];
        let menu =
            Retained::from_raw(msg_send![allocated, initWithTitle: &*NSString::from_str(title)])
                .expect("allocate settings menu");
        let _: () = msg_send![&menu, setAutoenablesItems: false];
        menu
    }
}

unsafe fn item(title: &str, action: Option<Action>, target: &AnyObject) -> Retained<AnyObject> {
    unsafe {
        let allocated: *mut AnyObject = msg_send![class!(NSMenuItem), alloc];
        let selector = action.map(|_| sel!(geothiteSettings:));
        let item = Retained::from_raw(msg_send![allocated,
            initWithTitle: &*NSString::from_str(title)
            action: selector
            keyEquivalent: &*NSString::from_str("")])
        .expect("allocate settings item");
        if let Some(action) = action {
            let _: () = msg_send![&item, setTag: action_tag(action)];
            let _: () = msg_send![&item, setTarget: target];
        }
        item
    }
}

struct Menu {
    root: Retained<AnyObject>,
    items: Vec<(Action, Retained<AnyObject>)>,
    camera_menu_item: Option<Retained<AnyObject>>,
}

impl Menu {
    unsafe fn new(target: &AnyObject, popup: bool) -> Self {
        unsafe {
            let root = menu("Settings");
            let mut result = Self {
                root,
                items: Vec::new(),
                camera_menu_item: None,
            };
            if popup {
                result.add("Settings", None, target);
            }
            let volume = menu("Volume");
            let volume_item = result.add("Volume", None, target);
            let _: () = msg_send![&volume_item, setSubmenu: &*volume];
            result.add_to(&volume, "Mute", Action::Mute, target);
            for level in (0..=100).step_by(10).rev() {
                result.add_to(&volume, &format!("{level}%"), Action::Volume(level), target);
            }
            result.separator();
            result.add("3D View", Some(Action::Toggle3d), target);
            result.add("First Person", Some(Action::ToggleFirstPerson), target);
            let camera = menu("Camera");
            let camera_item = result.add("Camera", None, target);
            let _: () = msg_send![&camera_item, setSubmenu: &*camera];
            result.camera_menu_item = Some(camera_item);
            for (label, preset) in [
                ("Default Orbit", Preset::Default),
                ("Close Orbit", Preset::Close),
                ("Wide Orbit", Preset::Wide),
            ] {
                result.add_to(&camera, label, Action::Camera(preset), target);
            }
            result.add_to(&camera, "Reset Camera", Action::ResetCamera, target);
            result.add("Fullscreen", Some(Action::Fullscreen), target);
            result.separator();
            result.add("Reset Settings", Some(Action::Reset), target);
            result
        }
    }

    unsafe fn add(
        &mut self,
        title: &str,
        action: Option<Action>,
        target: &AnyObject,
    ) -> Retained<AnyObject> {
        unsafe {
            let item = item(title, action, target);
            let _: () = msg_send![&self.root, addItem: &*item];
            if let Some(action) = action {
                self.items.push((action, item.clone()));
            }
            item
        }
    }

    unsafe fn add_to(&mut self, menu: &AnyObject, title: &str, action: Action, target: &AnyObject) {
        unsafe {
            let item = item(title, Some(action), target);
            let _: () = msg_send![menu, addItem: &*item];
            self.items.push((action, item));
        }
    }

    unsafe fn separator(&self) {
        unsafe {
            let separator: *mut AnyObject = msg_send![class!(NSMenuItem), separatorItem];
            let _: () = msg_send![&self.root, addItem: separator];
        }
    }

    fn sync(&self, prefs: Preferences, supports_3d: bool) {
        for (action, item) in &self.items {
            let checked = match *action {
                Action::Volume(volume) => volume == prefs.volume,
                Action::Mute => prefs.muted,
                Action::Toggle3d => prefs.view_3d,
                Action::ToggleFirstPerson => prefs.view_3d && prefs.camera.first_person,
                Action::Camera(preset) => prefs.selected_camera() == Some(preset),
                Action::Fullscreen => prefs.fullscreen,
                _ => false,
            };
            let enabled = supports_3d
                || !matches!(
                    action,
                    Action::Toggle3d
                        | Action::ToggleFirstPerson
                        | Action::Camera(_)
                        | Action::ResetCamera
                );
            // Both menus mirror actual renderer state, including F3/gamepad changes.
            unsafe {
                let _: () = msg_send![item, setState: isize::from(checked)];
                let _: () = msg_send![item, setEnabled: enabled];
                let _: () = msg_send![item, setHidden: !enabled];
            }
        }
        if let Some(item) = &self.camera_menu_item {
            // SAFETY: This is the retained NSMenuItem created with the Camera submenu.
            unsafe {
                let _: () = msg_send![item, setHidden: !supports_3d];
            }
        }
    }
}

/// NonSend ownership keeps every AppKit call and release on the main thread.
pub(super) struct NativeMenus {
    menubar: Retained<AnyObject>,
    root_item: Retained<AnyObject>,
    window: Retained<AnyObject>,
    accessory: Retained<AnyObject>,
    menus: [Menu; 2],
    _target: Retained<AnyObject>,
}

impl NativeMenus {
    pub fn new(title: &str) -> Option<Self> {
        // Called after Winit has created its real window and application menu.
        unsafe {
            let app: *mut AnyObject = msg_send![class!(NSApplication), sharedApplication];
            // Background-launched Winit windows need not be NSApp.mainWindow.
            // Match the production primary window without activating the app.
            let windows: *mut AnyObject = msg_send![app, windows];
            let count: usize = msg_send![windows, count];
            let expected = NSString::from_str(title);
            let mut selected = None;
            for index in 0..count {
                let window: *mut AnyObject = msg_send![windows, objectAtIndex: index];
                let title: *mut AnyObject = msg_send![window, title];
                let matches: bool = msg_send![title, isEqualToString: &*expected];
                if matches {
                    selected = Retained::retain(window);
                    break;
                }
            }
            let window = selected?;
            let menubar: *mut AnyObject = msg_send![app, mainMenu];
            let menubar = Retained::retain(menubar)?;
            let target = target();
            let menus = [Menu::new(&target, false), Menu::new(&target, true)];
            let root_item = item("Settings", None, &target);
            let _: () = msg_send![&root_item, setSubmenu: &*menus[0].root];

            let allocated: *mut AnyObject = msg_send![class!(NSPopUpButton), alloc];
            let frame = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(104.0, 26.0));
            let button = Retained::<AnyObject>::from_raw(
                msg_send![allocated, initWithFrame: frame pullsDown: true],
            )?;
            let _: () = msg_send![&button, setMenu: &*menus[1].root];
            let _: () = msg_send![&button, setTitle: &*NSString::from_str("Settings")];
            let _: () = msg_send![&button, setAutoenablesItems: false];
            // Closing a menu must return keyboard input to Winit's game view.
            let _: () = msg_send![&button, setRefusesFirstResponder: true];
            let allocated: *mut AnyObject =
                msg_send![class!(NSTitlebarAccessoryViewController), alloc];
            let accessory = Retained::<AnyObject>::from_raw(msg_send![allocated, init])?;
            let _: () = msg_send![&accessory, setView: &*button];
            // NSLayoutAttributeRight places it alongside the window title.
            let _: () = msg_send![&accessory, setLayoutAttribute: 2_isize];
            let _: () = msg_send![&menubar, addItem: &*root_item];
            let _: () = msg_send![&window, addTitlebarAccessoryViewController: &*accessory];
            Some(Self {
                menubar,
                root_item,
                window,
                accessory,
                menus,
                _target: target,
            })
        }
    }

    pub fn sync(&self, prefs: Preferences, supports_3d: bool) {
        for menu in &self.menus {
            menu.sync(prefs, supports_3d);
        }
    }
}

impl Drop for NativeMenus {
    fn drop(&mut self) {
        // Detach controls before releasing their non-retaining action target.
        unsafe {
            let _: () = msg_send![&self.menubar, removeItem: &*self.root_item];
            let controllers: *mut AnyObject =
                msg_send![&self.window, titlebarAccessoryViewControllers];
            let count: usize = msg_send![controllers, count];
            for index in 0..count {
                let controller: *mut AnyObject = msg_send![controllers, objectAtIndex: index];
                if std::ptr::eq(controller, &*self.accessory) {
                    let _: () = msg_send![&self.window, removeTitlebarAccessoryViewControllerAtIndex: index as isize];
                    break;
                }
            }
        }
    }
}
