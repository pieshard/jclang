# JCLang
Пародия JMCC на расте.

## Синтаксис
```ts
// function или process
function testfunc() {
  game a = 1;
}
event<player_join> {
  player::message(["hello", "world"]);
  testfunc();
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
