import AppKit
import SwiftUI

private extension View {
    func box(_ x: CGFloat, _ y: CGFloat, _ width: CGFloat, _ height: CGFloat) -> some View {
        frame(width: width, height: height).position(x: x + width / 2, y: y + height / 2)
    }
}

struct TodoRoot: View {
    @ObservedObject var store: TodoStore
    let registry: PartRegistry
    var nativeToolbar = false
    @State private var dialogVisible = false
    var palette: Palette { Palette(dark: store.dark) }
    var body: some View {
        GeometryReader { geometry in
            ZStack(alignment: .topLeading) {
                Color(nsColor: palette.background)
                if store.component == "todo.app" {
                    let top: CGFloat = nativeToolbar ? 0 : 56
                    if !nativeToolbar { appHeader(width: geometry.size.width) }
                    if store.route == "settings" {
                        settings.box(0, top, geometry.size.width, geometry.size.height - top)
                    } else {
                        list.box(0, top, 340, geometry.size.height - top)
                        if store.route == "detail" {
                            detail.box(340, top, geometry.size.width - 340, geometry.size.height - top)
                        } else {
                            Text("Choose a task to see its details.").foregroundStyle(Color(nsColor: palette.muted))
                                .box(340, top, geometry.size.width - 340, geometry.size.height - top)
                        }
                    }
                } else if store.component == "todo.list" {
                    list.box(0, 0, 340, geometry.size.height)
                } else if store.component == "todo.detail" {
                    detail.box(0, 0, geometry.size.width, geometry.size.height)
                } else { settings.box(0, 0, geometry.size.width, geometry.size.height) }
                if store.adding {
                    Color.black.opacity(0.6).ignoresSafeArea().onTapGesture { }
                    dialog
                        .scaleEffect(dialogVisible ? 1 : 0.9)
                        .opacity(dialogVisible ? 1 : 0)
                        .box((geometry.size.width - 400) / 2, (geometry.size.height - 223) / 2, 400, 223)
                        .onAppear {
                            withAnimation(.timingCurve(0, 0, 0.58, 1, duration: 0.2)) { dialogVisible = true }
                            DispatchQueue.main.asyncAfter(deadline: .now() + 0.02) {
                                if let field = registry.find("todo.list.newTitle") { field.window?.makeFirstResponder(field) }
                            }
                        }
                        .onDisappear { dialogVisible = false }
                }
            }
            .clipped()
        }
        .font(.system(size: 14))
        .foregroundStyle(Color(nsColor: palette.text))
        .tint(Color(nsColor: palette.accent))
        .environment(\.colorScheme, store.dark ? .dark : .light)
        .onChange(of: store.dark) { _, dark in
            if let window = registry.views.values.first(where: { $0.window != nil })?.window {
                window.appearance = NSAppearance(named: dark ? .darkAqua : .aqua)
                window.backgroundColor = Palette(dark: dark).background
            }
            (registry.find("todo.app.openSettings") as? NSButton)?.contentTintColor = Palette(dark: dark).accent
            (registry.find("todo.app.brand") as? NSTextField)?.textColor = Palette(dark: dark).accent
            (registry.find("todo.app.brandIcon") as? NSImageView)?.contentTintColor = Palette(dark: dark).accent
        }
    }
    func button(_ id: String, _ title: String, name: String? = nil, filled: Bool = false, plain: Bool = false, danger: Bool = false, disabled: Bool = false, action: @escaping () -> Void) -> some View {
        NativeButton(id: id, title: title, name: name, color: danger ? palette.danger : palette.accent, onAccent: palette.onAccent,
                     filled: filled, plain: plain, disabled: disabled, registry: registry, action: action)
    }
    func label(_ id: String, _ text: String, size: CGFloat = 14, role: NSAccessibility.Role = .staticText) -> some View {
        NativeLabel(id: id, text: text, role: role, size: size, color: size > 18 ? palette.text : palette.muted, registry: registry)
    }
    func appHeader(width: CGFloat) -> some View {
        ZStack(alignment: .topLeading) {
            HStack(spacing: 8) { Image(systemName: "checkmark.square"); Text("Todo").fontWeight(.semibold) }
                .font(.system(size: 18)).foregroundStyle(Color(nsColor: palette.accent)).box(20, 10, 90, 36)
            button("todo.app.openSettings", "Settings", plain: true) { store.navigate("settings") }.box(width - 123, 10, 107, 36)
            Color(nsColor: palette.divider).box(0, 55, width, 1)
        }.box(0, 0, width, 56)
        .accessibilityElement(children: .contain)
        .accessibilityLabel("Toolbar")
    }
    var list: some View {
        GeometryReader { g in
            ZStack(alignment: .topLeading) {
                Color(nsColor: palette.sidebar)
                // A subtle native sidebar material sits over the product's sidebar token.
                SidebarMaterial().opacity(0.04)
                Color(nsColor: palette.divider).box(339, 0, 1, g.size.height)
                label("todo.list.tasksHeading", "Tasks", size: 22, role: NSAccessibility.Role(rawValue: "AXHeading")).box(20, 18, 160, 32)
                button("todo.list.add", "＋ Add task", name: "Add task", filled: true) {
                    store.draft = ""; store.adding = true
                }.box(208, 16, 115, 36)
                NativeSegments(ids: ["filterAll", "filterActive", "filterDone"].map { "todo.list.\($0)" },
                               labels: ["All", "Active", "Done"], selected: ["all", "active", "done"].firstIndex(of: store.filter) ?? 0,
                               color: palette.accent, registry: registry) { store.chooseFilter(["all", "active", "done"][$0]) }
                    .box(16, 64, 307, 34)
                label("todo.list.left", "\(store.left) left", size: 12).box(20, 110, 299, 17)
                if store.visible.isEmpty {
                    label("todo.list.empty", store.empty).box(20, 167, 299, 20)
                } else {
                    ScrollView(.vertical) {
                        VStack(spacing: 0) {
                            ForEach(Array(store.visible.enumerated()), id: \.element.id) { i, task in
                                taskRow(task, position: i + 1)
                            }
                        }.padding(.bottom, 16)
                    }.box(8, 135, 323, max(0, g.size.height - 135))
                }
            }
        }
    }
    func taskRow(_ task: TodoTask, position: Int) -> some View {
        ZStack(alignment: .topLeading) {
            if store.component == "todo.app" && store.route == "detail" && store.taskID == task.id {
                RoundedRectangle(cornerRadius: 8).fill(Color(nsColor: palette.selected))
            }
            NativeButton(id: "todo.list.row\(position)Check", title: "", name: task.title, color: palette.accent,
                         checked: task.done, registry: registry) { store.setDone(task.id, !task.done) }.box(12, 11, 18, 18)
            NativeButton(id: "todo.list.row\(position)", title: task.title, color: task.done ? palette.muted : palette.text,
                         plain: true, registry: registry) { store.open(task.id) }.box(34, 0, 289, 40)
        }.frame(height: 40)
    }
    var detail: some View {
        ZStack(alignment: .topLeading) {
            if let task = store.tasks.first(where: { $0.id == store.taskID }) {
                Text("Task").font(.system(size: 22, weight: .medium)).frame(maxWidth: .infinity, alignment: .leading).box(32, 20, 480, 36)
                button("todo.detail.detailBack", "×", name: "Close", plain: true) { store.closeDetail() }.box(556, 20, 36, 36)
                Text("Title").font(.system(size: 12)).foregroundStyle(Color(nsColor: palette.muted)).frame(maxWidth: .infinity, alignment: .leading).box(32, 76, 560, 17)
                NativeField(id: "todo.detail.title", name: "Title", value: task.title, color: palette.text, registry: registry) { store.edit(task.id, "title", $0) }.box(32, 93, 560, 43)
                Text("Notes").font(.system(size: 12)).foregroundStyle(Color(nsColor: palette.muted)).frame(maxWidth: .infinity, alignment: .leading).box(32, 157, 560, 17)
                NativeNotes(id: "todo.detail.notes", value: task.notes, color: palette.text, registry: registry) { store.edit(task.id, "notes", $0) }.box(32, 174, 560, 108)
                Text("Done").frame(maxWidth: .infinity, alignment: .leading).box(32, 298, 450, 48)
                NativeSwitch(id: "todo.detail.done", name: "Done", on: task.done, color: palette.accent, registry: registry) { store.setDone(task.id, $0) }.box(540, 306, 52, 32)
                button("todo.detail.delete", "Delete", danger: true) { store.deleteTask() }.box(32, 370, 80, 36)
            } else {
                Text("This task no longer exists.").foregroundStyle(Color(nsColor: palette.muted)).padding(32)
            }
        }
    }
    var settings: some View {
        GeometryReader { g in
            let x = (g.size.width - 640) / 2 + 32
            ZStack(alignment: .topLeading) {
                button("todo.settings.settingsBack", "‹ Back to tasks", name: "Back to tasks", plain: true) {
                    store.report("closed"); if store.component == "todo.app" { store.navigate("list") }
                }.box(x - 12, 24, 145, 36)
                Text("Settings").font(.system(size: 28)).frame(maxWidth: .infinity, alignment: .leading).box(x, 72, 560, 40)
                Text("Theme").foregroundStyle(Color(nsColor: palette.accent)).frame(maxWidth: .infinity, alignment: .leading).box(x, 136, 560, 20)
                NativeSegments(ids: ["themeSystem", "themeLight", "themeDark"].map { "todo.settings.\($0)" },
                               labels: ["System", "Light", "Dark"], selected: ["system", "light", "dark"].firstIndex(of: store.theme) ?? 0,
                               color: palette.accent, registry: registry) { store.chooseTheme(["system", "light", "dark"][$0]) }.box(x, 164, 266, 34)
                Text("Show completed").frame(maxWidth: .infinity, alignment: .leading).box(x, 222, 450, 48)
                NativeSwitch(id: "todo.settings.showCompleted", name: "Show completed", on: store.showCompleted,
                             color: palette.accent, registry: registry) { store.toggleCompleted($0) }.box(x + 508, 230, 52, 32)
                Text("Off: All lists only the tasks left. Done still lists the done ones.")
                    .font(.system(size: 12)).foregroundStyle(Color(nsColor: palette.muted)).frame(maxWidth: .infinity, alignment: .leading).box(x, 270, 560, 18)
            }
        }
    }
    var dialog: some View {
        ZStack(alignment: .topLeading) {
            RoundedRectangle(cornerRadius: 20).fill(Color(nsColor: palette.raised))
            PartMarker(id: "todo.list.newTaskDialog", registry: registry).allowsHitTesting(false)
            Text("New task").font(.system(size: 24)).frame(maxWidth: .infinity, alignment: .leading).box(24, 24, 352, 34)
            Text("Title").font(.system(size: 12)).foregroundStyle(Color(nsColor: palette.muted)).frame(maxWidth: .infinity, alignment: .leading).box(24, 78, 352, 17)
            NativeField(id: "todo.list.newTitle", name: "Title", value: store.draft, color: palette.text, registry: registry,
                        change: { store.draft = $0 }, submit: { store.add() }).box(24, 95, 352, 43)
            button("todo.list.cancelAdd", "Cancel", plain: true) { store.adding = false }.box(222, 162, 80, 36)
            button("todo.list.confirmAdd", "Add", plain: true, disabled: store.draft.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty) { store.add() }.box(310, 162, 66, 36)
        }
        .frame(width: 400, height: 223)
        .accessibilityElement(children: .contain)
        .accessibilityLabel("New task")
    }
}
struct SidebarMaterial: NSViewRepresentable {
    func makeNSView(context: Context) -> NSVisualEffectView { let v = NSVisualEffectView(); v.material = .sidebar; v.blendingMode = .withinWindow; return v }
    func updateNSView(_ view: NSVisualEffectView, context: Context) { }
}
