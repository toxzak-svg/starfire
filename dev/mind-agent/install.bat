@echo off
echo ============================================
echo   Mind-Agent Service Installer
echo ============================================
echo.

REM Check for Python
python --version >nul 2>&1
if errorlevel 1 (
    echo ERROR: Python not found!
    pause
    exit /b 1
)

echo [1/3] Installing mind-agent...
pip install -e .

echo [2/3] Creating Windows Task...
schtasks /Create /TN "MindAgent" /XML "mind-agent-task.xml" /F >nul 2>&1
if errorlevel 1 (
    echo Could not create task with XML, trying manual...
    schtasks /Create /TN "MindAgent" /TR "pythonw.exe C:\dev\mind-agent\mind-agent-service.py" /SC ONLOGON /RL LIMITED /F
)

echo [3/3] Starting service...
schtasks /Run /TN "MindAgent"

echo.
echo ============================================
echo   Mind-Agent is now running!
echo.
echo To check status:
echo   schtasks /Query /TN "MindAgent"
echo.
echo To stop:
echo   schtasks /End /TN "MindAgent"
echo.
echo To uninstall:
echo   schtasks /Delete /TN "MindAgent" /F
echo ============================================

pause
