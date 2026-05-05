@echo off
:: Rollback script — delegates to gaming-mode.ps1
:: Preserved for backward compatibility; prefer running the .ps1 directly.

powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "& '%~dp0gaming-mode.ps1' -Rollback"
if %ERRORLEVEL% neq 0 (
    echo.
    echo [FAIL] PowerShell rollback failed. Try running manually:
    echo   powershell -ExecutionPolicy Bypass -File "%~dp0gaming-mode.ps1" -Rollback
    pause
    exit /b %ERRORLEVEL%
)

echo.
echo ================================================
echo Rollback Complete!
echo ================================================
echo.
echo Please restart your computer for changes to take effect.
echo.
pause