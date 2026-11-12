import SwiftUI

public struct TaskItem: Codable, Identifiable, Equatable {
    public let id: String
    public var title: String
    public var notes: String
    public var done: Bool
}

@MainActor public final class TodoStore: ObservableObject {
    @Published public var tasks: [TaskItem]
    @Published public var filter = "all"
    @Published public var theme: String
    @Published public var showCompleted: Bool
    private var nextID: Int
    public var onEvent: (String, [String: Any]) -> Void = { _, _ in }

    public init(seed: String = "three", theme: String = "system", showCompleted: Bool = true) {
        let url = Bundle(for: BundleToken.self).url(forResource: "seeds", withExtension: "json")!
        let fixtures = try! JSONDecoder().decode([String: [TaskItem]].self, from: Data(contentsOf: url))
        let initial = fixtures[seed] ?? []
        tasks = initial
        nextID = (initial.compactMap { Int($0.id.dropFirst()) }.max() ?? 0) + 1
        self.theme = theme
        self.showCompleted = showCompleted
    }
    public var visible: [TaskItem] {
        tasks.filter { filter == "active" ? !$0.done : filter == "done" ? $0.done : showCompleted || !$0.done }
    }
    public var left: Int { tasks.filter { !$0.done }.count }
    public var emptyText: String { filter == "done" ? "Nothing done yet" : tasks.isEmpty ? "No tasks yet" : "Nothing left to do" }
    public func add(_ title: String) {
        let title = title.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !title.isEmpty else { return }
        let id = "t\(nextID)"; nextID += 1
        tasks.append(TaskItem(id: id, title: title, notes: "", done: false))
        onEvent("added", ["id": id, "title": title])
    }
    public func change(_ id: String, _ field: String, _ value: Any) {
        guard let i = tasks.firstIndex(where: { $0.id == id }) else { return }
        switch field {
        case "title":
            guard let v = value as? String, tasks[i].title != v else { return }
            tasks[i].title = v; onEvent("renamed", ["id": id, "title": v])
        case "notes":
            guard let v = value as? String, tasks[i].notes != v else { return }
            tasks[i].notes = v; onEvent("notesChanged", ["id": id, "notes": v])
        default:
            guard let v = value as? Bool, tasks[i].done != v else { return }
            tasks[i].done = v; onEvent("toggled", ["id": id, "done": v])
        }
    }
    public func delete(_ id: String) { tasks.removeAll { $0.id == id }; onEvent("deleted", ["id": id]) }
    public func chooseFilter(_ value: String) {
        guard filter != value else { return }; filter = value; onEvent("filterChanged", ["filter": value])
    }
    public func chooseTheme(_ value: String) {
        guard theme != value else { return }; theme = value; onEvent("themeChanged", ["theme": value])
    }
    public func setCompleted(_ value: Bool) {
        guard showCompleted != value else { return }; showCompleted = value; onEvent("showCompletedChanged", ["show": value])
    }
}
private class BundleToken {}
