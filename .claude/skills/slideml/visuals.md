# Визуальные приёмы: как показать число

Число, которое несёт слайд, нужно **показать**, а не только написать. Крупная цифра без образа — это текст. Выберите приём по типу сообщения, скопируйте фрагмент и замените числа на свои.

Все фрагменты проверены сборкой. **Примеры написаны для вымышленной компании «Луч». Её тексты и числа не переносите на свои слайды.** Берите только приём и формулы размеров.

## Какой приём выбрать

| Сообщение | Приём |
|---|---|
| Доля, процент, «N из M» | V1 сетка единиц |
| Сравнение 2–3 значений одной меры (время, цена) | V2 полосы сравнения |
| Части целого, 2–4 части | V3 полоса долей |
| Экономия, снижение затрат | V4 экономия |
| Число получено расчётом | V5 формула |
| Текущее значение и цель | V6 шкала с целью |
| Этапы, узкое место, где теряем | V7 процесс с узким местом |
| Динамика во времени | график, архетип A5 |
| Рейтинг объектов | график, архетип A6 |

Правила:
- На каждом слайде с числами — минимум один приём из этой таблицы или график. Исключения: обложка, цитата, просьба.
- Размеры считайте по формуле из описания приёма. Длина полосы должна быть пропорциональна числу.
- В комментарии слайда пишите архетип и приём: `<!-- A7 · V1 · заголовок -->`.

## V1. Сетка единиц

**Когда:** Доля или процент: «7 % с ошибкой», «63 % закрыто без оператора», «3 из 10».

**Как:** `<waffle total="100" value="7"/>` — 100 квадратов, 7 акцентом. Для «2 из 367» используйте `total="367" highlight="101,120"`. Рядом крупное число и что значит одна клетка («1 квадрат = 1 монтаж»). Акцентные клетки — отдельная фигура `<id>-hi`, её можно анимировать.

**Типичная ошибка:** Писать «7 %» крупно без картинки; сетка без подписи, что значит клетка.

```xml
  <slide id="v1" bg="lt1" transition="fade" transition-dur="600">
    <use template="chrome" section="Луч · Качество" page="01"/>
    <text id="v1-title" x="48" y="72" w="864" h="auto" class="h1">7&#160;% монтажей требуют повторного выезда</text>
    <waffle id="v1-grid" x="48" y="168" w="312" h="312" total="100" value="7" cols="10" gap="6" fill="lt2" accent="accent1"/>
    <text id="v1-kpi" x="416" y="164" w="496" h="auto" class="kpi" size="96" color="accent1">7&#160;%</text>
    <text id="v1-what" x="416" y="290" w="496" h="auto" class="h2">7 из&#160;100 монтажей за&#160;квартал</text>
    <text id="v1-why" x="416" y="330" w="440" h="auto" class="body">Чаще всего — недотянутые крепления и&#160;ошибки в&#160;схеме подключения.</text>
    <text id="v1-key" x="416" y="452" w="496" h="auto" class="label">1 квадрат = 1 монтаж</text>
    <anim target="v1-grid-hi" effect="fade" start="after" dur="500"/>
  </slide>
```

## V2. Полосы сравнения

**Когда:** Два-три значения одной меры: время, цена, скорость («4 мин против 25 с», «160 ₽ против 38 ₽»).

**Как:** Горизонтальные полосы одной высоты, длина = 720 × значение / максимум. Значение крупно справа от полосы. Лучший вариант — акцентом, остальные `lt2`. Под полосами — разница («−7 дней») и причина.

**Типичная ошибка:** Две цифры рядом без масштаба; полосы, длина которых не пропорциональна значению.

```xml
  <slide id="v2" bg="lt1" transition="fade" transition-dur="600">
    <use template="chrome" section="Луч · Скорость монтажа" page="02"/>
    <text id="v2-title" x="48" y="72" w="864" h="auto" class="h1">Бригада Б монтирует объект в&#160;2,4 раза быстрее</text>
    <!-- Bar length = 720 × value / max. Here max = 12 days: A = 720, B = 720 × 5 / 12 = 300. -->
    <text id="v2-a-l" x="48" y="180" w="400" h="auto" class="label">Бригада А · дней на объект</text>
    <shape id="v2-a" x="48" y="200" w="720" h="56" fill="lt2" line="none"/>
    <text id="v2-a-v" x="784" y="210" w="128" h="auto" class="h1" color="dk2">12</text>
    <text id="v2-b-l" x="48" y="284" w="400" h="auto" class="label" color="accent1">Бригада Б · дней на объект</text>
    <shape id="v2-b" x="48" y="304" w="300" h="56" fill="accent1" line="none"/>
    <text id="v2-b-v" x="364" y="314" w="128" h="auto" class="h1" color="accent1">5</text>
    <text id="v2-delta" x="48" y="400" w="864" h="auto" class="h2" color="accent1">−7 дней на каждый объект</text>
    <text id="v2-why" x="48" y="436" w="864" h="auto" class="body">Бригада Б собирает каркас на&#160;складе заранее.</text>
    <anim target="v2-b" effect="wipe-left" start="after" dur="600"/>
  </slide>
```

