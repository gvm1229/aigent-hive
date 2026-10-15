# Version-qualified desktop coordination client. No app files, settings, or processes are changed.
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)
[Console]::InputEncoding = [Text.UTF8Encoding]::new($false)
Add-Type -TypeDefinition @'
using System;
using System.IO;
using System.IO.Pipes;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;
using System.Collections.Concurrent;
public sealed class HiveDesktopPipe : IDisposable {
    [DllImport("kernel32.dll", SetLastError=true)]
    static extern bool GetNamedPipeServerProcessId(IntPtr pipe, out uint pid);
    readonly NamedPipeClientStream pipe = new NamedPipeClientStream(".", "codex-ipc", PipeDirection.InOut, PipeOptions.Asynchronous);
    readonly BlockingCollection<string> frames = new BlockingCollection<string>(64);
    long bufferedBytes;
    public uint ServerPid;
    public volatile bool Alive = true;
    public HiveDesktopPipe() {
        pipe.Connect(700);
        if (!GetNamedPipeServerProcessId(pipe.SafePipeHandle.DangerousGetHandle(), out ServerPid)) throw new IOException("Cannot identify desktop peer");
    }
    public void Start() { var thread = new Thread(ReadLoop); thread.IsBackground = true; thread.Start(); }
    byte[] Exact(int size) {
        byte[] bytes = new byte[size]; int offset = 0;
        while (offset < size) { int n = pipe.Read(bytes, offset, size - offset); if (n == 0) throw new EndOfStreamException(); offset += n; }
        return bytes;
    }
    void ReadLoop() {
        try { while (Alive) {
            int size = BitConverter.ToInt32(Exact(4), 0);
            if (size < 2 || size > 16777216) throw new IOException("Invalid desktop frame size");
            string frame = new UTF8Encoding(false, true).GetString(Exact(size));
            if (Interlocked.Add(ref bufferedBytes, frame.Length * 2L) > 33554432 || !frames.TryAdd(frame)) throw new IOException("Desktop queue overflow");
        } } catch { Alive = false; }
    }
    public string Poll() { string value; if (!frames.TryTake(out value)) return null; Interlocked.Add(ref bufferedBytes, -value.Length * 2L); return value; }
    public void Send(string value) {
        byte[] bytes = Encoding.UTF8.GetBytes(value);
        if (bytes.Length > 16384) throw new IOException("Oversized control request");
        byte[] header = BitConverter.GetBytes(bytes.Length);
        pipe.Write(header, 0, 4); pipe.Write(bytes, 0, bytes.Length); pipe.Flush();
    }
    public void Dispose() { Alive = false; pipe.Dispose(); frames.Dispose(); }
}
public sealed class HiveDesktopInput {
    readonly BlockingCollection<string> commands = new BlockingCollection<string>(4);
    public volatile bool Alive = true;
    public HiveDesktopInput() { var thread = new Thread(ReadLoop); thread.IsBackground = true; thread.Start(); }
    void ReadLoop() { try { string line; while ((line = Console.In.ReadLine()) != null) {
        if (line.Length > 262144 || !commands.TryAdd(line)) break;
    } } finally { Alive = false; } }
    public string Poll() { string value; return commands.TryTake(out value) ? value : null; }
}
'@
$pipe = $null
try {
    $pipe = [HiveDesktopPipe]::new()
    $peer = Get-CimInstance Win32_Process -Filter ('ProcessId=' + $pipe.ServerPid)
    $packages = @(Get-AppxPackage -Name OpenAI.Codex | Where-Object PublisherId -eq '2p2nqsd0c76g0')
    $matching = @($packages | Where-Object {
        $peer.ExecutablePath -and $peer.ExecutablePath.StartsWith(([IO.Path]::GetFullPath($_.InstallLocation).TrimEnd('\') + '\'), [StringComparison]::OrdinalIgnoreCase)
    })
    $owner = Invoke-CimMethod -InputObject $peer -MethodName GetOwner
    if ($matching.Count -ne 1 -or $peer.Name -ne 'ChatGPT.exe' -or $owner.ReturnValue -ne 0 -or
        ($owner.Domain + '\' + $owner.User) -ne [Security.Principal.WindowsIdentity]::GetCurrent().Name) { throw 'Untrusted desktop pipe peer' }
    # This private coordination protocol is qualified only for this exact tested package.
    if ([string]$matching[0].Version -ne '26.1007.9446.0') { throw 'Desktop version requires control qualification' }
    $pin = [Diagnostics.Process]::GetProcessById([int]$pipe.ServerPid)
    $null = $pin.Handle
    if ($pin.MainModule.FileName -ne $peer.ExecutablePath -or
        [Math]::Abs(($pin.StartTime.ToUniversalTime() - $peer.CreationDate.ToUniversalTime()).TotalMilliseconds) -gt 1) { throw 'Desktop peer identity changed' }
    $pipe.Start()
    $client = 'initializing-client'
    $sessions = @{}
    $pending = @{}
    $stop = $null
    $catalogReady = $false
    $catalogFault = $false
    $catalogQueue = [Collections.Generic.Queue[string]]::new()
    $catalogSeen = @{}
    $catalogChecked = @{}
    $catalogInProgress = $false
    $catalogRoundFault = $false
    $catalogAt = -10000L
    $clock = [Diagnostics.Stopwatch]::StartNew()
    $lastPublish = -1000L
    function Send($value) { $pipe.Send((ConvertTo-Json -InputObject $value -Depth 12 -Compress)) }
    function Request([string]$method, $params, [int]$version, [string]$target, [string]$thread) {
        $id = [Guid]::NewGuid().ToString()
        $value = @{type='request';requestId=$id;sourceClientId=$client;method=$method;params=$params;version=$version;timeoutMs=1800}
        if ($target) { $value.targetClientId=$target }
        $pending[$id]=@{method=$method;thread=$thread;at=$clock.ElapsedMilliseconds}
        Send $value
    }
    function Follow([string]$id) {
        Send @{type='broadcast';sourceClientId=$client;method='thread-stream-following-changed';version=1;params=@{hostId='local';conversationId=$id;following=$true}}
    }
    function Track([string]$id) {
        if ($id -notmatch '^[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}$') {throw 'Invalid desktop thread identity'}
        if (-not $sessions.ContainsKey($id)) {
            if ($sessions.Count -ge 4096) {throw 'Too many desktop sessions'}
            $sessions[$id]=@{id=$id;ready=$false;dirty=$true;active=@();statuses=@{};lastResponse='none';lastStop=-2000L;interruptedTurn='';runtime='unknown';goal='';snapshotAt=-2000L;observedAt=-1L}
            Follow $id
        }
    }
    function Count-Unqualified {
        $count=0
        foreach ($id in $catalogSeen.Keys) {
            if (-not $sessions.ContainsKey($id) -and
                (-not $catalogChecked.ContainsKey($id) -or $clock.ElapsedMilliseconds-$catalogChecked[$id] -gt 8000)) {$count++}
        }
        return $count
    }
    function Visible-Turns($state) {
        $canonical=@()
        if ($state.turnHistory.kind -eq 'canonical') {
            $history=$state.turnHistory.history
            if ($null -eq $history.islands) {throw 'Missing canonical turn order'}
            foreach ($island in @($history.islands)) {
                foreach ($entry in @($island.entries)) {
                    $property=$history.entitiesByKey.PSObject.Properties[[string]$entry.value]
                    if ($null -eq $property) {throw 'Missing canonical turn entity'}
                    $turn=$property.Value
                    if ($turn.turnId -and $turn.status) {$canonical+=@($turn)}
                }
            }
        } else {return @($state.turns | Where-Object {$_.turnId -and $_.status})}
        $known=@{};foreach($turn in $canonical){$known[[string]$turn.turnId]=$true}
        $incoming=@{};$before=@{};$buffer=[Collections.Generic.List[object]]::new()
        foreach($turn in @($state.turns)) {
            if(-not $turn.turnId -or -not $turn.status){continue}
            $id=[string]$turn.turnId;$incoming[$id]=$turn
            if($known.ContainsKey($id)) {
                if($buffer.Count -gt 0){$before[$id]=@($before[$id])+@($buffer.ToArray());$buffer.Clear()}
            } else {$buffer.Add($turn)}
        }
        $ordered=[Collections.Generic.List[object]]::new()
        foreach($turn in $canonical) {
            $id=[string]$turn.turnId
            foreach($extra in @($before[$id])){if($null -ne $extra){$ordered.Add($extra)}}
            $status=[string]$turn.status;$overlay=$incoming[$id]
            if($null -ne $overlay -and -not ($status -ne 'inProgress' -and $overlay.status -eq 'inProgress' -and
                ($null -ne $turn.itemsPagination -or $null -ne $overlay.itemsPagination))) {$status=[string]$overlay.status}
            $ordered.Add([pscustomobject]@{turnId=$id;status=$status})
        }
        foreach($extra in $buffer){$ordered.Add($extra)}
        return $ordered.ToArray()
    }
    function Confirmed-Target($target,[long]$began) {
        $session=$sessions[$target.thread]
        return ($session.ready -and $session.observedAt -ge $began -and
            $session.statuses[$target.turn] -eq 'interrupted' -and $session.goal -ne 'active')
    }
    function Snapshot($snapshot) {
        if ($snapshot.hostId -ne 'local' -or -not $sessions.ContainsKey([string]$snapshot.conversationId)) { return }
        $session=$sessions[[string]$snapshot.conversationId]
        if ($snapshot.change.type -ne 'snapshot') {
            # Content deltas must not cause repeated full-history snapshots during reasoning.
            foreach ($patch in @($snapshot.change.patches)) {
                $path=@($patch.path)
                if ($path.Count -gt 0 -and $path[0] -is [string] -and $path[0].StartsWith('/')) {$path=@($path[0].TrimStart('/').Split('/'))}
                if ($path.Count -eq 0) {continue}
                $root=[string]$path[0];$leaf=[string]$path[-1]
                if ($root -in @('threadRuntimeStatus','threadGoal') -or
                    ($root -eq 'turnHistory' -and ($path.Count -le 4 -or ($path.Count -eq 5 -and $leaf -in @('status','turnId')))) -or
                    ($root -eq 'turns' -and ($path.Count -le 2 -or ($path.Count -eq 3 -and $leaf -in @('status','turnId'))))) {$session.dirty=$true;break}
            }
            return
        }
        $state=$snapshot.change.conversationState
        if ($state.id -ne $session.id -or $state.hostId -ne 'local') { throw 'Mismatched desktop snapshot' }
        $turns=@(Visible-Turns $state)
        $session.active=@()
        if($turns.Count -gt 0 -and $turns[-1].status -eq 'inProgress') {$session.active=@([string]$turns[-1].turnId)}
        $session.statuses=@{}
        foreach($turn in $turns){$session.statuses[[string]$turn.turnId]=[string]$turn.status}
        $session.observedAt=$clock.ElapsedMilliseconds
        $session.runtime=[string]$state.threadRuntimeStatus.type
        $session.goal=[string]$state.threadGoal.status
        $session.ready=$true;$session.dirty=$false;$session.snapshotAt=$clock.ElapsedMilliseconds
        # No prompt, tool output, history item, or file path is retained in this client.
    }
    Request 'initialize' @{clientType='aigent-hive-usage-monitor'} 0 '' ''
    $inputReader=[HiveDesktopInput]::new()
    while ($pipe.Alive -and $inputReader.Alive -and -not $pin.HasExited) {
        for ($count=0; $count -lt 32; $count++) {
            $frame=$pipe.Poll();if ($null -eq $frame) { break }
            $message=$frame | ConvertFrom-Json
            switch ($message.type) {
                'client-discovery-request' { Send @{type='client-discovery-response';requestId=$message.requestId;response=@{canHandle=$false}} }
                'response' {
                    $request=$pending[[string]$message.requestId]
                    if ($null -eq $request) { continue }
                    $pending.Remove([string]$message.requestId)
                    if ($request.method -eq 'initialize') {
                        if ($message.resultType -ne 'success' -or -not $message.result.clientId) { throw 'Desktop registration failed' }
                        $client=[string]$message.result.clientId
                    } elseif ($request.method -eq 'thread-owner-discovery') {
                        if ($message.resultType -eq 'success' -and $message.handledByClientId) {Track $request.thread;$catalogChecked[$request.thread]=$clock.ElapsedMilliseconds}
                        elseif ($message.resultType -eq 'error' -and $message.error -eq 'no-client-found') {$catalogChecked[$request.thread]=$clock.ElapsedMilliseconds}
                        else {$catalogReady=$false;$catalogFault=$true;$catalogRoundFault=$true}
                    } elseif ($request.method -eq 'thread-follower-interrupt-turn') {
                        $session=$sessions[$request.thread]
                        $session.lastResponse=if ($message.resultType -eq 'success' -and $message.result.ok -and -not $message.result.goalPauseError) { 'accepted' } else { 'failed' }
                        $session.interruptedTurn=[string]$message.result.interruptedTurnId
                        $session.dirty=$true
                    }
                }
                'broadcast' {
                    if ($message.sourceClientId -eq $client) { continue }
                    if ($message.method -eq 'thread-stream-following-changed' -and $message.version -eq 1 -and
                        $message.params.hostId -eq 'local' -and $message.params.following -eq $true) {
                        $id=[string]$message.params.conversationId
                        Track $id
                    } elseif ($message.method -eq 'thread-stream-state-changed' -and $message.version -eq 11) { Snapshot $message.params }
                }
            }
        }
        $line=$inputReader.Poll()
        if ($null -ne $line) {
            $command=$line | ConvertFrom-Json
            if ($command.operation -eq 'stop' -and $command.incident -match '^sha256:[a-f0-9]{64}$') {
                if ($null -eq $stop -or $stop.digest -ne $command.incident) {
                    $detected=[DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
                    if ($command.detected_at_ms -gt 0) {$detected=[long]$command.detected_at_ms}
                    $stop=@{began=$clock.ElapsedMilliseconds;digest=[string]$command.incident;detected_ms=$detected;settled=$false;targeted=@{};request_ms=$null}
                }
            } elseif ($command.operation -eq 'discover') {
                $catalogReady=($command.valid -eq $true);$catalogAt=$clock.ElapsedMilliseconds
                if ($catalogReady -and -not $catalogInProgress) {
                    if (@($command.ids).Count -gt 4096) {throw 'Catalog exceeds session limit'}
                    $catalogSeen=@{};$catalogInProgress=$true;$catalogRoundFault=$false
                    foreach ($id in @($command.ids)) {
                        if ($id -notmatch '^[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}$' -or $catalogSeen.ContainsKey($id)) {throw 'Invalid catalog identity'}
                        $catalogSeen[$id]=$true
                        if (-not $sessions.ContainsKey($id)) {$catalogQueue.Enqueue($id)}
                    }
                }
            } elseif ($command.operation -eq 'clear') { $stop=$null }
            else { throw 'Invalid desktop control operation' }
        }
        foreach ($id in @($pending.Keys)) { if ($clock.ElapsedMilliseconds - $pending[$id].at -gt 2000) { if($pending[$id].method -eq 'thread-owner-discovery'){$catalogReady=$false;$catalogFault=$true;$catalogRoundFault=$true};$pending.Remove($id) } }
        if ($clock.ElapsedMilliseconds-$catalogAt -gt 8000) {$catalogReady=$false;$catalogFault=$true}
        $available=16-@($pending.Values | Where-Object method -eq 'thread-owner-discovery').Count
        while ($client -ne 'initializing-client' -and $available -gt 0 -and $catalogQueue.Count -gt 0) {
            $id=$catalogQueue.Dequeue();Request 'thread-owner-discovery' @{hostId='local';conversationId=$id} 1 '' $id;$available--
        }
        if ($catalogInProgress -and $catalogQueue.Count -eq 0 -and @($pending.Values | Where-Object method -eq 'thread-owner-discovery').Count -eq 0) {
            $catalogInProgress=$false
            if (-not $catalogRoundFault) {$catalogFault=$false}
        }
        foreach ($session in @($sessions.Values)) {
            if (($session.dirty -and $clock.ElapsedMilliseconds - $session.snapshotAt -gt 200) -or $clock.ElapsedMilliseconds - $session.snapshotAt -gt 5000) { Follow $session.id;$session.snapshotAt=$clock.ElapsedMilliseconds }
            if ($null -ne $stop -and $session.ready -and ($session.active.Count -gt 0 -or $session.goal -eq 'active') -and
                $clock.ElapsedMilliseconds - $session.lastStop -ge 1000) {
                foreach ($turn in $session.active) {
                    $key=$session.id + ':' + $turn
                    if (-not $stop.targeted.ContainsKey($key) -and $stop.settled) { $stop.detected_ms=[DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds();$stop.request_ms=$null;$stop.settled=$false }
                    $stop.targeted[$key]=@{thread=$session.id;turn=$turn}
                }
                Request 'thread-follower-interrupt-turn' @{conversationId=$session.id;mode='user-stop'} 3 '' $session.id
                $session.lastStop=$clock.ElapsedMilliseconds
                if ($null -eq $stop.request_ms) { $stop.request_ms=[DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()-$stop.detected_ms }
            }
        }
        if ($client -ne 'initializing-client' -and $clock.ElapsedMilliseconds-$lastPublish -ge 250) {
            $activeCount=0;$unknown=0
            if(-not $catalogReady -or $catalogFault){$unknown++}
            $unknown += Count-Unqualified
            foreach ($session in $sessions.Values) { $activeCount+=$session.active.Count;if (-not $session.ready) {$unknown++} }
            $receipt=$null
            if ($null -ne $stop) {
                $confirmed=0
                foreach ($target in $stop.targeted.Values) {
                    $session=$sessions[$target.thread]
                    if (Confirmed-Target $target $stop.began) { $confirmed++ }
                }
                $hash=[Security.Cryptography.SHA256]::Create()
                try {$scope='sha256:'+([BitConverter]::ToString($hash.ComputeHash([Text.Encoding]::UTF8.GetBytes((@($stop.targeted.Keys | Sort-Object) -join '|'))))).Replace('-','').ToLowerInvariant()}finally{$hash.Dispose()}
                $stop.settled=($confirmed -gt 0 -and $confirmed -eq $stop.targeted.Count -and $activeCount -eq 0 -and $unknown -eq 0)
                $receipt=@{unknown_sessions=$unknown;active_turns=$activeCount;app_alive=$true;scope_digest=$scope;incident=$stop.digest;targeted=$stop.targeted.Count;confirmed=$confirmed;verified=($confirmed -gt 0 -and $confirmed -eq $stop.targeted.Count -and $activeCount -eq 0 -and $unknown -eq 0);elapsed_ms=[DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()-$stop.detected_ms;request_ms=$stop.request_ms}
            }
            $stopping=($null -ne $stop -or @($pending.Values | Where-Object method -eq 'thread-follower-interrupt-turn').Count -gt 0)
            $packet=@{control_stopping=$stopping;schema_version=1;connected=$true;app_alive=$true;server_pid=$pipe.ServerPid;known_sessions=$sessions.Count;active_turns=$activeCount;unknown_sessions=$unknown;stop=$receipt}
            [Console]::Out.WriteLine((ConvertTo-Json -InputObject $packet -Depth 6 -Compress));[Console]::Out.Flush()
            $lastPublish=$clock.ElapsedMilliseconds
        }
        Start-Sleep -Milliseconds 20
    }
} catch {
    # Stable error class only: no native payload, prompts, or host paths in durable state.
    [Console]::Out.WriteLine('{"schema_version":1,"connected":false,"app_alive":true,"error":"desktop-control-unavailable"}')
    exit 1
} finally {
    if ($null -ne $pipe) { $pipe.Dispose() }
    if ($null -ne $pin) { $pin.Dispose() }
}
