# Display-only Windows notice. Closing it never acknowledges an incident or resumes work.
$ErrorActionPreference='Stop'
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
[Console]::InputEncoding=[Text.UTF8Encoding]::new($false)
$request=[Console]::In.ReadToEnd() | ConvertFrom-Json
if($request.operation -ne 'notify'){throw 'Unsupported notice operation'}

function Read-NoticeState([string]$path) {
    for($attempt=0;$attempt -lt 3;$attempt++) {
        try {
            $stream=[IO.File]::Open($path,[IO.FileMode]::Open,[IO.FileAccess]::Read,([IO.FileShare]::ReadWrite -bor [IO.FileShare]::Delete))
            try {
                if($stream.Length -gt 65536){throw 'Notice state exceeds its limit'}
                $reader=[IO.StreamReader]::new($stream,[Text.Encoding]::UTF8,$true)
                try {return ($reader.ReadToEnd() | ConvertFrom-Json)} finally {$reader.Dispose()}
            } finally {$stream.Dispose()}
        } catch {if($attempt -eq 2){throw};Start-Sleep -Milliseconds 20}
    }
}

function Get-NoticeView($state,$context,[bool]$enabled,[bool]$readFailed=$false) {
    $current=$state.incident
    $matching=($null -ne $current -and $current.digest -eq $context.incident)
    if($matching){$context.reason=[string]$current.reason;$context.detected_at=[long]$current.detected_at}
    $reset=$context.reason -eq 'usage-reset'
    $sensor=$context.reason -eq 'sensor-unresponsive'
    $reason=if($reset){'사용량 초기화 감지'}elseif($context.reason -eq 'sensor-unresponsive'){'60초 동안 정상 사용량 조회 없음'}else{'사용량 보호 상태 확인 필요'}
    $view=@{
        Title=if($reset){"사용량 초기화를 감지해`n작업 중단을 요청했습니다"}elseif($sensor){"사용량을 확인할 수 없어`n작업 중단을 요청했습니다"}else{"사용량 보호 상태를 확인할 수 없어`n작업 중단을 요청했습니다"}
        Explanation=if($reset){"Aigent Hive가 사용량 초기화를 감지했습니다.`n새 할당량을 보호하기 위해 작업 중단을 요청했습니다."}elseif($sensor){"Aigent Hive가 60초 동안 사용량을 확인하지 못했습니다.`n한도 보호를 위해 작업 중단을 요청했습니다."}else{"Aigent Hive가 사용량 보호 상태를 확인하지 못했습니다.`n한도 보호를 위해 작업 중단을 요청했습니다."}
        Reason=$reason;Status='중단 상태 확인 중';StatusDetail='앱에서 작업 중단 결과를 확인하고 있습니다.';Tone='warning'
        App=if($state.app_alive){'Codex 앱은 열려 있습니다.'}else{'Codex 앱의 상태를 확인해 주세요.'}
        Automatic='이전 작업은 자동으로 다시 시작하지 않습니다.'
        Next="명령을 복사해 Codex 밖의 PowerShell에서 실행하세요.`n사용량 확인 후, 원하는 대화를 직접 재개해 주세요."
        CanCopy=($matching -and -not $readFailed -and $current.digest -match '^sha256:[a-f0-9]{64}$')
        When=if($context.detected_at -gt 0){[DateTimeOffset]::FromUnixTimeSeconds([long]$context.detected_at).ToLocalTime().ToString('yyyy.MM.dd  HH:mm:ss')}else{'확인 중'}
        Banner=if($context.preview){'화면 미리보기 · 실제 조회나 작업 중단 없음'}elseif($context.is_test){'격리 시험 · 실제 초기화가 아닌 시험 신호'}else{''}
        Details='';Footer='창을 닫아도 작업은 재개되지 않습니다.'
    }
    if(-not $readFailed -and $null -eq $current) {
        $view.Title='중단 기록을 확인했습니다';$view.Status='보호 잠금 해제';$view.Tone='success'
        $view.Explanation="이번 중단 기록의 확인이 완료되었습니다.`n자세한 중단 이유는 아래에 보관되어 있습니다."
        $view.StatusDetail="새 사용량 확인 후 보호 잠금을 해제했습니다.`n작업은 자동으로 다시 시작하지 않습니다."
        $view.Next="원하는 Codex 대화에서 작업을 직접 재개해 주세요.`n이 창을 닫아도 이전 작업은 자동으로 시작되지 않습니다."
    } elseif(-not $readFailed -and -not $matching) {
        $view.Status='새 보호 사건 발생';$view.StatusDetail='이 창은 이전 중단 기록입니다. 새 안내 창에서 최신 이유와 상태를 확인해 주세요.'
        $view.Next="최신 안내 창에서 새 중단 기록을 확인해 주세요.`n그 창에 표시된 재개 방법을 따라 주세요."
    } elseif($readFailed) {
        $view.Status='상태를 확인할 수 없습니다';$view.StatusDetail="중단 결과를 읽지 못했습니다.`n보호 잠금이 풀린 것으로 처리하지 않습니다."
        $view.Next="중단 기록을 다시 읽을 수 있을 때 확인해 주세요.`n현재 Codex 작업 상태를 먼저 점검해 주세요."
    } elseif($current.stop_confirmed -and $state.app_alive) {
        $view.Title=if($reset){"새 할당량을 보호하기 위해`n작업을 멈췄습니다"}else{"사용량 한도 보호를 위해`n작업을 멈췄습니다"}
        $view.Explanation=if($reset){"Aigent Hive가 사용량 초기화를 감지했습니다.`n새 할당량을 보호하기 위해 Codex 작업을 중단했습니다."}elseif($sensor){"Aigent Hive가 60초 동안 사용량을 확인하지 못했습니다.`n한도 보호를 위해 Codex 작업을 중단했습니다."}else{"Aigent Hive가 사용량 보호 상태를 확인하지 못했습니다.`n한도 보호를 위해 Codex 작업을 중단했습니다."}
        $view.Status='작업 중단 확인';$view.StatusDetail='중단 대상의 실행이 멈춘 것을 확인했습니다.';$view.Tone='success'
    } elseif($current.stop_error) {
        $view.Status='일부 중단 상태의 확인이 필요합니다'
        $view.StatusDetail=if($state.unknown_sessions -gt 0){"일부 대화의 최종 상태를 확인하지 못했습니다.`nCodex에서 해당 작업의 상태를 확인해 주세요."}else{"정해진 시간 안에 전체 중단을 확인하지 못했습니다.`nCodex에서 작업의 상태를 확인해 주세요."}
    } elseif($state.control_connected -and $state.active_turns -eq 0 -and $state.unknown_sessions -eq 0 -and $current.targeted_turns -eq 0) {
        $view.Status='현재 실행 중인 작업이 없습니다';$view.StatusDetail='작업 중단 요청을 유지하고 있습니다. 이미 멈춰 있던 작업을 새로 중단한 것으로 계산하지 않습니다.'
    }
    $connection=if($state.control_connected){'연결됨'}else{'확인 필요'}
    $monitor=if($enabled){'실행 중'}else{'꺼짐 · 이전 중단 기록'}
    $view.Details='중단 대상  '+[long]$current.targeted_turns+'개    확인  '+[long]$current.confirmed_turns+"개`n"+
        '현재 실행  '+[long]$state.active_turns+'개    추가 확인  '+[long]$state.unknown_sessions+"개`n"+
        '앱 제어  '+$connection+'    감시  '+$monitor
    return $view
}

