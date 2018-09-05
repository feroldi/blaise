# Pust

Pust é o nome do projeto da disciplina de compiladores.

## Gramática

```ebnf
<programa> ::= "program" <id> ";" { <declaração> } <lista-comando>

<declaração> ::= "let" <id> ":" <tipo> ";"

<tipo> ::= "int" | "bool" | "float" | "str"

<lista-comando> ::= <commando> ";" { <commando> ";" }

<comando> ::= <atribuição>
            | <leitura>
            | <escrita>
            | <composto>
            | <condicional>
            | <repetição>

<lista-parâmetro> ::= <id> { "," <id> }

<lista-expressão> ::= <expressão> { "," <expressão> }

<atribuição> ::= <id> "=" <expressão> ";"

<leitura> ::= "read" "(" <lista-parâmetro> ")" ";"
            | "readln" "(" <lista-parâmetro> ")" ";"

<escrita> ::= "write" "(" <lista-expressão> ")" ";"
            | "writeln" "(" <lista-expressão> ")" ";"

<composto> ::= "{" <lista-comando> "}"

<condicional> ::= "if" <expressão> <composto> ["else" <composto>]

<repetição> ::= "while" <expressão> <composto>

<expressão> ::=  <expressão> "+" <termo>
               | <expressão> "–" <termo>
               | <termo>

<termo> ::= <termo> "*" <igualdade>
          | <termo> "/" <igualdade>
          | <igualdade>

<igualdade> ::= <igualdade> "==" <relacional>
              | <igualdade> "!=" <relacional>
              | <relacional>

<relacional> ::= <relacional> "<" <fator>
               | <relacional> "<=" <fator>
               | <relacional> ">" <fator>
               | <relacional> ">=" <fator>
               | <fator>

<fator> ::= <num>
          | <id>
          | <string>
          | "(" <expressão> ")"

<num> ::= <inteiro> | <fracionário>
```

## Regex

```
<string> ::= "[^"]*"

<id> ::= [a-zA-Z_][a-zA-Z0-9_]*

<inteiro> ::= 0|([1-9][0-9]*)

<fracionário> ::= [0-9]+\.[0-9]+([Ee][+-]?[0-9]+)?
```

## Alfabeto

```
0 1 2 3 4 5 6 7 8 9
a b c d e f g h i j k l m n o p q r s t u v w x y z
A B C D E F G H I J K L M N O P Q R S T U V W X Y Z
{ } ( ) > < = ! " + - * / , ; :
```
