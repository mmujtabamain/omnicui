import SwiftUI
import UIKit

public enum TodoRoute: Hashable { case detail(String), settings }

public struct TodoRoot: View {
    @ObservedObject var store: TodoStore
    let component: String
    let taskID: String
    @State private var path: [TodoRoute] = []
    public init(store: TodoStore, component: String = "todo.app", taskID: String = "t1") {
        self.store = store; self.component = component; self.taskID = taskID
    }
    private func navigated(_ route: TodoRoute?) {
        switch route {
        case .detail(let id): store.onEvent("navigated", ["screen": "detail", "id": id])
        case .settings: store.onEvent("navigated", ["screen": "settings", "id": ""])
        case nil: store.onEvent("navigated", ["screen": "list", "id": ""])
        }
    }
    public var body: some View {
        NavigationStack(path: $path) {
            Group {
                switch component {
                case "todo.detail": DetailScreen(store: store, id: taskID, back: {})
                case "todo.settings": SettingsScreen(store: store, back: {})
                default:
                    ListScreen(store: store) { id in
                        store.onEvent("opened", ["id": id])
                        if component == "todo.app" { path.append(.detail(id)) }
                    }
                    .toolbar {
                        if component == "todo.app" {
                            ToolbarItem(placement: .topBarLeading) {
                                Button("Settings", systemImage: "gearshape") { path.append(.settings) }
                                    .accessibilityIdentifier("todo.app.openSettings")
                            }
                        }
                    }
                }
            }
            .navigationDestination(for: TodoRoute.self) { route in
                switch route {
                case .detail(let id): DetailScreen(store: store, id: id) { path.removeLast() }
                case .settings: SettingsScreen(store: store) { path.removeLast() }
                }
            }
        }
        .onChange(of: path) { _, value in if component == "todo.app" { navigated(value.last) } }
        .preferredColorScheme(store.theme == "system" ? nil : store.theme == "dark" ? .dark : .light)
    }
}

struct ListScreen: View {
    @ObservedObject var store: TodoStore
    let open: (String) -> Void
    @State private var adding = false
    @State private var newTitle = ""
    private func confirm() {
        guard !newTitle.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else { return }
        store.add(newTitle); adding = false
    }
    var body: some View {
        VStack(spacing: 8) {
            Picker("Filter", selection: Binding(get: { store.filter }, set: store.chooseFilter)) {
                Text("All").tag("all").accessibilityIdentifier("todo.list.filterAll")
                Text("Active").tag("active").accessibilityIdentifier("todo.list.filterActive")
                Text("Done").tag("done").accessibilityIdentifier("todo.list.filterDone")
            }.pickerStyle(.segmented).padding(.horizontal)
            Text("\(store.left) left").foregroundStyle(.secondary)
                .accessibilityIdentifier("todo.list.left").accessibilityAddTraits(.updatesFrequently)
            List {
                if store.visible.isEmpty {
                    Text(store.emptyText).foregroundStyle(.secondary).accessibilityIdentifier("todo.list.empty").accessibilityAddTraits(.updatesFrequently)
                }
                ForEach(Array(store.visible.enumerated()), id: \.element.id) { index, task in
                    HStack {
                        Button { store.change(task.id, "done", !task.done) } label: {
                            Image(systemName: task.done ? "checkmark.circle.fill" : "circle")
                                .font(.title2).foregroundStyle(task.done ? Color.accentColor : .secondary)
                        }
                        .buttonStyle(.borderless)
                        .accessibilityLabel(task.title)
                        .accessibilityValue(task.done ? "1" : "0")
                        .accessibilityAddTraits(.isToggle)
                        .accessibilityIdentifier("todo.list.row\(index + 1)Check")
                        Button { open(task.id) } label: {
                            Text(task.title).strikethrough(task.done).foregroundStyle(task.done ? Color.secondary : Color.primary)
                                .frame(maxWidth: .infinity, alignment: .leading).contentShape(Rectangle())
                        }.buttonStyle(.borderless).accessibilityIdentifier("todo.list.row\(index + 1)")
                    }
                    .swipeActions { Button("Delete", role: .destructive) { store.delete(task.id) } }
                }
            }.listStyle(.insetGrouped)
        }
        .navigationTitle("Tasks").navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .principal) {
                Text("Tasks").font(.headline).accessibilityAddTraits(.isHeader).accessibilityIdentifier("todo.list.tasksHeading")
            }
            ToolbarItem(placement: .topBarTrailing) {
                Button("Add task", systemImage: "plus") { newTitle = ""; adding = true }
                    .accessibilityIdentifier("todo.list.add")
            }
        }
        .sheet(isPresented: $adding) {
            AddTaskSheet(title: $newTitle, confirm: confirm, cancel: { adding = false })
        }
    }
}

