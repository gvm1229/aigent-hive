# Nonshipping display-only preview. Never connects to Codex or starts a monitor.
param(
    [Parameter(Mandatory=$true)][string]$SourceRoot,
    [Parameter(Mandatory=$true)][string]$FixtureRoot,
    [ValidateSet('reset','gap','partial','ack','unavailable')][string]$Case='reset',
    [ValidateRange(460,900)][int]$Width=560,
    [switch]$ExpandDetails,
    [switch]$CloseAfterCapture,
    [ValidateSet('button','escape')][string]$CloseWith='button'
)
$ErrorActionPreference='Stop'
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$source=[IO.Path]::GetFullPath($SourceRoot).TrimEnd('\')
$fixture=[IO.Path]::GetFullPath($FixtureRoot).TrimEnd('\')
if(-not $fixture.StartsWith((Join-Path $source 'tests\work\usage-notice-preview')+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Preview fixture outside owned test root'}
$check=$fixture
while($check){
    if((Test-Path -LiteralPath $check) -and ((Get-Item -LiteralPath $check).Attributes -band [IO.FileAttributes]::ReparsePoint)){throw 'Linked preview path refused'}
    $check=[IO.Path]::GetDirectoryName($check)
}
$utf8=[Text.UTF8Encoding]::new($false)
$config=Join-Path $fixture '.hive/runtime/usage-monitor'
$null=New-Item -ItemType Directory -Path $config -Force
$incident=@{digest='sha256:'+('a'*64);reason=if($Case -eq 'gap'){'sensor-unresponsive'}else{'usage-reset'};detected_at=[DateTimeOffset]::UtcNow.ToUnixTimeSeconds();stop_confirmed=$true;stop_error=$null;targeted_turns=4;confirmed_turns=4}
$state=@{incident=$incident;app_alive=$true;control_connected=$true;active_turns=0;unknown_sessions=0}
if($Case -eq 'partial'){$incident.stop_confirmed=$false;$incident.confirmed_turns=3;$incident.stop_error='session-scope-unverified';$state.unknown_sessions=1}
if($Case -eq 'ack'){$state.incident=$null}
$statePath=Join-Path $config 'state.json'
if($Case -ne 'unavailable'){[IO.File]::WriteAllText($statePath,($state|ConvertTo-Json -Depth 6),$utf8)}
[IO.File]::WriteAllText((Join-Path $config 'config.json'),'{"enabled":true}',$utf8)
$tokens=$null;$parseErrors=$null
$text=[IO.File]::ReadAllText((Join-Path $source 'crates/hive-cli/src/usage_monitor/notice.ps1'),[Text.Encoding]::UTF8)
$tree=[Management.Automation.Language.Parser]::ParseInput($text,[ref]$tokens,[ref]$parseErrors)
if($parseErrors){throw 'Notice parsing failed'}
$tree.FindAll({param($node)$node -is [Management.Automation.Language.FunctionDefinitionAst]},$false)|ForEach-Object {Invoke-Expression $_.Extent.Text}
function Preview-Hash([string]$path) {
    if(-not(Test-Path -LiteralPath $path)){return 'absent'}
    $stream=[IO.File]::OpenRead($path)
    try{return [BitConverter]::ToString([Security.Cryptography.SHA256]::Create().ComputeHash($stream)).Replace('-','').ToLowerInvariant()}
    finally{$stream.Dispose()}
}
$before=Preview-Hash $statePath
$context=@{incident=$incident.digest;reason=$incident.reason;detected_at=$incident.detected_at;is_test=$false;preview=$true}
$window=New-NoticeWindow $context $fixture
$window.Width=$Width
if($ExpandDetails){
    $window.FindName('Details').Parent.Parent.IsExpanded=$true
}
Add-Type -AssemblyName System.Drawing
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class HiveNoticeCapture {
 [StructLayout(LayoutKind.Sequential)] public struct RECT {public int Left,Top,Right,Bottom;}
 [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr window,out RECT rect);
 [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr window,IntPtr dc,uint flags);
 [DllImport("dwmapi.dll")] public static extern int DwmGetWindowAttribute(IntPtr window,int attribute,out RECT rect,int size);
 [DllImport("gdi32.dll")] public static extern bool SetViewportOrgEx(IntPtr dc,int x,int y,IntPtr previous);
}
'@
$capture=[Windows.Threading.DispatcherTimer]::new();$capture.Interval=[TimeSpan]::FromMilliseconds(600)
$capture.Tag=@{window=$window;fixture=$fixture;close=[bool]$CloseAfterCapture;case=$Case;width=$Width;state=$statePath;before=$before;close_with=$CloseWith}
$capture.Add_Tick({
    param($sender,$eventData)
    $sender.Stop();$data=$sender.Tag;$view=$data.window
    $handle=[Windows.Interop.WindowInteropHelper]::new($view).Handle
    $rect=[HiveNoticeCapture+RECT]::new()
    if(-not [HiveNoticeCapture]::GetWindowRect($handle,[ref]$rect)){throw 'Cannot measure preview window'}
    $visible=[HiveNoticeCapture+RECT]::new()
    if([HiveNoticeCapture]::DwmGetWindowAttribute($handle,9,[ref]$visible,16) -ne 0){$visible=$rect}
    $bitmap=[Drawing.Bitmap]::new($visible.Right-$visible.Left,$visible.Bottom-$visible.Top)
    $graphics=[Drawing.Graphics]::FromImage($bitmap);$dc=$graphics.GetHdc()
    $null=[HiveNoticeCapture]::SetViewportOrgEx($dc,$rect.Left-$visible.Left,$rect.Top-$visible.Top,[IntPtr]::Zero)
    try {$captured=[HiveNoticeCapture]::PrintWindow($handle,$dc,2)}finally{$graphics.ReleaseHdc($dc);$graphics.Dispose()}
    try {
        if(-not $captured){throw 'Cannot capture the preview window'}
        $bitmap.Save((Join-Path $data.fixture 'notice.png'),[Drawing.Imaging.ImageFormat]::Png)
    } finally {$bitmap.Dispose()}
    $dpi=[Windows.Media.VisualTreeHelper]::GetDpi($view)
    $receipt=@{schema_version=1;preview_only=$true;case=$data.case;logical_width=$data.width;actual_dpi_x=$dpi.PixelsPerInchX;actual_dpi_y=$dpi.PixelsPerInchY;apartment=[Threading.Thread]::CurrentThread.ApartmentState.ToString();image='notice.png';title=$view.Tag.controls.Title.Text;status=$view.Tag.controls.Status.Text;close_only=$data.close}
    if($data.close){
        if($data.close_with -eq 'escape') {
            $key=[Windows.Input.KeyEventArgs]::new([Windows.Input.Keyboard]::PrimaryDevice,[Windows.PresentationSource]::FromVisual($view),[Environment]::TickCount,[Windows.Input.Key]::Escape)
            $key.RoutedEvent=[Windows.Input.Keyboard]::PreviewKeyDownEvent;$view.RaiseEvent($key)
        } else {$view.Tag.controls.Close.RaiseEvent([Windows.RoutedEventArgs]::new([Windows.Controls.Button]::ClickEvent))}
        $receipt.close_with=$data.close_with
        $receipt.closed=(-not $view.IsVisible)
        $receipt.incident_unchanged=((Preview-Hash $data.state) -eq $data.before)
        if(-not $receipt.closed -or -not $receipt.incident_unchanged){throw 'Close changed the incident or left the window open'}
    }
    [IO.File]::WriteAllText((Join-Path $data.fixture 'preview.json'),($receipt|ConvertTo-Json -Depth 4),[Text.UTF8Encoding]::new($false))
    [Console]::Out.WriteLine(($receipt|ConvertTo-Json -Compress));[Console]::Out.Flush()
})
$window.Add_ContentRendered({$capture.Start()})
$null=$window.ShowDialog()
$capture.Stop()
