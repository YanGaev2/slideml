# SlideML (прототип): справка для агента

SlideML — XML-формат слайдов. Компилятор `slideml` превращает его в нативный `.pptx`. Каждый элемент становится родным объектом PowerPoint: фигурой, картинкой, графиком, таблицей, группой или анимацией. Растровых скриншотов нет.

## 1. Команды

`<скилл>` — папка скилла slideml. Команда `python <скилл>/scripts/check.py <deck.xml>` выполняет шаги 1–3 за один запуск.

1. Скомпилируйте презентацию:
   `<скилл>/bin/slideml.exe build <deck.xml> -o <out.pptx>`
2. Отрисуйте слайды через PowerPoint:
   `powershell -NoProfile -File <скилл>/scripts/render.ps1 -Pptx <out.pptx> -OutDir <dir>`
3. Нарисуйте рамки и `id` элементов поверх слайдов:
   `python <скилл>/scripts/overlay.py <out.manifest.json> <dir>`
4. Откройте `<dir>/slideN.png` и `<dir>/slideN.overlay.png` и посмотрите каждый слайд.

Команда 1 пишет рядом с `.pptx` файл `<out>.manifest.json`: вычисленные рамки всех элементов и метрики текста. Команда 1 также печатает предупреждения:
- `warning: … text overflows` — текст не помещается в блок. Компилятор измеряет текст по файлам шрифтов. Точность — 99,5 % строк по сравнению с PowerPoint.
- `warning: … font size 8pt is unreadable` — кегль меньше 9 pt.
- `lint: … overlap` — два текста наезжают друг на друга.
- `lint: … almost align` — левые края различаются на 0,5–4 pt. Сделайте их равными.

Исправьте каждое предупреждение до рендера. Команда 3 печатает переходы, анимации и строку `OVERFLOW` от самого PowerPoint.

На overlay-картинке синяя заливка показывает занятую текстом часть блока. Красная рамка означает переполнение. Серые рамки — фигуры, картинки и графики. Фиолетовый пунктир — группы.

Ошибка компиляции содержит строку, столбец и список допустимых значений. Исправьте исходник и повторите сборку.

## 2. Холст и единицы

- Слайд 16:9 имеет размер 960 × 540 pt. Начало координат — левый верхний угол.
- Все длины в пунктах (pt): координаты, кегль, толщина линии, радиус, отступы.
- Время в миллисекундах.
- Порядок элементов в файле — порядок слоёв. Первый элемент лежит ниже всех.

## 3. Цвет и заливка

| Запись | Значение |
|---|---|
| `#1E293B` | цвет RGB |
| `#1E293B/40` | цвет с непрозрачностью 40 % |
| `accent1`, `accent1/30` | слот темы: `tx1 tx2 bg1 bg2 dk1 lt1 dk2 lt2 accent1…accent6` |
| `white`, `black` | белый, чёрный |
| `none` | нет заливки |
| `linear(0, c1 0%, c2 100%)` | линейный градиент. Угол 0 — слева направо, 90 — сверху вниз |
| `radial(c1 0%, c2 100%)` | радиальный градиент из центра |
| `pattern(dotGrid, fg, bg)` | узор PowerPoint: `pct5…pct90`, `dotGrid`, `smGrid`, `lgGrid`, `dnDiag`, `wave`, `zigZag` и другие |

Атрибут `color` у текста тоже принимает градиент. Так получается градиентный текст.

## 4. Шрифты

Используйте только установленные шрифты с кириллицей:

- **Unbounded**, **Unbounded Medium**, **Unbounded SemiBold**, **Unbounded ExtraBold** — широкий геометрический, для заголовков.
- **Manrope**, **Manrope Medium**, **Manrope SemiBold**, **Manrope ExtraBold** — гротеск для текста.
- **Inter**, **Inter Medium**, **Inter SemiBold**.
- **Montserrat**, **Arial Black**, **Segoe UI**, **Segoe UI Semibold**, **Segoe UI Black**, **Bahnschrift**, **Bahnschrift SemiBold**.
- **Cascadia Mono** — моноширинный.

