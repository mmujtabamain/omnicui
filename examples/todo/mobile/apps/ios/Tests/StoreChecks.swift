import Foundation
import CryptoKit
import TodoUI

@main enum StoreChecks {
    @MainActor static func main() throws {
        let fixture = URL(fileURLWithPath: CommandLine.arguments[1])
        let digest = SHA256.hash(data: try Data(contentsOf: fixture)).map { String(format: "%02x", $0) }.joined()
        precondition(digest == "c1ace19804fea36d7f4a6e1b96c2e183c518741010339f09c058615992a51d2e")
        let store = TodoStore()
        var events: [(String, [String: Any])] = []
        store.onEvent = { events.append(($0, $1)) }
        precondition(store.left == 2 && store.visible.map(\.id) == ["t1", "t2", "t3"])
        store.add(" \n  "); precondition(events.isEmpty)
        store.add(" Water the plants \n")
        precondition(store.tasks.last!.id == "t4" && store.tasks.last!.title == "Water the plants")
        store.delete("t4"); store.add("Next")
        precondition(store.tasks.last!.id == "t5")
        store.setCompleted(false)
        precondition(store.visible.map(\.id) == ["t1", "t3", "t5"])
        store.chooseFilter("done")
        precondition(store.visible.map(\.id) == ["t2"])
        let count = events.count
        store.chooseFilter("done"); store.chooseTheme("system"); store.setCompleted(false)
        store.change("t1", "done", false); store.change("t1", "title", "Buy milk")
        precondition(events.count == count)
        store.change("t1", "title", " Buy milk ")
        store.change("t1", "notes", "Note")
        precondition(store.tasks[0].title == " Buy milk " && store.tasks[0].notes == "Note")
        precondition(events.suffix(2).map(\.0) == ["renamed", "notesChanged"])
        store.change("t1", "done", true)
        precondition(store.visible.map(\.id) == ["t1", "t2"])
        precondition(store.tasks.map(\.id) == ["t1", "t2", "t3", "t5"])
        let empty = TodoStore(seed: "empty")
        precondition(empty.emptyText == "No tasks yet" && empty.left == 0)
        empty.chooseFilter("done"); precondition(empty.emptyText == "Nothing done yet")
        empty.add("One"); precondition(empty.tasks[0].id == "t1")
        empty.chooseFilter("active"); empty.change("t1", "done", true)
        precondition(empty.emptyText == "Nothing left to do")
        let many = TodoStore(seed: "many")
        precondition(many.tasks.count == 20 && many.left == 14)
        print("Store checks passed: fixture SHA-256, monotonic IDs, trim, filters, order, exact-change events, edits, empty states, many seed")
    }
}
