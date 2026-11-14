> Уточнение текущей очереди: экспериментальные/неисправные/незавершённые функции отложены по последнему заданию. Этот реестр описывает GitHub-снимок; опубликованная beta проверена отдельно в PUBLISHED_RELEASE.md/PUBLISHED_MECHANICS.json. Рецепты двух источников нельзя смешивать.

# Реестр оригинального PowerCraft 2

Источник: `PowerCraft/2.0` commit `795369c5a03c7cf39307398a432a5d644698e1df`. Все **445** текстовых исходных/build-файлов скачаны через официальный GitHub-коннектор; размеры и Git blob ID каждого совпали с полным неусечённым деревом. В составе — **427 Java-файлов**. Копии находятся вне Git нового мода; оригинальная графика не включена.

`ORIGINAL_MECHANICS.json` содержит полный машинный индекс классов, публичных методов, констант, TODO-строк, **62** деклараций `PC_FieldObject`, **177** деклараций рецептов и **78** GUI/container/виджет-классов. Числа включают API и служебные объекты; это не 62 самостоятельных игровых блока и не 177 разных рецептов, одновременно доступных игроку. Рецепты medium/legacy и 3D учтены отдельно. Это статический анализ, не запуск старого Forge и не доказательство исправности каждого модуля.

Лексический индекс воспроизводится: `python3 scripts/inventory-original.py --source /путь/к/исходникам`. Перед обработкой он проверяет все файлы по `ORIGINAL_SOURCE_INDEX.json`. Комментарии исключены из поиска регистраций и рецептов. Сложные динамические/3D-обработчики помечены `manual_review_required`: точное размещение блоков и проверки смотрят в указанных классах.

## Категории оригинала

1. **Исходная реализация имеется.** Core, Transport, Machines, Deco, Net, Light, Hologram, Teleport имеют регистрацию и код поведения. Отдельные ограничения/TODO ниже учитываются отдельно; наличие кода не означает отсутствие ошибок.
2. **Частично реализовано или требует исправлений.** Автор прямо пометил Logic, Storage и Mobile как in development, Storage/Mobile — not working correctly. Они сейчас отложены; их исправление и перенос не входят в текущую очередь. В других модулях частичные особенности вынесены отдельно.
3. **Только планы/удалённое.** Weasel отсутствует и планировался rewrite; CheckPoints удалён; Integrations содержит только build.gradle и не подключён в settings.gradle; общая энергетика и Legacy Logic/Weasel — планы. ItemStorage — прежнее имя Storage, не дополнительный модуль.

## Зарегистрированные объекты и взаимодействия

### core