В атрибуте `font` значение `heading` означает шрифт заголовков темы, `body` — основной шрифт темы.

## 5. Структура

```xml
<deck lang="ru-RU">
  <theme font="Manrope" heading="Unbounded" dk1="#0B0B2E" lt1="#FFFFFF" dk2="…" lt2="…"
         accent1="…" accent2="…" accent3="…" accent4="…" accent5="…" accent6="…"/>
  <slide id="cover" bg="dk1" transition="fade" transition-dur="800"> … </slide>
</deck>
```

- `slide`: `id`, `bg` (любая заливка), `transition` (`fade`, `push`, `wipe`, `morph`), `transition-dur`, `transition-dir` (`u d l r`).
- `id` элемента уникален в пределах слайда. Анимация ссылается на элемент по `id`.

## 6. Элементы

Общие атрибуты рамки: `id x y w h`, `rot` (градусы), `flip` (`h`, `v`, `hv`).

Эффекты (у `shape`, `text`, `image`, `icon`):
- `shadow`: `soft`, `lift`, `strong`, `hard` (жёсткая тень без размытия, вправо-вниз), `none`, или `blur dist color [angle]`. Пример: `shadow="0 8 black 45"`.
- `glow="accent1/50 12"` — свечение: цвет и радиус.
- `soft-edge="40"` — размытый край. Большой эллипс с `radial(...)` и `soft-edge` даёт мягкое цветное пятно для фона «аврора».

### shape — фигура
- `geom` — один из 187 пресетов PowerPoint: `rect`, `roundRect`, `ellipse`, `triangle`, `chevron`, `homePlate`, `blockArc`, `donut`, `arc`, `star5`, `wave`, `plus` и другие.
- `radius` — радиус углов для `roundRect`.
- `adj="adj1=…;adj2=…"` — регуляторы пресета. Для `blockArc`: `adj1` — начальный угол, `adj2` — конечный угол, в 1/60000 градуса. 0 — направление «на 3 часа», углы идут по часовой стрелке. `adj3` — толщина кольца, от 0 до 50000.
- `path="M… L… C… A… Z"` и `viewbox="0 0 W H"` — свой контур в синтаксисе SVG path, поддержаны все команды.
- `fill`, `line="color width [dash]"` (dash: `dash`, `sysDash`, `dot`, `sysDot`, `lgDash`, `dashDot`).
- Текст внутри: `pad="t r b l"`, `anchor="top|middle|bottom"`, `wrap="0"` выключает перенос строк.

### text — текстовый блок
Это фигура без заливки и без отступов. По умолчанию текст прижат к верху блока.

Высота блока:
- `h="auto"` — компилятор ставит высоту по измеренному тексту. Используйте это для подписей и абзацев.
- `fit="shrink"` — компилятор уменьшает кегль, пока текст не поместится в `h`. Предупреждение сообщает масштаб.

Если символа нет в шрифте (например, `₽` в Unbounded и Manrope), компилятор сам берёт этот символ из шрифта, где он есть. Ничего не оборачивайте вручную.

### Текст внутри `shape`, `text`, `td`
- Пишите текст прямо в элементе или разбейте его на абзацы `<p>`.
- Внутри строки: `<span …>`, `<b>`, `<i>`, `<u>`, `<br/>`, `<a href="…">`.
- Атрибуты текста наследуются: элемент → `p` → `span`.
- Список атрибутов: `font`, `size`, `color`, `bold`, `italic`, `underline`, `caps`, `spacing` (трекинг в pt), `line-height` (множитель), `before`, `after` (pt), `align` (`left`, `center`, `right`, `justify`), `bullet` (`•`, `—`, `1.`), `indent`.

