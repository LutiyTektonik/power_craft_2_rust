# PowerCraft 2 Horizon

**0.6.0** добавляет сортирующую ленту Transport с фильтрами и делением стопок. В сборку также входит перенаправляющая лента 0.5.0. В Windows-прогоне 0.4.0 подтверждены разгрузка трёх лент в сундуки и срабатывание детектора от игрока и предмета. Полный перенос PowerCraft 2 продолжается.

| Блок | Движение | Особенность |
| --- | --- | --- |
| powercraft2:conveyor | Прежние 0,1 блока/тик | Передача в соседние контейнеры |
| powercraft2:speedy_conveyor | Прежние 0,2 блока/тик | С 0.3.1 тоже разгружается — по прямому запросу пользователя |
| powercraft2:brake_conveyor | Прежние 0,06 блока/тик | Redstone тормозит; передача в контейнеры сохранена |
| powercraft2:detector_conveyor | Как normal | Сигнал 15 при пересечении сущностями куба блока, 0 после ухода последней |
| powercraft2:redirector_conveyor | Как normal | Выбирает выход по редстоуну и соседям, разгружается по направлению установки |
| powercraft2:separator_conveyor | Как normal | Две группы фильтров, пропорциональное деление и разгрузка по выбранному выходу |

Разгрузка использует прежние семь vanilla-типов, правила сторон/слотов/остатка и общий лимит 64 предмета на сущность за тик. Скорость Speedy не менялась. Новый детектор выдаёт сигнал на все грани, обновляет соседей на переходах и после загрузки, меняет собственный powered стандартными пакетами. Своя сеть/физика/таймерный поток не добавлены. [Передача](docs/STORAGE_TRANSFER.md), [детектор и ограничения](docs/DETECTOR.md).

Принятые ID, данные/рецепты/модели старых лент и плавное client/server движение сохранены. Из 29 content-файлов базы 0.2.0 26 побайтно прежние, три JSON (языки и творческая вкладка) только дополнены без удаления старых записей. Новые ресурсы созданы нами. Пины не менялись:

```text
Horizon 9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0
Mixins  e59da9094f0c2fdda6c627fe9eff22dec6a906f0
Minecraft 26.3
```

## Установка и проверка

Для продолжения разработки на Windows 11 используйте **WSL2 / Ubuntu 24.04** и локальный Codex в каталоге проекта. Репозиторий — https://github.com/LutiyTektonik/power_craft_2_rust , ветка — **`work/transport-detector`**. В Ubuntu установите зависимости и клонируйте историю:

```sh
sudo apt-get update
sudo apt-get install --no-install-recommends -y git python3 curl ca-certificates build-essential rsync pkg-config libasound2-dev libudev-dev
mkdir -p ~/src
cd ~/src
git -c core.autocrlf=false clone --branch work/transport-detector https://github.com/LutiyTektonik/power_craft_2_rust.git powercraft2-horizon
cd powercraft2-horizon
git config core.autocrlf false
git fetch origin --tags
```

Затем выполните bootstrap и проверки ниже. Используются Rust **1.95.0**, Python **>=3.11**, Bash; ресурсы сборки загружаются заново и не входят в Git. После bootstrap в новом терминале выполните `source scripts/env.sh`. Откройте этот каталог локальным Codex внутри WSL и дайте ему прочитать **AGENTS.md**. Полная инструкция, восстановление Git bundle и состав резервного ZIP — [LOCAL_DEVELOPMENT.md](docs/LOCAL_DEVELOPMENT.md). Проверенная в игре база трёх лент сохранена тегом `powercraft2-v0.2.0-tested`; локальная ветка `work/transport-separator` содержит 0.6.0.

Обновить обе части до **0.6.0**, удалить дубликаты старых версий и выполнить native rebuild в Launcher. .hnmod содержит исходники, не готовую Windows DLL/EXE. Сначала копия мира: новые ленты появляются в сохранении только с соответствующей версией контента. [Windows 11](docs/WINDOWS11.md).

```sh
scripts/bootstrap.sh --install
source scripts/env.sh
scripts/check.sh
scripts/build.sh
NATIVE_NO_DEFAULT_FEATURES=1 scripts/check.sh --horizon
python3 scripts/prepare-windows.py
```

Результаты текущих проверок и хеши пакетов приведены в [отчёте](docs/VERIFICATION.json). Для игрового прогона двух новых лент см. [сценарии приёмки](docs/TESTING.md).

## Соответствие оригиналу

Speedy-разгрузка — явно согласованное дополнение, его нельзя удалять при дальнейшей сверке оригинала. Детектор восстанавливает опубликованные движение/обнаружение/redstone; упаковка/despawn-подправила, sticky boots и поршневое поведение пока не перенесены. Текущий detector использует штатную block entity, поэтому не двигается поршнями. Есть стоимость серверного опроса сущностей; массовая нагрузка ещё не профилирована.

Экспериментальные/неисправные/незавершённые функции оригинала (Logic/Storage/Mobile и проблемные подфункции) остаются отложенными. Остальные принятые адаптации — высота 1/4, плавная ограниченная скорость, штатный pickup/despawn и прежние medium-рецепты — сохраняются.

- [Прогресс](docs/PORTING_PROGRESS.md), [пять категорий сравнения](docs/PORTING_COMPARISON.json)
- [Опубликованный релиз](docs/PUBLISHED_RELEASE.md), [полный beta-реестр](docs/PUBLISHED_MECHANICS.json)
- [Архитектура](docs/ARCHITECTURE.md), [совместимость](docs/COMPATIBILITY.md), [лицензии](docs/LEGAL.md), [история](CHANGELOG.md)

Нет обязательных сторонних модов. Именованные хуки не переписывают функции движка. Читать GameLevel для AABB и менять клиентскую velocity приходится через документированные pin-specific мосты — это не стабильный API для других SHA. Оригинальные JAR/Java/графика и полные исходники Horizon в Git/архивы проекта не входят.
