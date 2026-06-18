# JCLang
Пародия JMCC на расте.

## Использование:
Формат: `jclang <файл.jc> -u -o`

Параметр `-u` выгружает код в облако, и даёт команду на импорт модуля.

Параметр `-o` выводит JSON модуля, который можно отправить в файл, к примеру: `jclang test.jc -o > output.json`

## Синтаксис
```ts
// function или process
function testfunc() {
  game a = 1;
}
event<player_join> {
  player::message(["hello", "world"]);
  testfunc();
  var minimessage_test = m"<red>hi";
  // также доступны: p (plain), j (json), l (legacy)(по умолчанию), m (minimessage)
  player::has_privilege!("WHITELISTED") { // ! инвертирует условие
    player::message([ value::name, " вы не в бс!" ]);
  }
}
```

## Переменные
```ts
// тип переменных может объявлятся где угодно
game a;

function test1() {
    local b = 123; // но значение может быть определено только под хэндлером
    local c = "hello";
}

function test2() {
    local b = 456; // обозначение переменных локальны
    test1(); // но если переопределить какую-то переменную то будет вызван варнинг
    // так как функция test1 установила в b значение 123
    player::message(c); // вызов test1 добавило c
}
```

## Фабрики
Делаются вызовом определённых функциий. Обратите внимание, что в них нельзя ложить переменные, только обычные значения
```ts
location(x, y, z, yaw, pitch);
vector(x, y, z);
potion(effect, amplifier, duration);
particle(texture, count, xspread, yspread);
sound(id, pitch, volume, source);
```