### image — картинка
`src` (путь от файла презентации, только `.jpg` и `.png`), `fit` (`cover`, `contain`, `fill`), `focus="60% 40%"`, `radius`, `geom` (например, `ellipse`), `line`, `opacity`, `tint` (`grayscale` или `duotone(#тёмный, #светлый)`), `alt`.

Фото по теме ищите и скачивайте командой `python <скилл>/scripts/fetch_image.py "<запрос по-английски>" <папка презентации>/assets --count 3`. Команда берёт фото с открытой лицензией и записывает авторов в `assets/SOURCES.md`.

### icon — векторная иконка Lucide
`<icon name="rocket" x y w h color="accent1" weight="2" fill="none"/>`
- `name` — имя иконки с lucide.dev, около 2 150 штук. Примеры: `users`, `chart-line`, `shield-check`, `zap`, `globe`, `target`, `wallet`, `calendar`, `cpu`, `sparkles`, `trending-up`, `mouse-pointer-2`, `star`, `play`, `cloud`, `minus`, `square`, `x`.
- `weight` — толщина штриха в единицах сетки иконки 24 × 24.
- Иконка — нативная векторная фигура. В PowerPoint её можно перекрасить. На неизвестное имя компилятор предлагает похожие имена.

### dots — полутоновое поле точек
`<dots x y w h step="9" r="1.6" fill="#9B8F82"/>` — одна фигура, внутри до 6000 точек.

### line — линия
`<line x1 y1 x2 y2 stroke="white/60 1 dash" head="oval" tail="triangle"/>`. Концы: `none`, `triangle`, `stealth`, `diamond`, `oval`, `arrow`.

### chart — нативный график с данными Excel
```xml
<chart type="column" x y w h labels="1" legend="none" gap="50" size="11" color="tx1/60">
  <cat>Q1, Q2, Q3, Q4</cat>
  <series name="Выручка" values="12, 18, 15, 24" color="accent1"
          colors="…по цвету на точку…" fill="linear(90, accent1 0%, accent2 100%)"/>
</chart>
```
- `type`: `column`, `bar`, `line`, `area`, `doughnut`, `pie`.
- Атрибуты графика: `labels`, `label-pos`, `label-color`, `label-size`, `label-bold`, `legend` (`none b t r l`), `gap`, `overlap`, `stacked`, `hole` (для `doughnut`), `format` (числовой формат Excel, например `0"%"`), `axis` (`cat`, `both`, `none`), `grid`, `grid-color`, `axis-color`, `min`, `max`, `smooth`, `font`, `size`, `color`.
- Атрибуты серии: `width`, `marker` (для `line`), `colors` (цвет или градиент на каждую точку), `label-points="5,6"` (подписи только у этих точек, счёт с 0).
- Несколько серий в `doughnut` дают концентрические кольца. Первая серия — внутреннее кольцо.
- `outline="none"` или `outline="white 2"` — контур сегментов `doughnut`/`pie`.
- `axis-color="none"` прячет линию оси. `label-font` задаёт шрифт подписей данных.

**Геометрия графика детерминирована.** Компилятор сам считает шкалу (`min`, `max`, `step` — их можно задать) и точную область построения. Область построения можно задать вручную: `plot="top right bottom left"` — отступы в pt внутри рамки графика. Поэтому компилятор знает координату каждого столбца и точки. Аннотации ставьте внутри `<chart>` по данным, а не по пикселям:

```xml
<chart id="orders" type="column" … labels="1">
  <cat>H1, H2, …, H13</cat>
  <series values="…" colors="tx1/15, …, accent1" label-points="6"/>
  <band from="H6" to="H7" fill="accent1/8"/>              <!-- подсветка диапазона категорий, под графиком -->
  <ref value="97" label="среднее 97" label-at="start"/>   <!-- линия уровня; stroke="tx1/45 1 dash" -->
  <note at="H7" w="160" fill="tx1" color="bg1">121 тыс. — пик квартала</note>  <!-- выноска к точке; dx, dy, series -->
</chart>
<chart type="doughnut" …><center size="30" sub="доля солнца">31%</center></chart>  <!-- текст в центре кольца -->
```

