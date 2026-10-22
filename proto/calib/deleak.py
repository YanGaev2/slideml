"""Replaces numbers and phrases from the Zerno test prompt in skill files with the neutral «Луч» example."""
from pathlib import Path

skill = Path("F:/pptx-ai-native/.claude/skills/slideml")
R = [
    ("«Выручка выросла на 58 % за три месяца», «Чек вырос на 10,5 %, рейтинг 4,8»", "«Ввод вырос на 68 % за квартал», «Средний проект вырос на 47 %»"),
    ("«+58 % сентябрь к июлю», а не просто «+58 %»", "«+68 % июнь к апрелю», а не просто «+68 %»"),
    ("(420 / 380 − 1 = +10,5 %, +40 ₽)", "(140 / 95 − 1 = +47 %, +45 кВт)"),
    ("(выручка на точку, на клиента)", "(выручка на клиента, ввод на регион)"),
    ("«1,9 млн ₽», а не «1,9»", "«5,2 МВт», а не «5,2»"),
    ("«нет данных за Q2»", "«нет данных за Q1»"),
    ("(«Выручка»), а вывод («Выручка выросла на 58 % к июлю»)", "(«Ввод»), а вывод («Ввод вырос на 68 % к апрелю»)"),
    ("«+58 % сентябрь к июлю». Месячный рост", "«+68 % июнь к апрелю». Месячный рост"),
    ('"latte art coffee"', '"solar panels roof"'),
    ("(«coffee shop interior», «barista», «cafe facade»)", "(«office interior», «factory workers», «city street»)"),
    ("`<note at=\"Сентябрь\">+58 % к июлю</note>`", "`<note at=\"Июнь\">+68 % к апрелю</note>`"),
    ("(например, месяц открытия)", "(например, месяц запуска)"),
    ("«Выручка», «Метрики», «План»", "«Выручка», «Метрики», «План»"),
]
for f in skill.glob("*.md"):
    s = f.read_text(encoding="utf-8")
    t = s
    for a, b in R:
        t = t.replace(a, b)
    if t != s:
        f.write_text(t, encoding="utf-8")
        print("updated", f.name)
for f in skill.glob("*.md"):
    s = f.read_text(encoding="utf-8")
    for bad in ("58 %", "420", "380 ₽", "1,9 млн", "4,8", "кофе", "Зерно", "latte"):
        if bad in s:
            print("LEAK", f.name, bad)