## V3. Полоса долей

**Когда:** Часть целого из 2–4 частей: «72 % / 28 %», «бот 63 % / оператор 37 %».

**Как:** Одна полоса на всю ширину 864, ширина сегмента = 864 × доля. Число доли внутри сегмента, расшифровка под сегментом той же ширины.

**Типичная ошибка:** Кольцевая диаграмма для двух долей; доли, которые в сумме не 100 %.

```xml
  <slide id="v3" bg="lt1" transition="fade" transition-dur="600">
    <use template="chrome" section="Луч · Клиенты" page="03"/>
    <text id="v3-title" x="48" y="72" w="864" h="auto" class="h1">72&#160;% выручки дают B2B-клиенты</text>
    <text id="v3-l" x="48" y="192" w="600" h="auto" class="label">Доля в выручке Q2</text>
    <!-- Segment width = 864 × share. B2B: 864 × 0,72 = 622; private: 864 − 622 = 242. -->
    <shape id="v3-b2b" x="48" y="216" w="622" h="128" fill="accent1" line="none" anchor="middle" align="left" pad="0 20" font="heading" size="32" color="lt1">72 %</shape>
    <shape id="v3-b2c" x="670" y="216" w="242" h="128" fill="lt2" line="none" anchor="middle" align="left" pad="0 20" font="heading" size="32" color="dk1">28 %</shape>
    <text id="v3-a-h" x="48" y="368" w="560" h="auto" class="h2">B2B: склады, заводы, торговые центры</text>
    <text id="v3-a-b" x="48" y="402" w="560" h="auto" class="body">Средний объект 240 кВт, договор на&#160;обслуживание.</text>
    <text id="v3-b-h" x="670" y="368" w="242" h="auto" class="h2">Частные дома</text>
    <text id="v3-b-b" x="670" y="402" w="242" h="auto" class="body">Средний объект 12 кВт.</text>
    <anim target="v3-b2b" effect="wipe-left" start="after" dur="700"/>
  </slide>
```

## V4. Экономия

**Когда:** Было → стало в деньгах или ресурсах, когда важна разница: «160 ₽ → 38 ₽, экономия 122 ₽».

**Как:** Полоса «было» на всю ширину. Полоса «стало» = 864 × новое / старое, сплошная акцентом. Остаток — та же длина с заштриховкой `pattern(wdUpDiag, accent1, lt1)` и подписью экономии под ним.

**Типичная ошибка:** Экономия только текстом; заштрихованная часть другой длины, чем разница.

```xml
  <slide id="v4" bg="lt1" transition="fade" transition-dur="600">
    <use template="chrome" section="Луч · Себестоимость" page="04"/>
    <text id="v4-title" x="48" y="72" w="864" h="auto" class="h1">Стоимость монтажа кВт снизилась на&#160;треть</text>
    <!-- Old cost = full width 864. New cost = 864 × 41 / 62 = 571. Savings = 864 − 571 = 293. -->
    <text id="v4-old-l" x="48" y="176" w="400" h="auto" class="label">Было · 62 ₽ за&#160;кВт</text>
    <shape id="v4-old" x="48" y="198" w="864" h="48" fill="lt2" line="none"/>
    <text id="v4-new-l" x="48" y="276" w="400" h="auto" class="label" color="accent1">Стало · 41 ₽ за&#160;кВт</text>
    <shape id="v4-new" x="48" y="298" w="571" h="48" fill="accent1" line="none"/>
    <shape id="v4-save" x="619" y="298" w="293" h="48" fill="pattern(wdUpDiag, accent1, lt1)" line="accent1 1"/>
    <text id="v4-save-l" x="619" y="360" w="293" h="auto" class="h2" color="accent1">−21 ₽ экономии</text>
    <text id="v4-why" x="48" y="420" w="540" h="auto" class="body">Закупка панелей напрямую у&#160;завода и&#160;сборка каркаса на&#160;складе.</text>
    <anim target="v4-save" effect="fade" start="after" dur="500"/>
    <anim target="v4-save-l" effect="fade" start="with" dur="500"/>
  </slide>
```

