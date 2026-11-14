> 0.3.0 сохраняет тот же Horizon/Mixins. Дополнительные публичные helpers insert_item API 20, item_entities/set_item_entity API 42; новых хуков/обновления движка нет. Официальная интеграция прошла; см. текущий VERIFICATION.json.

> Выпуск 0.2.0 сохраняет эти же пины. Официальная интеграция прошла; добавлены публичные Host::neighbor_signal и set_block_property_with, без повышения минимального API и без новых хуков. База 0.1.1 уже подтверждена пользователем в игре; новые ленты пока не приняты. Текущий отчёт — VERIFICATION.json.

> Актуальная цель 0.1.1 — фактическая сборка пользователя `9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0`, API 415. Исходный прототип 0.1.0 исследовался на f4d8d62; сохранённая ниже история исследования не является новой игровой проверкой.

# Версии Horizon и инструментария

Исследование выполнено 14 ноября 2026 года по официальным репозиториям и `playhorizon.dev/docs`. Первоначально исходники изучались через коннектор GitHub. При исправлении 0.1.1 разрешённый сетевой режим позволил официальному инструменту получить полное закреплённое дерево в игнорируемый native/target/ и выполнить настоящую проверку. Машиночитаемые пины — `upstream.lock.json`.

| Проект | Полный commit SHA |
| --- | --- |
| [Horizon](https://github.com/horizon-rust/horizon/commit/9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0) | `9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0` |
| [Horizon Mixins](https://github.com/horizon-rust/horizon-mixins/commit/e59da9094f0c2fdda6c627fe9eff22dec6a906f0) | `e59da9094f0c2fdda6c627fe9eff22dec6a906f0` |
| [Horizon Mods](https://github.com/horizon-rust/horizon-mods/commit/656f948e0d058d647676857625da3d4b66755929) | `656f948e0d058d647676857625da3d4b66755929` |
| [PowerCraft 2.0](https://github.com/PowerCraft/2.0/commit/795369c5a03c7cf39307398a432a5d644698e1df) | `795369c5a03c7cf39307398a432a5d644698e1df` |

Цель 0.1.1 выбрана по фактической работающей сборке пользователя, а не по текущему main. Официальный cargo check с модом на этом SHA прошёл. Новый игровой прогон и Windows-сборка ещё требуются; подробности в SMOOTH_MOVEMENT_VERIFICATION.json.

## Игра и формат данных

В `crates/horizon-core/src/version.rs`: Minecraft **26.3**, protocol **777**, DataVersion **5023**, resource pack format **97.1**, data pack format **121.0**. `STABLE=true` относится к версии Minecraft, а не к стабильности API модов. Workspace Horizon требует Rust 1.95 и edition 2024; для проекта закреплён stable Rust 1.95.0.

`docs/api/mod-files.md` обозначает content как **preview**, native code как **experimental**. Раздел typed trait mods в `docs/native-mods.md` помечен **draft**. У именованных хуков есть версионированный контракт, но весь нативный механизм нельзя называть стабильным публичным ABI.

## Реальные возможности API

| Область | Проверенные исходники и использование |
| --- | --- |
| `.hmod` | ZIP с `horizon-mod.toml`, `assets/`, `data/`. API 0.2, `capabilities=["registries"]`; исполняемого Rust-кода здесь нет |
| Регистрация | `data/powercraft2/horizon/block/*.json`, `creative_tab/*.json`; блок с `item=true` создаёт соответствующий предмет. Шаблон vanilla, свойства и формы — `horizon-mods/src/content.rs` |
| Идентификаторы | mod ID `[a-z0-9_]{1,64}`; `powercraft2` подходит. Resource location `powercraft2:conveyor` подходит; namespace уникален внутри загруженного набора |
| Мир | Typed `LevelRef` читает блок, свойства, загруженность. Нет загрузки чанка нашим чтением; неизвестное/незагруженное возвращает `None`/`false` |
| Тики | `entity_tick_listened` и `entity_tick_post`, версия 1, API 146. Серверные entity-ticking chunks; Post после всей семьи предметов, включая уже исчезнувший предмет |
| Сущности | `EntityRef` — временный handle, API 373; скорость, bounding box и on_ground — API 379. `set_velocity` отказывает клиенту, игроку, устаревшей сущности и нечисловым значениям |
| Размещение | `state_for_placement`, версия 1; настоящий `PlacementContext` имеет `yaw`. Штатные `state_block`/`state_with_property` проверены в `GameLevel` |
| Сохранение | Обычные block states сохраняются движком. API также имеет mod block entities, SNBT и attachments; для простого конвейера они не используются |
| Клиент/сервер | Контент `side=both`; авторитетная физика на сервере. Публичный setter не меняет клиента; в 0.1.1 отдельный tick-адаптер меняет только внутреннюю velocity клиента |
| `.hnmod` | ZIP `horizon-native.json` + `mixins/powercraft2/`; это исходный bundle, который лаунчер компилирует в отдельную игру |
| `horizon-mixin.toml` | `[mod]`, `[compatibility]`, один `[[module]]`, четыре именованных `[[injection]]` вида HEAD. Исходное дерево движка не редактируется |
| `compatibility.json` | Локальные пины game/hooks/Horizon/Mixins; официальный `native.py` использует их при упаковке и staging |

WebAssembly `mod.wasm` — legacy, в актуальном формате больше не запускается. Java Forge/Fabric-классы не являются Rust API и здесь не запускаются. Объединённый native+content `.hmod` поддерживается форматом, но для простоты и совместимости упаковки выбраны две части.

## Доказательства и проверки

`API_EVIDENCE.json` содержит каталожные записи хуков/методов и точные Git blob ID девяти прочитанных исходных файлов. Команда `python3 scripts/audit.py --sources /путь/к/чистым/исходникам` проверяет эти ID. Такой аудит обнаруживает несовпадающую ревизию исходников, но не заменяет Rust type checking.

`native/tools/native.py` и `game.py` получены побайтно из закреплённого Horizon Mods; `TOOL_PROVENANCE.json` фиксирует исходные blob ID, аудит проверяет их неизменность. Копии считаются внешним локальным инструментарием и исключены из Git: лицензия корня Horizon Mods не обнаружена. Для чистого checkout есть `python3 scripts/fetch-tools.py --download`. Использованные настоящие команды:

```sh
python3 native/tools/native.py --help
python3 native/tools/native.py package powercraft2
# При доступных инструментах и сети:
python3 native/tools/native.py check powercraft2
python3 native/tools/native.py build powercraft2
cargo horizon-mod check dist/powercraft2-0.1.0-content.hmod
```

Официальный `native.py check` скачивает pinned Horizon, применяет Mixins в staged-дерево, затем вызывает `cargo check -p horizon-client --bin horizon-client --locked`. Пакетирование не выполняет эту проверку. Отдельный режим `--horizon latest` здесь намеренно не используется.