private struct AddTaskSheet: View {
    @Binding var title: String
    let confirm: () -> Void
    let cancel: () -> Void
    @FocusState private var titleFocused: Bool
    var body: some View {
        NavigationStack {
            Form {
                TextField("Title", text: $title).accessibilityLabel("Title")
                    .accessibilityIdentifier("todo.list.newTitle").focused($titleFocused)
                    .submitLabel(.done).onSubmit(confirm)
            }
            .navigationTitle("New task").navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel", action: cancel).accessibilityIdentifier("todo.list.cancelAdd")
                }
                ToolbarItem(placement: .confirmationAction) {
                    Button("Add", action: confirm).disabled(title.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                        .accessibilityIdentifier("todo.list.confirmAdd")
                }
            }
        }
        .presentationDetents([.medium])
        .task { titleFocused = true }
    }
}

struct DetailScreen: View {
    @ObservedObject var store: TodoStore
    let id: String
    let back: () -> Void
    var task: TaskItem? { store.tasks.first { $0.id == id } }
    var body: some View {
        Form {
            if task != nil {
                TextField("Title", text: Binding(get: { self.task?.title ?? "" }, set: { store.change(id, "title", $0) }))
                    .accessibilityLabel("Title").accessibilityIdentifier("todo.detail.title")
                TextField("Notes", text: Binding(get: { self.task?.notes ?? "" }, set: { store.change(id, "notes", $0) }), axis: .vertical)
                    .lineLimit(3...8).accessibilityLabel("Notes").accessibilityIdentifier("todo.detail.notes")
                Toggle("Done", isOn: Binding(get: { self.task?.done ?? false }, set: { store.change(id, "done", $0) }))
                    .accessibilityIdentifier("todo.detail.done")
            }
        }
        .navigationTitle("Task").navigationBarTitleDisplayMode(.inline).navigationBarBackButtonHidden()
        .background(EdgeBackGesture())
        .toolbar {
            ToolbarItem(placement: .topBarLeading) {
                Button("Back", systemImage: "chevron.left") { store.onEvent("closed", ["id": id]); back() }
                    .accessibilityIdentifier("todo.detail.detailBack")
            }
            ToolbarItem(placement: .topBarTrailing) {
                Button("Delete", systemImage: "trash", role: .destructive) { store.delete(id); back() }
                    .accessibilityIdentifier("todo.detail.delete")
            }
        }
    }
}

struct SettingsScreen: View {
    @ObservedObject var store: TodoStore
    let back: () -> Void
    var body: some View {
        Form {
            Section("Theme") {
                Picker("Theme", selection: Binding(get: { store.theme }, set: store.chooseTheme)) {
                    Text("System").tag("system").accessibilityIdentifier("todo.settings.themeSystem")
                    Text("Light").tag("light").accessibilityIdentifier("todo.settings.themeLight")
                    Text("Dark").tag("dark").accessibilityIdentifier("todo.settings.themeDark")
                }.pickerStyle(.segmented)
            }
            Toggle("Show completed", isOn: Binding(get: { store.showCompleted }, set: store.setCompleted))
                .accessibilityIdentifier("todo.settings.showCompleted")
        }
        .navigationTitle("Settings").navigationBarTitleDisplayMode(.inline).navigationBarBackButtonHidden()
        .background(EdgeBackGesture())
        .toolbar {
            ToolbarItem(placement: .topBarLeading) {
                Button("Back", systemImage: "chevron.left") { store.onEvent("closed", [:]); back() }
                    .accessibilityIdentifier("todo.settings.settingsBack")
            }
        }
    }
}

// A named back control keeps its VoiceOver label stable. Restore UIKit's native
// interactive pop gesture, which hiding the automatic back item otherwise disables.
private struct EdgeBackGesture: UIViewControllerRepresentable {
    final class Controller: UIViewController {
        override func viewDidAppear(_ animated: Bool) {
            super.viewDidAppear(animated)
            navigationController?.interactivePopGestureRecognizer?.delegate = nil
            navigationController?.interactivePopGestureRecognizer?.isEnabled = true
        }
    }
    func makeUIViewController(context: Context) -> Controller { Controller() }
    func updateUIViewController(_ controller: Controller, context: Context) {}
}
