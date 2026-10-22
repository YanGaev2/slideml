# Архетипы слайдов

Каждый слайд стройте по одному архетипу. Выберите архетип по типу данных, скопируйте фрагмент и замените тексты и числа на свои. Координаты меняйте под свой текст: компилятор измеряет текст и предупредит о переполнении.

Все фрагменты проверены сборкой (`check.py`: 0 предупреждений). Они используют стили и шаблоны из `starter.xml`.

**Примеры написаны для вымышленной компании «Луч» (солнечные электростанции). Её тексты и числа не переносите на свои слайды.** Берите только раскладку.

## Какой архетип выбрать

| Что надо показать | Архетип |
|---|---|
| Главный вывод на обложке, тема физическая | A1 |
| Главный вывод на обложке, тема абстрактная (софт, финансы) | A3 как обложка |
| Одно число несёт слайд | A3 |
| Два значения одной метрики (было и стало) | A4 |
| 3 и больше точек во времени | A5 |
| Сравнение объектов | A6 |
| Часть из целого (2 из 5, доли) | A7 |
| Много метрик сразу | A8 |
| План, этапы, сроки | A9 |
| Отзыв, цитата, смена раздела | A13 |
| Просьба, решение, следующий шаг | A12 |
| Как работает процесс (3–5 шагов) | A14: ряд блоков со стрелками `line tail="triangle"`, главный шаг акцентом |
| Два-три варианта | A10: колонки одинаковой ширины с одинаковыми строками, победитель акцентом |

Правила:
- В деке из 5 слайдов — минимум 4 разных архетипа. Соседние слайды не повторяют архетип.
- Минимум один тёмный слайд или слайд с фото.
- Перед каждым `<slide>` пишите комментарий `<!-- A5 · заголовок -->`. `check.py` по нему проверяет разнообразие.

## A1. Обложка-фото с ответом

**Когда:** Слайд 1, тема физическая: место, товар, люди, еда, производство.

**Раскладка:** Фото на весь слайд, тонировка `duotone` в палитру, затемнение слева `linear(0, dk1/92 0%, … dk1/10 100%)`. Заголовок-вывод 44 pt в левой половине (w ≤ 640), под ним строка из 2–3 ключевых чисел. Шаблон `chrome-plain`. Подпись фото внизу слева.

**Типичная ошибка:** Три-четыре слоя надписей; иконка вместо фото; дельта без базы («+68 %») на обложке.

```xml
  <slide id="a1" bg="dk1" transition="fade" transition-dur="700">
    <image id="a1-photo" src="assets/photo.jpg" x="0" y="0" w="960" h="540" fit="cover" tint="duotone(#1E1A17, #E9D8C4)"/>
    <shape id="a1-shade" x="0" y="0" w="960" h="540" fill="linear(0, dk1/92 0%, dk1/60 55%, dk1/10 100%)" line="none"/>
    <use template="chrome-plain" section="Луч · Q2 2026" page="01"/>
    <text id="a1-label" x="48" y="214" w="560" h="auto" class="label" color="lt1/70">Отчёт совету директоров</text>
    <text id="a1-title" x="48" y="240" w="640" h="auto" class="display" size="44" color="lt1">Ввели 12&#160;МВт за&#160;квартал — больше, чем за&#160;весь 2025 год</text>
    <text id="a1-kpis" x="48" y="440" w="600" h="auto" class="body" color="lt1/80">Июнь 5,2 МВт · средний проект 140 кВт · 86 объектов</text>
    <text id="a1-credit" x="48" y="512" w="420" h="auto" class="credit">Фото: Unsplash</text>
  </slide>
```

## A3. Большое число

**Когда:** Одно число несёт весь слайд (итог, рекорд).

**Раскладка:** Число `kpi` 120 pt слева (единица — `<span size="44">`). Справа (x 640, w 272) — база и причина в 2–3 строки. Внизу полоса из трёх подтверждающих фактов: значение `h1` + подпись `label`.

**Типичная ошибка:** Число в карточке, вокруг пусто; нет базы сравнения.

