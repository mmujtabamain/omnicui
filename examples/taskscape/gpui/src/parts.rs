//! Part tags. GPUI has no accessibility tree yet, so the app says what its parts are itself, the
//! way egui apps tag widgets for AccessKit: a [`Part`] wraps an element with its name (the
//! contracts' names: `search`, `row3Check`), its role, and what it shows (text, accessible
//! name, checked, disabled, focus). While the registry is on, every frame records each drawn part
//! with its box on screen. The app itself never reads it; it is off unless a tool switches it on
//! (`enable`), and then costs one map insert per part per frame.

use std::cell::RefCell;
use std::collections::HashMap;
use std::time::Duration;

use gpui::{
    AnyElement, App, Bounds, Element, ElementId, FocusHandle, GlobalElementId, InspectorElementId, IntoElement, LayoutId,
    Pixels, SharedString, Window,
};

#[derive(Clone, Default, Debug)]
pub struct PartInfo {
    pub role: &'static str,
    pub text: Option<String>,
    pub name: Option<String>,
    pub checked: Option<bool>,
    pub disabled: bool,
    pub selected: Option<bool>,
    pub focused: bool,
    pub value: Option<f64>,
    /// The opacity the app draws the part with (its animation's), 1 when it isn't fading.
    pub opacity: f32,
    pub bounds: Bounds<Pixels>,
    /// The part's box clipped to what its scrolling containers show.
    pub visible: Bounds<Pixels>,
}

#[derive(Clone)]
pub struct PartMeta {
    pub name: SharedString,
    pub info: PartInfo,
    pub focus: Option<FocusHandle>,
}

/// One trace sample: when, where the part was, and its opacity.
#[derive(Clone, Copy, Debug)]
pub struct Sample {
    pub at: Duration,
    pub bounds: Bounds<Pixels>,
    pub opacity: f32,
}

#[derive(Default)]
struct Registry {
    enabled: bool,
    paints: u64,
    current: HashMap<SharedString, (PartInfo, Option<FocusHandle>)>,
    last: HashMap<SharedString, (PartInfo, Option<FocusHandle>)>,
    trace: Option<(String, Vec<Sample>)>,
}

thread_local! {
    static REG: RefCell<Registry> = RefCell::new(Registry::default());
}

/// Switches the registry on (a test adapter); the app leaves it off.
pub fn enable() {
    REG.with(|r| r.borrow_mut().enabled = true);
}
pub fn enabled() -> bool {
    REG.with(|r| r.borrow().enabled)
}

/// The parts of the last frame drawn.
pub fn get(name: &str) -> Option<(PartInfo, Option<FocusHandle>)> {
    REG.with(|r| r.borrow().last.get(name).cloned())
}
pub fn names() -> Vec<String> {
    REG.with(|r| r.borrow().last.keys().map(|k| k.to_string()).collect())
}
/// Frames drawn so far.
pub fn paints() -> u64 {
    REG.with(|r| r.borrow().paints)
}
/// Starts sampling a part at every frame drawn.
pub fn trace_start(part: &str) {
    REG.with(|r| r.borrow_mut().trace = Some((part.to_string(), Vec::new())));
}
pub fn trace_take() -> Vec<Sample> {
    REG.with(|r| r.borrow_mut().trace.take().map(|t| t.1).unwrap_or_default())
}

pub fn part(name: impl Into<SharedString>, role: &'static str) -> PartMeta {
    PartMeta { name: name.into(), info: PartInfo { role, opacity: 1.0, ..Default::default() }, focus: None }
}