| Объект / основной класс | Механика и ограничения | Категория |
| --- | --- | --- |
| `powerCrystal` / [`PCco_BlockPowerCrystal`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/core/src/main/java/powercraft/core/PCco_App.java#L37) | Кристалл: цветовые metadata, добыча/генерация и реакция на луч; источник компонентов. | 1 |
| `blockSaver` / [`PCco_BlockBlockSaver`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/core/src/main/java/powercraft/core/PCco_App.java#L39) | Сохранение блока в предмете с данными; потребуется безопасный whitelist NBT и восстановления. | 1 |
| `powerDust` / [`PCco_ItemPowerDust`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/core/src/main/java/powercraft/core/PCco_App.java#L41) | Пыль для рецептов; самостоятельной энергетической сети не создаёт. | 1 |
| `activator` / [`PCco_ItemActivator`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/core/src/main/java/powercraft/core/PCco_App.java#L43) | Инструмент активации механизмов и 3D-структур; расход/проверки на сервере. | 1 |
| `craftingTool` / [`PCco_ItemCraftingTool`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/core/src/main/java/powercraft/core/PCco_App.java#L45) | Переносной интерфейс крафта, варианты рецептов и работа с инвентарём. | 1 |
| `oreSniffer` / [`PCco_ItemOreSniffer`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/core/src/main/java/powercraft/core/PCco_App.java#L47) | Поиск руды и обратная связь игроку; ограничивать область сканирования. | 1 |
| `craftingCore` / [`PCco_ItemCraftingCore`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/core/src/main/java/powercraft/core/PCco_App.java#L49) | Компонент рецептов; собственная механика крафтера в классе предмета отсутствует. | 1 |
| `formationCore` / [`PCco_ItemFormationCore`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/core/src/main/java/powercraft/core/PCco_App.java#L51) | Компонент рецептов; не доказательство отдельного работающего строительного устройства. | 1 |
| `annihilationCore` / [`PCco_ItemAnnihilationCore`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/core/src/main/java/powercraft/core/PCco_App.java#L53) | Компонент рецептов; не отдельная машина разрушения. | 1 |
| `sensorCore` / [`PCco_ItemSensorCore`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/core/src/main/java/powercraft/core/PCco_App.java#L55) | Компонент рецептов; датчики реализованы отдельными блоками Net/Logic. | 1 |
| `spawnerSetter` / [`PCco_MobSpawnerSetter`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/core/src/main/java/powercraft/core/PCco_App.java#L57) | Служебный обработчик настройки спавнеров; не отдельный зарегистрированный предмет. | 1 |

### deco

| Объект / основной класс | Механика и ограничения | Категория |
| --- | --- | --- |
| `redstoneStorage` / [`PCde_BlockRedstoneStorage`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/deco/src/main/java/powercraft/deco/PCde_App.java#L26) | Блоки хранения redstone с metadata/рецептами преобразования. | 1 |
| `ironFrame` / [`PCde_BlockIronFrame`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/deco/src/main/java/powercraft/deco/PCde_App.java#L28) | Соединяемые железные рамы, зависящие от соседей формы/коллизии. | 1 |
| `chimney` / [`PCde_BlockChimney`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/deco/src/main/java/powercraft/deco/PCde_App.java#L30) | Дымоход, варианты и тик частиц дыма. | 1 |
| `platform` / [`PCde_BlockPlatform`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/deco/src/main/java/powercraft/deco/PCde_App.java#L32) | Платформа с ограждениями и выбором сторон при размещении. | 1 |
| `stairs` / [`PCde_BlockStairs`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/deco/src/main/java/powercraft/deco/PCde_App.java#L34) | Лестница/ступени с ориентацией и ограждениями. | 1 |

### hologram

| Объект / основной класс | Механика и ограничения | Категория |
| --- | --- | --- |
| `hologramBlockEmpty` / [`PChg_BlockHologramBlockEmpty`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/hologram/src/main/java/powercraft/hologram/PChg_App.java#L27) | Невидимый/голографический блок, выбор формы и видимости. | 1 |
| `hologramField` / [`PChg_BlockHologramField`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/hologram/src/main/java/powercraft/hologram/PChg_App.java#L29) | Поле с сохраняемым смещением, создание/рендер голограммы. | 1 |
| `hologramGlasses` / [`PChg_ItemArmorHologramGlasses`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/hologram/src/main/java/powercraft/hologram/PChg_App.java#L31) | Броня-очки для просмотра голограмм. | 1 |
| `hologramGlassesOverlay` / [`PChg_HologramGlassesOverlay`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/hologram/src/main/java/powercraft/hologram/PChg_AppClient.java#L33) | Клиентский overlay очков; служебный обработчик, не предмет. | 1 |

### light

| Объект / основной класс | Механика и ограничения | Категория |
| --- | --- | --- |
| `light` / [`PCli_BlockLight`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/light/src/main/java/powercraft/light/PCli_App.java#L34) | Цветной светильник, размещение на гранях, режимы stable/huge, GUI. | 1 |
| `lightningConductor` / [`PCli_BlockLightningConductor`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/light/src/main/java/powercraft/light/PCli_App.java#L36) | Молниеотвод с TileEntity и обновлением/сохранением состояния. | 1 |
| `laser` / [`PCli_BlockLaser`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/light/src/main/java/powercraft/light/PCli_App.java#L38) | Лазер, управление redstone, линза/состав, попадание в блоки и сущности. | 1 |
| `mirror` / [`PCli_BlockMirror`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/light/src/main/java/powercraft/light/PCli_App.java#L40) | Зеркало с цветом и направлением, отражение луча. | 1 |
| `prism` / [`PCli_BlockPrism`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/light/src/main/java/powercraft/light/PCli_App.java#L42) | Призма: выбранные грани, разделение/изменение луча. | 1 |
| `laserSensor` / [`PCli_BlockLaserSensor`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/light/src/main/java/powercraft/light/PCli_App.java#L44) | Датчик луча и redstone-выход. | 1 |
| `laserComposition` / [`PCli_ItemLaserComposition`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/light/src/main/java/powercraft/light/PCli_App.java#L46) | Предмет состава/настройки лазерного луча. | 1 |

### logic

| Объект / основной класс | Механика и ограничения | Категория |
| --- | --- | --- |
| `pulsar` / [`PClo_BlockPulsar`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/logic/src/main/java/powercraft/logic/PClo_App.java#L41) | Генератор импульсов с таймерами и настройками. | 2 |
| `gate` / [`PClo_BlockGate`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/logic/src/main/java/powercraft/logic/PClo_App.java#L43) | Семь вентилей NOT/AND/NAND/OR/NOR/XOR/XNOR и выбор входных сторон. | 2 |
| `flipFlop` / [`PClo_BlockFlipFlop`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/logic/src/main/java/powercraft/logic/PClo_App.java#L45) | Триггеры D/RS/T и случайный выход; сохраняемое состояние. | 2 |
| `delayer` / [`PClo_BlockDelayer`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/logic/src/main/java/powercraft/logic/PClo_App.java#L47) | FIFO-буфер и HOLD/медленный повторитель, задержки сигнала. | 2 |
| `special` / [`PClo_BlockSpecial`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/logic/src/main/java/powercraft/logic/PClo_App.java#L49) | Датчики дня/ночи/дождя, пустого/полного сундука и special-конфигурация. | 2 |
| `repeater` / [`PClo_BlockRepeater`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/logic/src/main/java/powercraft/logic/PClo_App.java#L51) | Пересечение, splitter, прямой/угловой и instant-повторители. | 2 |
| `wire` / [`PClo_BlockWire`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/logic/src/main/java/powercraft/logic/PClo_App.java#L53) | Провод redstone; порядок соседних обновлений требует отдельной проверки. | 2 |
| `pulsarCore` / [`PClo_ItemPulsarCore`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/logic/src/main/java/powercraft/logic/PClo_App.java#L55) | Компонент рецептов pulsar. | 2 |

### machines

| Объект / основной класс | Механика и ограничения | Категория |
| --- | --- | --- |
| `automaticWorkbench` / [`PCma_BlockAutomaticWorkbench`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/machines/src/main/java/powercraft/machines/PCma_App.java#L45) | Автокрафт: сетка рецепта, входной запас и выходной инвентарь. | 1 |
| `roaster` / [`PCma_BlockRoaster`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/machines/src/main/java/powercraft/machines/PCma_App.java#L47) | Обжиг предметов/сущностей и netherrack-режим; часть redstone-проверок закомментирована. | 1 |
| `replacer` / [`PCma_BlockReplacer`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/machines/src/main/java/powercraft/machines/PCma_App.java#L49) | Замена/разрушение/установка блока с offset и GUI; часть проверок разрешений в TODO. | 1 |
| `transmutabox` / [`PCma_BlockTransmutabox`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/machines/src/main/java/powercraft/machines/PCma_App.java#L51) | Преобразование предметов с progress/энергией самого устройства; не общая энергетическая система. | 1 |
| `xpBank` / [`PCma_BlockXPBank`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/machines/src/main/java/powercraft/machines/PCma_App.java#L53) | Хранение и выдача опыта игроку/уровней. | 1 |
| `blockBuilder` / [`PCma_BlockBlockBuilder`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/machines/src/main/java/powercraft/machines/PCma_App.java#L55) | Строитель: использование предметов/блоков через направления и луч. | 1 |
| `harvester` / [`PCma_BlockHarvester`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/machines/src/main/java/powercraft/machines/PCma_App.java#L57) | Сбор урожая/деревьев и добыча; серверные права на изменение мира. | 1 |
| `fishingMachine` / [`PCma_BlockFishingMachine`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/machines/src/main/java/powercraft/machines/PCma_App.java#L59) | 3D-структура автоматической рыбалки, состояние running и выдача предметов. | 1 |
| `chunkLoader` / [`PCma_BlockChunkLoader`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/machines/src/main/java/powercraft/machines/PCma_App.java#L61) | 3D-структура загрузчика чанков; нужны ограниченные tickets и выгрузка. | 1 |

### net

| Объект / основной класс | Механика и ограничения | Категория |
| --- | --- | --- |
| `sensor` / [`PCnt_BlockSensor`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/net/src/main/java/powercraft/net/PCnt_App.java#L28) | Датчики по группам сущностей, радиус, GUI и redstone-выход. | 1 |
| `radio` / [`PCnt_BlockRadio`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/net/src/main/java/powercraft/net/PCnt_App.java#L30) | Передатчик/приёмник с каналом, активностью и настройками отображения. | 1 |
| `portableTx` / [`PCnt_ItemRadioRemote`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/net/src/main/java/powercraft/net/PCnt_App.java#L32) | Ручной радиопередатчик выбранного канала. | 1 |
| `radioManager` / [`PCnt_RadioManager`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/net/src/main/java/powercraft/net/PCnt_App.java#L34) | Служебное сохраняемое состояние каналов/передатчиков; не игровой блок. | 1 |

### storage

| Объект / основной класс | Механика и ограничения | Категория |
| --- | --- | --- |
| `bigChest` / [`PCs_BlockBigChest`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/storage/src/main/java/powercraft/storage/PCs_App.java#L22) | Большой многоблочный сундук: общий инвентарь и мастер-части. | 2 |
| `compressor` / [`PCs_ItemCompressor`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/storage/src/main/java/powercraft/storage/PCs_App.java#L24) | Обычный и ender-access компрессор/доступ к содержимому; требуется исправление сохранения и атомарности. | 2 |

### teleport

| Объект / основной класс | Механика и ограничения | Категория |
| --- | --- | --- |
| `teleporter` / [`PCtp_BlockTeleporter`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/teleport/src/main/java/powercraft/teleport/PCtp_App.java#L27) | Телепортер, имена/точки назначения, измерения, GUI, менеджер WorldData. | 1 |
| `teleporterCore` / [`PCtp_ItemTeleporterCore`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/teleport/src/main/java/powercraft/teleport/PCtp_App.java#L29) | Компонент телепортера/рецептов. | 1 |

### transport

| Объект / основной класс | Механика и ограничения | Категория |
| --- | --- | --- |
| `conveyorBelt` / [`PCtr_BlockBeltNormal`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/transport/src/main/java/powercraft/transport/PCtr_App.java#L35) | Обычная лента: движение, упаковка, соседние инвентари, особые действия предметов и despawn-политика. | 1 |
| `speedyBelt` / [`PCtr_BlockBeltSpeedy`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/transport/src/main/java/powercraft/transport/PCtr_App.java#L37) | Быстрая лента: удвоенные boost/max-speed и общие правила края/упаковки. | 1 |
| `detectionBelt` / [`PCtr_BlockBeltDetector`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/transport/src/main/java/powercraft/transport/PCtr_App.java#L39) | Детекторная лента: движение, обнаружение сущностей, выход redstone 15 и повторный scheduled tick. | 1 |
| `breakBelt` / [`PCtr_BlockBeltBreak`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/transport/src/main/java/powercraft/transport/PCtr_App.java#L41) | Тормозная лента: 0,6 параметров обычной; redstone отключает тягу и гасит горизонтальную скорость. | 1 |
| `redirectionBelt` / [`PCtr_BlockBeltRedirector`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/transport/src/main/java/powercraft/transport/PCtr_App.java#L43) | Перенаправление по redstone/соседним транспортерам; TileEntity хранит выбранное направление для сущности. | 1 |
| `separationBelt` / [`PCtr_BlockBeltSeparator`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/transport/src/main/java/powercraft/transport/PCtr_App.java#L45) | Фильтры предметов, меню и сохранение; выбор направления по фильтру. | 1 |
| `ejectionBelt` / [`PCtr_BlockBeltEjector`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/transport/src/main/java/powercraft/transport/PCtr_App.java#L47) | Извлечение из инвентарей: предметы/стеки/всё, first/last/random, настройки и redstone. | 1 |
| `elevator` / [`PCtr_BlockElevator`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/transport/src/main/java/powercraft/transport/PCtr_App.java#L49) | Вертикальный транспорт в двух metadata-вариантах вверх/вниз. | 1 |
| `splitter` / [`PCtr_BlockSplitter`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/transport/src/main/java/powercraft/transport/PCtr_App.java#L51) | Объёмный разветвитель с фильтрами/выбором направлений и инвентарём настроек. | 1 |
| `slimeboots` / [`PCtr_ItemArmorStickyBoots`](https://github.com/PowerCraft/2.0/blob/795369c5a03c7cf39307398a432a5d644698e1df/modules/transport/src/main/java/powercraft/transport/PCtr_App.java#L53) | Липкие ботинки: исключение игрока из движения ленты. | 1 |

### Mobile: регистрация через структуру, а не PC_FieldObject

`PCmo_App`, `PCmo_MinerManager`, `PCmo_EntityMiner`, `PCmo_Command`, `PCmo_IMinerBrain`: два 3D-рецепта, программируемый шахтёр, команды перемещения/поворота/подъёма, копания, моста, работы с инвентарём/топливом, контейнер и клиентские/серверные пакеты. Категория 2 по README. Отсутствие PC_FieldObject у Mobile не означает отсутствие реализации. В новом моде пока не начато.

## Интерфейсы, сохранение и API

Игровые меню регистрируются в App-классах: CraftingTool; SeperationBelt и Splitter (исходное написание); Special; AutomaticWorkbench, BlockBuilder, Replacer, Roaster, Transmutabox; Miner; Radio; Teleporter. Дополнительно есть окна настройки Light/Net/Logic и API-виджеты. Полный список 78 классов находится в `interfaces` машинного реестра; они не считаются 78 игровыми экранами.

API содержит аннотации регистрации/конфигурации, PC_Block/PC_Item, PC_TileEntity, PC_WorldData, inventory helpers, 3D recipes, crafting/ore/fuel registries, GRES, пакеты и клиентский рендер. Загрузчик powercraft/ обнаруживает/устанавливает модули и регистрирует Forge hooks. В Horizon используем штатные реестры, block states, block entities/меню и серверную логику; Java framework и собственный загрузчик модулей не переносятся.

Направления/metadata, аннотированные NBT-поля TileEntity и WorldSavedData исходника — описание поведения. Совместимость Forge-сохранений не реализована. Новые инвентари/таймеры потребуют отдельной версии схем и мультиплеерных тестов.

## Подтверждённые ограничения оригинального исходника

- `PCtr_App`: legacy-рецепт separationBelt перечисляет ключ R, отсутствующий в шаблоне; его нельзя переносить механически. В этом выпуске выбраны medium-рецепты существующих трёх лент.
- `PCtr_BeltHelper.packItems`: сравнение ItemStack.isItemEqual не охватывает все теги; перенос должен использовать полное сравнение компонентов и лимиты стека, а не повторять потенциальное объединение разных NBT.
- `PCtr_TileEntityRedirectionBeltBase`: кэш направлений привязан к entity ID; в Horizon нельзя сохранять runtime ID после выгрузки/смены мира.
- `PCma_BlockReplacer`: проверки флагов NO_HARVEST/NO_BUILD частично закомментированы с TODO. Требуется проверка разрешений перед изменением мира.
- `PCma_BlockRoaster`: часть redstone-проверок находится в закомментированных TODO; нельзя считать все режимы подтверждёнными.
- `PCmo_EntityMiner`: есть TODO клиентского завершения движения и состояния программы; исходный Mobile сам автор считает проблемным.
- Core formation/annihilation/crafting/sensor cores имеют минимальные классы предметов и являются компонентами; это не готовые автономные машины с такими именами.

Указанные пункты проверены статически. Прочие TODO могут быть косметическими или обычным fallback, поэтому наличие TODO/return null само по себе не превращает весь класс в неработающий.

## Правила переноса ресурсов

См. `LEGAL.md`: оригинальный Java используется для изучения, тексты методов и графика не распространяются в новом моде. Реестр содержит имена, факты регистрации, данные рецептов и ссылки. Новые текстуры генерируются нами. Категории исходника и статусы переноса — разные сведения; текущие статусы приведены в `PORTING_PROGRESS.md`.