## V5. Формула

**Когда:** Число получено расчётом, и расчёт — аргумент: «26 тыс. × 122 ₽ = 3,2 млн ₽», «86 × 140 кВт = 12 МВт».

**Как:** Множители `kpi-muted` с подписями, знаки × и = между ними, результат — в акцентной плашке. Ниже — что следует из формулы и точный расчёт мелко.

**Типичная ошибка:** Результат без множителей: зритель не верит числу, которое не видит откуда.

```xml
  <slide id="v5" bg="lt1" transition="fade" transition-dur="600">
    <use template="chrome" section="Луч · Ввод" page="05"/>
    <text id="v5-title" x="48" y="72" w="864" h="auto" class="h1">12&#160;МВт дали 86 объектов по&#160;140&#160;кВт</text>
    <text id="v5-a" x="48" y="196" w="240" h="auto" class="kpi-muted" color="dk1">86</text>
    <text id="v5-a-l" x="48" y="270" w="240" h="auto" class="label">объектов за&#160;квартал</text>
    <text id="v5-x" x="268" y="196" w="60" h="auto" class="kpi-muted" align="center">×</text>
    <text id="v5-b" x="340" y="196" w="260" h="auto" class="kpi-muted" color="dk1">140<span size="28"> кВт</span></text>
    <text id="v5-b-l" x="340" y="270" w="260" h="auto" class="label">средний объект</text>
    <text id="v5-eq" x="592" y="196" w="60" h="auto" class="kpi-muted" align="center">=</text>
    <shape id="v5-res" x="664" y="176" w="248" h="128" radius="12" fill="accent1" line="none" anchor="middle" align="left" pad="0 20" font="heading" size="56" color="lt1">12<span size="24"> МВт</span></shape>
    <line id="v5-rule" x1="48" y1="352" x2="912" y2="352" stroke="dk1/15 0.75"/>
    <text id="v5-next" x="48" y="372" w="864" h="auto" class="h2">Чтобы удвоить ввод, нужно 172 объекта или&#160;280&#160;кВт на&#160;объект</text>
    <text id="v5-note" x="48" y="440" w="864" h="auto" class="body">86 × 140 кВт = 12 040 кВт ≈ 12 МВт.</text>
    <anim target="v5-res" effect="zoom" start="after" dur="500"/>
  </slide>
```

## V6. Шкала с целью

**Когда:** Текущее значение и цель на одной шкале: «7 % сейчас, цель 3 %».

**Как:** Горизонтальная шкала с подписями 0 / середина / максимум, x = 48 + 864 × значение / максимум. Текущее значение — заполненный кружок акцентом, цель — пустой кружок, стрелка от текущего к цели. Внизу — что даст достижение цели в числах.

**Типичная ошибка:** Цель, которой нет в данных пользователя: тогда ставьте `[уточнить]` на подписи цели.

```xml
  <slide id="v6" bg="lt1" transition="fade" transition-dur="600">
    <use template="chrome" section="Луч · Цель" page="06"/>
    <text id="v6-title" x="48" y="72" w="864" h="auto" class="h1">Снизим повторные выезды с&#160;7&#160;% до&#160;3&#160;%</text>
    <!-- Scale 0–10 % on x 48..912: x = 48 + 86,4 × value. 7 % → 653, 3 % → 307. -->
    <line id="v6-axis" x1="48" y1="272" x2="912" y2="272" stroke="dk1/30 2"/>
    <text id="v6-t0" x="48" y="288" w="80" h="auto" class="label">0 %</text>
    <text id="v6-t5" x="440" y="288" w="80" h="auto" class="label" align="center">5 %</text>
    <text id="v6-t10" x="832" y="288" w="80" h="auto" class="label" align="right">10 %</text>
    <shape id="v6-now" geom="ellipse" x="641" y="260" w="24" h="24" fill="accent1" line="none"/>
    <text id="v6-now-l" x="573" y="180" w="160" h="auto" class="h2" color="accent1" align="center">Сейчас 7 %</text>
    <shape id="v6-goal" geom="ellipse" x="295" y="260" w="24" h="24" fill="lt1" line="dk1 2.5"/>
    <text id="v6-goal-l" x="227" y="180" w="160" h="auto" class="h2" align="center">Цель 3 %</text>
    <line id="v6-arrow" x1="630" y1="244" x2="332" y2="244" stroke="accent1 2" tail="triangle"/>
    <text id="v6-how" x="48" y="352" w="864" h="auto" class="body">Как: чек-лист приёмки на&#160;объекте и&#160;фото каждого крепления в&#160;приложении. Срок — конец Q3.</text>
    <line id="v6-rule" x1="48" y1="408" x2="912" y2="408" stroke="dk1/15 0.75"/>
    <text id="v6-gain" x="48" y="424" w="864" h="auto" class="h2" color="accent1">−3 повторных выезда в&#160;квартал при&#160;86 объектах</text>
    <anim target="v6-arrow" effect="wipe-right" start="after" dur="600"/>
  </slide>
```