function Get-ResumeCommand([string]$root,[string]$incident) {
    if([string]::IsNullOrWhiteSpace($root) -or -not [IO.Path]::IsPathRooted($root) -or
       $root.IndexOfAny([char[]](0..31)) -ge 0 -or $incident -notmatch '^sha256:[a-f0-9]{64}$'){return $null}
    return "hive usage monitor acknowledge --user-root '"+$root.Replace("'","''")+"' --incident '"+$incident+"' --output json"
}

function Update-Notice($ui) {
    $state=$null;$enabled=$false;$readFailed=$false
    try {
        $state=Read-NoticeState (Join-Path $ui.root '.hive/runtime/usage-monitor/state.json')
        $config=Read-NoticeState (Join-Path $ui.root '.hive/runtime/usage-monitor/config.json')
        $enabled=[bool]$config.enabled
    } catch {$readFailed=$true}
    try {$view=Get-NoticeView $state $ui.context $enabled $readFailed}
    catch {$ui.context.detected_at=0L;$view=Get-NoticeView $null $ui.context $false $true}
    foreach($name in @('Title','Explanation','Reason','Status','StatusDetail','App','Automatic','Next','When','Banner','Details','Footer')){$ui.controls[$name].Text=$view[$name]}
    $ui.controls.BannerBox.Visibility=if($view.Banner){'Visible'}else{'Collapsed'}
    $ui.controls.CopyArea.Visibility=if($view.CanCopy){'Visible'}else{'Collapsed'}
    if(-not $view.CanCopy){$ui.copy_feedback=''}
    $ui.controls.CopyFeedback.Text=$ui.copy_feedback
    $ui.controls.CopyFeedback.Visibility=if($ui.copy_feedback){'Visible'}else{'Collapsed'}
    $success=$view.Tone -eq 'success'
    $ui.controls.StatusBox.Background=if($success){'#EDF5F0'}else{'#FFF5E5'}
    $ui.controls.Status.Foreground=if($success){'#22563E'}else{'#744B13'}
    $ui.controls.StatusDetail.Foreground=if($success){'#335D48'}else{'#704D21'}
    if($ui.context.is_test) {
        try {
            $progress=Read-NoticeState (Join-Path $ui.root 'result.json')
            if($progress.installation_phase -eq 'passed-installed-always-on'){$ui.controls.Details.Text+="`n실제 사용자 설치·상시 감시 완료"}
            elseif($progress.installation_phase -eq 'installing'){$ui.controls.Details.Text+="`n설치·상시 감시 확인 진행 중"}
            elseif($progress.phase -eq 'failed'){$ui.controls.Details.Text+="`n전체 수용 미완료 · 설치 미실행"}
        } catch {}
    }
}

