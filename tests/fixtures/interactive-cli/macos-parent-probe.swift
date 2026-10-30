// Read-only evidence for one synthetic updater child. Never a product authorization.
import Foundation
import Security
import Darwin

@_silgen_name("proc_pidpath")
func processPath(_ pid: Int32, _ buffer: UnsafeMutableRawPointer, _ size: UInt32) -> Int32

func inspect() -> [String: Any] {
    guard CommandLine.arguments.count == 4,
          let pid = Int32(CommandLine.arguments[1]) else { return [:] }
    let root = CommandLine.arguments[2]
    let hash = CommandLine.arguments[3]
    guard hash.count == 40, hash.allSatisfy({ $0.isHexDigit }) else { return [:] }
    var mib: [Int32] = [CTL_KERN, KERN_PROCARGS2, pid]
    let mibCount = UInt32(mib.count)
    var size = 0
    guard sysctl(&mib, mibCount, nil, &size, nil, 0) == 0,
          size >= 4, size <= 65_536 else { return [:] }
    var bytes = [UInt8](repeating: 0, count: size)
    let read = bytes.withUnsafeMutableBytes { buffer in
        sysctl(&mib, mibCount, buffer.baseAddress, &size, nil, 0)
    }
    guard read == 0, size >= 4, size <= bytes.count else { return [:] }
    let count = bytes.withUnsafeBytes { Int($0.loadUnaligned(as: Int32.self)) }
    guard (2...16).contains(count) else { return [:] }
    var offset = 4
    func nextString() -> String? {
        guard offset < size,
              let end = bytes[offset..<size].firstIndex(of: 0) else { return nil }
        let value = String(bytes: bytes[offset..<end], encoding: .utf8)
        offset = end + 1
        return value
    }
    guard let executable = nextString() else { return [:] }
    while offset < size && bytes[offset] == 0 { offset += 1 }
    var argv = [String]()
    for _ in 0..<count {
        guard let value = nextString(), value.utf8.count <= 4096 else { return [:] }
        argv.append(value)
    }
    // Do not parse or print the environment that follows argv in the kernel buffer.
    guard ["update", "install"].contains(argv[1]), argv.contains(root) else { return [:] }
    var image = [UInt8](repeating: 0, count: 4096)
    let pathStatus = image.withUnsafeMutableBytes { processPath(pid, $0.baseAddress!, UInt32($0.count)) }
    let imagePath = String(decoding: image.prefix(while: { $0 != 0 }), as: UTF8.self)
    var code: SecCode?
    let attributes: [String: Any] = ["pid": NSNumber(value: pid), "dynamicCode": true]
    let copied = SecCodeCopyGuestWithAttributes(nil, attributes as CFDictionary, SecCSFlags(rawValue: 0), &code)
    var requirement: SecRequirement?
    let parsed = SecRequirementCreateWithString("cdhash H\"\(hash)\"" as CFString, SecCSFlags(rawValue: 0), &requirement)
    let checked = copied == errSecSuccess && parsed == errSecSuccess
        ? SecCodeCheckValidity(code!, SecCSFlags(rawValue: 0), requirement) : OSStatus(-1)
    return ["argv0": argv[0], "action": argv[1], "kernel_executable": executable,
            "process_group": getpgid(pid), "session": getsid(pid),
            "image_path": imagePath, "image_path_status": pathStatus,
            "image_path_exists": FileManager.default.fileExists(atPath: imagePath),
            "code_copy_status": copied, "requirement_status": parsed,
            "code_validity_status": checked]
}

var observation = inspect()
// Observe unlink without pausing the original updater or changing its inputs.
if observation["action"] as? String == "update",
   observation["code_validity_status"] as? Int32 == 0,
   let path = observation["image_path"] as? String, path.contains("/.aigent-hive-"),
   let pid = Int32(CommandLine.arguments[1]) {
    let deadline = ProcessInfo.processInfo.systemUptime + 3
    var changes = [[String: Any]]()
    var previous = "\(observation["image_path_status"] ?? 0):\(path):\(FileManager.default.fileExists(atPath: path))"
    while ProcessInfo.processInfo.systemUptime < deadline && changes.count < 4 {
        var image = [UInt8](repeating: 0, count: 4096)
        let status = image.withUnsafeMutableBytes { processPath(pid, $0.baseAddress!, UInt32($0.count)) }
        let current = String(decoding: image.prefix(while: { $0 != 0 }), as: UTF8.self)
        let key = "\(status):\(current):\(FileManager.default.fileExists(atPath: current))"
        if key != previous {
            var next = inspect()
            next["observed_path_status"] = status
            next["process_present"] = getpgid(pid) != -1
            changes.append(next)
            previous = key
            if next["process_present"] as? Bool == false { break }
        }
        usleep(1000)
    }
    observation["path_changes"] = changes
}
let result = try JSONSerialization.data(withJSONObject: observation, options: [.sortedKeys])
print(String(decoding: result, as: UTF8.self))
