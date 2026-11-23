@echo off
setlocal
cd /d "%~dp0"
set "log=OpenTzar-ostatnia-proba.txt"
if exist "%log%" copy /y "%log%" "OpenTzar-poprzednia-proba.txt" >nul
echo OpenTzar TEST8 - %DATE% %TIME% >"%log%"
"OpenTzar.exe" --root "." --skip-intro --screen Random >"OpenTzar-konsola.txt" 2>&1
set "result=%errorlevel%"
type "OpenTzar-konsola.txt" >>"%log%"
echo Exit code: %result% >>"%log%"
if not "%result%"=="0" start "" notepad "%CD%\%log%"
exit /b %result%
