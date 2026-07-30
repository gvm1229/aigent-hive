@echo off
setlocal EnableExtensions DisableDelayedExpansion
set "HIVE_INSTALL_VERSION=__AIGENT_HIVE_VERSION__"
set "HIVE_INSTALL_URL=https://unpkg.com/aigent-hive@%HIVE_INSTALL_VERSION%/install.ps1"
set "HIVE_INSTALL_SCRIPT=%TEMP%\aigent-hive-install-%RANDOM%-%RANDOM%.ps1"

powershell.exe -NoLogo -NoProfile -NonInteractive -Command "$ProgressPreference='SilentlyContinue'; Invoke-WebRequest -UseBasicParsing -Uri $env:HIVE_INSTALL_URL -OutFile $env:HIVE_INSTALL_SCRIPT"
if errorlevel 1 (
  set "HIVE_INSTALL_EXIT=%ERRORLEVEL%"
  del /f /q "%HIVE_INSTALL_SCRIPT%" >nul 2>&1
  exit /b %HIVE_INSTALL_EXIT%
)

powershell.exe -NoLogo -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "%HIVE_INSTALL_SCRIPT%"
set "HIVE_INSTALL_EXIT=%ERRORLEVEL%"
del /f /q "%HIVE_INSTALL_SCRIPT%" >nul 2>&1
exit /b %HIVE_INSTALL_EXIT%