```xml
  <slide id="a3" bg="lt1" transition="fade" transition-dur="600">
    <use template="chrome" section="Луч · Итог квартала" page="02"/>
    <text id="a3-title" x="48" y="72" w="864" h="auto" class="h1">Ввод мощности за квартал — 12&#160;МВт</text>
    <text id="a3-kpi" x="40" y="168" w="560" h="auto" class="kpi">12<span size="44"> МВт</span></text>
    <!-- Bars: 272 pt = the larger value. 2025: 272 × 10 / 12 = 227. -->
    <text id="a3-b1-l" x="642" y="176" w="272" h="auto" class="label" color="accent1">Q2 2026 · 12 МВт</text>
    <shape id="a3-b1" x="642" y="196" w="272" h="28" fill="accent1" line="none"/>
    <text id="a3-b2-l" x="642" y="240" w="272" h="auto" class="label">Весь 2025 год · 10 МВт</text>
    <shape id="a3-b2" x="642" y="260" w="227" h="28" fill="lt2" line="none"/>
    <text id="a3-why" x="642" y="304" w="272" h="auto" class="body">За квартал — больше, чем за&#160;прошлый год.</text>
    <line id="a3-rule" x1="48" y1="368" x2="912" y2="368" stroke="dk1/15 0.75"/>
    <text id="a3-f1v" x="48" y="384" w="270" h="auto" class="h1">5,2 МВт</text>
    <text id="a3-f1l" x="48" y="428" w="270" h="auto" class="label">ввод в июне</text>
    <text id="a3-f2v" x="345" y="384" w="270" h="auto" class="h1">+68 %</text>
    <text id="a3-f2l" x="345" y="428" w="270" h="auto" class="label">июнь к апрелю</text>
    <text id="a3-f3v" x="642" y="384" w="270" h="auto" class="h1">86 объектов</text>
    <text id="a3-f3l" x="642" y="428" w="270" h="auto" class="label">было 61 в начале квартала</text>
  </slide>
```

## A4. Было → стало

**Когда:** Одна метрика изменилась: цена, чек, конверсия, размер.

**Раскладка:** «Было» `kpi-muted` серым слева (w 300), стрелка `line` с `tail="triangle"`, «стало» `kpi` 84 pt акцентом (w 464). Ниже строка дельты акцентом (абсолютная и в %), под ней причина.

**Типичная ошибка:** «Было» спрятано в подпись 10 pt.

```xml
  <slide id="a4" bg="lt1" transition="fade" transition-dur="600">
    <use template="chrome" section="Луч · Размер проекта" page="03"/>
    <text id="a4-title" x="48" y="72" w="864" h="auto" class="h1">Средний проект вырос на&#160;47&#160;%</text>
    <text id="a4-before-l" x="48" y="196" w="240" h="auto" class="label">Было</text>
    <text id="a4-before" x="48" y="222" w="300" h="auto" class="kpi-muted">95 кВт</text>
    <line id="a4-arrow" x1="356" y1="256" x2="428" y2="256" stroke="dk1 2" tail="triangle"/>
    <text id="a4-after-l" x="452" y="196" w="300" h="auto" class="label">Стало</text>
    <text id="a4-after" x="448" y="214" w="464" h="auto" class="kpi" size="84" color="accent1">140 кВт</text>
    <!-- Bars to scale: 864 pt = the larger value; before = 864 × 95 / 140 = 586. -->
    <shape id="a4-bar-before" x="48" y="318" w="586" h="14" fill="lt2" line="none"/>
    <shape id="a4-bar-after" x="48" y="340" w="864" h="14" fill="accent1" line="none"/>
    <text id="a4-delta" x="48" y="376" w="560" h="auto" class="h2" color="accent1">+45 кВт на каждый проект · +47 %</text>
    <text id="a4-why" x="48" y="412" w="560" h="auto" class="body">Почему вырос: [уточнить причину — доля складов и заводов].</text>
    <anim target="a4-after" effect="zoom" start="after" dur="500"/>
  </slide>
```

## A5. Тренд с выноской

**Когда:** Динамика 3–12 точек во времени.

**Раскладка:** `chart column` слева (w 560), прошлые точки `lt2`, ключевая — `accent1`, подписи с единицей через `format`. `<note>` с выводом у ключевой точки. Справа колонка (x 640, w 272): итог периода и причина.

**Типичная ошибка:** Заголовок пересказывает столбцы; подписи без единиц.

