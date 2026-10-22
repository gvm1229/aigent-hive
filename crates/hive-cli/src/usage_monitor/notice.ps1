# Notification-only Windows notice. It has no control over session execution.
$ErrorActionPreference='Stop'
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
[Console]::InputEncoding=[Text.UTF8Encoding]::new($false)
$request=[Console]::In.ReadToEnd() | ConvertFrom-Json
if($request.operation -ne 'notify'){throw 'Unsupported notice operation'}

function Read-NoticeState([string]$path) {
    if(-not ('HiveNoticeFiles' -as [type])) {
        Add-Type -TypeDefinition @'
using System;
using System.IO;
using System.Text;
using System.Runtime.InteropServices;
using Microsoft.Win32.SafeHandles;
public static class HiveNoticeFiles {
    [DllImport("kernel32.dll", CharSet=CharSet.Unicode, SetLastError=true)]
    static extern SafeFileHandle CreateFileW(string path, uint access, uint share, IntPtr security, uint mode, uint flags, IntPtr template);
    public static string ReadText(string path) {
        // Read-only, with delete sharing so the controller can publish atomically.
        // SafeFileHandle avoids legacy .NET path normalization of Windows verbatim paths.
        using(var handle=CreateFileW(path.Replace('/','\\'),0x80000000,7,IntPtr.Zero,3,0x80,IntPtr.Zero)) {
            if(handle.IsInvalid) throw new IOException("Notice state unavailable");
            using(var stream=new FileStream(handle,FileAccess.Read)) {
                if(stream.Length>65536) throw new IOException("Notice state exceeds its limit");
                using(var reader=new StreamReader(stream,Encoding.UTF8,true)) return reader.ReadToEnd();
            }
        }
    }
}
'@
    }
    for($attempt=0;$attempt -lt 3;$attempt++) {
        try {
            return ([HiveNoticeFiles]::ReadText($path) | ConvertFrom-Json)
        } catch {if($attempt -eq 2){throw};Start-Sleep -Milliseconds 20}
    }
}

