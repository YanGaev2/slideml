# SlideML: AI-native PowerPoint. Проектный документ

Статус: проект, версия 0.1. Дата: 2026-10-15.

Документ описывает Phase 1: библиотеку, CLI и MCP-сервер. Визуальный редактор описан только как будущая цель.

Термины документа:

- **Исходник** — файл `.sml` на языке SlideML.
- **Компилятор** — программа `sml`, которая строит `.pptx` из исходника.
- **Агент** — LLM, которая пишет и правит исходник через MCP или CLI.
- **Оракул** — инструмент `oracle/`. Он управляет PowerPoint через COM. PowerPoint — эталон.
- **Разрешённая модель** (resolved model) — результат компиляции до записи OOXML: абсолютные рамки, строки текста, итоговые размеры шрифта.
- **Тема-кит** (theme kit) — тема, мастер, макеты, токены и стили таблиц в одном пакете.
- **Токен** — имя из темы-кита: цвет, размер текста, отступ, эффект.
- **[verify]** — утверждение не проверено. Этап, который первым его касается, проверяет его через оракул.

---

## 1. Цели и не-цели

### 1.1 Цели

1. Агент создаёт и правит презентации. Презентации выглядят профессионально без ручной доработки.
2. Исходный формат понятен агенту. Агент правит элемент по `id`, а не переписывает файл.
3. Каждый объект в `.pptx` — родной объект PowerPoint. Пользователь может править фигуры, текст, таблицы, картинки, анимации и переходы.
4. PowerPoint открывает каждый файл без диалога восстановления.
5. Текст в PowerPoint для Windows переносится и помещается так же, как предсказал компилятор, для ≥ 99% блоков (порог M1). Точного совпадения для всех блоков мы не обещаем. Алгоритм PowerPoint закрыт. Запас подгонки поглощает пограничные случаи. PowerPoint для Mac и веб-версия могут переносить иначе (вопрос 14.6).
6. Компилятор сам считает геометрию. Он не полагается на пересчёт в PowerPoint при открытии.
7. Компилятор детерминирован: одинаковый вход даёт побайтно одинаковый `.pptx`.
8. Агент получает структурную обратную связь после каждой правки: ошибки, линт, измерения.

### 1.2 Не-цели Phase 1

1. Браузер, HTML, CSS или Playwright в конвейере экспорта. Никогда.
2. Растеризация как скрытый запасной вариант. Никогда. Неподдерживаемая конструкция — ошибка компиляции.
3. LibreOffice как эталон рендера.
4. Импорт произвольного чужого `.pptx` (P2).
5. Собственный растеризатор превью (P1). В Phase 1 превью делает PowerPoint.
6. Текст CJK и RTL (P2). Компилятор отклоняет такой текст с ошибкой `text/script-unsupported`.
7. Визуальный редактор и сборка WASM (P2).
8. Вызовы LLM внутри компилятора и MCP-сервера. LLM вызывает только оценочный стенд `evals/`.

---

## 2. Анализ конкурентов: почему их экспорт плох

### 2.1 Сводная таблица