Аннотации — нативные фигуры с `id` `<chart-id>-note0`, `<chart-id>-ref1` и т. д. (или свой `id`). Их можно анимировать.

### table — нативная таблица
```xml
<table x y w h cols="1fr 2fr" lines="hv" line="white/60 1" size="12" color="white">
  <tr><th>Заголовок</th><td>Текст</td></tr>
  <tr h="40" fill="white/5"><td colspan="2">Объединённая ячейка</td></tr>
</table>
```
- `cols` — ширины колонок в `fr` или pt.
- `lines`: `none`, `h` (между строками), `hv` (между строками и колонками), `rows` (h и внешние горизонтали), `all`.
- У ячеек и строк: `fill`, `anchor`, `pad`, `colspan` и атрибуты текста. Ячейка `th` по умолчанию жирная.
- PowerPoint растягивает строку, если текст в неё не помещается.

### Стили и шаблоны

Повторяемое оформление описывайте один раз:

```xml
<deck>
  <styles>
    <style name="h1" font="heading" size="40" color="white"/>
    <style name="caption" size="11" color="tx1/60"/>
  </styles>
  <templates>
    <template id="chrome">                       <!-- рамка, подвал, фон — на каждый слайд -->
      <shape id="bg-blob" … /> <text id="footer" … class="caption">{{page}}</text>
    </template>
    <template id="kpi">                          <!-- компонент с параметрами -->
      <shape x="0" y="0" w="200" h="90" radius="12"/>
      <text id="value" x="16" y="12" w="170" h="auto" class="h1">{{value}}</text>
    </template>
  </templates>
  <slide id="s1">
    <use template="chrome" page="01"/>
    <use template="kpi" dx="60" dy="160" prefix="k1-" value="140+"/>
    <text class="h1" x="60" y="60" w="600" h="auto">Заголовок</text>
  </slide>
</deck>
```

- `class="a b"` подставляет атрибуты стилей. Атрибуты самого элемента важнее стиля.
- `<use>` вставляет содержимое шаблона. `{{имя}}` заменяется атрибутом `<use>`.
- `dx`, `dy` сдвигают все координаты шаблона. `prefix` добавляется к `id` элементов шаблона.
- Без `prefix` элементы шаблона сохраняют свои `id` на каждом слайде. Поэтому Morph не двигает общую рамку и плавно двигает фон.

### group — группа
`<group id="card1"> … </group>` — нативная группа PowerPoint. Анимация группы двигает все её элементы вместе. Группы можно вкладывать.

## 7. Анимации и переходы

`<anim target="id" effect="float-up" start="after" delay="0" dur="600"/>` — прямой потомок `slide`.

- `effect`: `appear`, `fade`, `float-up`, `float-down`, `fly-left`, `fly-right`, `fly-top`, `fly-bottom`, `wipe-left`, `wipe-right`, `wipe-top`, `wipe-bottom`, `zoom`, `expand`.
- `start`: `click` — по щелчку; `with` — вместе с предыдущим эффектом; `after` — после предыдущего эффекта.
- Если первый эффект слайда имеет `start="after"` или `start="with"`, он начинается сразу после перехода.
- Шаблоны эффектов сняты с PowerPoint. PowerPoint показывает их как родные эффекты, а не как «Custom».

**Morph.** Дайте элементу один и тот же `id` на двух соседних слайдах. На втором слайде задайте `transition="morph"`. PowerPoint плавно переведёт положение, размер и цвет элемента.

## 8. Ограничения прототипа

- Высота строки в PowerPoint равна `1,2 × size × line-height`. Компилятор учитывает это сам.
- Нет заметок докладчика, нет видео, нет анимации списка по абзацам.
- Только одна анимация на элемент.
