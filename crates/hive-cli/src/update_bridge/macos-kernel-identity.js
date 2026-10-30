// Read-only kernel identity of one parent. No Apple events, environment or file reads.
ObjC.import('Foundation');
ObjC.bindFunction('csops', ['int', ['int', 'unsigned int', 'void *', 'unsigned long']]);

function run(arguments) {
    if (arguments.length !== 1 || !/^[1-9][0-9]{0,9}$/.test(arguments[0])) {
        throw new Error('invalid parent pid');
    }
    var pid = Number(arguments[0]);
    if (pid > 2147483647) throw new Error('invalid parent pid');
    var before = Ref('unsigned int');
    var after = Ref('unsigned int');
    var hash = $.NSMutableData.dataWithLength(20);
    // XNU CS_OPS_STATUS=0, CS_OPS_CDHASH=5. Never accept an operation from input.
    if ($.csops(pid, 0, before, 4) !== 0 ||
        $.csops(pid, 5, hash.mutableBytes, 20) !== 0 ||
        $.csops(pid, 0, after, 4) !== 0 || before[0] !== after[0]) {
        throw new Error('parent kernel identity unavailable or changed');
    }
    return JSON.stringify({pid: pid, flags: after[0],
        code_directory_hash: ObjC.unwrap(hash.base64EncodedStringWithOptions(0))});
}
