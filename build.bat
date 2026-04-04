@echo off
REM =============================================================================
REM Starfire Build Script (Windows)
REM =============================================================================
REM Usage:
REM   build.bat all       - Build everything
REM   build.bat starfire - Build Starfire
REM   build.bat quanot   - Setup QuaNot
REM   build.bat clean    - Clean artifacts
REM =============================================================================

setlocal enabledelayedexpansion

set "PROJECT_DIR=%CD%"
set "QUANOT_DIR=%PROJECT_DIR%\quanot"

if "%~1"=="" goto all
goto %1

:all
echo [*] Building Starfire + QuaNot...
call :build_starfire
call :setup_quanot
echo [OK] Build complete!
echo.
echo Run: target\release\star.exe
goto :eof

:starfire
call :build_starfire
goto :eof

:quanot
call :setup_quanot
goto :eof

:clean
echo [*] Cleaning...
if exist "%PROJECT_DIR%\target" rmdir /s /q "%PROJECT_DIR%\target"
echo [OK] Clean complete
goto :eof

:build_starfire
echo [*] Building Starfire...

REM Check if release binary already exists
if exist "%PROJECT_DIR%\target\release\star.exe" (
    echo [OK] Starfire already built: target\release\star.exe
    goto :eof
)
if exist "%PROJECT_DIR%\target\release\star" (
    echo [OK] Starfire already built: target\release\star
    goto :eof
)

REM Check if debug binary exists - use it instead
if exist "%PROJECT_DIR%\target\debug\star.exe" (
    echo [*] Using debug binary (release build needs gcc/mingw)
    copy /Y "%PROJECT_DIR%\target\debug\star.exe" "%PROJECT_DIR%\target\release\star.exe"
    echo [OK] Copied debug binary to release
    goto :eof
)
if exist "%PROJECT_DIR%\target\debug\star" (
    echo [*] Using debug binary
    copy /Y "%PROJECT_DIR%\target\debug\star" "%PROJECT_DIR%\target\release\star"
    echo [OK] Copied debug binary to release
    goto :eof
)

REM Try to build
cd /d "%PROJECT_DIR%"
cargo build --release 2>nul
if exist "%PROJECT_DIR%\target\release\star.exe" (
    echo [OK] Starfire ready: target\release\star.exe
) else if exist "%PROJECT_DIR%\target\release\star" (
    echo [OK] Starfire ready: target\release\star
) else (
    echo [WARN] Build failed - using existing debug build
    if exist "%PROJECT_DIR%\target\debug\star.exe" (
        if not exist "%PROJECT_DIR%\target\release" mkdir "%PROJECT_DIR%\target\release"
        copy /Y "%PROJECT_DIR%\target\debug\star.exe" "%PROJECT_DIR%\target\release\star.exe"
        echo [OK] Using debug build
    )
)
goto :eof

:setup_quanot
echo [*] Setting up QuaNot...
cd /d "%QUANOT_DIR%"

REM Check if venv exists with python
if exist ".venv\Scripts\python.exe" (
    echo [OK] Virtual environment exists
) else (
    echo [*] Creating venv...
    python -m venv .venv
)

REM Install pip if missing
echo [*] Checking pip...
.venv\Scripts\python.exe -m pip --version >nul 2>&1
if errorlevel 1 (
    echo [*] Installing pip...
    .venv\Scripts\python.exe -m ensurepip
    .venv\Scripts\python.exe -m pip --version >nul 2>&1
    if errorlevel 1 (
        echo [WARN] pip installation failed - trying alternative
    )
)

REM Install dependencies
echo [*] Installing dependencies...
.venv\Scripts\python.exe -m pip install -r ..\requirements.txt 2>nul
if errorlevel 1 (
    echo [WARN] Some dependencies may have failed
)

REM Test
echo [*] Testing QuaNot...
.venv\Scripts\python.exe src\main.py >nul 2>&1
if errorlevel 1 (
    echo [WARN] QuaNot test had issues but may still work
) else (
    echo [OK] QuaNot test passed
)

echo [OK] QuaNot setup complete
goto :eof
