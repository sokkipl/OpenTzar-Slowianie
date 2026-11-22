# OpenTzar — Słowianie (TEST3 / Windows x64)

Prywatna gałąź testowa projektu opartego na ReTzar 1.1.0 (GPL-3.0-or-later), rozszerzona o Słowian jako czwartą nację.

## Jak uzyskać gotowy ZIP z EXE

1. Pobierz przygotowany w rozmowie ChatGPT plik `OpenTzar_TEST3_Zrodla_Build.zip` (nie zamieniaj go na starsze `Zrodla.zip` z TEST2).
2. W GitHub wybierz **Add file → Upload files** i prześlij ten ZIP do głównego katalogu repozytorium (obok `README.md`).
3. Kliknij **Commit changes**. Wysłanie tego pliku automatycznie uruchamia workflow `OpenTzar TEST3 — Windows x64`.
4. Wejdź w zakładkę **Actions**, wybierz najnowszą kompilację i poczekaj na wynik (zielony znacznik oznacza poprawną kompilację, nie zweryfikowaną grę).
5. Z sekcji **Artifacts** pobierz `OpenTzar_TEST3_Win64`. Po otwarciu archiwum pobierz gotowy `OpenTzar_TEST3_Windows_x64.zip`.
6. Rozpakuj do osobnego folderu i uruchom `START_SLOWIANIE.cmd` albo `OpenTzar.exe`.

Projekt **nie zawiera** plików `Tzar.exe`, `ReTzar.dat` ani archiwów gry `.wdt` i nie powinien modyfikować instalacji GOG. Pierwsze uruchomienie wymaga wskazania/zatwierdzenia legalnie zainstalowanego Tzara. Domyślna ścieżka to `C:\Program Files\GOG Galaxy\Games\Tzar`.

**Uwaga:** Źródła TEST2 obejmują też wbudowane lokalizacje/fonty o nie w pełni ustalonych prawach. Repozytorium i artefakty pozostają **prywatne do osobistych testów**, bez publicznego wydawania kompilacji do czasu zakończenia audytu pochodzenia. Kompilacja nie potwierdza naprawy `DATA/IMAGES.INI` ani działania czwartej nacji — wymaga to testu na oryginalnej instalacji GOG.

## Problemy

Gdy GitHub Actions pokaże czerwony błąd, zajrzyj do logów kroku kompilacji. Po uruchomieniu gry przy problemach z mapą dołącz `OpenTzar-ostatnia-proba.txt`. Nie publikuj oryginalnego EXE Tzara ani plików gry.

## Źródła i licencja

Kod na bazie [ColCarlo/ReTzar](https://github.com/ColCarlo/ReTzar), licencja GPL-3.0-or-later (pełny tekst `LICENSE` w archiwum źródeł). ReTzar nie jest oficjalnym produktem Haemimont Games, Take-Two ani FX Interactive.