```xml
  <slide id="a5" bg="lt1" transition="fade" transition-dur="600">
    <use template="chrome" section="Луч · Ввод мощности" page="04"/>
    <text id="a5-title" x="48" y="72" w="864" h="auto" class="h1">Ввод рос каждый месяц: июнь +68&#160;% к&#160;апрелю</text>
    <chart id="a5-chart" type="column" x="48" y="168" w="560" h="312" labels="1" label-bold="1" label-size="14" size="12" color="dk2" gap="70" format="0.0&quot; МВт&quot;">
      <cat>Апрель, Май, Июнь</cat>
      <series name="Ввод, МВт" values="3.1, 3.7, 5.2" colors="lt2, lt2, accent1"/>
      <note at="Июнь" w="170" size="12" fill="dk1" color="lt1">+68 % к апрелю</note>
    </chart>
    <text id="a5-total-l" x="640" y="196" w="272" h="auto" class="label">Итого за квартал</text>
    <text id="a5-total" x="640" y="220" w="272" h="auto" class="h1" size="36">12 МВт</text>
    <text id="a5-why" x="640" y="280" w="272" h="auto" class="body">Рост совпал с&#160;запуском второй монтажной бригады [уточнить].</text>
    <anim target="a5-chart" effect="wipe-bottom" start="after" dur="800"/>
  </slide>
```

## A6. Рейтинг

**Когда:** Сравнение объектов между собой (регионы, продукты, каналы).

**Раскладка:** `chart bar`, по убыванию, лидер — `accent1`, остальные `lt2`, ось без линии (`axis-color="none"`). Справа вывод.

**Типичная ошибка:** Вертикальные столбцы с длинными подписями; все одного цвета.

```xml
  <slide id="a6" bg="lt1" transition="fade" transition-dur="600">
    <use template="chrome" section="Луч · Регионы" page="05"/>
    <text id="a6-title" x="48" y="72" w="864" h="auto" class="h1">Краснодар даёт треть ввода [пример данных]</text>
    <chart id="a6-chart" type="bar" x="48" y="168" w="560" h="312" labels="1" label-size="13" size="13" color="dk1" gap="45" axis-color="none">
      <cat>Краснодар, Ростов, Ставрополь, Воронеж, Крым</cat>
      <series name="Ввод, кВт" values="4100, 2700, 2300, 1700, 1200" colors="accent1, lt2, lt2, lt2, lt2"/>
    </chart>
    <text id="a6-note" x="640" y="196" w="272" h="auto" class="body">Числа на этом слайде — пример раскладки. Берите только данные пользователя.</text>
  </slide>
```

## A7. Часть из целого

**Когда:** «2 из 5», доли до 5–7 частей.

**Раскладка:** N плиток в ряд на всю ширину (5 × 156 pt, зазор 21). Выделенные — `accent1`, с номером внутри. Под рядом линии-скобки с подписями групп. Ниже — один производный факт.

**Типичная ошибка:** Пять одинаковых иконок без данных.

```xml
  <slide id="a7" bg="lt1" transition="fade" transition-dur="600">
    <use template="chrome" section="Луч · География" page="06"/>
    <text id="a7-title" x="48" y="72" w="864" h="auto" class="h1">Работаем в&#160;5 регионах, два из&#160;них новые</text>
    <shape id="a7-t1" x="48" y="184" w="156" h="176" radius="10" fill="lt2" line="none" anchor="bottom" pad="16" align="left" font="heading" size="28" color="dk1">1</shape>
    <shape id="a7-t2" x="225" y="184" w="156" h="176" radius="10" fill="lt2" line="none" anchor="bottom" pad="16" align="left" font="heading" size="28" color="dk1">2</shape>
    <shape id="a7-t3" x="402" y="184" w="156" h="176" radius="10" fill="lt2" line="none" anchor="bottom" pad="16" align="left" font="heading" size="28" color="dk1">3</shape>
    <shape id="a7-t4" x="579" y="184" w="156" h="176" radius="10" fill="accent1" line="none" anchor="bottom" pad="16" align="left" font="heading" size="28" color="lt1">4</shape>
    <shape id="a7-t5" x="756" y="184" w="156" h="176" radius="10" fill="accent1" line="none" anchor="bottom" pad="16" align="left" font="heading" size="28" color="lt1">5</shape>
    <line id="a7-b1" x1="48" y1="380" x2="558" y2="380" stroke="dk2 1"/>
    <text id="a7-b1t" x="48" y="392" w="510" h="auto" class="label">Работали до Q2</text>
    <line id="a7-b2" x1="579" y1="380" x2="912" y2="380" stroke="accent1 1.5"/>
    <text id="a7-b2t" x="579" y="392" w="333" h="auto" class="label" color="accent1">Открыты в Q2</text>
    <text id="a7-fact" x="48" y="432" w="864" h="auto" class="body">Ввод на регион в июне ≈ 1 МВт (5,2 МВт / 5, без учёта даты входа в регион).</text>
    <anim target="a7-t4" effect="zoom" start="after" dur="400"/>
    <anim target="a7-t5" effect="zoom" start="after" dur="400"/>
  </slide>
```

