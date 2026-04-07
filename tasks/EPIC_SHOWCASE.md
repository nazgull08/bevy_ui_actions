# Epic: Character Sheet Showcase

**Цель:** Комплексный пример `character_sheet.rs`, демонстрирующий все виджеты bevy_ui_actions в одном связном RPG-интерфейсе. Для WebGL сайта библиотеки.

**Расположение:** `examples/character_sheet.rs` в bevy_ui_actions
**Feature:** `viewport3d` (для 3D превью)

## Концепт

RPG character sheet: левая панель с 3D превью персонажа + правая панель с табами (Stats / Equipment / Lore). Экипировка предметов отражается на 3D модели. Все 14 виджетов библиотеки задействованы в одном интерфейсе.

### Layout

```
┌──────────────────────────────────────────────────────────┐
│              [Stats]  [Equipment]  [Lore]    ← Tabs      │
├───────────────────┬──────────────────────────────────────┤
│                   │                                      │
│   Viewport3d      │  Tab content:                        │
│   (drag-rotate)   │                                      │
│                   │  Stats: bars + attributes             │
│   ┌─────────┐    │  Equipment: slots + inventory list    │
│   │ 3D model│    │  Lore: dialogue + hypertext           │
│   │ реагирует│    │                                      │
│   │на экипир.│    │                                      │
│   └─────────┘    │                                      │
│                   │                                      │
│  HP ████░░ 75/100 │                                      │
│  MP ███░░░ 40/80  │                                      │
│                   │                                      │
├───────────────────┘                                      │
│  [tooltip / status bar]                                  │
└──────────────────────────────────────────────────────────┘
```

### Покрытие виджетов

| Виджет | Где используется |
|--------|-----------------|
| UiTheme + TextRole | Единый стиль везде |
| NodeExt (row/column/centered) | Layout всех секций |
| Panel | Фон секций |
| Tabs | Stats / Equipment / Lore навигация |
| Viewport3d | 3D превью персонажа (drag-rotate) |
| ProgressBar | HP, MP, Stamina, XP |
| Drag & Drop | Инвентарь → Equipment слоты |
| ScrollView | Список предметов в инвентаре |
| ListView | Выбор предмета с selection |
| Rich Tooltip | Stat comparison при наведении на предмет |
| Modal | Подтверждение замены / выбрасывания предмета |
| DialogueBox | Лор NPC |
| HyperText | Кликабельные ссылки между темами |
| TopicRegistry | Накопление знаний |
| InteractiveVisual | Hover/press/selected feedback |
| OnClick / OnRightClick | Действия с предметами |
| Selected | Выделение атрибута / предмета |

## Задачи

### Task 01 — Scaffold + Viewport3d манекен
Базовый layout: left panel (Viewport3d) + right panel (заглушка). 3D манекен из примитивов (capsule body, sphere head, cylinder limbs). Drag-to-rotate. Мини-бары HP/MP под viewport.

**Покрытие:** Viewport3d, ProgressBar, NodeExt, Panel, UiTheme

### Task 02 — Tabs + Stats tab
TabGroup с тремя вкладками (Stats / Equipment / Lore). Stats tab: progress bars (HP/MP/Stamina/XP), attribute grid (STR/DEX/INT/VIT) с Selected highlight, кнопки +/- level-up.

**Покрытие:** Tabs, ProgressBar, Selected, InteractiveVisual, OnClick

### Task 03 — Equipment tab: slots + inventory
6 equipment slots (Head, Chest, MainHand, OffHand, Legs, Ring) как DropTarget. ScrollView с инвентарём предметов (15-20 items). Drag предмет из списка → drop в слот = экипировка. Swap при занятом слоте.

**Покрытие:** Drag & Drop, ScrollView, ListView, DropTarget, Draggable

### Task 04 — Viewport3d реагирует на экипировку
При экипировке: меч = прямоугольник в правой руке, щит = диск в левой, шлем = конус на голове, броня = изменение цвета торса. Снятие = убираем визуал.

**Покрытие:** Viewport3d (dynamic children), sync system

### Task 05 — Rich Tooltips + Modal
При наведении на предмет в инвентаре — rich tooltip со stat comparison (vs текущая экипировка). При right-click на экипированный предмет — modal "Unequip / Drop?".

**Покрытие:** Rich Tooltip (StatDiff), Modal, OnRightClick

### Task 06 — Lore tab
DialogueBox с NPC-рассказчиком. HyperText ссылки между записями лора (оружие, монстры, мир). TopicRegistry с 8-10 записями. Тематика — RPG фэнтези, связанная с предметами из инвентаря.

**Покрытие:** DialogueBox, HyperText, TopicRegistry, TopicDiscovered

### Task 07 — Visual polish
Единый тёмный RPG-стиль (тёмно-серый + золотые акценты). InteractiveVisual на всех интерактивных элементах. Полировка spacing, цветов, border. Финальная проверка всех взаимодействий.

**Покрытие:** UiTheme тюнинг, InteractiveVisual, Panel presets, BorderStyle

### Task 08 — GLB модели (опционально)
Замена примитивного манекена на GLB модель. Реальная экипировка оружия на attachment points. Asset pipeline для примера.

**Покрытие:** Viewport3d + GLB, реальная 3D экипировка

## Принципы

- **Один .rs файл** — пример должен быть self-contained
- **Нет внешних ассетов** (до Task 08) — всё из примитивов Bevy
- **Data-driven** — предметы, статы, лор в const/static структурах внутри файла
- **Инкрементально** — каждый таск компилируется и работает самостоятельно
- **WebGL-совместимо** — без compute, без heavy textures

## Зависимости

- bevy_ui_actions v0.2.0 с feature `viewport3d`
- Bevy 0.16.1
