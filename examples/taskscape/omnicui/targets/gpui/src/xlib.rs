//! The X server the adapter's windows live on, reached through Xlib and the XTEST extension,
//! loaded at run time (dlopen), so that nothing here changes how the GPUI crates are built.
//!
//! - Input is real: XTEST makes the server deliver pointer and key events exactly as a mouse
//!   and a keyboard would, so GPUI's own X11 platform code receives them, resolves them with
//!   xkbcommon, and runs its own hit testing, focus, key bindings and text input.
//! - Screenshots read the window's pixels back from the server (XGetImage): what the renderer
//!   presented, with nothing drawn over it.
#![allow(non_upper_case_globals, clippy::missing_safety_doc)]

use std::ffi::{c_char, c_int, c_uint, c_ulong, c_void, CString};

use anyhow::{anyhow, bail, Result};

type Display = c_void;
pub type Window = c_ulong;
type KeySym = c_ulong;
type KeyCode = u8;

#[repr(C)]
struct XImage {
    width: c_int,
    height: c_int,
    xoffset: c_int,
    format: c_int,
    data: *mut c_char,
    byte_order: c_int,
    bitmap_unit: c_int,
    bitmap_bit_order: c_int,
    bitmap_pad: c_int,
    depth: c_int,
    bytes_per_line: c_int,
    bits_per_pixel: c_int,
    red_mask: c_ulong,
    green_mask: c_ulong,
    blue_mask: c_ulong,
    // followed by obdata and the function table; never read here
}

struct Fns {
    open_display: unsafe extern "C" fn(*const c_char) -> *mut Display,
    default_root: unsafe extern "C" fn(*mut Display) -> Window,
    flush: unsafe extern "C" fn(*mut Display) -> c_int,
    sync: unsafe extern "C" fn(*mut Display, c_int) -> c_int,
    string_to_keysym: unsafe extern "C" fn(*const c_char) -> KeySym,
    keysym_to_keycode: unsafe extern "C" fn(*mut Display, KeySym) -> KeyCode,
    keycode_to_keysym: unsafe extern "C" fn(*mut Display, KeyCode, c_int) -> KeySym,
    get_image: unsafe extern "C" fn(*mut Display, Window, c_int, c_int, c_uint, c_uint, c_ulong, c_int) -> *mut XImage,
    destroy_image: unsafe extern "C" fn(*mut XImage) -> c_int,
    free: unsafe extern "C" fn(*mut c_void) -> c_int,
    set_input_focus: unsafe extern "C" fn(*mut Display, Window, c_int, c_ulong) -> c_int,
    raise_window: unsafe extern "C" fn(*mut Display, Window) -> c_int,
    fake_key: unsafe extern "C" fn(*mut Display, c_uint, c_int, c_ulong) -> c_int,
    fake_button: unsafe extern "C" fn(*mut Display, c_uint, c_int, c_ulong) -> c_int,
    fake_motion: unsafe extern "C" fn(*mut Display, c_int, c_int, c_int, c_ulong) -> c_int,
    query_extension: unsafe extern "C" fn(*mut Display, *mut c_int, *mut c_int, *mut c_int, *mut c_int) -> c_int,
    query_tree: unsafe extern "C" fn(*mut Display, Window, *mut Window, *mut Window, *mut *mut Window, *mut c_uint) -> c_int,
    fetch_name: unsafe extern "C" fn(*mut Display, Window, *mut *mut c_char) -> c_int,
    translate: unsafe extern "C" fn(*mut Display, Window, Window, c_int, c_int, *mut c_int, *mut c_int, *mut Window) -> c_int,
}

pub struct X {
    dpy: *mut Display,
    f: Fns,
}

unsafe fn sym<T>(lib: *mut c_void, name: &str) -> Result<T> {
    let c = CString::new(name).unwrap();
    let p = libc::dlsym(lib, c.as_ptr());
    if p.is_null() {
        bail!("{name} is missing");
    }
    Ok(std::mem::transmute_copy(&p))
}

