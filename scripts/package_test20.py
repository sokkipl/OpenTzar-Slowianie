"""Create the Windows player folder from the registered game resources."""
import re
import shutil
import sys
from pathlib import Path

src, binary, dest = map(Path, sys.argv[1:4])
dest.mkdir(parents=True, exist_ok=True)
shutil.copyfile(binary, dest / 'OpenTzar.exe')
paths = re.findall(r'\("([^"]+)",\s*include_bytes!', (src/'src/runtime_art.rs').read_text('utf-8'))
if len(set(paths)) != len(paths) or not paths:
    raise ValueError('Invalid resource registry')
for rel in paths:
    if not re.fullmatch(r'(art|src)/[A-Za-z0-9_/.-]+', rel) or '..' in Path(rel).parts:
        raise ValueError(rel)
    output = dest/'src'/rel
    output.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(src/rel, output)
shutil.copyfile(src/'LICENSE', dest/'LICENSE.txt')
cmd = """@echo off
setlocal
cd /d "%~dp0"
if not exist "OpenTzar.exe" goto missing
start "" /wait "%~dp0OpenTzar.exe" --ot-check-assets
if errorlevel 1 goto invalid
start "" /wait "%~dp0OpenTzar.exe" --root "%~dp0." --skip-intro --screen Random
if not errorlevel 1 exit /b 0
echo Blad podczas uruchamiania gry. Sprawdz pliki diagnostyczne w tym folderze.
pause
exit /b 1
:missing
echo Brak OpenTzar.exe. Rozpakuj caly ZIP przed uruchomieniem.
pause
exit /b 1
:invalid
echo Blad kontroli zasobow.
if exist runtime-art-v1.error type runtime-art-v1.error
pause
exit /b 1
"""
(dest/'START_SLOWIANIE.cmd').write_bytes(cmd.replace('\n','\r\n').encode('ascii'))
(dest/'CZYTAJ.txt').write_text('OpenTzar TEST20 — Windows x64\nRozpakuj cały ZIP do osobnego folderu i uruchom START_SLOWIANIE.cmd.\nPrzy pierwszym starcie wskaż główny folder oryginalnego Tzara z GOG, np. C:\\Program Files\\GOG Galaxy\\Games\\Tzar.\nGra czyta zasoby oraz czcionki bezpośrednio z Twojej instalacji.\n', encoding='utf-8-sig')
print(f'{len(paths)} resources')