function Get-NoticeView($state,$context,[bool]$enabled,[bool]$readFailed=$false) {
    $current=$state.incident
    $matching=($null -ne $current -and $current.digest -eq $context.incident)
    if($matching){$context.reason=[string]$current.reason;$context.detected_at=[long]$current.detected_at;$context.threshold_details=@($current.threshold_details);$context.query_error=$current.query_error}
    $reset=$context.reason -eq 'usage-reset'
    $sensor=$context.reason -eq 'sensor-unresponsive'
    $threshold=$context.reason -eq 'usage-threshold'
    $reason=if($reset){'사용량 초기화 감지'}elseif($context.reason -eq 'sensor-unresponsive'){'60초 동안 정상 사용량 조회 없음'}else{'사용량 보호 상태 확인 필요'}
    $view=@{
        Title=if($reset){"사용량 초기화를 감지해`n작업 중단을 요청했습니다"}elseif($sensor){"사용량을 확인할 수 없어`n작업 중단을 요청했습니다"}else{"사용량 보호 상태를 확인할 수 없어`n작업 중단을 요청했습니다"}
        Explanation=if($reset){"Aigent Hive가 사용량 초기화를 감지했습니다.`n새 할당량을 보호하기 위해 작업 중단을 요청했습니다."}elseif($sensor){"Aigent Hive가 60초 동안 사용량을 확인하지 못했습니다.`n한도 보호를 위해 작업 중단을 요청했습니다."}else{"Aigent Hive가 사용량 보호 상태를 확인하지 못했습니다.`n한도 보호를 위해 작업 중단을 요청했습니다."}
        Reason=$reason;Status='중단 상태 확인 중';StatusDetail='앱에서 작업 중단 결과를 확인하고 있습니다.';Tone='warning'
        App=if($state.app_alive){'Codex 앱은 열려 있습니다.'}else{'Codex 앱의 상태를 확인해 주세요.'}
        Automatic="이전 작업은 자동으로 다시 시작하지 않습니다.`n이미 시작한 일반 외부 명령은 계속 실행될 수 있습니다."
        Next="Codex에서 원하는 대화를 재개하거나 새 메시지를 보내세요.`n별도의 확인이나 해제 절차는 없습니다."
        When=if($context.detected_at -gt 0){[DateTimeOffset]::FromUnixTimeSeconds([long]$context.detected_at).ToLocalTime().ToString('yyyy.MM.dd  HH:mm:ss')}else{'확인 중'}
        Banner=if($context.preview){'화면 미리보기 · 실제 조회나 작업 중단 없음'}elseif($context.is_test){'격리 시험 · 실제 초기화가 아닌 시험 신호'}else{''}
        Details='';Footer='창을 닫아도 작업은 재개되지 않습니다.'
    }
    $thresholdText='설정한 사용량 하한에 도달했습니다.'
    if($threshold) {
        $conditions=@($context.threshold_details | Where-Object {$null -ne $_} | ForEach-Object {
            $window=if($_.window -eq 'weekly'){'주간'}elseif($_.window -eq 'session'){'5시간'}else{'구독 한도'}
            '남은 사용량 {0:0.##}% / 하한 {1}% · {2}' -f [double]$_.remaining_percent,[int]$_.threshold_percent,$window
        } | Sort-Object -Unique)
        if($conditions.Count){$thresholdText=($conditions | Select-Object -First 3) -join "`n"}
        if($conditions.Count -gt 3){$thresholdText+="`n그 밖의 적용 기준: "+($conditions.Count-3)+'개'}
        $view.Title="설정한 사용량 하한에 도달해`n작업 중단을 요청했습니다"
        $view.Explanation=$thresholdText+"`nAigent Hive가 해당 Codex 작업에 중단을 요청했습니다."
        $view.Reason='설정한 사용량 하한 도달'
    }
    if(-not $readFailed -and $null -eq $current -and $context.reason -eq 'monitor-fault') {
        $view.Status='보호 기록을 읽지 못해 중단을 요청했습니다';$view.Tone='warning'
        $view.StatusDetail='사용량 보호 기록을 확인하지 못했습니다. 실제 중단 확인 결과는 기록하지 못했습니다.'
    } elseif(-not $readFailed -and $null -eq $current) {
        $view.Status='이전 중단 안내';$view.Tone='warning'
        $view.StatusDetail='현재 중단 기록을 찾지 못했습니다. 아래의 원래 중단 이유를 참고해 주세요.'
    } elseif(-not $readFailed -and -not $matching) {
        $view.Status='새 보호 사건 발생';$view.StatusDetail='이 창은 이전 중단 기록입니다. 새 안내 창에서 최신 이유와 상태를 확인해 주세요.'
        $view.Next="최신 안내 창에서 새 중단 기록을 확인해 주세요.`nCodex에서는 원하는 대화를 직접 재개할 수 있습니다."
    } elseif($readFailed) {
        $view.Status='상태를 확인할 수 없습니다';$view.StatusDetail="중단 결과를 읽지 못했습니다.`n이 창에서 작업을 차단하거나 재개하지 않습니다."
        $view.Next="중단 기록을 다시 읽을 수 있을 때 확인해 주세요.`n현재 Codex 작업 상태를 먼저 점검해 주세요."
    } elseif($current.stop_confirmed -and $state.app_alive) {
        $view.Title=if($reset){"새 할당량을 보호하기 위해`n작업을 멈췄습니다"}else{"사용량 한도 보호를 위해`n작업을 멈췄습니다"}
        $view.Explanation=if($reset){"Aigent Hive가 사용량 초기화를 감지했습니다.`n새 할당량을 보호하기 위해 Codex 작업을 중단했습니다."}elseif($sensor){"Aigent Hive가 60초 동안 사용량을 확인하지 못했습니다.`n한도 보호를 위해 Codex 작업을 중단했습니다."}else{"Aigent Hive가 사용량 보호 상태를 확인하지 못했습니다.`n한도 보호를 위해 Codex 작업을 중단했습니다."}
        $view.Status='작업 중단 확인';$view.StatusDetail='중단 대상의 실행이 멈춘 것을 확인했습니다.';$view.Tone='success'
        if($threshold){$view.Title="설정한 사용량 하한에 도달해`n작업을 멈췄습니다";$view.Explanation=$thresholdText+"`nAigent Hive가 해당 Codex 작업의 중단을 확인했습니다."}
    } elseif($current.stop_error) {
        $view.Status='일부 중단 상태의 확인이 필요합니다'
        $view.StatusDetail=if($state.unknown_sessions -gt 0){"일부 대화의 최종 상태를 확인하지 못했습니다.`nCodex에서 해당 작업의 상태를 확인해 주세요."}else{"정해진 시간 안에 전체 중단을 확인하지 못했습니다.`nCodex에서 작업의 상태를 확인해 주세요."}
    } elseif($state.control_connected -and $state.active_turns -eq 0 -and $state.unknown_sessions -eq 0 -and $current.targeted_turns -eq 0) {
        $view.Status='감지 당시 실행 중인 작업이 없었습니다';$view.StatusDetail='이미 멈춰 있던 작업을 새로 중단한 것으로 계산하지 않습니다.'
    }
    $connection=if($state.control_connected){'연결됨'}else{'확인 필요'}
    $monitor=if($enabled){'실행 중'}else{'꺼짐 · 이전 중단 기록'}
    $view.Details='중단 대상  '+[long]$current.targeted_turns+'개    확인  '+[long]$current.confirmed_turns+"개`n"+
        '현재 실행  '+[long]$state.active_turns+'개    추가 확인  '+[long]$state.unknown_sessions+"개`n"+
        '앱 제어  '+$connection+'    감시  '+$monitor
    if($sensor -and $context.query_error) {
        $errorClass=[string]$context.query_error
        $cause=if($errorClass -match 'timeout|timed out'){'조회가 제한 시간 안에 끝나지 않았습니다.'}elseif($errorClass -match 'exited|process failed'){'조회 프로그램이 종료되었습니다.'}elseif($errorClass -match 'account|identity'){'사용 중인 계정을 일치시켜 확인하지 못했습니다.'}elseif($errorClass -match 'regression|schedule|window'){'사용량 자료의 시각이나 구간을 다시 확인하지 못했습니다.'}else{'받은 사용량 자료를 검증하지 못했습니다.'}
        $view.Explanation+="`n"+$cause
    }
    return $view
}