## A8. Сводка метрик

**Когда:** 4–8 метрик одним взглядом (отчёт).

**Раскладка:** `table` на всю ширину: метрика / факт / база / изменение. Изменение — акцентом и жирным. Нет данных — серым «нет данных».

**Типичная ошибка:** Таблица без базы сравнения.

```xml
  <slide id="a8" bg="lt1" transition="fade" transition-dur="600">
    <use template="chrome" section="Луч · Сводка" page="07"/>
    <text id="a8-title" x="48" y="72" w="864" h="auto" class="h1">Три из четырёх метрик выросли</text>
    <table id="a8-table" x="48" y="168" w="864" h="240" cols="2fr 1fr 1fr 1fr" lines="h" line="dk1/15 0.75" size="16" color="dk1" pad="6 8 6 0">
      <tr h="40"><th font="Cascadia Mono" size="11" caps="1" color="dk2" bold="0">Метрика</th><th font="Cascadia Mono" size="11" caps="1" color="dk2" bold="0" align="right">Q2 2026</th><th font="Cascadia Mono" size="11" caps="1" color="dk2" bold="0" align="right">База</th><th font="Cascadia Mono" size="11" caps="1" color="dk2" bold="0" align="right">Изменение</th></tr>
      <tr h="50"><td>Ввод в июне</td><td align="right">5,2 МВт</td><td align="right" color="dk2">3,1 МВт (апрель)</td><td align="right" color="accent1" bold="1">+68 %</td></tr>
      <tr h="50"><td>Средний проект</td><td align="right">140 кВт</td><td align="right" color="dk2">95 кВт</td><td align="right" color="accent1" bold="1">+47 %</td></tr>
      <tr h="50"><td>Регионов</td><td align="right">5</td><td align="right" color="dk2">3</td><td align="right" color="accent1" bold="1">+2</td></tr>
      <tr h="50"><td>Доля B2B-клиентов</td><td align="right">72 %</td><td align="right" color="dk2">нет данных</td><td align="right" color="dk2">—</td></tr>
    </table>
  </slide>
```

## A9. Дорожная карта

**Когда:** План, этапы, сроки.

**Раскладка:** Ось `line` на всю ширину, 2–5 вех `ellipse` 12 pt. Над осью — срок `label`, под осью — действие `h2` и цель `body`. «Сейчас» — акцентом. Внизу строка «Цель к концу периода».

**Типичная ошибка:** Шаги без сроков и без цели в цифрах.