unsafe fn open_lib(names: &[&str]) -> Result<*mut c_void> {
    for n in names {
        let c = CString::new(*n).unwrap();
        let h = libc::dlopen(c.as_ptr(), libc::RTLD_NOW | libc::RTLD_GLOBAL);
        if !h.is_null() {
            return Ok(h);
        }
    }
    Err(anyhow!("none of {names:?} could be loaded (Xlib and the XTEST client library are needed)"))
}

const ZPixmap: c_int = 2;
const AllPlanes: c_ulong = !0;
const RevertToParent: c_int = 2;
const CurrentTime: c_ulong = 0;

impl X {
    /// Opens the display named by `DISPLAY`.
    pub fn open() -> Result<X> {
        unsafe {
            let x11 = open_lib(&["libX11.so.6", "libX11.so"])?;
            let xtst = open_lib(&["libXtst.so.6", "libXtst.so"])?;
            let f = Fns {
                open_display: sym(x11, "XOpenDisplay")?,
                default_root: sym(x11, "XDefaultRootWindow")?,
                flush: sym(x11, "XFlush")?,
                sync: sym(x11, "XSync")?,
                string_to_keysym: sym(x11, "XStringToKeysym")?,
                keysym_to_keycode: sym(x11, "XKeysymToKeycode")?,
                keycode_to_keysym: sym(x11, "XKeycodeToKeysym")?,
                get_image: sym(x11, "XGetImage")?,
                destroy_image: sym(x11, "XDestroyImage").or_else(|_| sym(x11, "XFree"))?,
                free: sym(x11, "XFree")?,
                set_input_focus: sym(x11, "XSetInputFocus")?,
                raise_window: sym(x11, "XRaiseWindow")?,
                fake_key: sym(xtst, "XTestFakeKeyEvent")?,
                fake_button: sym(xtst, "XTestFakeButtonEvent")?,
                fake_motion: sym(xtst, "XTestFakeMotionEvent")?,
                query_extension: sym(xtst, "XTestQueryExtension")?,
                query_tree: sym(x11, "XQueryTree")?,
                fetch_name: sym(x11, "XFetchName")?,
                translate: sym(x11, "XTranslateCoordinates")?,
            };
            let dpy = (f.open_display)(std::ptr::null());
            if dpy.is_null() {
                bail!("can't open the X display {:?}", std::env::var("DISPLAY").unwrap_or_default());
            }
            let (mut a, mut b, mut c, mut d) = (0, 0, 0, 0);
            if (f.query_extension)(dpy, &mut a, &mut b, &mut c, &mut d) == 0 {
                bail!("the X server has no XTEST extension, so there is no way to send it input");
            }
            Ok(X { dpy, f })
        }
    }

    pub fn sync(&self) {
        unsafe {
            (self.f.sync)(self.dpy, 0);
        }
    }

    pub fn focus(&self, w: Window) {
        unsafe {
            (self.f.raise_window)(self.dpy, w);
            (self.f.set_input_focus)(self.dpy, w, RevertToParent, CurrentTime);
            (self.f.sync)(self.dpy, 0);
        }
    }

    /// Moves the pointer to a point on the screen (root coordinates).
    pub fn move_to(&self, x: i32, y: i32) {
        unsafe {
            (self.f.fake_motion)(self.dpy, -1, x, y, CurrentTime);
            (self.f.flush)(self.dpy);
        }
    }

    pub fn button(&self, button: u32, down: bool) {
        unsafe {
            (self.f.fake_button)(self.dpy, button, down as c_int, CurrentTime);
            (self.f.flush)(self.dpy);
        }
    }

    fn keycode(&self, keysym: KeySym) -> Option<KeyCode> {
        let k = unsafe { (self.f.keysym_to_keycode)(self.dpy, keysym) };
        (k != 0).then_some(k)
    }

