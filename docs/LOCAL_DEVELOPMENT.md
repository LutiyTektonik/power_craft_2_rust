# Продолжение разработки на Windows 11

Репозиторий: https://github.com/LutiyTektonik/power_craft_2_rust . Ветка `work/transport-detector` содержит 0.4.0. Продолжение 0.5.0 с перенаправляющей лентой ведётся локально в `work/transport-redirector`. Проверенная пользователем игровая база трёх лент — тег `powercraft2-v0.2.0-tested`.

## Окружение

Скрипты проекта рассчитаны на Bash/Linux. Для Windows 11 используйте WSL2 с Ubuntu 24.04; игра и Horizon Launcher остаются на Windows. Это документированный маршрут разработки, а не заявление о проведённой здесь Windows/WSL-проверке. Linux-сборка игры не заменяет native rebuild в Windows Launcher.

Если WSL ещё нет, в PowerShell администратора выполните `wsl --install -d Ubuntu-24.04`, перезагрузитесь при запросе и закончите настройку пользователя Ubuntu. Дальнейшие команды выполняются **в Ubuntu**, не в PowerShell. Храните исходники в Linux-файловой системе WSL (`~/src`), чтобы не терять права исполнения и производительность. Предусмотрите несколько гигабайт для инструментов; полная интеграция требует не менее 6 ГиБ свободного места перед стартом и может занять больше при компиляции. Лимит параллельной сборки — два задания.

```sh
sudo apt-get update
sudo apt-get install --no-install-recommends -y git python3 curl ca-certificates build-essential rsync pkg-config libasound2-dev libudev-dev
mkdir -p ~/src
cd ~/src
git -c core.autocrlf=false clone --branch work/transport-detector https://github.com/LutiyTektonik/power_craft_2_rust.git powercraft2-horizon
cd powercraft2-horizon
git config core.autocrlf false
git fetch origin --tags
scripts/bootstrap.sh --install
source scripts/env.sh
scripts/check.sh
```

Bootstrap использует имеющиеся подходящие инструменты, устанавливает закреплённый Rust 1.95.0 с rustfmt/Clippy при отсутствии, получает два официальных Python-инструмента по проверенным Git blob ID и собирает `cargo-horizon-mod` из закреплённой ревизии. Требуются сеть и Python >=3.11 (в Ubuntu 24.04 — 3.12). Если Rust ставится в `.tools/`, в каждом новом терминале нужен `source scripts/env.sh`. Кэши и исходники движка не переносятся: они воспроизводимо загружаются заново. `bootstrap.sh` без `--install` только проверяет окружение и может завершиться ошибкой на свежем клоне.

Для быстрого старта без сборки CLI Horizon можно сначала установить Rust штатным rustup (https://rustup.rs), выполнить `rustup toolchain install 1.95.0 --profile minimal --component rustfmt --component clippy`, затем `python3 scripts/fetch-tools.py --download` и `scripts/check.sh`. Полный bootstrap остаётся необходимым для упаковки и интеграционных проверок.

## Локальный Codex

Откройте каталог `~/src/powercraft2-horizon` в локальном Codex, который работает внутри WSL, либо через IDE с подключением к WSL и расширением Codex. Не открывайте вместо него корень пользовательского каталога или каталог с распакованными кэшами. Для CLI запускайте уже установленный `codex` из этого каталога в WSL. Авторизация Codex/GitHub выполняется штатно на компьютере; облачные токены и настройки учётных записей не входят в перенос.

Первое поручение Codex: «Прочитай AGENTS.md, README.md, docs/STATUS.md, docs/PORTING_PROGRESS.md, docs/VERIFICATION.json. Проверь окружение и scripts/check.sh. Продолжай из текущей версии, сохрани ID, физику, Speedy-разгрузку и проверенные конвейеры. Не меняй пины Horizon. Новые игровые функции сейчас не добавляй без задания». Перед изменениями создайте свою ветку: `git switch -c work/local-development`. Не переписывайте опубликованные коммиты и теги.

## Сборка и проверка

```sh
source scripts/env.sh
scripts/check.sh --static                 # Python, структура; не Rust/Horizon
scripts/check.sh                          # Python, fmt, Clippy, Rust-тесты
scripts/build.sh --source-only            # исходные пакеты; не компиляция игры
scripts/build.sh                          # Rust release, пакеты и официальная валидация
NATIVE_NO_DEFAULT_FEATURES=1 scripts/check.sh --horizon
python3 scripts/prepare-windows.py        # Windows test kit после сборки
```

Последняя проверка компилирует реальный закреплённый Horizon, скачивает зависимости и может быть тяжёлой. Не считать её выполненной после одного `cargo test`. Актуальные пины находятся в `upstream.lock.json`, `compatibility.json` и `horizon-mixin.toml`; не заменять их `main`. Устанавливаемые файлы: `dist/powercraft2-0.5.0.hnmod` и `dist/powercraft2-0.5.0-content.hmod`. Копировать их из WSL в Windows можно через Проводник (`explorer.exe dist`). Установку и native rebuild выполнять по [WINDOWS11.md](WINDOWS11.md). `.hnmod` содержит исходники, а не готовую DLL. Игровые действия — [TESTING.md](TESTING.md).

## Резервный комплект без доступа к GitHub

Подготовленный `powercraft2-local-transfer.zip` содержит:

- `source/` — все отслеживаемые исходники, свои PNG, тесты, документацию, CI, Cargo.lock, toolchain и манифесты сборки;
- `project.bundle` — все локальные ветки и теги с полной историей, без секретов и конфигурации авторизации;
- `artifacts/` — неизменённые установочные пары текущей версии и проверенной 0.2.0, текущий Windows test kit;
- `TRANSFER_MANIFEST.json` — commit, ветка, refs и SHA-256 каждого файла комплекта;
- `RESTORE.txt` — краткие команды восстановления.

Распакуйте ZIP в отдельный каталог. Для продолжения **с историей** клонируйте bundle, а не просто откройте `source/`:

```sh
git clone --branch work/transport-detector ./project.bundle powercraft2-horizon
cd powercraft2-horizon
git remote set-url origin https://github.com/LutiyTektonik/power_craft_2_rust.git
git fetch origin --tags
git status --short
```

`git fetch` можно пропустить без сети. Clone bundle создаёт рабочую ветку и remote tracking refs остальных веток; при необходимости они восстанавливаются через `git switch --track origin/ИМЯ`. Теги доступны сразу. Не копируйте поверх клона `source/`: это самостоятельный снимок тех же файлов. Для прежней подтверждённой базы используйте отдельный checkout/worktree тега `powercraft2-v0.2.0-tested`; не понижайте версию мира с detector без подходящей резервной копии.

Повторное создание комплекта из чистого закоммиченного проекта с сохранённым `dist/`: `python3 scripts/export-local.py`. Скрипт проверяет SHA-256 пакетов текущей версии по сохранённому отчёту и отказывается включать незакоммиченные изменения. На чистом GitHub-клоне для комплекта сначала нужны установочные пакеты в `dist/`; исходники и Git-история уже полностью доступны через clone. ZIP из кнопки GitHub «Download ZIP» **не содержит историю Git**.

Не включены `target/`, `.cache/`, `.tools/`, пользовательские миры и настройки Launcher, исходники/ресурсы сторонних проектов, старые JAR, учётные данные. Миры при необходимости копируются отдельно штатными средствами Launcher. Cloud-абсолютные пути в старых журналах — исторические сведения; команды выше используют текущий каталог проекта.