```xml
  <slide id="a9" bg="lt1" transition="fade" transition-dur="600">
    <use template="chrome" section="Луч · План Q3" page="08"/>
    <text id="a9-title" x="48" y="72" w="864" h="auto" class="h1">В Q3 выходим в&#160;шестой регион и&#160;запускаем сервис</text>
    <line id="a9-axis" x1="48" y1="248" x2="912" y2="248" stroke="dk1 1.25"/>
    <shape id="a9-m1" geom="ellipse" x="42" y="242" w="12" h="12" fill="accent1" line="none"/>
    <text id="a9-d1" x="48" y="200" w="260" h="auto" class="label" color="accent1">Сейчас · июнь</text>
    <text id="a9-a1" x="48" y="272" w="260" h="auto" class="h2">5 регионов, 5,2 МВт в месяц</text>
    <shape id="a9-m2" geom="ellipse" x="330" y="242" w="12" h="12" fill="dk1" line="none"/>
    <text id="a9-d2" x="336" y="200" w="260" h="auto" class="label">[срок]</text>
    <text id="a9-a2" x="336" y="272" w="260" h="auto" class="h2">Выйти в&#160;шестой регион</text>
    <text id="a9-g2" x="336" y="324" w="260" h="auto" class="body">Цель: [ввод в регионе в месяц]</text>
    <shape id="a9-m3" geom="ellipse" x="618" y="242" w="12" h="12" fill="dk1" line="none"/>
    <text id="a9-d3" x="624" y="200" w="288" h="auto" class="label">[срок]</text>
    <text id="a9-a3" x="624" y="272" w="288" h="auto" class="h2">Запустить сервисный контракт</text>
    <text id="a9-g3" x="624" y="324" w="288" h="auto" class="body">Цель: [доля сервиса в выручке]</text>
    <line id="a9-goal-rule" x1="48" y1="408" x2="912" y2="408" stroke="dk1/15 0.75"/>
    <text id="a9-goal" x="48" y="424" w="864" h="auto" class="h2" color="accent1">Цель к концу Q3: [ввод в месяц] и 6 регионов</text>
  </slide>
```

## A13. Цитата на фото

**Когда:** Голос клиента, смена раздела, эмоциональная пауза.

**Раскладка:** Фото на весь слайд с затемнением, цитата `quote` 36 pt слева (w 640), источник под ней, подпись фото.

**Типичная ошибка:** Цитата без источника.

```xml
  <slide id="a13" bg="dk1" transition="fade" transition-dur="700">
    <image id="a13-photo" src="assets/photo2.jpg" x="0" y="0" w="960" h="540" fit="cover" tint="duotone(#1E1A17, #E9D8C4)"/>
    <shape id="a13-shade" x="0" y="0" w="960" h="540" fill="linear(0, dk1/90 0%, dk1/70 60%, dk1/30 100%)" line="none"/>
    <use template="chrome-plain" section="Луч · Клиенты" page="09"/>
    <text id="a13-q" x="48" y="168" w="640" h="auto" class="quote">«Цитата клиента или сотрудника, которая подтверждает главный тезис»</text>
    <text id="a13-src" x="48" y="400" w="640" h="auto" class="body" color="lt1/70">— Имя, должность, компания [уточнить]</text>
    <text id="a13-credit" x="48" y="512" w="420" h="auto" class="credit">Фото: Unsplash</text>
  </slide>
```

## A12. Просьба

**Когда:** Последний слайд: решение, деньги, следующий шаг.

**Раскладка:** Тёмный фон, `chrome-plain`. Заголовок `display`, 1–3 пронумерованные просьбы: номер `h1` акцентом + текст `h2` (w 760) с суммой и сроком. Контакт внизу.

**Типичная ошибка:** Глаголы без чисел и сроков («развиваться дальше»).

```xml
  <slide id="a12" bg="dk1" transition="fade" transition-dur="700">
    <use template="chrome-plain" section="Луч · Что дальше" page="10"/>
    <text id="a12-title" x="48" y="72" w="600" h="auto" class="display" color="lt1">Что мы просим</text>
    <text id="a12-n1" x="48" y="224" w="60" h="auto" class="h1" color="accent1">1</text>
    <text id="a12-a1" x="112" y="228" w="760" h="auto" class="h2" color="lt1">[Сумма] на выход в&#160;шестой регион до&#160;[срок]</text>
    <text id="a12-n2" x="48" y="296" w="60" h="auto" class="h1" color="accent1">2</text>
    <text id="a12-a2" x="112" y="300" w="760" h="auto" class="h2" color="lt1">Решение по&#160;сервисному контракту до&#160;[срок]</text>
    <text id="a12-n3" x="48" y="368" w="60" h="auto" class="h1" color="accent1">3</text>
    <text id="a12-a3" x="112" y="372" w="760" h="auto" class="h2" color="lt1">Следующий отчёт — [дата]</text>
    <text id="a12-contact" x="48" y="456" w="600" h="auto" class="body" color="lt1/70">Контакт: [имя, почта, телефон]</text>
  </slide>
```
