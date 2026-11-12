import AppKit
import SwiftUI

struct TodoTask: Codable, Identifiable {
    var id: String
    var title: String
    var notes: String
    var done: Bool
}

@MainActor final class TodoStore: ObservableObject {
    @Published var tasks: [TodoTask]
    @Published var filter = "all"
    @Published var theme: String
    @Published var showCompleted: Bool
    @Published var route = "list"
    @Published var taskID: String
    @Published var adding = false
    @Published var draft = ""
    var nextID: Int
    var events: [[String: Any]] = []
    let component: String
    @Published var scheme: String
    init(component: String, props: [String: Any], scheme: String, seeds: URL) throws {
        let fixtures = try JSONDecoder().decode([String: [TodoTask]].self, from: Data(contentsOf: seeds))
        let initial = fixtures[props["seed"] as? String ?? "three"] ?? []
        tasks = initial
        nextID = (initial.compactMap { Int($0.id.dropFirst()) }.max() ?? 0) + 1
        theme = props["theme"] as? String ?? "system"
        showCompleted = props["showCompleted"] as? Bool ?? true
        taskID = props["taskId"] as? String ?? "t1"
        self.component = component
        self.scheme = scheme
    }
    var dark: Bool { theme == "dark" || (theme == "system" && scheme == "dark") }
    var visible: [TodoTask] { tasks.filter { filter == "active" ? !$0.done : filter == "done" ? $0.done : showCompleted || !$0.done } }
    var left: Int { tasks.filter { !$0.done }.count }
    var empty: String { filter == "done" ? "Nothing done yet" : tasks.isEmpty ? "No tasks yet" : "Nothing left to do" }
    func report(_ name: String, _ fields: [String: Any] = [:]) { events.append(["name": name, "fields": fields]) }
    func navigate(_ screen: String, id: String = "") {
        guard route != screen || (screen == "detail" && taskID != id) else { return }
        route = screen
        if screen == "detail" { taskID = id }
        report("navigated", ["screen": screen, "id": id])
    }
    func back() { if adding { adding = false } else if component == "todo.app" { navigate("list") } }
    func chooseFilter(_ value: String) { guard filter != value else { return }; filter = value; report("filterChanged", ["filter": value]) }
    func chooseTheme(_ value: String) { guard theme != value else { return }; theme = value; report("themeChanged", ["theme": value]) }
    func toggleCompleted(_ value: Bool) { guard showCompleted != value else { return }; showCompleted = value; report("showCompletedChanged", ["show": value]) }
    func setDone(_ id: String, _ value: Bool) {
        guard let i = tasks.firstIndex(where: { $0.id == id }), tasks[i].done != value else { return }
        tasks[i].done = value; report("toggled", ["id": id, "done": value])
    }
    func edit(_ id: String, _ field: String, _ value: String) {
        guard let i = tasks.firstIndex(where: { $0.id == id }) else { return }
        if field == "title" { guard tasks[i].title != value else { return }; tasks[i].title = value }
        else { guard tasks[i].notes != value else { return }; tasks[i].notes = value }
        report(field == "title" ? "renamed" : "notesChanged", ["id": id, field: value])
    }
    func add() {
        let title = draft.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !title.isEmpty else { return }
        let id = "t\(nextID)"; nextID += 1
        tasks.append(TodoTask(id: id, title: title, notes: "", done: false))
        report("added", ["id": id, "title": title]); adding = false; draft = ""
    }
    func open(_ id: String) { report("opened", ["id": id]); if component == "todo.app" { navigate("detail", id: id) } }
    func closeDetail() { report("closed", ["id": taskID]); if component == "todo.app" { navigate("list") } }
    func deleteTask() { let id = taskID; tasks.removeAll { $0.id == id }; report("deleted", ["id": id]); if component == "todo.app" { navigate("list") } }
}

struct Palette {
    let dark: Bool
    func color(_ light: UInt32, _ dark: UInt32) -> NSColor {
        let n = self.dark ? dark : light
        return NSColor(srgbRed: CGFloat((n >> 16) & 255) / 255, green: CGFloat((n >> 8) & 255) / 255, blue: CGFloat(n & 255) / 255, alpha: 1)
    }
    var background: NSColor { color(0xF8F9FF, 0x111418) }
    var sidebar: NSColor { color(0xF2F3F9, 0x191C20) }
    var text: NSColor { color(0x191C20, 0xE1E2E8) }
    var muted: NSColor { color(0x43474E, 0xC3C6CF) }
    var accent: NSColor { color(0x36618E, 0xA0CAFD) }
    var onAccent: NSColor { color(0xFFFFFF, 0x003258) }
    var selected: NSColor { color(0xD7E3F7, 0x3B4858) }
    var raised: NSColor { color(0xE6E8EE, 0x272A2F) }
    var danger: NSColor { color(0xBA1A1A, 0xFFB4AB) }
    var divider: NSColor { color(0xC3C6CF, 0x43474E) }
}
