# Version-qualified desktop coordination client. No app files, settings, or processes are changed.
param([string]$QualificationThread='')
$ErrorActionPreference = 'Stop'
if($QualificationThread -and $QualificationThread -notmatch '^[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}$'){throw 'Invalid qualification scope'}
$ProgressPreference = 'SilentlyContinue'
[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)
[Console]::InputEncoding = [Text.UTF8Encoding]::new($false)
function Assert-DesktopVersion([string]$version) {
    if($version -notin @('26.1013.4894.0','26.1016.6573.0')){throw 'Desktop version requires control qualification'}
}
Add-Type -ReferencedAssemblies 'System.Web.Extensions','System.Core' -TypeDefinition @'
using System;
using System.IO;
using System.IO.Pipes;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;
using System.Collections.Concurrent;
using System.Collections;
using System.Collections.Generic;
using System.Web.Script.Serialization;
public static class HiveDesktopJson {
    static IDictionary<string,object> Map(object value) { return value as IDictionary<string,object>; }
    static object Get(object value, string key) {
        var map=Map(value); object found;
        return map!=null && map.TryGetValue(key,out found) ? found : null;
    }
    static Dictionary<string,object> Pick(object value, params string[] keys) {
        var result=new Dictionary<string,object>(StringComparer.Ordinal);
        var map=Map(value);
        if(map!=null) foreach(string key in keys) if(map.ContainsKey(key)) result[key]=map[key];
        return result;
    }
    static IList Array(object value) { return value as IList ?? new object[0]; }
    static object Turn(object value) {
        var result=Pick(value,"turnId","status");
        if(Get(value,"itemsPagination")!=null) result["itemsPagination"]=true;
        return result;
    }
    static object State(object value) {
        var state=Pick(value,"id","hostId");
        state["threadRuntimeStatus"]=Pick(Get(value,"threadRuntimeStatus"),"type");
        state["threadGoal"]=Pick(Get(value,"threadGoal"),"status");
        var turns=new List<object>();
        foreach(object turn in Array(Get(value,"turns"))) turns.Add(Turn(turn));
        state["turns"]=turns;
        object original=Get(value,"turnHistory");
        var history=Pick(original,"kind");
        if((string)Get(original,"kind")=="canonical") {
            object content=Get(original,"history");
            var entities=new Dictionary<string,object>(StringComparer.Ordinal);
            var input=Map(Get(content,"entitiesByKey"));
            if(input==null || Get(content,"islands")==null) throw new IOException("Invalid turn order metadata");
            foreach(var pair in input) entities[pair.Key]=Turn(pair.Value);
            var islands=new List<object>();
            foreach(object island in Array(Get(content,"islands"))) {
                var entries=new List<object>();
                foreach(object entry in Array(Get(island,"entries"))) entries.Add(Pick(entry,"value"));
                islands.Add(new Dictionary<string,object>{{"entries",entries}});
            }
            history["history"]=new Dictionary<string,object>{{"entitiesByKey",entities},{"islands",islands}};
        }
        state["turnHistory"]=history;
        return state;
    }
    public static string Compact(string frame) {
        var json=new JavaScriptSerializer { MaxJsonLength=67108864, RecursionLimit=256 };
        object source=json.DeserializeObject(frame);
        var result=Pick(source,"type","requestId","sourceClientId","method","version","resultType","handledByClientId","error");
        string type=Get(source,"type") as string;
        if(type=="response") result["result"]=Pick(Get(source,"result"),"clientId","ok","goalPauseError","interruptedTurnId");
        else if(type=="broadcast") {
            object parameters=Get(source,"params");
            var filtered=Pick(parameters,"hostId","conversationId","following");
            if((string)Get(source,"method")=="thread-stream-state-changed") {
                object change=Get(parameters,"change");
                var selected=Pick(change,"type");
                if((string)Get(change,"type")=="snapshot") selected["conversationState"]=State(Get(change,"conversationState"));
                else {
                    var patches=new List<object>();
                    foreach(object patch in Array(Get(change,"patches"))) patches.Add(Pick(patch,"path"));
                    selected["patches"]=patches;
                }
                filtered["change"]=selected;
            }
            result["params"]=filtered;
        }
        string output=json.Serialize(result);
        if(Encoding.UTF8.GetByteCount(output)>262144) throw new IOException("Desktop metadata exceeds limit");
        return output;
    }
}
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
            if (size < 2 || size > 67108864) throw new IOException("Invalid desktop frame size");
            string frame = HiveDesktopJson.Compact(new UTF8Encoding(false, true).GetString(Exact(size)));
            if (Interlocked.Add(ref bufferedBytes, frame.Length * 2L) > 33554432 || !frames.TryAdd(frame)) throw new IOException("Desktop queue overflow");
        } } catch { Alive = false; }
    }
    public string Poll() { string value; if (!frames.TryTake(out value)) return null; Interlocked.Add(ref bufferedBytes, -value.Length * 2L); return value; }
    public void Send(string value) {
        byte[] bytes = Encoding.UTF8.GetBytes(value);
        if (bytes.Length > 16384) throw new IOException("Oversized control request");
        byte[] frame = new byte[bytes.Length + 4];
        Buffer.BlockCopy(BitConverter.GetBytes(bytes.Length), 0, frame, 0, 4);
        Buffer.BlockCopy(bytes, 0, frame, 4, bytes.Length);
        var write = pipe.WriteAsync(frame, 0, frame.Length);
        if (!write.Wait(500)) { Alive = false; pipe.Dispose(); throw new IOException("Desktop write timeout"); }
        write.GetAwaiter().GetResult();
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
    Assert-DesktopVersion ([string]$matching[0].Version)
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
        $pending[$id]=@{method=$method;thread=$thread;at=$clock.ElapsedMilliseconds;incident=if($null -ne $stop){$stop.digest}else{''}}
        Send $value
    }
    function Follow([string]$id, [bool]$following=$true) {
        Send @{type='broadcast';sourceClientId=$client;method='thread-stream-following-changed';version=1;params=@{hostId='local';conversationId=$id;following=$following}}
        $sessions[$id].subscribed=$following
    }
    function Track([string]$id) {
        if($QualificationThread -and $id -ne $QualificationThread){return}
        if ($id -notmatch '^[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}$') {throw 'Invalid desktop thread identity'}
        if (-not $sessions.ContainsKey($id)) {
            if ($sessions.Count -ge 4096) {throw 'Too many desktop sessions'}
            $sessions[$id]=@{id=$id;ready=$false;dirty=$false;subscribed=$false;active=@();statuses=@{};lastResponse='none';lastStop=-2000L;incident='';runtime='unknown';goal='';snapshotAt=-2000L;observedAt=-1L}
        }
    }
    function Session-Digest([string]$id) {
        $hash=[Security.Cryptography.SHA256]::Create()
        try {return 'sha256:'+([BitConverter]::ToString($hash.ComputeHash([Text.Encoding]::UTF8.GetBytes(('codex'+[char]0+$id))))).Replace('-','').ToLowerInvariant()}finally{$hash.Dispose()}
    }
    function New-Stop([string]$digest,[long]$detected,$scope=$null) {
        $began=$clock.ElapsedMilliseconds
        $age=[Math]::Max([long]0,[long]([DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()-$detected))
        $eligible=@{};foreach($id in @($sessions.Keys)+@($catalogSeen.Keys)){if($id -and ($null -eq $scope -or (Session-Digest $id) -in @($scope))){$eligible[$id]=$true}}
        return @{eligible=$eligible;scope=$scope;requested=@{};began=$began;verifyUntil=$began+[Math]::Max(0,5000-$age);digest=$digest;detected_ms=$detected;age_at_begin=$age;settled=$false;targeted=@{};request_ms=$null}
    }
    function Stop-Age {return $stop.age_at_begin+$clock.ElapsedMilliseconds-$stop.began}
    function Release-Subscriptions {
        foreach($session in @($sessions.Values)) {
            if($session.subscribed){Follow $session.id $false}
        }
    }
    function Register-Target([string]$id,[string]$turn,[long]$requestedAt=-1) {
        if($null -eq $stop -or $turn -notmatch '^[A-Za-z0-9_-]{1,128}$'){return}
        $key=$id+':'+$turn
        if($stop.targeted.ContainsKey($key)){return}
        $stop.targeted[$key]=@{thread=$id;turn=$turn}
        if($requestedAt -ge 0){$stop.request_ms=[Math]::Max([long]$stop.request_ms,$stop.age_at_begin+$requestedAt-$stop.began)}
    }
    function Interrupt-Response($request,$message) {
        $session=$sessions[$request.thread]
        if($null -eq $session -or $null -eq $stop -or $request.incident -ne $stop.digest){return}
        $session.lastResponse=if ($message.resultType -eq 'success' -and $message.result.ok -and -not $message.result.goalPauseError) { 'accepted' } else { 'failed' }
        $turn=[string]$message.result.interruptedTurnId
        $fresh=$turn -and -not $stop.targeted.ContainsKey($request.thread+':'+$turn)
        if($message.resultType -eq 'success' -and $message.result.ok) {Register-Target $request.thread $turn $request.at}
        if($fresh -or $session.lastResponse -eq 'failed'){$session.dirty=$true}
    }
    function Stop-Finished {return ($null -ne $stop -and ($stop.settled -or $clock.ElapsedMilliseconds -ge $stop.verifyUntil))}
    function Advance-Stop {
        if($null -eq $stop){return}
        $selected=@($sessions.Values | Where-Object {$stop.eligible.ContainsKey($_.id)})
        foreach($session in $selected) {
            if($stop.eligible.ContainsKey($session.id) -and -not $stop.requested.ContainsKey($session.id) -and $clock.ElapsedMilliseconds -lt $stop.verifyUntil) {
                $stop.requested[$session.id]=$true;
                Request 'thread-follower-interrupt-turn' @{conversationId=$session.id;mode='user-stop'} 3 '' $session.id
                $session.lastStop=$clock.ElapsedMilliseconds
                if($session.incident -ne $stop.digest){
                    $stop.request_ms=[Math]::Max([long]$stop.request_ms,(Stop-Age))
                    $session.ready=$false;$session.lastResponse='none';$session.incident=$stop.digest
                }
            }
        }
        if((Stop-Age) -lt 1800 -and @($selected | Where-Object lastResponse -eq 'none').Count -gt 0){return}
        $waiting=@($selected | Where-Object {$_.subscribed -and -not $_.ready}).Count -gt 0
        $unread=@($selected | Where-Object ready -eq $false).Count -gt 0
        foreach($session in $selected) {
            if($clock.ElapsedMilliseconds -ge $stop.verifyUntil -or $stop.settled) {
                if($session.subscribed){Follow $session.id $false}
            } elseif(-not $waiting -and (-not $session.subscribed -or (-not $unread -and $session.ready -and $session.dirty -and $clock.ElapsedMilliseconds-$session.snapshotAt -ge 1000))) {
                Follow $session.id
                $session.ready=$false;$waiting=$true;$session.snapshotAt=$clock.ElapsedMilliseconds
            }
        }
    }
    function Count-Unqualified {
        $count=0
        foreach ($id in $catalogSeen.Keys) {
            if($null -ne $stop -and $null -ne $stop.scope -and -not $stop.eligible.ContainsKey($id)){continue}
            if (-not $sessions.ContainsKey($id) -and
                (-not $catalogChecked.ContainsKey($id) -or $clock.ElapsedMilliseconds-$catalogChecked[$id] -ge 20000)) {$count++}
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
        if(-not $session.subscribed -or ($null -ne $stop -and $clock.ElapsedMilliseconds -ge $stop.verifyUntil)){return}
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
        foreach($turn in $session.active){Register-Target $session.id $turn $session.lastStop}
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
        $drainUntil=$clock.ElapsedMilliseconds+20
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
                        Interrupt-Response $request $message
                    }
                }
                'broadcast' {
                    if ($message.sourceClientId -eq $client) { continue }
                    if ($message.method -eq 'thread-stream-following-changed' -and $message.version -eq 1 -and
                        $message.params.hostId -eq 'local' -and $message.params.following -eq $true) {
                        $id=[string]$message.params.conversationId
                        if(( -not $QualificationThread -or $id -eq $QualificationThread) -and $id -match '^[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}$' -and -not $sessions.ContainsKey($id) -and -not $catalogSeen.ContainsKey($id)) {
                            $catalogSeen[$id]=$true;$catalogQueue.Enqueue($id)
                        }
                    } elseif ($message.method -eq 'thread-stream-state-changed' -and $message.version -eq 11) { Snapshot $message.params }
                }
            }
            if($clock.ElapsedMilliseconds -ge $drainUntil){break}
        }
        $line=$inputReader.Poll()
        if ($null -ne $line) {
            $command=$line | ConvertFrom-Json
            if ($command.operation -eq 'stop' -and $command.incident -match '^sha256:[a-f0-9]{64}$') {
                if ($null -eq $stop -or $stop.digest -ne $command.incident) {
                    $detected=[DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
                    if ($command.detected_at_ms -gt 0) {$detected=[long]$command.detected_at_ms}
                    Release-Subscriptions
                    $stop=New-Stop ([string]$command.incident) $detected $command.scope
                }
            } elseif ($command.operation -eq 'discover') {
                $catalogReady=($command.valid -eq $true);$catalogAt=$clock.ElapsedMilliseconds
                if ($catalogReady -and -not $catalogInProgress) {
                    if (@($command.ids).Count -gt 4096) {throw 'Catalog exceeds session limit'}
                    $catalogSeen=@{};$catalogInProgress=$true;$catalogRoundFault=$false
                    foreach ($id in @($command.ids)) {
                        if ($id -notmatch '^[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}$' -or $catalogSeen.ContainsKey($id)) {throw 'Invalid catalog identity'}
                        if($QualificationThread -and $id -ne $QualificationThread){throw 'Catalog outside qualification scope'}
                        $catalogSeen[$id]=$true
                        if (-not $sessions.ContainsKey($id)) {$catalogQueue.Enqueue($id)}
                    }
                }
            } elseif ($command.operation -eq 'clear') {Release-Subscriptions;$stop=$null}
            else { throw 'Invalid desktop control operation' }
        }
        foreach ($id in @($pending.Keys)) { if ($clock.ElapsedMilliseconds - $pending[$id].at -gt 2000) { if($pending[$id].method -eq 'thread-owner-discovery'){$catalogReady=$false;$catalogFault=$true;$catalogRoundFault=$true};$pending.Remove($id) } }
        if ($clock.ElapsedMilliseconds-$catalogAt -ge 20000) {$catalogReady=$false;$catalogFault=$true}
        # Do not spend the interruption deadline draining a large ownership catalog.
        Advance-Stop
        $available=16-@($pending.Values | Where-Object method -eq 'thread-owner-discovery').Count
        while ($client -ne 'initializing-client' -and $available -gt 0 -and $catalogQueue.Count -gt 0) {
            $id=$catalogQueue.Dequeue();Request 'thread-owner-discovery' @{hostId='local';conversationId=$id} 1 '' $id;$available--
        }
        if ($catalogInProgress -and $catalogQueue.Count -eq 0 -and @($pending.Values | Where-Object method -eq 'thread-owner-discovery').Count -eq 0) {
            $catalogInProgress=$false
            if (-not $catalogRoundFault) {$catalogFault=$false}
        }
        if($null -eq $stop) {
            foreach($session in @($sessions.Values)) {
                if((-not $session.subscribed -or $session.dirty -or $clock.ElapsedMilliseconds-$session.snapshotAt -ge 15000) -and $clock.ElapsedMilliseconds-$session.lastStop -ge 1000) {
                    if($session.subscribed){Follow $session.id $false};Follow $session.id
                    $session.lastStop=$clock.ElapsedMilliseconds
                }
            }
        }
        if ($client -ne 'initializing-client' -and $clock.ElapsedMilliseconds-$lastPublish -ge 250) {
            $activeCount=0;$unknown=0
            if(($null -eq $stop -or $null -eq $stop.scope) -and (-not $catalogReady -or $catalogFault)){$unknown++}
            if($null -ne $stop -and $null -ne $stop.scope -and $stop.eligible.Count -ne @($stop.scope).Count){$unknown++}
            $unknown += Count-Unqualified
            foreach ($session in $sessions.Values) {
                if($null -eq $stop -or $stop.eligible.ContainsKey($session.id)){$activeCount+=$session.active.Count;if ($null -ne $stop -and -not $session.ready) {$unknown++}}
            }
            $receipt=$null
            if ($null -ne $stop) {
                $confirmed=0;$confirmedSessions=@()
                foreach ($target in $stop.targeted.Values) {
                    $session=$sessions[$target.thread]
                    if (Confirmed-Target $target $stop.began) { $confirmed++;$confirmedSessions+=(Session-Digest $target.thread) }
                }
                $hash=[Security.Cryptography.SHA256]::Create()
                try {$scope='sha256:'+([BitConverter]::ToString($hash.ComputeHash([Text.Encoding]::UTF8.GetBytes((@($stop.targeted.Keys | Sort-Object) -join '|'))))).Replace('-','').ToLowerInvariant()}finally{$hash.Dispose()}
                $stop.settled=($confirmed -gt 0 -and $confirmed -eq $stop.targeted.Count -and $activeCount -eq 0 -and $unknown -eq 0)
                if($stop.settled){Release-Subscriptions}
                $receipt=@{finished=(Stop-Finished);unknown_sessions=$unknown;active_turns=$activeCount;app_alive=$true;scope_digest=$scope;confirmed_sessions=@($confirmedSessions | Sort-Object -Unique);incident=$stop.digest;targeted=$stop.targeted.Count;confirmed=$confirmed;verified=($confirmed -gt 0 -and $confirmed -eq $stop.targeted.Count -and $activeCount -eq 0 -and $unknown -eq 0);elapsed_ms=(Stop-Age);request_ms=$stop.request_ms}
            }
            $stopping=(($null -ne $stop -and -not (Stop-Finished)) -or @($pending.Values | Where-Object method -eq 'thread-follower-interrupt-turn').Count -gt 0)
            $activity=@($sessions.Values | Where-Object {$_.ready -and -not $_.dirty -and $_.active.Count -gt 0} | ForEach-Object {@{id=$_.id;turn=$_.active[-1]}})
            $packet=@{activity_observed=($unknown -eq 0);active_sessions=$activity;control_stopping=$stopping;schema_version=1;connected=$true;app_alive=$true;server_pid=$pipe.ServerPid;known_sessions=$sessions.Count;active_turns=$activeCount;unknown_sessions=$unknown;stop=$receipt}
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