    /// The keycode for a keysym name, and whether the keysym is on the key's shifted level.
    pub fn key_for(&self, name: &str) -> Result<(KeyCode, bool)> {
        let c = CString::new(name).unwrap();
        let ks = unsafe { (self.f.string_to_keysym)(c.as_ptr()) };
        if ks == 0 {
            bail!("no keysym {name}");
        }
        let code = self.keycode(ks).ok_or_else(|| anyhow!("no key on this keyboard types {name}"))?;
        let base = unsafe { (self.f.keycode_to_keysym)(self.dpy, code, 0) };
        Ok((code, base != ks))
    }

    pub fn key(&self, code: KeyCode, down: bool) {
        unsafe {
            (self.f.fake_key)(self.dpy, code as c_uint, down as c_int, CurrentTime);
            (self.f.flush)(self.dpy);
        }
    }

    /// The window's pixels, as RGBA rows, from the server.
    pub fn capture(&self, w: Window, width: u32, height: u32) -> Result<Vec<u8>> {
        unsafe {
            (self.f.sync)(self.dpy, 0);
            let img = (self.f.get_image)(self.dpy, w, 0, 0, width, height, AllPlanes, ZPixmap);
            if img.is_null() {
                bail!("XGetImage failed (the window is not mapped, or is smaller than {width}×{height})");
            }
            let im = &*img;
            if im.bits_per_pixel != 32 {
                (self.f.destroy_image)(img);
                bail!("unexpected {} bits per pixel", im.bits_per_pixel);
            }
            let (rs, gs, bs) = (im.red_mask.trailing_zeros(), im.green_mask.trailing_zeros(), im.blue_mask.trailing_zeros());
            let mut out = Vec::with_capacity((width * height * 4) as usize);
            for y in 0..im.height {
                let row = im.data.add((y * im.bytes_per_line) as usize) as *const u32;
                for x in 0..im.width {
                    let p = *row.add(x as usize) as c_ulong;
                    out.extend_from_slice(&[(p >> rs) as u8, (p >> gs) as u8, (p >> bs) as u8, 255]);
                }
            }
            (self.f.destroy_image)(img);
            let _ = self.f.free;
            Ok(out)
        }
    }

    /// The window with this title, anywhere under the root (GPUI's X11 window gives no handle).
    pub fn find(&self, title: &str) -> Option<Window> {
        unsafe { self.find_under(self.root(), title) }
    }

    unsafe fn find_under(&self, w: Window, title: &str) -> Option<Window> {
        let mut name: *mut c_char = std::ptr::null_mut();
        if (self.f.fetch_name)(self.dpy, w, &mut name) != 0 && !name.is_null() {
            let n = std::ffi::CStr::from_ptr(name).to_string_lossy().into_owned();
            (self.f.free)(name as *mut c_void);
            if n == title {
                return Some(w);
            }
        }
        let (mut root, mut parent, mut kids, mut n) = (0, 0, std::ptr::null_mut(), 0u32);
        if (self.f.query_tree)(self.dpy, w, &mut root, &mut parent, &mut kids, &mut n) == 0 {
            return None;
        }
        let list: Vec<Window> = (0..n as usize).map(|i| *kids.add(i)).collect();
        if !kids.is_null() {
            (self.f.free)(kids as *mut c_void);
        }
        list.into_iter().rev().find_map(|k| self.find_under(k, title))
    }

    /// Where a window's top-left corner is on the screen.
    pub fn origin(&self, w: Window) -> (i32, i32) {
        let (mut x, mut y, mut child) = (0, 0, 0);
        unsafe {
            (self.f.translate)(self.dpy, w, self.root(), 0, 0, &mut x, &mut y, &mut child);
        }
        (x, y)
    }

    pub fn root(&self) -> Window {
        unsafe { (self.f.default_root)(self.dpy) }
    }
}