impl PartMeta {
    pub fn text(mut self, t: impl Into<String>) -> Self {
        self.info.text = Some(t.into());
        self
    }
    pub fn name(mut self, n: impl Into<String>) -> Self {
        self.info.name = Some(n.into());
        self
    }
    pub fn checked(mut self, c: bool) -> Self {
        self.info.checked = Some(c);
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.info.disabled = d;
        self
    }
    pub fn selected(mut self, s: bool) -> Self {
        self.info.selected = Some(s);
        self
    }
    pub fn value(mut self, v: f64) -> Self {
        self.info.value = Some(v);
        self
    }
    pub fn opacity(mut self, o: f32) -> Self {
        self.info.opacity = o;
        self
    }
    /// The part takes keyboard focus through this handle; `focused` is read from it.
    pub fn focus(mut self, f: &FocusHandle, window: &Window) -> Self {
        self.info.focused = f.is_focused(window);
        self.focus = Some(f.clone());
        self
    }
    pub fn focused(mut self, f: bool) -> Self {
        self.info.focused = f;
        self
    }
    pub fn wrap(self, child: impl IntoElement) -> Part {
        Part { meta: Some(self), child: child.into_any_element() }
    }
}

/// An element with its part tag.
pub struct Part {
    meta: Option<PartMeta>,
    child: AnyElement,
}

impl IntoElement for Part {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

impl Element for Part {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }
    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }
    fn request_layout(&mut self, _: Option<&GlobalElementId>, _: Option<&InspectorElementId>, window: &mut Window, cx: &mut App) -> (LayoutId, ()) {
        (self.child.request_layout(window, cx), ())
    }
    fn prepaint(&mut self, _: Option<&GlobalElementId>, _: Option<&InspectorElementId>, bounds: Bounds<Pixels>, _: &mut (), window: &mut Window, cx: &mut App) {
        if let Some(meta) = self.meta.take() {
            REG.with(|r| {
                let mut r = r.borrow_mut();
                if r.enabled && !r.current.contains_key(&meta.name) {
                    let mut info = meta.info;
                    info.bounds = bounds;
                    info.visible = bounds.intersect(&window.content_mask().bounds);
                    r.current.insert(meta.name, (info, meta.focus));
                }
            });
        }
        self.child.prepaint(window, cx);
    }
    fn paint(&mut self, _: Option<&GlobalElementId>, _: Option<&InspectorElementId>, _: Bounds<Pixels>, _: &mut (), _: &mut (), window: &mut Window, cx: &mut App) {
        self.child.paint(window, cx);
    }
}

/// The root of a window's tree: marks where a frame starts and ends, for the registry.
pub struct Frame {
    child: AnyElement,
}

pub fn frame(child: impl IntoElement) -> Frame {
    Frame { child: child.into_any_element() }
}

impl IntoElement for Frame {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

impl Element for Frame {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }
    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }
    fn request_layout(&mut self, _: Option<&GlobalElementId>, _: Option<&InspectorElementId>, window: &mut Window, cx: &mut App) -> (LayoutId, ()) {
        REG.with(|r| r.borrow_mut().current.clear());
        (self.child.request_layout(window, cx), ())
    }
    fn prepaint(&mut self, _: Option<&GlobalElementId>, _: Option<&InspectorElementId>, _: Bounds<Pixels>, _: &mut (), window: &mut Window, cx: &mut App) {
        self.child.prepaint(window, cx);
    }
    fn paint(&mut self, _: Option<&GlobalElementId>, _: Option<&InspectorElementId>, _: Bounds<Pixels>, _: &mut (), _: &mut (), window: &mut Window, cx: &mut App) {
        self.child.paint(window, cx);
        // Deferred children (menus, overlays) are prepainted after this element's own prepaint,
        // so the frame's parts are complete only now.
        REG.with(|r| {
            let mut r = r.borrow_mut();
            if !r.enabled {
                return;
            }
            r.paints += 1;
            r.last = std::mem::take(&mut r.current);
            let now = crate::motion::now();
            let Registry { trace, last, .. } = &mut *r;
            if let Some((name, samples)) = trace {
                if let Some((info, _)) = last.get(name.as_str()) {
                    samples.push(Sample { at: now, bounds: info.bounds, opacity: info.opacity });
                }
            }
        });
    }
}
