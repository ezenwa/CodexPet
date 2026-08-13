param(
    [switch]$Startup,
    [switch]$CloseWithCodex,
    [int]$CodexProcessId = 0
)
Add-Type -AssemblyName PresentationFramework, PresentationCore, WindowsBase
$script:appVersion = [Version]'1.2.1'
$script:releaseApiUrl = 'https://api.github.com/repos/ezenwa/CodexPet/releases/latest'
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class CodexPetWindowNative {
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hwnd, out RECT rect);
    [DllImport("user32.dll")] public static extern IntPtr SendMessage(IntPtr hwnd, int msg, IntPtr wParam, IntPtr lParam);
    [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr hwnd);
    [DllImport("user32.dll")] public static extern int GetSystemMetricsForDpi(int index, uint dpi);
}
'@
$root = Split-Path -Parent $MyInvocation.MyCommand.Path
$canonicalInstallDir = Join-Path $env:LOCALAPPDATA 'CodexPet'
$canonicalPetScript = Join-Path $canonicalInstallDir 'CodexPet.ps1'
$currentRootPath = [IO.Path]::GetFullPath($root).TrimEnd('\')
$canonicalRootPath = [IO.Path]::GetFullPath($canonicalInstallDir).TrimEnd('\')
# Once CodexPet is installed, reject accidental launches from the development
# folder. Such a copy would otherwise acquire the global mutex and prevent the
# installed watcher from controlling or closing the visible pet.
if ((Split-Path -Leaf $currentRootPath) -ne 'CodexPetSelector' -and
    -not $currentRootPath.Equals($canonicalRootPath,[StringComparison]::OrdinalIgnoreCase) -and
    (Test-Path -LiteralPath $canonicalPetScript)) {
    exit 0
}
$powerShellExe = Join-Path $env:SystemRoot 'System32\WindowsPowerShell\v1.0\powershell.exe'
$pidFile = Join-Path $root 'codexpet.pid'
$closeRequestFile = Join-Path $root 'close-with-codex.request'
$stateDir = Join-Path ([Environment]::GetFolderPath('ApplicationData')) 'CodexPet'
New-Item -ItemType Directory -Path $stateDir -Force | Out-Null
$positionFile = Join-Path $stateDir 'window-position.json'
$selectedPetFile = Join-Path $stateDir 'selected-pet.txt'
$languageFile = Join-Path $stateDir 'language.txt'
. (Join-Path $root 'CodexPet-State.ps1')
$script:language = $(if ([Globalization.CultureInfo]::CurrentUICulture.TwoLetterISOLanguageName -eq 'es') { 'es' } else { 'en' })
if (Test-Path -LiteralPath $languageFile) {
    $savedLanguage = (Get-Content -LiteralPath $languageFile -Raw -ErrorAction SilentlyContinue).Trim().ToLowerInvariant()
    if ($savedLanguage -in @('en', 'es')) { $script:language = $savedLanguage }
}
$script:translations = @{
    es = @{
        StatusWorking='CODEX TRABAJANDO'; StatusInput='NECESITA ATENCIÓN'; StatusReady='TAREA TERMINADA'; StatusFailed='CODEX BLOQUEADO'; StatusIdle='CODEX EN ESPERA'; StatusOffline='CODEX DESCONECTADO'
        ChoosePet='Elegir mascota'; StartWindows='Iniciar con Windows'; CheckUpdates='Buscar actualizaciones...'; ClosePet='Cerrar mascota'; Language='Idioma'; English='Inglés'; Spanish='Español'
        InvalidVersion='GitHub devolvió una versión no válida: {0}'; NewVersionMessage="Hay una nueva versión de CodexPet: v{0}.`n`nVersión instalada: v{1}.`n`n¿Quieres abrir la página oficial de descarga?"; UpdateAvailable='Actualización disponible'
        UpToDateMessage='CodexPet está actualizado (v{0}).'; UpdateTitle='Buscar actualizaciones'; UpdateError="No fue posible buscar actualizaciones.`n`n{0}"
    }
    en = @{
        StatusWorking='CODEX WORKING'; StatusInput='NEEDS ATTENTION'; StatusReady='TASK COMPLETE'; StatusFailed='CODEX BLOCKED'; StatusIdle='CODEX IDLE'; StatusOffline='CODEX OFFLINE'
        ChoosePet='Choose pet'; StartWindows='Start with Windows'; CheckUpdates='Check for updates...'; ClosePet='Close pet'; Language='Language'; English='English'; Spanish='Spanish'
        InvalidVersion='GitHub returned an invalid version: {0}'; NewVersionMessage="A new CodexPet version is available: v{0}.`n`nInstalled version: v{1}.`n`nDo you want to open the official download page?"; UpdateAvailable='Update available'
        UpToDateMessage='CodexPet is up to date (v{0}).'; UpdateTitle='Check for updates'; UpdateError="Unable to check for updates.`n`n{0}"
    }
}
function Get-CodexPetText([string]$key) { return [string]$script:translations[$script:language][$key] }
$legacyPositionFile = Join-Path $root 'window-position.json'
if (-not (Test-Path -LiteralPath $positionFile) -and (Test-Path -LiteralPath $legacyPositionFile)) {
    Copy-Item -LiteralPath $legacyPositionFile -Destination $positionFile -Force -ErrorAction SilentlyContinue
}
$createdNew = $false
$singleInstance = New-Object System.Threading.Mutex($true, 'Local\CodexPet.Ezenwa', [ref]$createdNew)
if (-not $createdNew) {
    if ($CloseWithCodex) {
        $(if ($CodexProcessId -gt 0) { $CodexProcessId } else { 'close-with-codex' }) |
            Add-Content -LiteralPath $closeRequestFile -Encoding ascii
    }
    $singleInstance.Dispose()
    exit 0
}
$PID | Set-Content -LiteralPath $pidFile -Encoding ascii
$startupFolder = [Environment]::GetFolderPath('Startup')
$shortcutPath = Join-Path $startupFolder 'Codex Pet.lnk'
$scriptPath = $MyInvocation.MyCommand.Path
$watcherPath = Join-Path $root 'CodexPet-Watcher.ps1'
$watcherLauncher = Join-Path $root 'Start-CodexPetWatcher.vbs'
$wscriptExe = Join-Path $env:SystemRoot 'System32\wscript.exe'
$runKeyPath = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run'
$runValueName = 'CodexPetWatcher'
$watcherCommand = "`"$wscriptExe`" //B //Nologo `"$watcherLauncher`""
[xml]$xaml = @'
<Window xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation" xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml" WindowStyle="None" AllowsTransparency="True" Background="#01000000" Topmost="True" ShowActivated="False" ShowInTaskbar="False" Width="168" Height="140" MinWidth="120" MinHeight="100" MaxWidth="347" MaxHeight="289" ResizeMode="CanResize" Left="40" Top="40">
  <Viewbox Stretch="Uniform"><Grid Width="210" Height="175"><Border x:Name="Card" Margin="8" CornerRadius="24" Background="#E61A1B26" BorderBrush="#7AA2F7" BorderThickness="2">
    <Border.Effect><DropShadowEffect Color="#7AA2F7" BlurRadius="22" ShadowDepth="0" Opacity="0.75"/></Border.Effect>
    <Grid><Grid.RowDefinitions><RowDefinition Height="*"/><RowDefinition Height="42"/></Grid.RowDefinitions>
      <Grid x:Name="PetCanvas" Grid.Row="0">
        <Image x:Name="Pet" Width="116" Height="125" Stretch="Uniform" HorizontalAlignment="Center" VerticalAlignment="Center" Margin="0,-2,0,0" RenderTransformOrigin="0.5,0.5"/>
        <TextBlock x:Name="PetFallback" Text="🐈‍⬛" FontFamily="Segoe UI Emoji" FontSize="76" Width="126" TextAlignment="Center" HorizontalAlignment="Center" VerticalAlignment="Center" Margin="0,-3,0,0" Visibility="Collapsed"/>
        <Ellipse x:Name="Pulse" Width="13" Height="13" HorizontalAlignment="Right" VerticalAlignment="Top" Margin="0,18,17,0" Fill="#7AA2F7" Stroke="#F2F4FF" StrokeThickness="1"/>
      </Grid>
      <Border Grid.Row="1" Background="#CC24283B" CornerRadius="0,0,22,22"><StackPanel Orientation="Horizontal" HorizontalAlignment="Center" VerticalAlignment="Center">
        <TextBlock x:Name="Status" Text="CODEX EN ESPERA" Foreground="#C0CAF5" FontFamily="JetBrainsMono Nerd Font" FontWeight="Bold" FontSize="14"/>
      </StackPanel></Border>
    </Grid>
  </Border></Grid></Viewbox>
</Window>
'@
$reader = New-Object System.Xml.XmlNodeReader $xaml
$window = [Windows.Markup.XamlReader]::Load($reader)
$window.WindowStartupLocation = [Windows.WindowStartupLocation]::Manual
$script:restoringWindowPosition = $true
function Update-PetMaximumSize {
    try {
        if (-not $petFrames -or $petFrames.Count -eq 0) { return }
        $presentationSource=[Windows.PresentationSource]::FromVisual($window)
        if (-not $presentationSource -or -not $presentationSource.CompositionTarget) { return }
        $deviceTransform=$presentationSource.CompositionTarget.TransformToDevice
        $dpiScaleX=[Math]::Max(1.0,[double]$deviceTransform.M11)
        $dpiScaleY=[Math]::Max(1.0,[double]$deviceTransform.M22)
        # The image occupies 116x125 units inside a 210x175 design canvas.
        # Limit its physical device-pixel footprint to the source bitmap pixels.
        $maximumScale=[Math]::Min(
            $petFrames[0].PixelWidth/(116.0*$dpiScaleX),
            $petFrames[0].PixelHeight/(125.0*$dpiScaleY)
        )
        $window.MaxWidth=[Math]::Floor((210.0*$maximumScale)*100.0)/100.0
        $window.MaxHeight=[Math]::Floor((175.0*$maximumScale)*100.0)/100.0
        if ($window.Width -gt $window.MaxWidth) { $window.Width=$window.MaxWidth }
        if ($window.Height -gt $window.MaxHeight) { $window.Height=$window.MaxHeight }
    } catch {}
}
function Save-WindowPosition([switch]$Force) {
    if ($script:restoringWindowPosition -and -not $Force) { return }
    try {
        $position = @{
            Left=[Math]::Round($window.Left,2)
            Top=[Math]::Round($window.Top,2)
            Width=[Math]::Round($window.Width,2)
            Height=[Math]::Round($window.Height,2)
        } | ConvertTo-Json
        $temporaryPositionFile = "$positionFile.tmp"
        $position | Set-Content -LiteralPath $temporaryPositionFile -Encoding utf8
        Move-Item -LiteralPath $temporaryPositionFile -Destination $positionFile -Force
    } catch {}
}
if (Test-Path -LiteralPath $positionFile) {
    try {
        $savedPosition = Get-Content -LiteralPath $positionFile -Raw -ErrorAction Stop | ConvertFrom-Json -ErrorAction Stop
        $left = [double]$savedPosition.Left
        $top = [double]$savedPosition.Top
        if ($null -ne $savedPosition.Width) { $window.Width=[Math]::Min($window.MaxWidth,[Math]::Max($window.MinWidth,[double]$savedPosition.Width)) }
        if ($null -ne $savedPosition.Height) { $window.Height=[Math]::Min($window.MaxHeight,[Math]::Max($window.MinHeight,[double]$savedPosition.Height)) }
        $visibleWidth=[Math]::Min(48.0,$window.Width); $visibleHeight=[Math]::Min(48.0,$window.Height)
        $minimumLeft=[System.Windows.SystemParameters]::VirtualScreenLeft-$window.Width+$visibleWidth
        $maximumLeft=[System.Windows.SystemParameters]::VirtualScreenLeft+[System.Windows.SystemParameters]::VirtualScreenWidth-$visibleWidth
        $minimumTop=[System.Windows.SystemParameters]::VirtualScreenTop
        $maximumTop=[System.Windows.SystemParameters]::VirtualScreenTop+[System.Windows.SystemParameters]::VirtualScreenHeight-$visibleHeight
        $window.Left=[Math]::Min($maximumLeft,[Math]::Max($minimumLeft,$left))
        $window.Top=[Math]::Min($maximumTop,[Math]::Max($minimumTop,$top))
    } catch {}
}
$positionSaveTimer=New-Object Windows.Threading.DispatcherTimer
$positionSaveTimer.Interval=[TimeSpan]::FromMilliseconds(350)
$positionSaveTimer.Add_Tick({
    $positionSaveTimer.Stop()
    Save-WindowPosition
})
$window.Add_LocationChanged({
    $positionSaveTimer.Stop()
    $positionSaveTimer.Start()
})
$window.Add_SizeChanged({
    $positionSaveTimer.Stop()
    $positionSaveTimer.Start()
})
$window.Add_PreviewMouseLeftButtonUp({
    $positionSaveTimer.Stop()
    Save-WindowPosition
})
$window.Add_SourceInitialized({
    $script:restoringWindowPosition = $false
    $helper = New-Object Windows.Interop.WindowInteropHelper($window)
    $script:windowHandle = $helper.Handle
    $source = [Windows.Interop.HwndSource]::FromHwnd($helper.Handle)
    $script:resizeHook = [Windows.Interop.HwndSourceHook]{
        param([IntPtr]$hwnd, [int]$msg, [IntPtr]$wParam, [IntPtr]$lParam, [ref]$handled)
        if ($msg -eq 0x0084) { # WM_NCHITTEST
            $value=$lParam.ToInt64(); $x=[int]($value -band 0xffff); $y=[int](($value -shr 16) -band 0xffff)
            if ($x -ge 32768) { $x-=65536 }; if ($y -ge 32768) { $y-=65536 }
            $rect=New-Object CodexPetWindowNative+RECT
            if (-not [CodexPetWindowNative]::GetWindowRect($hwnd,[ref]$rect)) { return [IntPtr]::Zero }
            $viewScale=[Math]::Min($window.ActualWidth/210.0,$window.ActualHeight/175.0)
            $deviceTransform=$source.CompositionTarget.TransformToDevice
            $insetX=8.0*$viewScale*$deviceTransform.M11; $insetY=8.0*$viewScale*$deviceTransform.M22
            $sensitiveRadius=3.0+(8.0*$viewScale)
            $bandX=$sensitiveRadius*$deviceTransform.M11; $bandY=$sensitiveRadius*$deviceTransform.M22
            $left=[Math]::Abs($x-($rect.Left+$insetX)) -le $bandX
            $right=[Math]::Abs($x-($rect.Right-$insetX)) -le $bandX
            $top=[Math]::Abs($y-($rect.Top+$insetY)) -le $bandY
            $bottom=[Math]::Abs($y-($rect.Bottom-$insetY)) -le $bandY
            $hit=1
            if ($top -and $left) { $hit=13 } elseif ($top -and $right) { $hit=14 }
            elseif ($bottom -and $left) { $hit=16 } elseif ($bottom -and $right) { $hit=17 }
            if ($hit -ne 1) { $handled.Value=$true; return [IntPtr]$hit }
        }
        return [IntPtr]::Zero
    }
    $source.AddHook($script:resizeHook)
    Update-PetMaximumSize
})
$window.Add_DpiChanged({
    $window.Dispatcher.BeginInvoke([Action]{ Update-PetMaximumSize }) | Out-Null
})
function Get-ResizeHit($position) {
    $viewScale=[Math]::Min($window.ActualWidth/210.0,$window.ActualHeight/175.0)
    $inset=8.0*$viewScale; $band=3.0+(8.0*$viewScale)
    $left=[Math]::Abs($position.X-$inset) -le $band
    $right=[Math]::Abs($position.X-($window.ActualWidth-$inset)) -le $band
    $top=[Math]::Abs($position.Y-$inset) -le $band
    $bottom=[Math]::Abs($position.Y-($window.ActualHeight-$inset)) -le $band
    if ($top -and $left) { return 13 }; if ($top -and $right) { return 14 }
    if ($bottom -and $left) { return 16 }; if ($bottom -and $right) { return 17 }
    return 0
}
$window.Add_PreviewMouseMove({
    $hit=Get-ResizeHit $_.GetPosition($window)
    $window.Cursor = switch ($hit) {
        { $_ -in 13,17 } { [Windows.Input.Cursors]::SizeNWSE; break }
        { $_ -in 14,16 } { [Windows.Input.Cursors]::SizeNESW; break }
        default { [Windows.Input.Cursors]::Arrow }
    }
})
$window.Add_PreviewMouseLeftButtonDown({
    $hit=Get-ResizeHit $_.GetPosition($window)
    if ($hit -ne 0 -and $script:windowHandle -ne [IntPtr]::Zero) {
        $_.Handled=$true
        [void][CodexPetWindowNative]::SendMessage($script:windowHandle,0x00A1,[IntPtr]$hit,[IntPtr]::Zero)
        $positionSaveTimer.Stop()
        Save-WindowPosition -Force
    }
})
$card = $window.FindName('Card'); $pet = $window.FindName('Pet'); $petFallback = $window.FindName('PetFallback'); $pulse = $window.FindName('Pulse')
$statusText = $window.FindName('Status')
$petCatalog = [ordered]@{
    'codex'       = 'Codex'
    'bsod'        = 'BSOD'
    'dewey'       = 'Dewey'
    'fireball'    = 'Fireball'
    'null-signal' = 'Null Signal'
    'rocky'       = 'Rocky'
    'seedy'       = 'Seedy'
    'stacky'      = 'Stacky'
}
$script:petFrames = @()
$script:selectedPet = 'codex'
$script:petMenuItems = @{}
$script:lastRenderedFrame = -1
if (Test-Path -LiteralPath $selectedPetFile) {
    $savedPet=(Get-Content -LiteralPath $selectedPetFile -Raw -ErrorAction SilentlyContinue).Trim()
    if ($petCatalog.Contains($savedPet)) { $script:selectedPet=$savedPet }
}
function Set-PetSelection([string]$petKey) {
    if (-not $petCatalog.Contains($petKey)) { return }
    $petFrameRoot=Join-Path $root "assets\pets\$petKey"
    $petFrameFiles=@(Get-ChildItem -LiteralPath $petFrameRoot -Recurse -Filter 'frame_*.png' -File -ErrorAction SilentlyContinue | Sort-Object Name)
    $newFrames=@()
    foreach ($frameFile in $petFrameFiles) {
        try {
            $bitmap=New-Object Windows.Media.Imaging.BitmapImage
            $bitmap.BeginInit(); $bitmap.CacheOption=[Windows.Media.Imaging.BitmapCacheOption]::OnLoad
            $bitmap.UriSource=New-Object System.Uri($frameFile.FullName); $bitmap.EndInit(); $bitmap.Freeze()
            $newFrames += $bitmap
        } catch {}
    }
    if ($newFrames.Count -eq 0) { return }
    $script:petFrames=$newFrames
    $script:selectedPet=$petKey
    $pet.Source=$script:petFrames[0]
    $script:lastRenderedFrame=0
    $pet.Visibility='Visible'; $petFallback.Visibility='Collapsed'
    $petKey | Set-Content -LiteralPath $selectedPetFile -Encoding ascii
    foreach ($key in $script:petMenuItems.Keys) { $script:petMenuItems[$key].IsChecked=($key -eq $petKey) }
    if ($animationClock) { $animationClock.Restart() }
    Update-PetMaximumSize
}
Set-PetSelection $script:selectedPet
if ($script:petFrames.Count -eq 0) { $pet.Visibility='Collapsed'; $petFallback.Visibility='Visible' }
$idleAnimation = [pscustomobject]@{
    Frames    = @(0,1,2,3,4,5)
    Durations = @(1680,660,660,840,840,1920)
    LoopStart = 0
}
function New-CodexStateAnimation([int]$row, [int]$frameCount, [int]$frameDuration, [int]$finalFrameDuration) {
    $frames = @(); $durations = @()
    for ($column=0; $column -lt $frameCount; $column++) {
        $frames += ($row*8+$column)
        $durations += $(if ($column -eq $frameCount-1) { $finalFrameDuration } else { $frameDuration })
    }
    [pscustomobject]@{ Frames=$frames; Durations=$durations; LoopStart=0 }
}
function New-CodexCompletionAnimation([int]$row, [int]$frameCount, [int]$frameDuration, [int]$finalFrameDuration) {
    $primary=New-CodexStateAnimation $row $frameCount $frameDuration $finalFrameDuration
    $frames=@(); $durations=@()
    for ($repeat=0; $repeat -lt 3; $repeat++) { $frames += $primary.Frames; $durations += $primary.Durations }
    $loopStart=$frames.Count
    $frames += $idleAnimation.Frames; $durations += $idleAnimation.Durations
    [pscustomobject]@{ Frames=$frames; Durations=$durations; LoopStart=$loopStart }
}
$petAnimations = @{
    Idle    = $idleAnimation
    Offline = $idleAnimation
    Failed  = New-CodexStateAnimation 5 8 140 240
    Input   = New-CodexStateAnimation 6 6 150 260
    Working = New-CodexStateAnimation 7 6 120 220
    Ready   = New-CodexCompletionAnimation 8 6 150 280
}
$animationClock = [Diagnostics.Stopwatch]::StartNew()
function Get-CodexAnimationFrame($animation, [long]$elapsedMs) {
    $total=0L; foreach ($duration in $animation.Durations) { $total += $duration }
    if ($elapsedMs -ge $total) {
        $loopOffset=0L
        for ($i=0; $i -lt $animation.LoopStart; $i++) { $loopOffset += $animation.Durations[$i] }
        $loopDuration=$total-$loopOffset
        if ($loopDuration -gt 0) { $elapsedMs=$loopOffset+(($elapsedMs-$loopOffset)%$loopDuration) }
    }
    $cursor=0L
    for ($i=0; $i -lt $animation.Frames.Count; $i++) {
        $cursor += $animation.Durations[$i]
        if ($elapsedMs -lt $cursor) { return $animation.Frames[$i] }
    }
    return $animation.Frames[-1]
}
$script:currentState = ''
$script:sessionStateCache = New-CodexPetSessionState
$script:lastSessionDiscovery = [DateTime]::MinValue
$script:latestSessionFile = $null
function Set-State([string]$state, [switch]$Force) {
    if ($script:currentState -eq $state -and -not $Force) { return }
    $styles = @{ Working=@('#7AA2F7',(Get-CodexPetText 'StatusWorking')); Input=@('#E0AF68',(Get-CodexPetText 'StatusInput')); Ready=@('#9ECE6A',(Get-CodexPetText 'StatusReady')); Failed=@('#F7768E',(Get-CodexPetText 'StatusFailed')); Idle=@('#BB9AF7',(Get-CodexPetText 'StatusIdle')); Offline=@('#565F89',(Get-CodexPetText 'StatusOffline')) }
    $s=$styles[$state]; $brush=[Windows.Media.BrushConverter]::new().ConvertFromString($s[0])
    $pulse.Fill=$brush; $card.BorderBrush=$brush
    $card.Effect.Color=([Windows.Media.ColorConverter]::ConvertFromString($s[0])); $statusText.Text=$s[1]
    if ($script:currentState -ne $state) {
        $script:currentState=$state
        $animationClock.Restart()
    }
}
function Get-CodexState {
    if (@(Get-Process -Name codex -ErrorAction SilentlyContinue).Count -eq 0) { return 'Offline' }
    $now=Get-Date
    if (-not $script:latestSessionFile -or ($now-$script:lastSessionDiscovery).TotalSeconds -ge 5) {
        $script:latestSessionFile=Get-ChildItem -LiteralPath "$env:USERPROFILE\.codex\sessions" -Recurse -File -ErrorAction SilentlyContinue |
            Sort-Object LastWriteTime -Descending |
            Select-Object -First 1
        $script:lastSessionDiscovery=$now
    }
    $latest=$script:latestSessionFile
    if (-not $latest) { return 'Idle' }
    $latest=Get-Item -LiteralPath $latest.FullName -ErrorAction SilentlyContinue
    if (-not $latest) { return 'Idle' }
    $state=Read-CodexPetSessionChanges $script:sessionStateCache $latest
    if ($state -eq 'Ready' -and ($now-$latest.LastWriteTime).TotalMinutes -ge 10) { return 'Idle' }
    return $state
}
function Find-CodexPetUpdate {
    try {
        $headers = @{
            Accept       = 'application/vnd.github+json'
            'User-Agent' = "CodexPet/$($script:appVersion)"
        }
        $release = Invoke-RestMethod -Uri $script:releaseApiUrl -Headers $headers -Method Get -TimeoutSec 15
        $tag = [string]$release.tag_name
        $versionText = $tag -replace '^[vV]', ''
        $latestVersion = $null
        if (-not [Version]::TryParse($versionText, [ref]$latestVersion)) {
            throw ((Get-CodexPetText 'InvalidVersion') -f $tag)
        }

        if ($latestVersion -gt $script:appVersion) {
            $answer = [Windows.MessageBox]::Show(
                ((Get-CodexPetText 'NewVersionMessage') -f $latestVersion, $script:appVersion),
                (Get-CodexPetText 'UpdateAvailable'),
                [Windows.MessageBoxButton]::YesNo,
                [Windows.MessageBoxImage]::Information
            )
            if ($answer -eq [Windows.MessageBoxResult]::Yes) {
                Start-Process ([string]$release.html_url)
            }
        } else {
            [Windows.MessageBox]::Show(
                ((Get-CodexPetText 'UpToDateMessage') -f $script:appVersion),
                (Get-CodexPetText 'UpdateTitle'),
                [Windows.MessageBoxButton]::OK,
                [Windows.MessageBoxImage]::Information
            ) | Out-Null
        }
    } catch {
        [Windows.MessageBox]::Show(
            ((Get-CodexPetText 'UpdateError') -f $_.Exception.Message),
            'CodexPet',
            [Windows.MessageBoxButton]::OK,
            [Windows.MessageBoxImage]::Warning
        ) | Out-Null
    }
}
$context=New-Object Windows.Controls.ContextMenu
$petSelectorItem=New-Object Windows.Controls.MenuItem
foreach ($petKey in $petCatalog.Keys) {
    $petItem=New-Object Windows.Controls.MenuItem
    $petItem.Header=$petCatalog[$petKey]; $petItem.Tag=$petKey; $petItem.IsCheckable=$true
    $petItem.IsChecked=($petKey -eq $script:selectedPet)
    $petItem.Add_Click({ param($sender,$eventArgs) Set-PetSelection ([string]$sender.Tag) })
    $script:petMenuItems[$petKey]=$petItem
    [void]$petSelectorItem.Items.Add($petItem)
}
$startupItem=New-Object Windows.Controls.MenuItem; $startupItem.IsCheckable=$true; $startupItem.IsChecked=((Test-Path -LiteralPath $shortcutPath) -or ($null -ne (Get-ItemProperty -Path $runKeyPath -Name $runValueName -ErrorAction SilentlyContinue)))
$startupItem.Add_Click({
    if ($startupItem.IsChecked) {
        if (Test-Path -LiteralPath $shortcutPath) { Remove-Item -LiteralPath $shortcutPath -Force }
        New-Item -Path $runKeyPath -Force | Out-Null
        Set-ItemProperty -Path $runKeyPath -Name $runValueName -Value $watcherCommand
    } else {
        if (Test-Path -LiteralPath $shortcutPath) { Remove-Item -LiteralPath $shortcutPath -Force }
        Remove-ItemProperty -Path $runKeyPath -Name $runValueName -ErrorAction SilentlyContinue
    }
})
$updateItem=New-Object Windows.Controls.MenuItem; $updateItem.Add_Click({ Find-CodexPetUpdate })
$languageItem=New-Object Windows.Controls.MenuItem
$englishLanguageItem=New-Object Windows.Controls.MenuItem; $englishLanguageItem.IsCheckable=$true
$spanishLanguageItem=New-Object Windows.Controls.MenuItem; $spanishLanguageItem.IsCheckable=$true
[void]$languageItem.Items.Add($englishLanguageItem); [void]$languageItem.Items.Add($spanishLanguageItem)
$exitItem=New-Object Windows.Controls.MenuItem; $exitItem.Add_Click({$window.Close()})
function Set-CodexPetLanguage([string]$language) {
    if ($language -notin @('en', 'es')) { return }
    $script:language=$language
    $language | Set-Content -LiteralPath $languageFile -Encoding ascii
    $petSelectorItem.Header=Get-CodexPetText 'ChoosePet'
    $startupItem.Header=Get-CodexPetText 'StartWindows'
    $updateItem.Header="$(Get-CodexPetText 'CheckUpdates') (v$($script:appVersion))"
    $languageItem.Header=Get-CodexPetText 'Language'
    $englishLanguageItem.Header=Get-CodexPetText 'English'
    $spanishLanguageItem.Header=Get-CodexPetText 'Spanish'
    $exitItem.Header=Get-CodexPetText 'ClosePet'
    $englishLanguageItem.IsChecked=($language -eq 'en')
    $spanishLanguageItem.IsChecked=($language -eq 'es')
    if ($script:currentState) { Set-State $script:currentState -Force }
}
$englishLanguageItem.Add_Click({ Set-CodexPetLanguage 'en' })
$spanishLanguageItem.Add_Click({ Set-CodexPetLanguage 'es' })
Set-CodexPetLanguage $script:language
[void]$context.Items.Add($petSelectorItem); [void]$context.Items.Add((New-Object Windows.Controls.Separator)); [void]$context.Items.Add($startupItem); [void]$context.Items.Add($updateItem); [void]$context.Items.Add($languageItem); [void]$context.Items.Add((New-Object Windows.Controls.Separator)); [void]$context.Items.Add($exitItem); $card.ContextMenu=$context
$window.Add_MouseLeftButtonDown({
    # The animation/state timer shares WPF's UI thread with DragMove. Suspending
    # it prevents session scans and frame updates from interrupting native drag.
    $timer.Stop()
    try { $window.DragMove() }
    finally {
        # DragMove blocks until the mouse button is released, so persist the
        # final coordinates here instead of relying only on a delayed tick.
        $positionSaveTimer.Stop()
        Save-WindowPosition -Force
        $timer.Start()
    }
})
$window.Add_MouseDoubleClick({Start-Process 'wt.exe' -ArgumentList 'codex'})
$script:phase=0.0; $script:tick=0
$script:closeWithCodexEnabled=$true
# CodexPet is session-scoped by default. This also covers direct launches from
# old shortcuts that do not yet include -CloseWithCodex.
$script:codexWasRunning=$true
$timer=New-Object Windows.Threading.DispatcherTimer; $timer.Interval=[TimeSpan]::FromMilliseconds(80)
$timer.Add_Tick({
    $script:phase+=0.0933333333
    if ($petFrames.Count -gt 0) {
        $animation=$petAnimations[$script:currentState]
        if (-not $animation) { $animation=$petAnimations.Idle }
        $frameIndex=Get-CodexAnimationFrame $animation $animationClock.ElapsedMilliseconds
        if ($frameIndex -lt $petFrames.Count -and $frameIndex -ne $script:lastRenderedFrame) {
            $pet.Source=$petFrames[$frameIndex]
            $script:lastRenderedFrame=$frameIndex
        }
    }
    $pulse.Opacity=0.45+([Math]::Sin($script:phase*1.4)+1)*0.275
    $script:tick++
    # State discovery parses Codex session logs. Once every two seconds is responsive
    # without repeatedly blocking WPF's dispatcher during normal interaction.
    if ($script:tick%25-eq 1) {
        if (Test-Path -LiteralPath $closeRequestFile) {
            $script:closeWithCodexEnabled=$true
            Remove-Item -LiteralPath $closeRequestFile -Force -ErrorAction SilentlyContinue
        }
        # Codex creates transient codex.exe helpers during an active turn. Keep
        # the pet alive until the whole Codex process group has disappeared.
        $codexIsRunning=@(Get-Process -Name codex -ErrorAction SilentlyContinue).Count -gt 0
        if ($codexIsRunning) { $script:codexWasRunning=$true }
        elseif ($script:closeWithCodexEnabled -and $script:codexWasRunning) { $window.Close(); return }
        Set-State (Get-CodexState)
    }
})
$window.Add_Closing({
    $timer.Stop()
    $positionSaveTimer.Stop()
    Save-WindowPosition -Force
})
$window.Add_Closed({
    if (Test-Path -LiteralPath $pidFile) { Remove-Item -LiteralPath $pidFile -Force }
    if (Test-Path -LiteralPath $closeRequestFile) { Remove-Item -LiteralPath $closeRequestFile -Force -ErrorAction SilentlyContinue }
    $singleInstance.ReleaseMutex()
    $singleInstance.Dispose()
})
# Synchronize the visible label, colors, and animation before the window is
# shown. Subsequent timer ticks keep the state current as the session changes.
Set-State (Get-CodexState)
$timer.Start(); [void]$window.ShowDialog()