function Update-Notice($ui) {
    $state=$null;$enabled=$false;$readFailed=$false
    try {
        $state=Read-NoticeState ([IO.Path]::Combine($ui.root,'.hive\runtime\usage-monitor\state.json'))
        $config=Read-NoticeState ([IO.Path]::Combine($ui.root,'.hive\runtime\usage-monitor\config.json'))
        $enabled=[bool]$config.enabled
    } catch {$readFailed=$true}
    try {$view=Get-NoticeView $state $ui.context $enabled $readFailed}
    catch {$ui.context.detected_at=0L;$view=Get-NoticeView $null $ui.context $false $true}
    foreach($name in @('Title','Explanation','Status','StatusDetail','App','Automatic','Next','When','Banner','Footer')){$ui.controls[$name].Text=$view[$name]}
    $ui.controls.BannerBox.Visibility=if($view.Banner){'Visible'}else{'Collapsed'}
    $success=$view.Tone -eq 'success'
    $ui.controls.StatusBox.Background=if($success){'#EDF5F0'}else{'#FFF5E5'}
    $ui.controls.Status.Foreground=if($success){'#22563E'}else{'#744B13'}
    $ui.controls.StatusDetail.Foreground=if($success){'#335D48'}else{'#704D21'}

}

function New-NoticeWindow($context,[string]$root,[string]$logo) {
    Add-Type -AssemblyName PresentationFramework
    Add-Type -AssemblyName PresentationCore
    Add-Type -AssemblyName WindowsBase
    [xml]$xaml=@'
<Window xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation" xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml"
 Title="Aigent Hive — 사용량 보호 상태" Width="560" MinWidth="460" MinHeight="420" SizeToContent="Height"
 WindowStartupLocation="CenterScreen" ResizeMode="NoResize" Background="#FBFCFA" Foreground="#202B27"
 FontFamily="Malgun Gothic" FontSize="13" UseLayoutRounding="True" SnapsToDevicePixels="True">
 <Window.Resources>
  <Style x:Key="CloseAction" TargetType="Button">
   <Setter Property="Background" Value="#205D4B"/><Setter Property="Foreground" Value="White"/>
   <Setter Property="FontWeight" Value="SemiBold"/><Setter Property="FontSize" Value="13"/>
   <Setter Property="MinWidth" Value="112"/><Setter Property="Height" Value="40"/><Setter Property="Padding" Value="18,0"/>
   <Setter Property="BorderBrush" Value="#205D4B"/><Setter Property="BorderThickness" Value="2"/>
   <Setter Property="Template"><Setter.Value><ControlTemplate TargetType="Button">
    <Border x:Name="Frame" Background="{TemplateBinding Background}" BorderBrush="{TemplateBinding BorderBrush}" BorderThickness="{TemplateBinding BorderThickness}" CornerRadius="6" Padding="{TemplateBinding Padding}">
     <ContentPresenter HorizontalAlignment="Center" VerticalAlignment="Center"/>
    </Border>
    <ControlTemplate.Triggers>
     <Trigger Property="IsMouseOver" Value="True"><Setter TargetName="Frame" Property="Background" Value="#174B3B"/></Trigger>
     <Trigger Property="IsPressed" Value="True"><Setter TargetName="Frame" Property="Background" Value="#113E30"/></Trigger>
     <Trigger Property="IsKeyboardFocused" Value="True"><Setter TargetName="Frame" Property="BorderBrush" Value="#A3D6BF"/></Trigger>
     <Trigger Property="IsEnabled" Value="False"><Setter TargetName="Frame" Property="Opacity" Value="0.5"/></Trigger>
    </ControlTemplate.Triggers>
   </ControlTemplate></Setter.Value></Setter>
  </Style>
 </Window.Resources>
 <Grid>
  <Grid.RowDefinitions><RowDefinition Height="*"/><RowDefinition Height="Auto"/></Grid.RowDefinitions>
  <ScrollViewer VerticalScrollBarVisibility="Auto" HorizontalScrollBarVisibility="Disabled">
   <StackPanel Margin="28,26,28,24">
    <Border x:Name="BannerBox" Background="#FFF3D8" CornerRadius="6" Padding="12,8" Margin="0,0,0,22" Visibility="Collapsed">
     <TextBlock x:Name="Banner" Foreground="#715018" FontSize="12" TextWrapping="Wrap"/>
    </Border>
    <Grid>
     <Grid.ColumnDefinitions><ColumnDefinition Width="44"/><ColumnDefinition Width="16"/><ColumnDefinition Width="*"/></Grid.ColumnDefinitions>
     <Image x:Name="Logo" Width="44" Height="44" VerticalAlignment="Top" Stretch="Uniform" AutomationProperties.Name="Aigent Hive 공식 로고"/>
     <TextBlock x:Name="Title" Grid.Column="2" FontSize="21" FontWeight="Bold" LineHeight="31" TextWrapping="Wrap"/>
    </Grid>
    <TextBlock x:Name="Explanation" Margin="0,18,0,22" Foreground="#4E5C55" FontSize="14" LineHeight="24" TextWrapping="Wrap"/>
    <Border x:Name="StatusBox" Background="#FFF5E5" CornerRadius="12" Padding="16,14">
     <StackPanel>
      <TextBlock x:Name="Status" FontSize="14" FontWeight="Bold" TextWrapping="Wrap"/>
      <TextBlock x:Name="StatusDetail" FontSize="12.5" Margin="0,6,0,0" LineHeight="21" TextWrapping="Wrap"/>
     </StackPanel>
    </Border>
    <StackPanel Margin="0,18,0,0">
     <TextBlock x:Name="App" LineHeight="23" TextWrapping="Wrap"/>
     <TextBlock x:Name="Automatic" LineHeight="23" TextWrapping="Wrap"/>
    </StackPanel>
    <TextBlock Text="다시 작업하려면" FontWeight="Bold" Margin="0,20,0,6"/>
    <TextBlock x:Name="Next" Foreground="#4E5C55" LineHeight="22" TextWrapping="Wrap"/>
    <StackPanel Orientation="Horizontal" Margin="0,18,0,0">
     <TextBlock Text="감지 시각  " Foreground="#5D6962" FontSize="12"/>
     <TextBlock x:Name="When" Foreground="#5D6962" FontSize="12"/>
    </StackPanel>
   </StackPanel>
  </ScrollViewer>
  <Border Grid.Row="1" Background="#F1F4F0" BorderBrush="#DEE5DF" BorderThickness="0,1,0,0" Padding="24,16">
   <Grid>
    <Grid.ColumnDefinitions><ColumnDefinition Width="*"/><ColumnDefinition Width="Auto"/></Grid.ColumnDefinitions>
    <TextBlock x:Name="Footer" VerticalAlignment="Center" Foreground="#56635B" FontSize="11.5" TextWrapping="Wrap" Margin="0,0,18,0"/>
    <Button x:Name="Close" Grid.Column="1" Content="안내 닫기" Style="{StaticResource CloseAction}" IsDefault="True" AutomationProperties.Name="안내 닫기, 작업 재개 없음" ToolTip="이 안내 창만 닫습니다. 작업 상태는 바뀌지 않습니다."/>
   </Grid>
  </Border>
 </Grid>
</Window>
'@
    $window=[Windows.Markup.XamlReader]::Load([Xml.XmlNodeReader]::new($xaml))
    $stream=[IO.MemoryStream]::new([Convert]::FromBase64String($logo))
    try {
        $image=[Windows.Media.Imaging.BitmapImage]::new();$image.BeginInit()
        $image.CacheOption=[Windows.Media.Imaging.BitmapCacheOption]::OnLoad
        $image.StreamSource=$stream;$image.EndInit();$image.Freeze()
    } finally {$stream.Dispose()}
    $window.Icon=$image;$window.FindName('Logo').Source=$image
    $window.MaxHeight=[Math]::Max(420,[Windows.SystemParameters]::WorkArea.Height-48)
    $controls=@{}
    foreach($name in @('Title','Explanation','Status','StatusDetail','App','Automatic','Next','When','Banner','BannerBox','StatusBox','Footer','Close')){$controls[$name]=$window.FindName($name)}
    $timer=[Windows.Threading.DispatcherTimer]::new();$timer.Interval=[TimeSpan]::FromSeconds(1)
    $ui=@{window=$window;controls=$controls;context=$context;root=$root;timer=$timer}
    $window.Tag=$ui;$timer.Tag=$ui;$controls.Close.Tag=$window
    $timer.Add_Tick({param($sender,$eventData) Update-Notice $sender.Tag})
    $controls.Close.Add_Click({param($sender,$eventData) $sender.Tag.Close()})
    $window.Add_PreviewKeyDown({param($sender,$eventData) if($eventData.Key -eq 'Escape'){$sender.Close();$eventData.Handled=$true}})
    $window.Add_ContentRendered({param($sender,$eventData) Update-Notice $sender.Tag;[Console]::Out.WriteLine((@{notified=$true;close_only=($sender.ResizeMode.ToString() -eq 'NoResize')}|ConvertTo-Json -Compress));[Console]::Out.Flush();$sender.Tag.timer.Start()})
    $window.Add_Closed({param($sender,$eventData) $sender.Tag.timer.Stop()})
    Update-Notice $ui
    return $window
}

try {
    $context=@{incident=[string]$request.incident;reason=[string]$request.reason;detected_at=[long]$request.detected_at;is_test=$false;preview=($request.preview -eq $true)}
    try {$context.is_test=(Read-NoticeState ([IO.Path]::Combine([string]$request.user_root,'.hive\install\codex.json'))).acceptance_fixture -eq $true}catch{}
    $window=New-NoticeWindow $context ([string]$request.user_root) ([string]$request.logo_png_base64)
    $null=$window.ShowDialog()
} catch {
    Add-Type -AssemblyName System.Windows.Forms
    [Console]::Out.WriteLine('{"notified":true,"fallback":true}');[Console]::Out.Flush()
    $text=[string]$request.message+"`r`n`r`nAigent Hive의 사용량 보호 안내입니다. 닫아도 작업은 자동으로 재개되지 않습니다."
    $null=[Windows.Forms.MessageBox]::Show($text,'Aigent Hive — 작업 중단 안내','OK','Information')
}