## V7. Процесс с узким местом

**Когда:** Этапы с длительностью или потерями, одно узкое место: «монтаж 9 дней из 21», «7 % ошибок на маршрутизации».

**Как:** Плашки этапов в ряд со стрелками, узкое место — акцентом. Под ними полоса времени: длина сегмента = 864 × длительность / сумма. Ниже — как сократим узкое место.

**Типичная ошибка:** Этапы одинакового веса; нет числа, которое объясняет, почему это узкое место.

```xml
  <slide id="v7" bg="lt1" transition="fade" transition-dur="600">
    <use template="chrome" section="Луч · Процесс" page="07"/>
    <text id="v7-title" x="48" y="72" w="864" h="auto" class="h1">Монтаж — узкое место: 9 дней из&#160;21</text>
    <shape id="v7-s1" x="48" y="176" w="186" h="112" radius="10" fill="lt2" line="none" anchor="middle" align="left" pad="0 16" font="heading" size="15" color="dk1"><p>Заявка</p><p font="body" size="14" color="dk2" before="4">2 дня</p></shape>
    <shape id="v7-s2" x="274" y="176" w="186" h="112" radius="10" fill="lt2" line="none" anchor="middle" align="left" pad="0 16" font="heading" size="15" color="dk1"><p>Проект</p><p font="body" size="14" color="dk2" before="4">5 дней</p></shape>
    <shape id="v7-s3" x="500" y="176" w="186" h="112" radius="10" fill="accent1" line="none" anchor="middle" align="left" pad="0 16" font="heading" size="15" color="lt1"><p>Монтаж</p><p font="body" size="14" color="lt1" before="4">9 дней</p></shape>
    <shape id="v7-s4" x="726" y="176" w="186" h="112" radius="10" fill="lt2" line="none" anchor="middle" align="left" pad="0 16" font="heading" size="15" color="dk1"><p>Подключение</p><p font="body" size="14" color="dk2" before="4">5 дней</p></shape>
    <line id="v7-a1" x1="238" y1="232" x2="268" y2="232" stroke="dk1 1.5" tail="triangle"/>
    <line id="v7-a2" x1="464" y1="232" x2="494" y2="232" stroke="dk1 1.5" tail="triangle"/>
    <line id="v7-a3" x1="690" y1="232" x2="720" y2="232" stroke="dk1 1.5" tail="triangle"/>
    <!-- Time bar: 864 pt = 21 days, 41,14 pt per day. 2 → 82, 5 → 206, 9 → 370, 5 → 206. -->
    <text id="v7-tl" x="48" y="324" w="400" h="auto" class="label">Дни от&#160;заявки до&#160;подключения</text>
    <shape id="v7-t1" x="48" y="348" w="82" h="28" fill="lt2" line="lt1 2"/>
    <shape id="v7-t2" x="130" y="348" w="206" h="28" fill="lt2" line="lt1 2"/>
    <shape id="v7-t3" x="336" y="348" w="370" h="28" fill="accent1" line="lt1 2"/>
    <shape id="v7-t4" x="706" y="348" w="206" h="28" fill="lt2" line="lt1 2"/>
    <text id="v7-how" x="48" y="404" w="864" h="auto" class="body">Вторая бригада сократит монтаж до&#160;5 дней, а&#160;весь цикл — до&#160;17 дней.</text>
    <anim target="v7-s3" effect="zoom" start="after" dur="400"/>
  </slide>
```