| Инструмент | Как делает PPTX | Что теряется | Источник |
|---|---|---|---|
| Marp | Картинка слайда; режим `--pptx-editable` идёт через PDF и LibreOffice | Текст не правится, нет анимаций, темы ломаются, заметки теряются | [marp-cli](https://github.com/marp-team/marp-cli), [PR #626](https://github.com/marp-team/marp-cli/pull/626), [marp-vscode #139](https://github.com/marp-team/marp-vscode/issues/139) |
| Slidev | Картинка слайда; режим `pptx-editable` пересобирает фигуры | SVG, KaTeX, градиенты CSS становятся картинками; другие переносы строк; нет анимаций | [sli.dev/guide/exporting](https://sli.dev/guide/exporting) |
| pandoc / Quarto | Заполняет плейсхолдеры макетов шаблона | Нет свободной вёрстки, стилей, анимаций; ошибки с колонками и картинками | [quarto pptx](https://quarto.org/docs/presentations/powerpoint.html), [pandoc #5258](https://github.com/jgm/pandoc/issues/5258), [#4586](https://github.com/jgm/pandoc/issues/4586) |
| Typst (touying, typ2pptx) | PNG на слайд или обратный разбор SVG | Только картинки или эвристики; таблицы не родные | [touying-exporter](https://github.com/touying-typ/touying-exporter), [typ2pptx](https://github.com/touying-typ/typ2pptx) |
| Gamma | Веб-карточки, конвертация при экспорте | Сложные макеты — одна картинка; шрифты падают на Calibri; анимации теряются | [slidegmm](https://www.slidegmm.ai/en/blog/gamma-export-powerpoint-quality-guide) (данные конкурента) |
| Beautiful.ai | Smart Slides, экспорт «Editable» или «Images» | Нет анимаций и переходов; текст делится на блоки; графики — картинки | [support.beautiful.ai](https://support.beautiful.ai/hc/en-us/articles/30629528652685) |
| Pitch | Прямой экспорт | Нет анимаций и переходов; шрифты падают на Calibri | [help.pitch.com](https://help.pitch.com/en/articles/6713988-export-a-presentation-to-power-point) |
| Presenton | HTML-шаблоны, геометрия из браузера, запись через python-pptx | Метрики текста браузера; сложные макеты расходятся | [presenton](https://github.com/presenton/presenton) |
| html2pptx, dom-to-pptx | Playwright считывает рамки и вызывает pptxgenjs | Текст вне `<p>` теряется; градиенты и иконки растеризуются | [html2pptx.md](https://github.com/tfriedel/claude-office-skills/blob/main/public/pptx/html2pptx.md), [dom-to-pptx](https://github.com/atharva9167j/dom-to-pptx) |
| python-pptx | Прямая запись OOXML | Нет API анимаций; `normAutofit` без масштаба даёт переполнение | [#1106](https://github.com/scanny/python-pptx/issues/1106), [#1021](https://github.com/scanny/python-pptx/issues/1021) |
| pptxgenjs | Прямая запись OOXML, мастера | Нет анимаций; нет градиентов; `fit:'shrink'` не работает; дубли `cNvPr/@id` | [#102](https://github.com/gitbrent/PptxGenJS/issues/102), [#312](https://github.com/gitbrent/PptxGenJS/issues/312), [#1532](https://github.com/gitbrent/PptxGenJS/issues/1532) |
| Google Slides API | 52 типа запросов `batchUpdate` | Ни одного запроса для анимаций и переходов | [API reference](https://developers.google.com/workspace/slides/api/reference/rest/v1/presentations/request) |
| Aspose.Slides | Полная объектная модель OOXML | Работает, но это низкоуровневый закрытый API без дизайна | [docs.aspose.com](https://docs.aspose.com/slides/net/shape-animation/) |
| @office-kit/pptx | TSX DSL в родную модель OOXML, 95 пресетов анимации, Morph | До версии 1.0; нет сложных деревьев анимации; превью не точное | [office-kit/pptx](https://github.com/office-kit/pptx) |
| PPTAgent (исследование) | Правка шаблонных слайдов через узкий API | Нужны LibreOffice и Chromium | [arXiv 2501.03936](https://arxiv.org/abs/2501.03936) |

Числа для Gamma и Beautiful.ai публикуют конкуренты. Считайте их ориентировочными.

### 2.2 Общие причины плохого экспорта

1. **Растеризация.** Инструмент кладёт на слайд картинку слайда. Править нечего.
2. **Промежуточный формат после рендера.** Путь через PDF или SVG теряет абзацы, списки и таблицы. Конвертер угадывает их обратно.
3. **Браузер решает геометрию.** PowerPoint переносит текст по своим метрикам. Текст переполняет рамку.
4. **CSS без аналога в DrawingML.** Фильтры, `clip-path`, `::before`, flex и сложная анимация становятся картинкой или пропадают.
5. **Автоподбор.** PowerPoint не пересчитывает `normAutofit` при открытии. Голый `<a:normAutofit/>` означает масштаб 100%. Генератор без своего измерения текста выпускает переполнение.
6. **Шрифты.** Веб-шрифты не встроены. Замена на Calibri меняет переносы.
7. **Нет мастеров и плейсхолдеров.** Файл открывается, но Reset Slide, смена темы и Slide Master не работают.
8. **Нет модели анимации.** Популярные библиотеки не пишут `<p:timing>`.
9. **Графики как картинки.**
10. **Ошибки записи XML.** Неверный порядок элементов, дубли id, битые связи вызывают диалог восстановления.
11. **Неверный эталон.** LibreOffice пересчитывает автоподбор и скрывает переполнение, которое PowerPoint покажет.

### 2.3 Выводы для SlideML

- Словарь исходника отображается 1:1 в PresentationML и DrawingML. Конструкция без отображения — ошибка.
- Компилятор сам меряет текст по файлам шрифтов и калибруется по PowerPoint.
- Темы, мастера, макеты и плейсхолдеры — объекты первого класса.
- Анимации строятся только из шаблонов, которые записал сам PowerPoint.
- Эталон — PowerPoint через COM.

---

## 3. Ключевые решения

### 3.1 Оценка трёх предложений

| Критерий | fidelity-first | agent-first | mvp-risk-first |
|---|---|---|---|
| Точность экспорта | 9 | 8 | 8 |
| Эргономика агента и качество результата | 7 | 9 | 7 |
| Простота и реализуемость | 5 | 6 | 9 |
| Работа с рисками | 7 | 7 | 9 |
| **Итого** | **28** | **30** | **33** |

Основа документа — **mvp-risk-first**. Причина: оно сначала проверяет рискованные ставки (текст, анимации, шрифты, диалог восстановления) через оракул и только потом строит язык. Это снижает риск потратить месяцы на язык, который не даёт точного экспорта.

Из **agent-first** взяты:

- диагностика с готовыми операциями исправления (`fixes[].ops`);
- ошибки `css/*` с переводом CSS в словарь SlideML;
- области сетки `at="c1-6 r2-7"`;
- таблица правил линтера с кодами и порогами;
- инструмент `catalog` и семантические роли `role="decor"`;
- оверлей с id и зонами переполнения в превью.

Из **fidelity-first** взяты:

- имена Morph `!!key`, разделы (`p14:sectionLst`);
- детерминированные GUID из хеша `id`;
- семантическое сравнение после пересохранения как отдельный сигнал потери данных.

### 3.2 Язык ядра: Rust

Решение: продукт (парсер, модель, вёрстка, текстовый движок, запись OOXML, CLI, MCP) пишем на Rust. Оракул пишем на C# .NET 8 без AOT. Оракул — тестовый инструмент. Он не входит в продукт.

Обоснование:

1. **MCP.** `rmcp` — официальный SDK уровня Tier 1. У C++ нет официального SDK.
2. **Диагностика для агента.** `miette` даёт позиции в исходнике, метки и подсказки. В C# и C++ равного нет.
3. **Закрытые типы.** Каждый узел IR — `enum` или `struct`. `match` обязан покрыть все случаи. Новый вариант OOXML показывает все места, которые нужно обновить.
4. **Один статический бинарник.** Позже ядро собирается в WASM для веб-редактора. Путь WASM в C# слабый (SkiaSharp и HarfBuzzSharp под NativeAOT-LLVM).
5. **Точность текста не зависит от языка.** Она зависит от файлов шрифтов, шейпера уровня HarfBuzz и калибровки.

Отвергнуто:

- **C#.** Хороший второй вариант. Он выигрывает, только если продукт навсегда останется на Windows без WASM. Его плюсы (COM, OpenXmlValidator) мы используем в оракуле.
- **C++.** Нет официального MCP SDK, тяжёлая сборка Skia на Windows, нет инструментов диагностики, риск ошибок памяти в парсере недоверенного ввода.

Набор крейтов (точные версии в `Cargo.lock`):

| Задача | Крейт | Примечание |
|---|---|---|
| Шейпинг | `harfrust` | Не `rustybuzz` (RUSTSEC-2026-0206) |
| Разбор шрифтов, метрики | `skrifa` / `read-fonts` | Не `ttf-parser` (RUSTSEC-2026-0192) |
| Точки переноса UAX #14 | `icu_segmenter` | Плюс наши правила Office |
| Разбор исходника | `roxmltree` | Позиции узлов для диагностики |
| Правка исходника | собственный CST на `quick-xml` | Сохраняет комментарии и форматирование |
| Запись XML | собственные эмиттеры на `quick-xml` | Порядок детей по схеме |
| Пакет | `zip` | Фиксированные метки времени |
| Картинки | `imagesize`, `image` | Размеры и проверка PNG/JPEG |
| Диагностика | `miette` | |
| MCP | `rmcp` 3.x, точная версия | Спецификация 2026-07-28, stateless |
| Схемы инструментов | `serde` + `schemars` | Один источник для схем MCP |

Этап M0 проверяет, что эти крейты собираются вместе **[verify]**.

### 3.3 Синтаксис исходника: строгий XML с HTML-текстом

Решение: SlideML — строгий XML. Inline-текст и таблицы используют теги HTML. Элементы вёрстки и фигур — свои. Каждый из них отображается 1:1 в OOXML. CSS не принимается.

Обоснование:

1. XML поддерживает смешанный текст без массивов run.
2. Каждый узел может иметь `id`. Правки адресуют `id`, а не строки файла.
3. Ошибка получает `line:col` и список допустимых значений.
4. Теги `<b> <i> <a> <ul> <li> <table> <td>` дают модели сильные априорные знания. Они точно отображаются в `a:r`, `a:p`, `a:tbl`.
5. PPTAgent показал: HTML-подобный вид даёт около +20 пунктов успеха против сырого OOXML.

Отвергнуто:

- **HTML+CSS.** Модель пишет полный CSS: фильтры, `overflow:hidden`, flex. Подмножество означает расхождение превью и PowerPoint или постоянные отказы. AeSlides показал, что агенты прячут дефекты через CSS.
- **YAML/JSON.** Смешанный текст становится массивами run. Ошибки отступа YAML молча меняют структуру. JSON используем только для операций патча.
- **Markdown.** Не выражает геометрию, id, анимации. Возможен как импорт черновика (P1).
- **Свой DSL.** Нет априорных знаний у модели.
- **TSX/код.** Презентация становится программой. Её нельзя проверить статически и править по id без запуска.

Отличие от fidelity-first: в Phase 1 язык имеет **один** словарь. Отдельный «core vocabulary» не нужен. Агент читает разрешённую модель через `view="resolved"` в JSON. Второй словарь появится вместе с импортом (P2), если он будет нужен.

Риск: прямого сравнения генерации XML, HTML и YAML для слайдов в литературе нет. Этап M5 проверяет выбор A/B-тестом.

### 3.4 IR: три представления

1. **CST** — дерево исходника без потерь: комментарии, порядок атрибутов, позиции. Патч применяется к CST. Файл пишется из CST.
2. **Модель исходника** — типизированные, проверенные узлы с токенами. Геометрии нет.
3. **Разрешённая модель** — абсолютные рамки в EMU, `cNvPr id`, строки текста, итоговые `sz`, высоты строк таблиц. Её читают запись OOXML, линтер, вид `resolved` и будущий растеризатор. Поэтому они не могут разойтись.

Правила:

- Цвета хранятся как ссылки на тему. Hex вычисляется только для проверки контраста.
- Единицы внутри IR — целые числа в кодировке OOXML: `Emu(i64)`, `Angle60k(i32)`, `Pct1000(i32)`, `HundredthPt(i32)`.
- `id` исходника записывается в `p:cNvPr/@name`. Исключение — элемент с `morph-key`: его имя равно `!!<morph-key>`, потому что Morph сопоставляет фигуры по имени. Числовой `cNvPr id` назначается на каждом слайде по порядку z, начиная с 2 (`id="1"` занимает корень `p:spTree`).
- Манифест хранит пару «`cNvPr id` слайда → `id` исходника» для каждой фигуры. Оракул и будущий повторный импорт сопоставляют фигуры через манифест, а не только через `name`.
- `p:timing` в Phase 1 — заполненные шаблоны XML (раздел 3.7). Типизированное дерево `p:timing` и узел `Opaque` для неизвестного XML появятся вместе с импортом (P2). Сейчас у них нет потребителя.

### 3.5 Текст: компилятор меряет и пишет явный `sz`

Решение:

1. Компилятор меряет каждый текстовый блок по файлу шрифта из реестра.
2. Компилятор пишет явный `sz` на каждом run и в каждом `endParaRPr`.
3. Для политики `shrink` компилятор пишет голый `<a:normAutofit/>`. PowerPoint открывает файл с нашими размерами. После правки пользователем PowerPoint сам ведёт автоподбор.
4. Для политик `error` и `none` компилятор пишет `<a:noAutofit/>`. Для `grow` пишет вычисленную высоту и `<a:spAutoFit/>`.
5. Политика по умолчанию для текста агента — `error`. Переполнение возвращается агенту как диагностика с исправлениями. Заголовки по умолчанию — `shrink` с `min-size`.

Отвергнуто:

- **Вычисленный `fontScale` как основной механизм.** Округление `fontScale` — вторая переменная. Поведение Google Slides и Keynote с ним не проверено. Явный `sz` означает: файл говорит то, что показывает.
- **Пересчёт через COM (`TextFrame2.AutoSize`) в основном пути.** Только Windows, не детерминирован, прячет ошибки движка. Используем только для калибровки.

Компромисс: явный `sz` на плейсхолдерах мешает «Reset Slide» вернуть размер из макета. Плейсхолдер сохраняет тип и позицию. В P1 добавим `size-inheritance="native"` для текста, который помещается в размер макета.

### 3.6 Превью: PowerPoint в Phase 1, свой растеризатор в P1

Решение:

- В Phase 1 агент получает два вида обратной связи. Первый — геометрический линт и вид `resolved` за миллисекунды. Второй — PNG от PowerPoint через оракул по запросу (около 1 с на слайд при тёплом PowerPoint).
- PNG кешируется по хешу разрешённой модели слайда.
- Свой растеризатор (`tiny-skia`, `resvg`) входит в P1. Он становится превью по умолчанию только после прохода порогов SSIM против PowerPoint (раздел 8).

Отвергнуто:

- **Свой растеризатор в P0** (fidelity-first, agent-first). Работа большая, а эталон доступен на этой машине. Неточное превью учит агента не тому.
- **Скриншот браузера.** Это ровно тот провал, о котором говорит пользователь.

### 3.7 Анимации: только шаблоны от PowerPoint

Решение:

1. Оракул вызывает `MainSequence.AddEffect` для каждого эффекта, направления, триггера и режима по абзацам.
2. Оракул сохраняет файл и извлекает канонические поддеревья `p:par` и `p:bldP`.
3. Скрипт превращает их в шаблоны с параметрами `{spid}`, `{ctnId}`, `{dur}`, `{delay}`, `{pRg}`.
4. Компилятор заполняет только эти шаблоны. Новые деревья поведения он не собирает.
5. Каркас `p:timing` (`tmRoot`, `mainSeq`, группы клика, узлы `clickEffect`/`withEffect`/`afterEffect`, `prevCondLst`/`nextCondLst`, `bldLst`) тоже берётся из файлов PowerPoint. Компилятор собирает только этот каркас: он нумерует `cTn id`, назначает `grpId` и считает задержки `after`. Правила сборки M2 проверяет на комбинированных последовательностях.

Причина: парсер PowerPoint принимает XML, который движок показа молча игнорирует. Неверная метка пресета показывает в Animation Pane «Custom».

Отвергнуто: генерация деревьев из примитивов `p:set`/`p:anim`. Риск тихой потери слишком высок.

Переходы: `fade`, `push`, `wipe` пишутся как ECMA-376. `morph` пишется в `mc:AlternateContent`: Choice с `p159:morph`, Fallback с `p:fade`. Пары Morph задаёт `morph-key`, который становится именем `!!key`.

### 3.8 Шрифты: закрытый реестр

Решение: компилятор принимает только шрифты из реестра `registry/fonts/`. Иначе — ошибка `font/unknown` со списком допустимых шрифтов.

- **Tier A** — шрифты OFL, которые мы встраиваем: Inter, Source Sans 3, IBM Plex Sans, Noto Sans, JetBrains Mono. Встраивание — после спайка M3.
- **Tier B** — системные шрифты Windows и Office для Mac: Calibri, Arial, Georgia, Segoe UI (нет на macOS), Cambria, Verdana, Consolas. Мы меряем их по файлам из `C:\Windows\Fonts` и не встраиваем.

Компилятор всегда пишет `latin`, `ea` и `cs` явно. Кириллица использует слот `latin`.

Правила реестра:

1. Реестр принимает только статические файлы TTF. Вариативный шрифт (таблица `fvar`) — ошибка. Поддержка вариативных шрифтов в PowerPoint ненадёжна.
2. Для каждого семейства реестр хранит начертания regular, bold, italic и bold italic. Если начертания нет, `<b>` или `<i>` для этого семейства — ошибка `font/missing-face`. Причина: PowerPoint синтезирует ложное начертание, и его ширины мы не меряем.
3. Компилятор проверяет, что шрифт содержит каждый глиф текста. Если глифа нет, — ошибка `font/missing-glyph`. Причина: PowerPoint берёт глиф из другого шрифта, и переносы расходятся.
4. Имя в `fonts` — имя семейства из таблицы `name` (nameID 1), например `Inter SemiBold`. Это имя видит PowerPoint **[verify M3]**.

---

## 4. Формат исходника SlideML

### 4.1 Общие правила

1. Кодировка UTF-8. DTD и сущности запрещены. Управляющие символы запрещены, кроме табуляции и перевода строки.
2. Корень — `<deck version="0.1">`. Пространство имён необязательно.
3. Любой элемент может иметь `id`. Формат `[a-z][a-z0-9-]*`. `id` уникален в пределах презентации.
4. `sml fmt --assign-ids` дописывает недостающие `id` вида `s2-e3`.
5. Неизвестный элемент или атрибут — ошибка с подсказкой «did you mean».
6. Атрибут `style=` и свойства CSS — ошибка `css/*` с переводом.
7. Пробелы в тексте сворачиваются как в HTML. `<br/>` даёт перенос строки внутри абзаца (`a:br`).
8. Порядок элементов в слайде — порядок z. Атрибут `layer="back|front"` переносит элемент в начало или конец дерева.

### 4.2 Единицы

| Запись | Значение |
|---|---|
| `24`, `24pt` | пункты (число без единицы — пункты) |
| `px` | пиксели при 96 dpi |
| `in`, `cm`, `emu` | дюймы, сантиметры, EMU |
| `%` | доля рамки родителя |
| `fr` | доля свободного места в контейнере |
| `ms`, `s` | время |
| `deg` | градусы |

### 4.3 Цвета, заливки, линии

Цвет:

- Слот темы: `tx1 tx2 bg1 bg2 accent1 … accent6 hlink folHlink`. Компилятор пишет `a:schemeClr`. Цвет остаётся цветом темы в PowerPoint.
- Слоты `dk1 lt1 dk2 lt2` допустимы только в `<colors>`. В других местах — ошибка `token/raw-slot` с подсказкой (`dk1` → `tx1`).
- Литерал: `#3A86FF`. Компилятор пишет `a:srgbClr val="3A86FF"`. Линтер даёт `DS001` (info).
- Модификаторы: `accent1:tint(40%)`, `accent1:shade(20%)`, `tx1:alpha(60%)`, `accent2:lum(80%,20%)`. Они дают `a:tint`, `a:shade`, `a:alpha`, `a:lumMod`/`a:lumOff`.
- Семантические псевдонимы темы-кита: `positive`, `negative`, `muted`. Каждый отображается в слот с модификатором.

Прозрачность задаётся только модификатором `alpha(...)`. Отдельного атрибута `opacity` нет. `alpha(55%)` означает 55% непрозрачности и даёт `<a:alpha val="55000"/>`.

Заливка `fill`:

| Запись | OOXML |
|---|---|
| `fill="accent1"` | `a:solidFill` |
| `fill="none"` | `a:noFill` |
| `fill="linear(90deg, tx1:alpha(85%) 0%, tx1:alpha(0%) 70%)"` | `a:gradFill` + `a:lin ang` |
| `fill="radial(circle, accent1 0%, bg1 100%)"` (P1) | `a:gradFill` + `a:path path="circle"` |

Линия `line`: `line="accent6 1.5pt sysDash"`, `line="none"`. Части: цвет, толщина, штрих (11 значений `prstDash`). Концы стрелок: `head="triangle|stealth|diamond|oval|arrow"`, `tail="..."`.

### 4.4 Размещение

Размещение задаёт рамку элемента. Предпочтительный порядок:

1. Плейсхолдер макета: `<title>`, `<subtitle>`, `<body>`. Рамку даёт макет.
2. Именованная область макета: `area="body"`, `area="slide"`, `area="safe"`.
3. Контейнер: `<stack>`, `<row>`, `<columns>`, `<grid>`.
4. Область сетки темы: `at="c1-6 r2-7"` (столбцы 1–6, строки 2–7 сетки 12×8).
5. Абсолютные координаты: `x y w h`. Линтер проверяет их по сетке (`GEO004`).

Если элемент имеет `area` или `at` и также `w` или `h`, то `w` и `h` заменяют размер области. Элемент прижат к левому верхнему углу области. Пример: `at="c1-2 r7-7" h="6pt"` — полоса 6 pt у верхнего края строки 7.

`deck size="16:9"` даёт `sldSz cx="12192000" cy="6858000"` (13,333 × 7,5 in). `size="4:3"` даёт `cx="9144000" cy="6858000"`.

### 4.5 Элементы

#### Структура презентации

| Элемент | Атрибуты | Назначение |
|---|---|---|
| `deck` | `version`, `size="16:9\|4:3\|<w>x<h>"`, `lang` | Корень |
| `theme` | `id`, `extends="builtin:ink"` | Тема-кит или переопределение встроенного кита |
| `colors` | `dk1 lt1 dk2 lt2 accent1..accent6 hlink folHlink`, псевдонимы | Цветовая схема (12 слотов) |
| `fonts` | `heading`, `body` | Шрифтовая схема, имена из реестра |
| `scale` | `text="display:60 h1:40 h2:28 h3:22 body:18 small:14 caption:11"`, `space="xs:4 s:8 m:16 l:32 xl:48"` | Шкалы токенов |
| `grid` | `cols`, `rows`, `margin`, `gutter` | Сетка темы |
| `effect` | `name`, `value` | Токен эффекта, например `soft` |
| `master` | `id`, `background` | Мастер (свой кит) |
| `layout` | `id`, `name`, `type` (`title`, `obj`, `twoObj`, `titleOnly`, `secHead`, `blank`, ...) | Макет |
| `region` | `name`, `at` или `x y w h` | Именованная область макета |
| `placeholder` | `type` (`title`, `ctrTitle`, `subTitle`, `body`, `pic`, `dt`, `ftr`, `sldNum`), `idx`, размещение, текстовые атрибуты | Плейсхолдер макета |
| `section` | `name` | Раздел; содержит `slide` (P1) |
| `slide` | `id`, `layout`, `transition`, `transition-duration`, `transition-dir`, `hidden`, `background` | Слайд |
| `notes` | — | Заметки докладчика; inline-текст или `<p>` |
| `lint` | `disable="CODE,CODE"` | Отключение правил; видно в `get_outline` |

#### Плейсхолдеры слайда

| Элемент | Плейсхолдер |
|---|---|
| `title` | `title` или `ctrTitle` (по типу макета) |
| `subtitle` | `subTitle` |
| `body` | `body idx=1`; атрибут `slot="2"` выбирает второй плейсхолдер содержимого (макет `two-content`); может содержать текст или контейнеры |

#### Визуальные элементы

| Элемент | Ключевые атрибуты |
|---|---|
| `shape` | `preset` (187 имён `prstGeom`), `adj="adj1:12000"`, `radius` (для `roundRect`), `fill`, `line`, `shadow`, `glow` (P1), `rotate`, `flip="h\|v"`, `pad`, `anchor="top\|middle\|bottom"`, `fit`, `min-size`, `morph-key`, `role`, `link` |
| `image` | `src` (PNG, JPEG; SVG в P1), `sizing="cover\|contain\|stretch"`, `focus="60% 40%"`, `crop="l t r b"`, `radius`, `alt`, `morph-key`, `role` |
| `table` | `style`, `header`, `band="rows\|cols"`, `cols="2fr 1fr"`, `cell-pad`, `size` |
| `tr`, `th`, `td` | `colspan`, `rowspan`, `align`, `valign`, `fill`, `color`, `size`, `b` |
| `group` | `id`; дети в координатах слайда (P1) |
| `connector` | `from="id@right"`, `to="id@left"`, `geom="straight\|bent\|curved"`, `line`, `head`, `tail` (P1) |
| `chart` | `type`, данные в дочерних `series` (P1) |

Атрибут `sizing` относится только к картинкам. Атрибут `fit` относится только к тексту.

#### Контейнеры (без собственной фигуры в PPTX)

| Элемент | Атрибуты | Правило |
|---|---|---|
| `stack` | `dir="v\|h"`, `gap`, `align`, `justify` | Дети по главной оси: `auto` (по содержимому), фиксированный размер или `Nfr`. Ничего не переносится и не сжимается. Нехватка места — `LAY001`. |
| `row` | как `stack` | Краткая запись `stack dir="h"` |
| `columns` | `ratio="3fr 2fr"`, `gap` | Столбцы по долям |
| `grid` | `cols`, `rows`, `gap` | Дети по порядку или по `cell="2/1"` |
| `frame` | `at` или `x y w h` | Абсолютная рамка одного ребёнка |

Контейнер с атрибутом `group="1"` становится `p:grpSp` (P1).

#### Текст

| Элемент | Атрибуты | OOXML |
|---|---|---|
| `p` | `align`, `level`, `bullet`, `numbering`, `space-before`, `space-after`, `line-spacing`, `size`, `color`, `font` | `a:p` + `a:pPr` |
| `ul`, `ol` | `gap`, `size`, `fit`, `min-size`, `marker` | Текстовая фигура; каждый `li` — `a:p` |
| `li` | `level` | `a:p` с маркером темы-кита |
| `b i u s sup sub code` | — | свойства `a:rPr` |
| `span` | `color`, `size`, `font`, `b`, `i`, `highlight` | `a:r` с `a:rPr` |
| `a` | `href` или `slide="id"` | `a:hlinkClick` + связь |
| `br` | — | `a:br` |

Текст напрямую внутри `shape`, `title` и `td` — один абзац.

Текстовые атрибуты рамки: `fit="error|shrink|grow|none"`, `min-size`, `pad` (вставки `lIns` и т. д.), `anchor`, `wrap="square|none"`, `columns` (P1).

#### Анимация

`<timeline>` — упорядоченный список эффектов, как в Animation Pane.

| Элемент | Атрибуты |
|---|---|
| `enter` | `target`, `effect` (`appear`, `fade`, `fly-in`, `wipe`, `zoom`, `float-in`), `from` (`top`, `right`, `bottom`, `left`), `by="whole\|paragraph"`, `start="click\|with\|after"`, `duration`, `delay` |
| `exit` | как `enter`; эффекты `disappear`, `fade`, `fly-out`, `wipe`, `zoom`, `float-out` |
| `emphasis` | P1: `pulse`, `grow-shrink`, `spin`, `transparency` |
| `motion` | P1: `path` |

Значение `effect` берётся из реестра пресетов. Неизвестная комбинация — ошибка `anim/unknown-preset`. Направление `from` допустимо только там, где его даёт PowerPoint. Например, у `zoom` нет `from`, у `float-in` есть только `top` и `bottom`. Точный список даёт `catalog kind="animations"`.

`morph-key` уникален в пределах слайда. Повтор на одном слайде — ошибка `ref/duplicate-morph-key`.

### 4.6 Ошибки CSS с переводом

| Ввод агента | Ответ компилятора |
|---|---|
| `style="background-color:#fff"` | `css/style-attr: CSS is not supported. Use fill="#FFFFFF" or fill="bg1".` |
| `opacity="0.5"` | `css/opacity: Use the color modifier fill="accent1:alpha(50%)".` |
| `overflow="hidden"` | `css/overflow: Text cannot hide overflow. Use fit="shrink" min-size="14" or reduce text.` |
| `class="card"` | `css/class: Classes are not supported. Use theme tokens or a component.` |
| `display="flex"` | `css/display: Use <row> or <stack dir="h">.` |

### 4.7 Полный пример

```xml
<deck version="0.1" size="16:9" lang="ru-RU">
  <theme id="brand" extends="builtin:ink">
    <colors dk1="#14161A" lt1="#FFFFFF" dk2="#1F2A44" lt2="#F3F5F9"
            accent1="#3A86FF" accent2="#FF6B35" accent3="#2EC4B6"/>
    <fonts heading="Inter SemiBold" body="Inter"/>
  </theme>

  <!-- Слайд 1: фото на весь слайд, затемнение, плейсхолдеры, Morph-ключ -->
  <slide id="cover" layout="cover" transition="fade" transition-duration="600ms">
    <image id="hero" src="assets/hero.jpg" area="slide" sizing="cover" focus="60% 40%"
           layer="back" role="decor" alt="Ночной город над рекой"/>
    <shape id="scrim" preset="rect" area="slide" layer="back" role="decor" line="none"
           fill="linear(90deg, tx1:alpha(85%) 0%, tx1:alpha(0%) 70%)"/>
    <title id="cover-title" morph-key="headline" color="bg1">
      Итоги Q3: <span color="accent2">родные</span> слайды без скриншотов
    </title>
    <subtitle id="cover-sub" color="bg1:alpha(80%)">Обзор продукта · октябрь 2026</subtitle>
    <shape id="brand-bar" preset="rect" at="c1-2 r7-7" h="6pt" fill="accent2"
           line="none" morph-key="brand-bar"/>
    <notes>Начните с цитаты клиента. <b>Не</b> читайте заголовок.</notes>
  </slide>

  <!-- Слайд 2: столбцы, список, KPI-карточка, тень, анимация, Morph -->
  <slide id="why" layout="title-content" transition="morph" transition-duration="800ms">
    <title id="why-title" morph-key="headline">Почему это работает</title>
    <columns id="why-cols" area="body" ratio="3fr 2fr" gap="l">
      <ul id="points" size="body" fit="error" gap="m">
        <li id="p1">Вёрстка <b>описана</b>, геометрия <i>вычислена</i>.</li>
        <li id="p2">У каждого элемента есть <span color="accent1" b="1">стабильный id</span>.</li>
        <li id="p3">Переносы строк <u>совпадают</u> с PowerPoint.</li>
        <li id="p4">Подробности в <a href="https://example.com/spec">спецификации</a>.</li>
      </ul>
      <stack id="right" gap="m">
        <shape id="kpi" preset="roundRect" radius="12pt" fill="accent1" shadow="soft"
               pad="l" anchor="middle" h="1fr">
          <p id="kpi-value" align="center" size="display" color="bg1"><b>87%</b></p>
          <p id="kpi-label" align="center" size="small" color="bg1:alpha(85%)">меньше затрат на правки</p>
        </shape>
        <image id="shot" src="assets/editor.png" sizing="cover" h="1fr" radius="8pt"
               alt="Скриншот редактора"/>
      </stack>
    </columns>
    <shape id="brand-bar-2" preset="rect" at="c1-12 r8-8" h="4pt" fill="accent2"
           line="none" morph-key="brand-bar"/>
    <timeline>
      <enter target="points" effect="fade" by="paragraph" start="click" duration="400ms"/>
      <enter target="kpi" effect="zoom" start="click" duration="500ms"/>
      <enter target="shot" effect="fly-in" from="bottom" start="with" duration="500ms"/>
    </timeline>
    <notes>Четыре клика для пунктов, затем один клик для карточки.</notes>
  </slide>

  <!-- Слайд 3: родная таблица с объединённой ячейкой, переход push -->
  <slide id="numbers" layout="title-only" transition="push" transition-dir="left">
    <title id="numbers-title">Квартал за кварталом</title>
    <table id="pnl" area="body" style="ink-minimal" header="1" band="rows"
           cols="2fr 1fr 1fr 1fr" size="body">
      <tr><th>Метрика</th><th align="right">Q1</th><th align="right">Q2</th><th align="right">Q3</th></tr>
      <tr><td>Выручка</td><td align="right">$1,2 млн</td><td align="right">$1,5 млн</td>
          <td align="right"><b>$1,9 млн</b></td></tr>
      <tr><td>Валовая маржа</td><td align="right">61%</td><td align="right">63%</td>
          <td align="right" fill="accent3" color="bg1">67%</td></tr>
      <tr><td colspan="4" size="small" color="tx2">Данные не аудированы. Год заканчивается в декабре.</td></tr>
    </table>
    <timeline>
      <enter target="pnl" effect="wipe" from="top" start="after" duration="400ms"/>
    </timeline>
    <notes>Покажите на ячейку маржи Q3. Это единственное выделенное значение.</notes>
  </slide>
</deck>
```

`id` уникален во всей презентации. Поэтому полоса на втором слайде имеет `id="brand-bar-2"`. Пару Morph задаёт общий `morph-key`.

---

## 5. Архитектура

### 5.1 Структура репозитория

```
F:\pptx-ai-native\
  Cargo.toml                  # workspace; точные версии harfrust/skrifa/icu_segmenter/rmcp
  crates/
    sml-core/                 # синтаксис, CST, патчи, таблицы словаря, модель, темы-киты,
      src/{syntax,schema,      #   вёрстка, текстовый движок, линтер
           model,resolve,
           layout,text,lint}/
    sml-ooxml/                # эмиттеры по порядку схемы, rels, content types,
                              #   шаблоны timing, EOT, zip
    sml-cli/                  # бинарник `sml`, включая `sml mcp` (rmcp)
  oracle/                     # C# .NET 8: PowerPoint COM + OpenXmlValidator, JSON-lines
    Oracle.csproj
  registry/
    fonts/<family>/           # *.ttf, LICENSE, metrics.json (lineFactor, build)
    animations/               # шаблоны от PowerPoint + manifest.json
    themes/ink/, themes/paper/
    presets/presetShapeDefinitions.xml   # из ECMA-376 Part 1
  corpus/
    text/                     # калибровочные спецификации + эталоны PowerPoint по build
    decks/                    # примеры презентаций с эталонной геометрией
    broken/                   # специально испорченные пакеты для самопроверки оракула
  evals/                      # оценочный стенд агентов (единственный код с вызовами LLM)
  docs/                       # этот документ, справочник словаря (генерируется), ADR
```

Три крейта Rust. Текстовый движок живёт в `sml-core`. Он переходит в свой крейт, когда его отдельно потребуют растеризатор P1 и WASM.

### 5.2 Конвейер компиляции

```
.sml → parse → validate → expand → resolve → layout ⇄ text → lint → lower → package → [verify]
```

| Шаг | Что делает | Ошибки |
|---|---|---|
| 1. parse | Строит CST с позициями. Отклоняет DTD, сущности и управляющие символы. | `syntax/*` с `line:col` |
| 2. validate | Проверяет элементы, атрибуты, типы и перечисления по таблицам словаря. Проверяет уникальность `id`, цели анимаций, пары `morph-key`, файлы картинок, шрифты, 187 пресетов и имена `adj`. | `schema/*`, `ref/*`, `font/*`, `asset/*`, `css/*` |
| 3. expand | Раскрывает макеты кита и компоненты (P1) в обычные узлы с производными `id`. | `cmp/*` |
| 4. resolve | Применяет токены. Привязывает плейсхолдеры к макету. Раскрывает эффекты (`shadow="soft"`). | `token/*` |
| 5. layout | Делит области контейнерами. Считает рамки в EMU. Размер `auto` спрашивает у текстового движка. | `layout/*` (`LAY*`) |
| 6. text | Шейпинг, переносы, высоты строк, политика подгонки (раздел 7). | `text/*` (`TXT*`) |
| 7. lint | Правила по разрешённой модели (раздел 10). | предупреждения |
| 8. lower | Пишет части OOXML. Порядок детей — по `xsd:sequence`. Заполняет шаблоны анимаций. Пишет `mc:AlternateContent`. Дедуплицирует медиа по SHA-256. | только внутренние ошибки |
| 9. package | `[Content_Types].xml` первым. Нет записей каталогов. Фиксированные метки времени. Пишет `out.manifest.json`. | — |

Инвариант записи: у каждого типа OOXML ровно один эмиттер в Rust. Табличный тест сверяет порядок детей эмиттера с последовательностью XSD. Порядок в Phase 1 копируем вручную из ECMA-376 Part 1. Генерацию кода из XSD отложили: валидатор ловит ошибки, а кода меньше.

Манифест `out.manifest.json` содержит: слайды, фигуры (`name`, тип, рамка, текст, тип плейсхолдера), строки текста, эффекты анимации, переходы, заметки. Оракул сверяет с ним PowerPoint.

### 5.3 Пакет OOXML

Компилятор всегда пишет:

- `[Content_Types].xml`, `_rels/.rels`, `docProps/core.xml`, `docProps/app.xml` (даты `dcterms:created` и `dcterms:modified` — постоянное значение; текущее время нарушается цель 1.1.7);
- `ppt/presentation.xml` с `sldMasterIdLst`, `notesMasterIdLst`, `sldIdLst`, `sldSz`, `notesSz` (обязателен), `defaultTextStyle`;
- `ppt/presProps.xml`, `ppt/viewProps.xml`, `ppt/tableStyles.xml`;
- `ppt/theme/theme1.xml` (12 цветов, `majorFont`/`minorFont` с `latin ea cs`, `fmtScheme` с минимум тремя записями в каждом списке);
- `ppt/theme/theme2.xml` для мастера заметок;
- `ppt/slideMasters/slideMaster1.xml` с `clrMap` и `txStyles`;
- `ppt/slideLayouts/slideLayoutN.xml`;
- `ppt/slides/slideN.xml`, `ppt/notesSlides/notesSlideN.xml`, `ppt/notesMasters/notesMaster1.xml`;
- `ppt/media/*`, позже `ppt/charts/*`, `ppt/embeddings/*.xlsx`, `ppt/fonts/*.fntdata`.

Правила id:

- `p:sldId/@id` — от 256.
- `sldMasterId` и все `sldLayoutId` — от 2147483648, уникальны вместе.
- `cNvPr/@id` — уникален на слайде, с 2.
- `cTn/@id` — уникален на слайде.
- GUID (разделы, `a16:creationId`) — из хеша `id` исходника.

---

## 6. Отображение в OOXML

| Элемент SlideML | Конструкция PresentationML / DrawingML | Приоритет |
|---|---|---|
| `deck size` | `p:sldSz cx cy`, `p:notesSz` | P0 |
| `theme/colors` | `a:clrScheme` (dk1, lt1, dk2, lt2, accent1–6, hlink, folHlink) | P0 |
| `theme/fonts` | `a:fontScheme/majorFont, minorFont` с `latin ea cs` | P0 |
| токены `scale`, `space`, `grid` | Нет в OOXML. Компилятор подставляет числа. | P0 |
| `effect` (токен) | `a:effectLst/a:outerShdw` с фиксированными значениями | P0 |
| `master` | `p:sldMaster` + `p:clrMap` + `p:txStyles` | P0 |
| `layout` | `p:sldLayout type preserve="1"` + `a:masterClrMapping` | P0 |
| `placeholder` | `p:sp` с `p:nvPr/p:ph type idx` в макете | P0 |
| `slide` | `p:sld`, связь со `slideLayout` | P0 |
| `title`, `subtitle`, `body` | `p:sp` с `p:ph`; пустой `p:spPr`, если геометрия не переопределена | P0 |
| `shape preset adj` | `p:sp`/`a:prstGeom prst` + `a:avLst/a:gd` | P0 |
| `radius` | `adj` для `roundRect` = радиус / меньшая сторона × 100000; больше 50000 — ошибка `schema/radius-too-large` | P0 |
| `rotate`, `flip` | `a:xfrm rot flipH flipV` | P0 |
| `fill` сплошной, линейный | `a:solidFill`, `a:gradFill` + `a:lin` | P0 |
| `fill` радиальный | `a:gradFill` + `a:path` | P1 |
| модификаторы цвета | `a:alpha`, `a:tint`, `a:shade`, `a:lumMod`, `a:lumOff` | P0 |
| `line`, `head`, `tail` | `a:ln w` + заливка + `a:prstDash` + `a:headEnd`/`a:tailEnd` | P0 |
| `shadow` | `a:outerShdw` | P0 |
| `glow`, мягкие края, отражение | `a:glow`, `a:softEdge`, `a:reflection` | P1 |
| `pad`, `anchor`, `wrap` | `a:bodyPr lIns tIns rIns bIns anchor wrap` | P0 |
| `fit="shrink"` | явный `sz` + `<a:normAutofit/>` | P0 |
| `fit="error"`, `fit="none"` | явный `sz` + `<a:noAutofit/>` | P0 |
| `fit="grow"` | вычисленная высота `a:ext cy` + `<a:spAutoFit/>` | P0 |
| `p` | `a:p` + `a:pPr algn lvl marL indent` + `a:lnSpc`/`a:spcBef`/`a:spcAft` + `a:endParaRPr sz` | P0 |
| `ul/li`, `ol/li` | `a:p` с `a:buChar` или `a:buAutoNum`, висячий отступ `marL`/`indent` | P0 |
| `b i u s sup sub` | `a:rPr b i u="sng" strike baseline` | P0 |
| `span color size font` | `a:rPr sz` + заливка + `a:latin/a:ea/a:cs` | P0 |
| `a href` | `a:hlinkClick r:id` + связь `hyperlink TargetMode="External"` | P0 |
| `a slide` | `a:hlinkClick action="ppaction://hlinksldjump"` + связь `slide` | P1 |
| `br` | `a:br` | P0 |
| `image` | `p:pic` + `p:blipFill/a:blip r:embed` + `a:stretch` | P0 |
| `sizing="cover"` + `focus` | `a:srcRect l t r b` из размеров пикселей и точки фокуса | P0 |
| `image radius` | `a:prstGeom prst="roundRect"` в `p:spPr` | P0 |
| `alt` | `p:cNvPr descr` | P0 |
| `image src=*.svg` | `a:blip` с PNG (resvg) + `asvg:svgBlip` в `a:extLst` | P1 |
| SVG-иконки | `a:custGeom` | P1 |
| `table` | `p:graphicFrame` + `a:tbl` + `a:tblGrid/a:gridCol w` | P0 |
| `tr` | `a:tr h` = измеренная высота. PowerPoint считает `h` минимумом и сам увеличивает строку под текст (раздел 7.3) | P0 |
| `td colspan`/`rowspan` | `gridSpan`/`rowSpan` + ячейки `hMerge`/`vMerge` | P0 |
| `td fill`, рамки | `a:tcPr` + заливка + `a:lnL/lnR/lnT/lnB` | P0 |
| `table style` | `a:tblPr firstRow bandRow bandCol` + `a:tableStyleId` `{2D5ABB26-0587-4C30-8999-92F81FD0307C}` («No Style, No Grid», встроенный стиль) + явный `a:tcPr` в каждой ячейке. Стиль кита — макрос компилятора, а не стиль в `tableStyles.xml`. Компромисс: вкладка Table Design в PowerPoint не знает стиль кита | P0 |
| контейнеры | Нет фигуры. Только рамки детей. | P0 |
| `group`, `stack group="1"` | `p:grpSp`, `chOff = off`, `chExt = ext` | P1 |
| `connector` | `p:cxnSp` + `a:stCxn/a:endCxn` + вычисленные `xfrm`, flips, `adj` | P1 |
| `chart` | `p:graphicFrame` + `c:chart` + `c:chartSpace` с кешами + встроенный `.xlsx` | P1 |
| `notes` | `p:notes` + плейсхолдер `body idx=1` + `notesMaster` + `theme2` | P0 |
| `transition="fade\|push\|wipe"` | `p:transition spd` + `p:fade`/`p:push dir`/`p:wipe dir` | P0 |
| `transition-duration` | `p14:dur` в `mc:AlternateContent`, `mc:Ignorable="p14"` | P0 |
| `transition="morph"` | `mc:AlternateContent`: Choice `p159:morph option="byObject"`, Fallback `p:fade` | P0 |
| `morph-key="k"` | `p:cNvPr name="!!k"`; `id` исходника хранится в манифесте (раздел 3.4) | P0 |
| `timeline/enter`, `exit` | `p:timing` из шаблона пресета + `p:bldP spid grpId build` | P0 |
| `start="click\|with\|after"` | группа клика / `withEffect` / `afterEffect` с вычисленной задержкой | P0 |
| `by="paragraph"` | `p:txEl/p:pRg` на каждый абзац + `p:bldP build="p"` | P0 |
| `emphasis`, `motion` | шаблоны `presetClass="emph"`, `"path"` | P1 |
| интерактивный триггер | `p:seq nodeType="interactiveSeq"` | P1 |
| `section` | `p14:sectionLst` в `p:extLst` `presentation.xml` | P1 |
| футер, номер слайда | `p:hf` + плейсхолдеры `dt`/`ftr`/`sldNum` | P1 |
| встраивание шрифта | `p:embeddedFontLst` + `ppt/fonts/*.fntdata` (EOT) + `embedTrueTypeFonts="1"` | P1 (после спайка M3) |
| видео, аудио | `p:pic` + `a:videoFile` + `p14:media` + постер | P1 |
| узоры, 3D, WordArt | `a:pattFill`, `a:scene3d`/`a:sp3d`, `a:prstTxWarp` | P2 |
| SmartArt | Не пишем. Компоненты дают группы родных фигур. | никогда |

---

## 7. Измерение и подгонка текста

### 7.1 Исходные факты

Microsoft не публикует алгоритм вёрстки текста. Ниже — гипотезы из открытых проектов. Этап M1 проверяет каждую через оракул.

| Аспект | Гипотеза | Источник |
|---|---|---|
| Вставки по умолчанию | `lIns=rIns=91440`, `tIns=bIns=45720` EMU | ECMA-376 |
| Одинарный интервал | 1,2 × наибольший размер run в строке; другие данные: 1,1755–1,2037 em или ascent+descent+gap | [rdocx #222](https://github.com/tensorbee/rdocx/pull/222), [office2pdf #1071](https://github.com/developer0hye/office2pdf/issues/1071), [betteroffice #313](https://github.com/openooxml/betteroffice/issues/313) |
| Положение базовой линии | доля `usWinAscent / (usWinAscent + usWinDescent)` | [office-open-xml-viewer #1621](https://github.com/yukiyokotani/office-open-xml-viewer/pull/1621) |
| Ширины глифов | округление до 1/8 pt | [rdocx #222](https://github.com/tensorbee/rdocx/pull/222) |
| Переносы | UAX #14 с отличиями: перенос после `/`; нет переноса после дефиса между цифрами; висячий пробел в конце | [spine #423](https://github.com/mgilbir/spine/pull/423) |
| Автоподбор | PowerPoint применяет сохранённый масштаб; без атрибутов — 100% | [ooxml-common #10](https://github.com/uvrt/ooxml-common/pull/10) |
| `spcBef` первого абзаца | возможно, игнорируется | **[verify]** |
| Кернинг | PowerPoint кернит run, только если `sz` ≥ порога `kern` (каскад мастера часто даёт `kern="1200"`). Для статических шрифтов он берёт пары из старой таблицы `kern`, а не из GPOS | [betteroffice #861](https://github.com/openooxml/betteroffice/pull/861), [pptx2svg #14](https://github.com/uvrt/pptx2svg/pull/14), [MS Q&A](https://learn.microsoft.com/en-us/answers/questions/5891213/microsoft-powerpoint-incomplete-opentype-gpos-kern) |
| Лигатуры и другие функции OpenType | неизвестно, применяет ли PowerPoint `liga` по умолчанию | **[verify]** |
| Текстовая рамка фигуры | для фигур кроме `rect` текст вписан в прямоугольник `a:rect` из определения пресета, а не в рамку фигуры | ECMA-376 Part 1, `presetShapeDefinitions.xml` |

### 7.2 Алгоритм для каждого текстового блока

1. **Разрешить стиль.** Взять размер, шрифт, интервалы и вставки из модели. Не полагаться на наследование размера от макета.
2. **Шейпинг.** `harfrust` с файлом шрифта из реестра. Выключить функцию GPOS `kern`. Применить пары из таблицы `kern`, только если `sz` ≥ порога `kern`. Компилятор пишет `kern` явно на каждом run. Лигатуры — по результату M1. Округлить ширины до 1/8 pt **[verify]**. Учесть `spc`.
3. **Точки переноса.** `icu_segmenter` (UAX #14) плюс правила Office:
   - разрешить перенос после `/`;
   - запретить перенос после дефиса между цифрами;
   - не считать висячие пробелы в ширину;
   - не начинать строку с пробела, запятой или дефиса;
   - принудительно рвать слово длиннее строки.
4. **Строки.** Жадно набирать строки. Доступная ширина = ширина текстовой рамки − `lIns` − `rIns` − `marL` − `indent`. Для `rect` текстовая рамка равна рамке фигуры. Для других пресетов компилятор вычисляет `a:rect` по формулам `gdLst` из `presetShapeDefinitions.xml`. Для этого нужен вычислитель формул пресетов (17 операторов). M4 проверяет его на `ellipse`, `roundRect`, `chevron`, `hexagon` через `measure`.
5. **Высота.** Высота строки = `lineFactor(шрифт) × max sz в строке × spcPct` или `spcPts`. Прибавить `spcBef`, `spcAft`, `tIns`, `bIns`.
6. **Запас.** Начальный запас — 2% доступной высоты. Он уменьшается по данным калибровки каждого шрифта.
7. **Политика подгонки.**
   - `error` — переполнение даёт `TXT001` с мерами и исправлениями.
   - `shrink` — двоичный поиск по целым пунктам до `min-size`. Записать явный `sz` на каждом run. Записать `<a:normAutofit/>`. Если при `min-size` текст не помещается — `TXT003`.
   - `grow` — записать вычисленную высоту и `<a:spAutoFit/>`.
   - `none` — `<a:noAutofit/>`, переполнение — предупреждение.
8. **Результат.** Для каждого блока разрешённая модель хранит строки (текст, базовая линия, ширина, высота), применённый размер, `fill_ratio` и `overflow_pt`.

### 7.3 Таблицы

- Ширины столбцов дают `cols` (`fr`, `pt`, `%`).
- Высота строки = максимум по ячейкам: измеренный текст + `marT` + `marB`.
- Компилятор пишет высоту в `a:tr h`. Высоту рамки `graphicFrame` он пишет как сумму строк.
- PowerPoint считает `a:tr h` минимумом. Если наша мера меньше, PowerPoint увеличивает строку, и таблица растёт вниз за пределы области. Поэтому таблицы используют тот же запас, что и текст (7.2, шаг 6). Если таблица с запасом не помещается в область — ошибка `LAY002`.
- Модель высоты строк калибруется по COM `Rows(i).Height`.

### 7.4 Калибровка

1. Генератор пишет корпус нашим же записывателем. Перекрёстное произведение: шрифты {Calibri, Arial, Georgia, Segoe UI, Inter (только встроенный, после M3)} × размеры {10, 12, 14, 18, 24, 32, 44, 60, 96} × 5 ширин × `spcPct` {90, 100, 120, 150} и `spcPts` × выравнивание {l, ctr, r} × маркеры да/нет × смешанные run × латиница и кириллица. Тексты: lorem, русская проза, URL, числа с дефисами, дроби через `/`.
2. Около 500 блоков на шрифт.
3. Оракул командой `calibrate-text` читает `Lines(i).Text` и `BoundLeft/Top/Width/Height`.
4. Скрипт подбирает `lineFactor` и проверяет округление 1/8 pt. COM не даёт базовую линию. Поэтому `ascentShare` подбирается в P1 по растру, когда он нужен растеризатору.
5. Результат записывается в `registry/fonts/<family>/metrics.json` с номером сборки PowerPoint.

---

## 8. Превью и проверка через PowerPoint COM

### 8.1 Оракул

`oracle/` — долгоживущий процесс C# .NET 8. Он читает запросы JSON-lines из stdin и пишет результаты в stdout.

Внимание: PowerPoint — COM-сервер с одним экземпляром на сессию. Если пользователь уже открыл PowerPoint, оракул подключается к тому же процессу. `Application.Quit()` или завершение `POWERPNT.EXE` закрывает презентации пользователя без сохранения ([codex #40072](https://github.com/openai/codex/issues/40072), [Microsoft](https://learn.microsoft.com/en-us/previous-versions/office/troubleshoot/office-developer/use-visual-c-automate-run-program-instance)). Правила оракула:

1. Оракул закрывает только те презентации, которые открыл сам. Он никогда не вызывает `Quit()`.
2. При старте оракул проверяет, работает ли `POWERPNT.EXE`. Если PowerPoint запустил пользователь, супервизор не завершает процесс по тайм-ауту. Он возвращает ошибку `oracle/busy-user-session` и ждёт.
3. Супервизор завершает `POWERPNT.EXE` после тайм-аута (по умолчанию 30 с), только если этот процесс запустил оракул.
4. Оракул копирует входной `.pptx` во временный файл и открывает копию. Так открытый у пользователя файл не блокирует сборку.

| Команда | Что делает | Результат |
|---|---|---|
| `validate <pptx>` | `OpenXmlValidator`, формат Office 2019 | список ошибок; норма — 0. Ложное срабатывание попадает в список исключений, только если PowerPoint даёт ту же ошибку на своём файле. Каждое исключение имеет причину. Знает ли валидатор `p159:morph`, проверяет M0 **[verify]** |
| `open-check <pptx>` | `DisplayAlerts=ppAlertsNone`; `Presentations.Open(path, ReadOnly, Untitled=false, WithWindow=false)`; ловит `COMException`; ищет диалоговые окна `POWERPNT` через `EnumWindows` | `{opened, repaired_or_blocked, error}` |
| `inventory <pptx>` | Читает слайды, фигуры, текст, плейсхолдеры, эффекты, переходы, заметки. Сверяет с манифестом. | расхождения |
| `resave-diff <pptx>` | `SaveAs` во временный файл; нормализация (порядок атрибутов, `a16:creationId`, `dirty`, `err`, rsid); семантическое сравнение по частям | удалённые и изменённые элементы. PowerPoint при сохранении добавляет много своего (`extLst`, `p14:modId`, списки шрифтов темы, `app.xml`). Такие добавления описывает список разрешённых изменений. Удаление или новое значение нашего элемента — сбой |
| `render <pptx> [slides]` | `Slide.Export(path, "PNG", 1920, H)`; `H` — из пропорций `sldSz` (1080 для 16:9, 1440 для 4:3) | пути PNG |
| `measure <pptx>` | Для каждой фигуры: `Name`, рамка, `TextFrame2.TextRange.Lines(i)` с `.Text` и `BoundLeft/Top/Width/Height`; для таблиц `Rows(i).Height`. COM не даёт положение базовой линии. Оракул меряет только рамки строк | JSON геометрии |
| `anim-dump <pptx>` | `TimeLine.MainSequence(i)`: `EffectType`, `Exit`, `Timing.TriggerType`, `Duration`, `TriggerDelayTime`, `Shape.Name`, абзац; `SlideShowTransition.EntryEffect`, `Duration` | JSON анимаций |
| `harvest-anim <spec>` | `AddEffect` для каждой комбинации; извлечение `p:par` и `p:bldP` | шаблоны реестра |
| `calibrate-text <corpus>` | `measure` для корпуса | JSON для подбора констант |

Оракул работает в интерактивной сессии пользователя. COM не работает из службы. CI — self-hosted runner на этой машине, одна последовательная очередь.

### 8.2 Ворота проверки

Каждая фикстура и каждая примерная презентация проходит все ворота:

1. **Схема.** `validate` даёт 0 ошибок.
2. **Открытие.** `open-check` успешен. `inventory` совпадает с манифестом. `resave-diff` не содержит изменений вне списка разрешённых.
3. **Геометрия (главные ворота).**
   - Число строк и текст каждой строки совпадают точно.
   - Верх и высота каждой строки — в пределах ±0,5 pt.
   - Ширина строки — в пределах 1% или 1 pt.
   - Флаг переполнения совпадает.
   - Высоты строк таблиц — в пределах ±1 pt.
4. **Анимации.** `anim-dump` совпадает с `<timeline>`. Ни один эффект не отображается как «Custom».
5. **Растр (вторичные ворота).** SSIM PNG против эталона, привязанного к номеру сборки PowerPoint. Он ловит изменения поведения после обновления Office.

### 8.3 Метрика сравнения растров

Phase 1 использует метрику только для дрейфа Office. Эталон и проверяемый PNG дал один и тот же PowerPoint. Поэтому достаточно простого сравнения:

1. Оба PNG одного размера. Перевести в шкалу серого.
2. Пиксельная разница: доля пикселей с |Δ| > 20/255. Провал — больше 0,5% площади.
3. Провал называет `id` фигур, рамки которых пересекают отличающиеся пиксели.

SSIM, тепловые карты и пороги для растеризатора появятся в P1 вместе с растеризатором. Начальные пороги P1: SSIM ≥ 0,985 для фигур и картинок, ≥ 0,97 для слайдов с текстом.

### 8.4 Превью для агента

- Phase 1: `render` вызывает оракул. PNG кешируется по хешу разрешённой модели слайда.
- Режим `overlay` рисует поверх PNG: метки `id`, рамки, зоны переполнения красной штриховкой, пары пересечений, линии сетки. Оверлей рисует Rust по разрешённой модели. Для этого достаточно прямоугольников `tiny-skia`. `tiny-skia` не рисует текст: подписи `id` строятся из контуров глифов `skrifa`. Полный растеризатор слайда не нужен.
- P1: свой растеризатор рисует разрешённую модель. Он становится превью по умолчанию после прохода порогов 8.3 на корпусе.

---

## 9. CLI и MCP-инструменты

### 9.1 CLI `sml`

| Команда | Назначение |
|---|---|
| `sml new <dir> --theme ink` | Создать каркас презентации |
| `sml check <deck.sml> [--json]` | Разбор, проверка, вёрстка, линт; файл не пишется |
| `sml build <deck.sml> -o out.pptx [--verify]` | Полная компиляция; пишет `out.manifest.json`; `--verify` запускает ворота 8.2 |
| `sml lint <deck.sml> [--slide id] [--json]` | Только линт |
| `sml fmt <deck.sml> [--assign-ids]` | Канонический формат, генерация `id` |
| `sml inspect <deck.sml> [--slide id] [--resolved]` | Оглавление или разрешённая модель в JSON |
| `sml patch <deck.sml> ops.json` | Применить операции патча |
| `sml render <deck.sml> --slide id [--overlay] -o dir/` | PNG через оракул |
| `sml vocab [--format md\|json]` | Справочник словаря для агента (~2k токенов), генерируется из таблиц |
| `sml fonts list\|add <ttf>` | Реестр шрифтов: `fsType`, контуры TrueType, статус калибровки |
| `sml verify <out.pptx>` | Ворота оракула для готового файла |
| `sml mcp` | MCP-сервер по stdio |

### 9.2 Правила MCP-сервера

1. Файл на диске — источник истины. Сервер хранит только кеш рендеров.
2. Каждый изменяющий инструмент принимает `base_version` (SHA-256 файла). Если файл изменился, сервер возвращает `conflict` и текущее оглавление. Так правки пользователя не теряются.
3. Запись атомарна: временный файл, затем переименование.
4. Пакет операций атомарен: если одна операция падает, файл не меняется.
5. Каждый инструмент объявляет `outputSchema` и возвращает `structuredContent`. Картинки — блоки image content.
6. Каждый изменяющий инструмент возвращает диагностику, сводку линта и изменённые рамки. Второй вызов не нужен.
7. Инструменты отображаются 1:1 в функции библиотеки.
8. Сервер не вызывает LLM.

### 9.3 Инструменты

| Инструмент | Вход | Выход |
|---|---|---|
| `get_vocabulary` | `{topic?}` | `{markdown, version}` |
| `catalog` | `{kind: "themes"\|"layouts"\|"presets"\|"animations"\|"transitions"\|"fonts"\|"tokens", query?}` | `{items:[{name, description, params?, regions?, thumbnail_uri?}]}` |
| `create_deck` | `{path, theme, size?, lang?}` | `{path, version}` |
| `get_outline` | `{path}` | `{version, theme, slides:[{id, layout, title, node_ids, word_count, effect_count, lint_counts}], disabled_rules}` |
| `get_slide` | `{path, slide_id, view: "source"\|"resolved"\|"both"}` | `{version, xml?, shapes?:[{id, kind, box_pt, z, lines?, applied_size_pt?, fill_ratio?, overflow_pt?}]}` |
| `add_slide` | `{path, base_version, after?, layout, xml?}` | `{version, slide_id, assigned_ids, diagnostics, lint}` |
| `patch` | `{path, base_version, ops, dry_run?}` | `{ok, version, assigned_ids, diagnostics, lint, changed:[{id, box_pt_before, box_pt_after}]}` |
| `lint` | `{path, slide_id?, rules?}` | `{version, diagnostics, score:{errors, warnings}}` |
| `render` | `{path, slide_ids, overlay?, width_px?}` | image на каждый слайд + `{version, items:[{slide_id, png_path, cached}]}` |
| `build_pptx` | `{path, out, verify?}` | `{out, bytes, manifest_path, verify?:{validator_errors, opened, inventory_mismatches, resave_diff, geometry_mismatches}}` |

### 9.4 Схема операции патча

```json
{
  "$id": "urn:slideml:op",
  "oneOf": [
    {"type":"object","required":["op","id","attrs"],
     "properties":{"op":{"const":"set_attrs"},"id":{"type":"string"},
                   "attrs":{"type":"object","additionalProperties":{"type":"string"}}}},
    {"type":"object","required":["op","id","names"],
     "properties":{"op":{"const":"unset_attrs"},"id":{"type":"string"},
                   "names":{"type":"array","items":{"type":"string"}}}},
    {"type":"object","required":["op","id","xml"],
     "properties":{"op":{"const":"replace_inner"},"id":{"type":"string"},"xml":{"type":"string"}}},
    {"type":"object","required":["op","ref","position","xml"],
     "properties":{"op":{"const":"insert"},"ref":{"type":"string"},
                   "position":{"enum":["before","after","first_child","last_child"]},
                   "xml":{"type":"string"}}},
    {"type":"object","required":["op","id","ref","position"],
     "properties":{"op":{"const":"move"},"id":{"type":"string"},"ref":{"type":"string"},
                   "position":{"enum":["before","after","first_child","last_child"]}}},
    {"type":"object","required":["op","id"],
     "properties":{"op":{"const":"delete"},"id":{"type":"string"}}},
    {"type":"object","required":["op","id","new_id"],
     "properties":{"op":{"const":"duplicate"},"id":{"type":"string"},"new_id":{"type":"string"}}}
  ]
}
```

Компилятор проверяет XML-фрагменты в операциях по схеме целевого контекста. Например, `insert` в `<ul>` принимает только `<li>`.

### 9.5 Схема диагностики

```json
{
  "code": "TXT001",
  "severity": "error",
  "slide": "why",
  "id": "points",
  "path": "slide#why > columns#why-cols > ul#points",
  "span": {"line": 31, "col": 7},
  "message": "Text needs 212pt but the box has 186pt at size body (18pt).",
  "measured": {"overflow_pt": 26, "fill_ratio": 1.14, "lines": 9, "max_chars_hint": 180},
  "expected": null,
  "fixes": [
    {"title": "Use size=small",
     "ops": [{"op": "set_attrs", "id": "points", "attrs": {"size": "small"}}]},
    {"title": "Allow shrink to 14pt",
     "ops": [{"op": "set_attrs", "id": "points", "attrs": {"fit": "shrink", "min-size": "14"}}]}
  ]
}
```

Агент применяет исправление одним вызовом `patch`. Ни одна сторона не разбирает свободный текст.

---

## 10. Линтер и цикл агента

### 10.1 Правила P0

Линтер работает по разрешённой модели, а не по словам агента.

| Код | Правило | Порог |
|---|---|---|
| TXT001 | Переполнение при `fit="error"` | любое |
| TXT002 | Применено сжатие (info): исходный → итоговый размер | — |
| TXT003 | Сжатие дошло до `min-size`, текст не помещается | ошибка |
| TXT004 | Минимальный размер | основной текст < 14 pt, подпись < 10 pt |
| TXT005 | Последняя строка заголовка — одно слово | предупреждение |
| LAY001 | Дети не помещаются в контейнер | ошибка |
| LAY002 | Таблица с запасом высоты строк не помещается в область | ошибка |
| GEO001 | Элемент вне слайда (кроме `area="slide"`) | любое |
| GEO002 | Пересечение элементов содержимого (`role="decor"` и `layer="back"` исключены) | > 2% меньшей площади |
| GEO003 | Почти выровненные края или центры | 0,5–4 pt |
| GEO004 | Абсолютная рамка вне сетки | предупреждение |
| GEO005 | Заголовки одного макета стоят по-разному | > 1 pt |
| GEO006 | Содержимое вне безопасной области кита | предупреждение |
| CLR001 | Контраст текста с фоном | WCAG 4,5:1; 3:1 для ≥ 24 pt |
| DS001 | Литеральный цвет вне палитры | info |
| DS002 | Больше 2 семейств шрифтов или больше 4 размеров на слайде | предупреждение |
| DS003 | Отступ вне шкалы | info |
| DEN001 | Слов на слайде | > 60 предупреждение, > 100 ошибка |
| DEN002 | Больше 6 пунктов или пункт длиннее 2 строк | предупреждение |
| DEN003 | Заголовок не крупнее основного текста | предупреждение |
| IMG001 | Эффективное разрешение картинки | < 150 dpi |
| IMG002 | Искажение пропорций при `sizing="stretch"` | > 1% |
| A11Y001 | Картинка без `alt` | предупреждение |
| ANI001 | Цель анимации не существует, или `by="paragraph"` без абзацев | ошибка |
| ANI002 | Morph без пары `morph-key` на предыдущем слайде | предупреждение |
| ANI003 | Больше 8 кликов на слайде | предупреждение |
| PH001 | Пустой плейсхолдер (в PowerPoint видно «Click to add text») | предупреждение |

Для CLR001 фон под текстом считается так: сплошная заливка — точно; картинка без затемнения — «неизвестно» в Phase 1; картинка с `role="decor"` фигурой-затемнением — по цвету и `alpha` затемнения.

Правило отключается через `<lint disable="DEN001"/>`. Каждое отключение видно в `get_outline`.

### 10.2 Цикл «сгенерировал → посмотрел → исправил»

Сервер передаёт этот порядок в поле `instructions`:

1. Вызвать `get_vocabulary` один раз.
2. Вызвать `catalog` и выбрать тему и макеты.
3. Вызвать `create_deck`, затем `add_slide` для каждого слайда.
4. Прочитать `diagnostics` и `lint` в каждом ответе.
5. Применить `fixes` через `patch`, пока ошибок не станет 0.
6. Вызвать `get_slide view="resolved"` для слайдов с предупреждениями.
7. Вызвать `render overlay=true` для каждого слайда и проверить вид. Линтер не видит визуальную иерархию, баланс и качество картинок. Это видит только агент с поддержкой изображений. Стоимость — около 1 с на слайд.
8. Вызвать `build_pptx verify=true`.

Если модель агента не принимает изображения, шаг 7 невозможен. Тогда качество держат только киты, токены и линтер.

### 10.3 Как достигается качество дизайна

1. **Темы-киты.** В Phase 1 — два кита: `ink` и `paper`. Каждый содержит мастер, 10 макетов (`cover`, `section`, `title-content`, `two-content`, `title-only`, `blank`, `kpi`, `table`, `quote`, `image-left`), шкалы, сетку, эффекты, стили таблиц.
2. **Образцы.** `get_vocabulary topic="layouts"` возвращает для каждого макета один образец слайда на SlideML с хорошим дизайном. Образцы проходят ворота 8.2. Агент копирует образец и меняет содержимое. Образцы дают модели больше, чем описание правил.
3. **Токены.** Агент выбирает размеры и отступы из шкал. Произвольные числа дают предупреждения.
4. **Семантическая вёрстка.** Агент не планирует координаты. Это делает компилятор.
5. **Линтер.** Плотность, иерархия, контраст и выравнивание проверяются числами.
6. **Просмотр PNG.** Агент смотрит каждый слайд (10.2, шаг 7).
7. **Компоненты (P1).** `kpi`, `stat-row`, `quote`, `step-flow`, `comparison-card` раскрываются в родные фигуры.

---

## 11. Приоритеты фич

| Приоритет | Фичи |
|---|---|
| **P0** | Пакет, rels, content types. `presentation.xml` с id, `sldSz`, `notesSz`. Тема с полным `fmtScheme`. Мастер и макеты с плейсхолдерами (`title`, `ctrTitle`, `subTitle`, `body`). Киты `ink` и `paper`. `sp`, `pic`, `graphicFrame`-таблица. 187 пресетов `prstGeom` с проверкой имён `adj`. Сплошная и линейная градиентная заливка. Линии, штрихи, стрелки. Тень-токен. Модификаторы цвета. Текст: run, маркеры, нумерация, уровни, выравнивание, интервалы, ссылки. Текстовый движок с политиками `error`, `shrink`, `grow`, `none`. Таблицы с вычисленной высотой строк, `colspan`/`rowspan`, заливкой ячеек. PNG и JPEG с `cover`, `contain`, кадрированием. Заметки. Переходы `fade`, `push`, `wipe`, `morph`. Анимации входа и выхода: appear, fade, fly-in (4 направления), wipe (4 направления), zoom, float; старт click/with/after; по абзацам. Шрифты Tier B. Контейнеры и области сетки. Ошибки `css/*`. CLI, MCP, линтер, рендер через COM, ворота проверки. |
| **P1** | Родные графики (bar, line, pie, doughnut, area, scatter) с кешами и `.xlsx`. Группы. Свой растеризатор `tiny-skia`. Радиальный градиент. Glow, мягкие края, отражение. SVG с PNG-заменой. Иконки SVG → `custGeom`. Соединители с привязкой. Эмфаза и пути движения. Интерактивные триггеры. Переходы p14/p15. Разделы. Колонтитулы. Видео и аудио. Встраивание шрифтов Tier A (если M3 прошёл раньше — переходит в P0). Компоненты. Импорт черновика из Markdown. Повторный импорт своего `.pptx` по `cNvPr name` и части `customXml` с исходником **[verify: PowerPoint сохраняет `customXml`]**. `size-inheritance="native"`. |
| **P2** | Импорт чужого `.pptx` с узлами `Opaque`. Узоры. 3D. WordArt. Chartex. Zoom-слайды. Художественные эффекты картинок. Анимация графиков. CJK и RTL. WASM и визуальный редактор. Проверки на Mac и в веб-версии. |
| **Никогда** | Растеризация как запасной вариант. CSS. Запись SmartArt. Новые деревья анимации из примитивов. `effectDag`. |

---

## 12. План этапов с критериями приёмки

Оценки сроков даны для одного инженера с агентами. Это оценки, не обязательства. M1, M2 и M3 идут параллельно.

### M0. Оракул и минимальный пакет (неделя 1)

Объём: оракул со всеми командами, кроме `harvest-anim` и `calibrate-text`. Супервизор. `sml-ooxml` пишет минимальную презентацию: мастер, пустой макет, текстовые блоки, заметки. Сборка набора крейтов.

Критерии:

1. `cargo build` с точными версиями `harfrust`, `skrifa`, `icu_segmenter`, `roxmltree`, `quick-xml`, `zip`, `miette`, `rmcp` проходит.
2. Минимальная презентация: 0 ошибок валидатора, открытие без восстановления, `resave-diff` содержит только разрешённые изменения, экспорт PNG.
3. Оракул помечает как сбой каждый файл из `corpus/broken/`: неверный порядок детей, нет `notesSz`, дубль `cNvPr/@id`, висячий `r:id`. 10 корректных файлов не помечены.
4. Супервизор восстанавливается после принудительного завершения PowerPoint.
5. Пользователь держит открытой несохранённую презентацию. Оракул выполняет все команды. Презентация пользователя остаётся открытой, правки не теряются.
6. Две компиляции одного входа дают побайтно одинаковый файл.
7. Документирован способ, которым PowerPoint 16 сообщает о восстановлении при `WithWindow=false`.

### M1. Текстовый движок против PowerPoint (недели 1–3, главный риск)

Объём: генератор корпуса, `calibrate-text`, движок из раздела 7. M1 калибрует шрифты Tier B. Inter не установлен на машине: без встраивания PowerPoint покажет замену, и калибровка измерит не тот шрифт. Поэтому Inter калибруется после M3 по презентации со встроенным Inter.

Критерии «go»:

1. Число строк и текст строк совпадают точно для ≥ 99% блоков каждого шрифта. Каждое расхождение классифицировано.
2. Верх и высота строк в пределах ±0,5 pt для ≥ 98% строк.
3. Флаг переполнения совпадает с PowerPoint для 100% блоков вне полосы запаса ±2%.
4. Известно, верна ли гипотеза 1/8 pt, игнорируется ли `spcBef` первого абзаца, как работает порог `kern` и применяет ли PowerPoint лигатуры.

План «no-go»:

1. Увеличить запас до 5%. Сузить реестр до шрифтов, которые проходят.
2. Если и это не помогает — добавить явную опцию `--fit-with-powerpoint` (COM пересчитывает AutoSize). Сообщить пользователю об ограничении. Не делать опцию поведением по умолчанию.

### M2. Реестр анимаций и переходов (недели 2–3)

Объём: `harvest-anim` для списка P0. Шаблоны с параметрами. Комбинированные последовательности: click/with/after, по абзацам, задержки, 2+ анимированные фигуры на слайде. Переходы `fade`, `push`, `wipe`, `morph`.

Критерии:

1. Для каждой комбинации эффект × направление × старт × режим `anim-dump` нашего файла совпадает с `anim-dump` эталона от PowerPoint. Ни одного «Custom».
2. `resave-diff` части `p:timing` не содержит изменений вне списка разрешённых после нормализации `cTn id`.
3. Переходы читаются с верным `EntryEffect` и длительностью.
4. Morph связывает фигуры по `!!name`. Проверка — ручной показ одной презентации, результат записан в `registry/animations/manifest.json`.
5. Скрытие фигуры до входа проверено ручным показом для 10 семейств шаблонов. Результат записан.

### M3. Спайк встраивания шрифтов (неделя 3)

Объём: обёртка TTF в несжатый EOT в `.fntdata`. `embeddedFontLst`, `embedTrueTypeFonts="1"`, content type. Inter не установлен на машине.

Критерии:

1. PNG презентации с встроенным Inter явно отличается от замены на Calibri. После этого корпус M1 для Inter проходит критерии «go» M1.
2. Открытие и пересохранение без восстановления. `resave-diff` показывает, что шрифт сохранён.
3. Компилятор отклоняет шрифт с ограничением `fsType` или с контурами CFF. Сообщение объясняет причину.

План «no-go»: Phase 1 работает только на шрифтах Tier B. Киты переходят на Segoe UI, Calibri и Georgia. Встраивание уходит в P1.

### M4. Язык SlideML и полная компиляция P0 (недели 4–7)

Объём: парсер, CST, таблицы словаря, диагностика с исправлениями, ошибки `css/*`, киты `ink` и `paper`, контейнеры, области сетки, картинки, таблицы, заметки. Подключение движка M1, реестра M2 и шрифтов M3. `fmt`, `inspect`, `vocab`, `patch`.

Критерии:

1. Пример из 4.7 и 10 презентаций корпуса (все фичи P0) проходят ворота 8.2.
2. Высота строк таблиц из COM `Rows(i).Height` в пределах ±1 pt от нашего `a:tr h`.
3. Каждая из 30 заложенных ошибок (опечатка в атрибуте, неизвестный пресет, нет цели, неверная единица, неизвестный шрифт, CSS) даёт диагностику с позицией и верной подсказкой.
4. Патчи сохраняют комментарии и форматирование исходника.
5. Человек открывает пример 4.7 в PowerPoint, правит заголовок, меняет цвета темы, нажимает Reset Slide. Правка заголовка и смена цветов темы работают как в родном файле. Для Reset Slide чек-лист записывает фактический результат: позиция плейсхолдеров возвращается к макету, а размер текста зависит от явного `sz` (раздел 3.5). Диалога восстановления нет ни после одного действия.
6. Текст в `ellipse`, `roundRect`, `chevron` и `hexagon` переносится как в PowerPoint (ворота 8.2, пункт 3).

### M5. MCP и оценка агента (недели 8–9)

Объём: `sml mcp` со всеми инструментами 9.3. Кеш рендеров. Конфликты `base_version`. Стенд `evals/`.

Правила кода `evals/` (единственный код с вызовами LLM):

- не задавать `max_tokens` и аналоги;
- читать ответ модели только через вызовы инструментов со схемой или structured output, без регулярных выражений;
- не обрезать ответ модели;
- не выключать thinking; передавать reasoning-блоки дальше без изменений.

Критерии:

1. 10 брифов (питч, квартальный обзор, обучение, презентация на русском и др.): 10 из 10 презентаций проходят ворота 8.2.
2. В конце 0 ошибок `TXT001`, `TXT003`, `GEO002`.
3. Медиана циклов исправления на слайд ≤ 2.
4. Слепая оценка людьми против базовой линии (агент пишет код pptxgenjs) по рубрике (иерархия, выравнивание, плотность, контраст, единообразие): наш результат не хуже на ≥ 7 из 10.
5. Правка `.sml` пользователем во время работы агента даёт `conflict`, а не потерю данных.
6. A/B на 5 брифах: SlideML против исходника на подмножестве HTML. Метрики: доля валидных сборок, дефекты линта, циклы исправления, токены. Результат записан в ADR.

### M6+. P1 в таком порядке

1. Графики с `.xlsx`. Критерий: «Edit Data» открывается, кеши рендерятся, валидатор — 0 ошибок.
2. Группы и соединители.
3. Свой растеризатор. Критерий: SSIM ≥ 0,985 для фигур и картинок, ≥ 0,97 для текста против COM на корпусе.
4. SVG с PNG-заменой, иконки `custGeom`.
5. Эмфаза и пути движения.
6. Повторный импорт своего `.pptx`. Критерий: импорт → сборка даёт тот же манифест.

---

## 13. Риски и меры

| Риск | Вероятность / влияние | Мера | Ранний сигнал | Запасной план |
|---|---|---|---|---|
| Метрики текста расходятся с PowerPoint | высокая / критическое | Калибровка M1, закрытый реестр, явный `sz`, политика `error`, запас | доля совпадений M1 | Сузить шрифты, увеличить запас, явная опция COM-подгонки |
| XML анимации принят, но не играет или «Custom» | средняя / высокое | Только шаблоны от PowerPoint; ворота `anim-dump` | проверка M2 | Сократить список эффектов до прошедших |
| Встроенный шрифт не рендерится | средняя / высокое | Спайк M3 на неустановленном шрифте | PNG M3 | Только Tier B; изменить киты |
| Диалог восстановления в редком случае | средняя / критическое | Один эмиттер на тип, порядок по схеме, ворота на каждой сборке, ночной фаззинг случайных валидных исходников | ночной фаззинг | Блокировать релиз; причина становится фикстурой `broken/` |
| Оракул закрывает презентации пользователя (PowerPoint — один экземпляр на сессию) | высокая / критическое | Правила 8.1: нет `Quit()`, нет завершения чужого процесса, работа с копией файла | тест M0, пункт 5 | Оракул отказывается работать, пока PowerPoint пользователя открыт |
| PowerPoint увеличивает строки таблицы сверх нашей меры | средняя / среднее | Запас высоты, `LAY002`, ворота `Rows(i).Height` | M4, пункт 2 | Увеличить запас для таблиц |
| Текст в PowerPoint для Mac и веб-версии переносится иначе | высокая / среднее | Цель 1.1.5 ограничена Windows; вопрос 14.6 | — | Отдельная калибровка на каждой платформе (P2) |
| JPEG с тегом EXIF Orientation показан повёрнутым или с неверными пропорциями | средняя / среднее | Компилятор читает тег EXIF. Ориентация, отличная от 1, — ошибка `asset/exif-rotated` с подсказкой повернуть файл. Как PowerPoint обрабатывает тег — **[verify M4]** | M4 | — |
| COM зависает или нестабилен | высокая / среднее | Один последовательный процесс, сторож, тайм-ауты, интерактивная сессия | тест M0 | Один повтор, затем карантин фикстуры с отчётом |
| Обновление Office меняет поведение | средняя / среднее | Эталоны по номеру сборки, сравнение старых и новых эталонов | смена номера сборки | Перебазировать после ревью |
| Агенты делают некрасивые презентации | средняя / высокое | Киты, токены, линтер, оценка M5 с людьми | оценки M5 | Больше макетов, строже линт, рецепты слайдов как макеты |
| Агенты пишут CSS и HTML вне словаря | высокая / среднее | Ошибки `css/*` с переводом, «did you mean», короткий `get_vocabulary`, переименование частых ошибок по данным оценки | статистика ошибок M5 | Добавить псевдонимы атрибутов |
| Расход токенов на XML | средняя / низкое | Атрибуты вместо детей, токены, короткие имена, патчи вместо переписывания, компактные виды | токены в M5 | Импорт черновика из Markdown (P1) |
| Явный `sz` мешает Reset Slide | высокая / низкое | Компромисс записан; тип и позиция плейсхолдера сохраняются | ручная проверка M4 | `size-inheritance="native"` (P1) |
| Нестабильность крейтов шрифтов Rust | средняя / среднее | Точные версии, обновление набором, повтор калибровки после обновления | `cargo update` | Остаться на зафиксированных версиях |
| Изменения спецификации `rmcp` | низкая / низкое | Тонкий слой MCP | — | Зафиксировать версию |
| Оракул только на Windows | определённая / среднее | Линт и вид `resolved` работают везде; COM — на одном runner | — | Растеризатор P1 |
| Morph и p14/p159 различаются по версиям Office | средняя / низкое | `mc:AlternateContent` с валидным Fallback; линт-info «morph needs PowerPoint 2019+» | — | — |

---

## 14. Открытые вопросы к пользователю

1. **Темы.** Достаточно ли двух наших китов (`ink`, `paper`) в первом релизе? Или нужен импорт корпоративного `.potx` как мастера с первого дня? Второй вариант добавляет этап.
2. **Политика подгонки по умолчанию.** Я выбрал `error` для основного текста агента и `shrink` для заголовков. Альтернатива — `shrink` до `min-size` для всего текста. Какой вариант вам ближе?
3. **Визуальный редактор.** Это веб-редактор? Если да, сборка WASM и свой растеризатор переходят из P2 и P1 ближе к началу.
4. **Языки текста.** Phase 1 покрывает латиницу и кириллицу. Нужны ли CJK или арабский раньше P2?
5. **Шрифты.** Можно ли встраивать OFL-шрифты (Inter, IBM Plex) во все презентации? Нужны ли конкретные корпоративные шрифты?
6. **Где открывают презентации.** Только PowerPoint для Windows? Или нужны Mac, веб-версия, Keynote, Google Slides? Каждая цель требует отдельной проверки.
7. **Оценка в M5.** Какую модель и какого провайдера использовать для агентов в `evals/`? Кто делает слепую оценку людьми?
8. **Графики и иконки.** В P0 нет родных графиков и иконок: оба — P1. Деловая презентация без графиков и иконок выглядит бедно. Нужно ли перенести графики bar, line и pie и набор иконок `custGeom` в P0? Это добавляет около 2–3 недель до M5.
9. **Картинки.** Агент берёт картинки только из файлов на диске. Нужен ли инструмент поиска или генерации картинок? Сейчас он вне скоупа.
10. **Импорт.** Насколько срочен сценарий «человек правит в PowerPoint, агент продолжает»? Сейчас импорт своего файла — P1, чужого — P2.