function New-NoticeWindow($context,[string]$root) {
    Add-Type -AssemblyName PresentationFramework
    Add-Type -AssemblyName PresentationCore
    Add-Type -AssemblyName WindowsBase
    [xml]$xaml=@'
<Window xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation" xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml"
 Title="Aigent Hive — 사용량 보호 상태" Width="560" MinWidth="460" MinHeight="420" SizeToContent="Height"
 WindowStartupLocation="CenterScreen" ResizeMode="CanResize" Background="#FBFCFA" Foreground="#202B27"
 FontFamily="Malgun Gothic" FontSize="13" UseLayoutRounding="True" SnapsToDevicePixels="True">
 <Window.Icon><DrawingImage><DrawingImage.Drawing><DrawingGroup>
  <GeometryDrawing Brush="#205D4B" Geometry="M16,1 L29,6 29,17 C29,25 22,29 16,32 C10,29 3,25 3,17 L3,6 Z"/>
  <GeometryDrawing Brush="#FFFFFF" Geometry="M11,10 L14,10 14,22 11,22 Z M18,10 L21,10 21,22 18,22 Z"/>
 </DrawingGroup></DrawingImage.Drawing></DrawingImage></Window.Icon>
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
  <Style x:Key="CopyAction" TargetType="Button" BasedOn="{StaticResource CloseAction}">
   <Setter Property="Background" Value="#F1F4F0"/><Setter Property="Foreground" Value="#205D4B"/>
   <Setter Property="BorderBrush" Value="#C8D4CB"/><Setter Property="Height" Value="36"/>
   <Style.Triggers>
    <Trigger Property="IsMouseOver" Value="True"><Setter Property="Foreground" Value="White"/></Trigger>
    <Trigger Property="IsKeyboardFocused" Value="True"><Setter Property="BorderBrush" Value="#205D4B"/></Trigger>
   </Style.Triggers>
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
     <Border Width="44" Height="44" CornerRadius="12" Background="#E6EFE9" VerticalAlignment="Top">
      <Path Data="M7,4 L21,4 21,15 C21,21 15,24 14,24 C13,24 7,21 7,15 Z M11,9 L11,16 M17,9 L17,16" Stroke="#205D4B" StrokeThickness="1.8" StrokeLineJoin="Round" Stretch="Uniform" Width="24" Height="26"/>
     </Border>
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
    <StackPanel x:Name="CopyArea" Margin="0,12,0,0">
     <Button x:Name="Copy" Content="재개 명령 복사" Style="{StaticResource CopyAction}" HorizontalAlignment="Left" AutomationProperties.Name="재개 명령 복사, 명령 실행 없음"/>
     <TextBlock x:Name="CopyFeedback" Foreground="#4E5C55" FontSize="12" Margin="0,8,0,0" TextWrapping="Wrap" Visibility="Collapsed"/>
    </StackPanel>
    <Expander Header="상태 자세히 보기" Margin="-4,18,0,0" Foreground="#4E5C55" AutomationProperties.Name="사용량 보호 상태 자세히 보기">
     <StackPanel Margin="24,10,0,0">
      <Grid>
     <Grid.ColumnDefinitions><ColumnDefinition Width="72"/><ColumnDefinition Width="*"/></Grid.ColumnDefinitions>
     <Grid.RowDefinitions><RowDefinition Height="Auto"/><RowDefinition Height="Auto"/></Grid.RowDefinitions>
     <TextBlock Text="중단 이유" Foreground="#5D6962" Margin="0,0,0,9"/>
     <TextBlock x:Name="Reason" Grid.Column="1" FontWeight="SemiBold" TextWrapping="Wrap" Margin="0,0,0,9"/>
     <TextBlock Text="감지 시각" Grid.Row="1" Foreground="#5D6962"/>
     <TextBlock x:Name="When" Grid.Row="1" Grid.Column="1" TextWrapping="Wrap"/>
      </Grid>
      <TextBlock x:Name="Details" Margin="0,12,0,0" LineHeight="23" TextWrapping="Wrap" FontSize="12"/>
     </StackPanel>
    </Expander>
   </StackPanel>
  </ScrollViewer>
  <Border Grid.Row="1" Background="#F1F4F0" BorderBrush="#DEE5DF" BorderThickness="0,1,0,0" Padding="24,16">
   <Grid>
    <Grid.ColumnDefinitions><ColumnDefinition Width="*"/><ColumnDefinition Width="Auto"/></Grid.ColumnDefinitions>
    <TextBlock x:Name="Footer" VerticalAlignment="Center" Foreground="#56635B" FontSize="11.5" TextWrapping="Wrap" Margin="0,0,18,0"/>
    <Button x:Name="Close" Grid.Column="1" Content="안내 닫기" Style="{StaticResource CloseAction}" IsDefault="True" AutomationProperties.Name="안내 닫기, 작업 재개 없음" ToolTip="이 안내 창만 닫습니다. 보호 잠금과 작업 상태는 바뀌지 않습니다."/>
   </Grid>
  </Border>
 </Grid>
</Window>
'@
    $window=[Windows.Markup.XamlReader]::Load([Xml.XmlNodeReader]::new($xaml))
    $window.MaxHeight=[Math]::Max(420,[Windows.SystemParameters]::WorkArea.Height-48)
    $controls=@{}
    foreach($name in @('Title','Explanation','Reason','Status','StatusDetail','App','Automatic','Next','When','Banner','BannerBox','StatusBox','Details','Footer','Close','Copy','CopyArea','CopyFeedback')){$controls[$name]=$window.FindName($name)}
    $timer=[Windows.Threading.DispatcherTimer]::new();$timer.Interval=[TimeSpan]::FromSeconds(1)
    $ui=@{window=$window;controls=$controls;context=$context;root=$root;timer=$timer;copy_feedback=''}
    $window.Tag=$ui;$timer.Tag=$ui;$controls.Close.Tag=$window
    $controls.Copy.Tag=$ui
    $controls.Copy.Add_Click({
        param($sender,$eventData)
        $ui=$sender.Tag
        try {
            $state=Read-NoticeState (Join-Path $ui.root '.hive/runtime/usage-monitor/state.json')
            if($state.incident.digest -ne $ui.context.incident){throw 'Incident changed'}
            $command=Get-ResumeCommand $ui.root $ui.context.incident
            if(-not $command){throw 'Invalid resume command'}
            [Windows.Clipboard]::SetText($command)
            $ui.copy_feedback='복사했습니다. Codex 밖의 PowerShell에 붙여넣어 실행하세요.'
        } catch {$ui.copy_feedback='명령을 복사하지 못했습니다. 현재 중단 기록을 확인한 뒤 다시 눌러 주세요.'}
        Update-Notice $ui
    })
    $timer.Add_Tick({param($sender,$eventData) Update-Notice $sender.Tag})
    $controls.Close.Add_Click({param($sender,$eventData) $sender.Tag.Close()})
    $window.Add_PreviewKeyDown({param($sender,$eventData) if($eventData.Key -eq 'Escape'){$sender.Close();$eventData.Handled=$true}})
    $window.Add_ContentRendered({param($sender,$eventData) Update-Notice $sender.Tag;[Console]::Out.WriteLine('{"notified":true}');[Console]::Out.Flush();$sender.Tag.timer.Start()})
    $window.Add_Closed({param($sender,$eventData) $sender.Tag.timer.Stop()})
    Update-Notice $ui
    return $window
}

try {
    $context=@{incident=[string]$request.incident;reason=[string]$request.reason;detected_at=0L;is_test=$false;preview=($request.preview -eq $true)}
    try {$context.is_test=(Read-NoticeState (Join-Path ([string]$request.user_root) '.hive/install/codex.json')).acceptance_fixture -eq $true}catch{}
    $window=New-NoticeWindow $context ([string]$request.user_root)
    $null=$window.ShowDialog()
} catch {
    Add-Type -AssemblyName System.Windows.Forms
    [Console]::Out.WriteLine('{"notified":true,"fallback":true}');[Console]::Out.Flush()
    $text=[string]$request.message+"`r`n`r`nAigent Hive의 사용량 보호 안내입니다. 닫아도 작업은 자동으로 재개되지 않습니다."
    $null=[Windows.Forms.MessageBox]::Show($text,'Aigent Hive — 작업 중단 안내','OK','Information')
}
