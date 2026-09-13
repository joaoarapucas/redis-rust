# Banco de Dados: Rust + Lua

Fazer um banco de dados chave-valor em memória no estilo Redis ou Memcached em Rust com um sistema de extensões em Lua.

Este banco deverá ser feito em Rust. Deve-se estender o banco de dados com a VM do LUA.

## Interface de uso

O trabalho entrega **um executável** (`banco-memoria`, ou o nome que você der ao projeto). Ao ser executado sem argumentos, ele abre um terminal interativo próprio e fica lendo comandos do usuário, um por linha, até receber `EXIT`.

```
$ ./banco-memoria
> ADD cpf_zezinho 12345678909
OK
> GET cpf_zezinho
123.456.789-09
> GET chave_inexistente
ERRO: chave inexistente
> EXIT
$
```

São **apenas 3 comandos**:

| Comando | Efeito |
|---|---|
| `ADD chave valor` | Adiciona o dado no banco. Responde `OK` em caso de sucesso. |
| `GET chave` | Busca a chave no banco e exibe o valor na tela. |
| `EXIT` | Encerra o programa. |

Regras da interface:

- O programa **imprime um prompt** (`> `) e aguarda cada comando. Ele só termina com `EXIT` — nenhum outro comando encerra o processo.
- Os comandos são escritos em maiúsculas. `ADD`, `GET` e `EXIT`.
- Em `ADD`, o valor é tudo que vem depois do primeiro espaço após a chave, então o valor pode conter espaços. A chave, não.
- Cada comando responde em uma linha: `OK`, o valor buscado, ou uma mensagem de erro começando com `ERRO:` e explicando o motivo.
- **Nenhuma entrada derruba o programa.** Comando desconhecido, comando incompleto, linha em branco, valor rejeitado por uma extensão — em todos os casos o erro é reportado e o prompt volta. Só `EXIT` encerra.
- O banco é em memória: ao sair, os dados se perdem. Não há persistência em disco.
- O programa também deve funcionar com a entrada vinda de um *pipe* em vez do teclado, para permitir rodar um roteiro de comandos de uma vez:
  `cat roteiro.txt | ./banco-memoria`
  Nesse caso, o fim da entrada encerra o programa como se fosse um `EXIT`.

## Sistema de extensões

O ponto central deste trabalho **não são as extensões, é a interface entre Rust e Lua**. O motor em Rust não pode conhecer nenhuma extensão específica.

- **Descoberta em tempo de execução**: no boot, o motor deve varrer o diretório `extensions/` e carregar *todo* arquivo `.lua` que encontrar. Não existe lista fixa de extensões no código Rust.
- **Auto-registro**: cada extensão em Lua se registra declarando qual prefixo de chave ela trata e quais operações implementa (ADD, GET ou ambas). O protocolo de registro é decisão sua — projete-o e explique-o no `README.md`.
- **Nenhum literal de prefixo no Rust**: as strings `cpf`, `data` ou qualquer outro nome de extensão **não podem aparecer no código Rust**. Se aparecerem, o sistema de extensões não é genérico e o critério não é atendido.
- **Sem recompilar**: acrescentar ou remover um arquivo `.lua` do diretório `extensions/` deve mudar o comportamento do banco sem nenhuma recompilação.
- Todas as chamadas devem ser passadas à VM do Lua.
- **Retorno de transformações**: se a extensão fizer uma transformação de dado ou um cálculo, este valor deve ser devolvido ao Rust em uma estrutura específica do Rust.
- **Estruturas de erro**: se algum dado ou entrada for inválido, ou se a operação não for possível, além de sinalizar o insucesso deve-se explicar o motivo. Este valor também deve ser devolvido em uma estrutura específica do Rust.

### Consulta ao banco a partir da extensão

Nem toda validação depende só do valor recebido: algumas dependem do que já está guardado. O sistema de extensões precisa dar conta disso.

O motor deve oferecer às extensões em Lua uma forma de **consultar o banco durante a validação**. Como essa consulta é exposta é decisão de vocês — projetem junto com o protocolo de registro.

Duas restrições:

- **A extensão não recebe uma cópia do banco.** Ela pergunta ao banco no momento em que precisa, e a resposta tem que refletir o estado atual. Passar a base inteira como argumento para o Lua a cada chamada não atende.
- **A função exposta ao Lua é genérica.** Ela consulta o banco; ela não sabe o que é CPF, data ou qualquer outro prefixo. Vale a mesma regra da seção anterior.

Das extensões obrigatórias, **apenas o validador de CPF usa esta consulta** — veja a regra de unicidade abaixo. O formatador de data valida só o que recebe.

Este é o ponto mais difícil do trabalho, e é de propósito: no instante em que a extensão faz a pergunta, o comando `ADD` já está no meio de uma operação sobre esse mesmo banco. Resolver isso é parte do exercício. O `README.md` deve explicar como vocês resolveram.

### Verificação na correção

Na correção, **será colocada no diretório `extensions/` uma extensão que você nunca viu**, com um prefixo de chave próprio. Ela seguirá exatamente o mesmo contrato das extensões obrigatórias abaixo: valida na escrita, formata na leitura.

Ela deve funcionar sem recompilar o projeto e sem tocar em uma linha de Rust. Se for preciso alterar o código Rust para acomodá-la, o sistema de extensões não cumpriu seu propósito.

Duas coisas que valem saber de antemão:

- **Ela será escrita seguindo apenas o `README.md` de vocês.** Ninguém vai abrir o código Rust do trabalho para descobrir o protocolo de registro. Se a documentação não explicar como registrar uma extensão nova, ela não terá como ser escrita — e o critério não é atendido.
- **Ela usa a consulta ao banco** descrita acima. Uma consulta que só funcione para o CPF não vai atendê-la.

A extensão que você mesmo vai propor (seção abaixo) serve como seu ensaio para isso: se você precisar abrir o código Rust para fazê-la funcionar, o desenho ainda não está certo.

## Extensões obrigatórias

- Extensão: Validador de CPF
    - ADD: Se a chave for no formato `cpf_*`, deve-se validar se o dígito verificador do CPF é válido. A entrada deve ser feita apenas com os 11 números, sem formatação. http://clubes.obmep.org.br/blog/a-matematica-nos-documentos-a-matematica-dos-cpfs
    - ADD, unicidade: **o mesmo CPF não pode ser gravado sob duas chaves diferentes.** Se o número já existir no banco em outra chave, a operação falha e o erro deve dizer em qual chave ele já está. Regravar o mesmo CPF na mesma chave é permitido. Esta é a regra que exige a consulta ao banco descrita acima.
    - GET: formatar o cpf para o formato: `000.000.000-00`
    - Exemplo:
        1. `ADD cpf_zezinho 12345678909`
        2. `GET cpf_zezinho` => `123.456.789-09`
- Extensão: Formatador de Data
    - ADD, formato: Se a chave for no formato `data_*`, o valor deve estar em ISO8601, exatamente como `aaaa-mm-dd` (2022-10-23): quatro dígitos de ano, dois de mês e dois de dia, com zero à esquerda. Formatos frouxos como `2023-1-5` são inválidos.
    - ADD, calendário: não basta o formato — **a data tem que existir de verdade**. O mês tem que estar entre `01` e `12`, e o dia tem que caber no número de dias daquele mês: 31 dias em janeiro, março, maio, julho, agosto, outubro e dezembro; 30 em abril, junho, setembro e novembro; 28 ou 29 em fevereiro, conforme o ano. Portanto `2023-04-31` e `2023-13-01` são inválidas, mesmo estando no formato certo.
    - ADD, ano bissexto: um ano é bissexto se for divisível por 4, **exceto** quando for divisível por 100 e não for divisível por 400. Assim `2024` e `2000` são bissextos e aceitam `02-29`; `1900` e `2100` não são. Uma implementação que testa apenas `ano % 4 == 0` erra o ano de 1900 e está incorreta.
    - Esta extensão valida apenas o valor recebido — não consulta o banco.
    - GET: Formatar no formato brasileiro dd/mm/aaaa
    - Exemplo:
        1. `ADD data_nascimento_zezinho 2000-01-23`
        2. `GET data_nascimento_zezinho` => `23/01/2000`

## Extensão proposta por você

Além das duas obrigatórias, **o grupo deve propor e implementar uma terceira extensão, de tema livre**.

- Ela trata um prefixo de chave próprio, diferente de `cpf_` e `data_`.
- Ela segue o mesmo contrato das outras: faz alguma verificação no `ADD` e alguma transformação no `GET`.
- Ela não pode ser uma variação das obrigatórias — validar CNPJ, RG, PIS ou título de eleitor é o mesmo exercício do CPF com outros pesos, e não conta.
- Ela precisa exercitar alguma coisa que as duas obrigatórias não exercitam. O README deve dizer o que é.

Esta extensão é a prova de que o sistema de extensões é mesmo genérico: ela foi escrita depois do motor, por você, e tem que ter entrado sem alterar uma linha de Rust.

Junto dela, entregue os casos de teste dela — as entradas válidas e as inválidas, no mesmo formato do `casos_teste.txt`.

## Restrições de implementação

A validação é o conteúdo do exercício, então ela é para ser feita na mão:

- **É proibido usar crates de conveniência para validação ou formatação**: `chrono`, `time`, `regex`, e qualquer crate cuja função seja validar ou formatar datas ou documentos. A regra vale pelo propósito do crate, não pelo nome — não adianta procurar um equivalente fora da lista.
- Pela mesma razão, não use bibliotecas Lua de terceiros para estas tarefas.
- O `mlua` (ou equivalente para embarcar a VM do Lua) é permitido, obviamente — é o objeto do trabalho.

### Organização do código

O motor não pode ser um `main.rs` único. Estas quatro responsabilidades ficam em módulos separados, cada uma no seu arquivo:

- **Leitura da entrada**: o laço que imprime o prompt, lê a linha e escreve a resposta.
- **Interpretação do comando**: transformar uma linha de texto em `ADD`, `GET` ou `EXIT` — ou em erro de sintaxe.
- **Armazenamento**: guardar e recuperar pares chave-valor.
- **Ponte com o Lua**: carregar o diretório `extensions/`, registrar as extensões e despachar as chamadas.

Os nomes dos módulos e a forma de organizá-los são de vocês. O que vale é a regra abaixo.

**O crate do Lua pode ser importado em um único módulo.** Se `use mlua::...` aparecer em mais de um arquivo, as responsabilidades não estão separadas: quer dizer que o armazenamento, ou a leitura da entrada, sabe que existem extensões. Só a ponte pode saber.

O `README.md` deve listar os módulos e dizer quem depende de quem.

## Casos de teste

O arquivo `casos_teste.txt` acompanha este enunciado e traz as entradas com as quais o trabalho será exercitado, junto do resultado esperado de cada uma.

Ele inclui casos que passam despercebidos numa implementação apressada — CPFs de dígitos repetidos, a regra de século do ano bissexto, meses e dias fora de faixa. Rode todos antes de entregar.

## Documentação

**Tudo o que este enunciado pede deve estar documentado no `README.md` do repositório.** Entrega sem documentação sofre desconto de pontos.

O `README.md` precisa cobrir, no mínimo:

- **Como compilar e como executar** o projeto, do zero, em uma máquina que só tem Rust instalado.
- **O protocolo de registro de extensões** que você projetou: como uma extensão em Lua declara o prefixo que trata e as operações que implementa, e o que o motor em Rust espera receber de volta.
- **Como acrescentar uma extensão nova**, passo a passo, por alguém que não escreveu o projeto. Quem seguir estas instruções tem que conseguir colocar uma extensão qualquer para funcionar, sem falar com vocês.
- **As estruturas de retorno** de sucesso e de erro no Rust, e como um erro levantado dentro do Lua chega até a mensagem `ERRO:` na tela.
- **A extensão que vocês propuseram**: o que ela faz, por que escolheram essa, e o que ela exercita que o CPF e a data não exercitam.
- **Como a extensão consulta o banco** durante a validação, e como vocês resolveram o fato de o `ADD` já estar operando sobre esse mesmo banco no momento da consulta.
- **Os módulos do projeto** e quem depende de quem.
- **As decisões de projeto** que vocês tomaram e que o enunciado deixou em aberto, com o motivo de cada uma.

Documentação é sobre o *seu* projeto. Um README genérico, que descreveria qualquer trabalho parecido sem citar as decisões que vocês tomaram, conta como ausência de documentação.

## Entrega

**O único entregável é a URL de um repositório público no GitHub.** Não se entrega arquivo, anexo nem PDF por fora — tudo o que for avaliado tem que estar versionado no repositório.

**Uma entrega por grupo.** O grupo escolhe um repositório e entrega uma única URL. Entregas repetidas de integrantes diferentes contam como uma só, e vale a primeira recebida.

**O `README.md` deve começar com o nome completo de todos os integrantes do grupo.** Quem não estiver listado no README não recebe nota por este trabalho, mesmo que apareça no histórico de commits do repositório.

Estrutura esperada:

```
seu-repositorio/
├── Cargo.toml
├── README.md                        # integrantes + a documentação exigida acima
├── src/
│   └── ...                          # o motor em Rust, em módulos separados
├── extensions/
│   ├── cpf.lua
│   ├── data.lua
│   └── <sua_extensao>.lua           # a extensão que vocês propuseram
├── casos_teste.txt                  # o que acompanha este enunciado
└── casos_teste_<sua_extensao>.txt   # os casos da extensão de vocês
```

Este trabalho deve seguir:

- [Política de uso de ferramentas generativas de IA](https://crivelaro.notion.site/Pol-tica-de-uso-de-ferramentas-generativas-de-IA-1b53bb4e12a54b4aa06eaa02e62192f4?pvs=74)
- [Política antiplágio](https://crivelaro.notion.site/Pol-tica-antipl-gio-5187d7b1ab514bfb8424ac0fcfb59dba?pvs=74)

## Referências

Bancos chave-valor, o que são e como se constroem:

- [Implementing a Key-Value Store – Part 1: What are key-value stores, and why implement one?](https://codecapsule.com/2012/11/07/implementing-a-key-value-store-part-1-what-are-key-value-stores-and-why-implement-one/)
- [How Does a Database Work? | Let's Build a Simple Database](https://cstack.github.io/db_tutorial/)
- [Build a Blazingly Fast Key-Value Store with Rust](https://www.tunglevo.com/note/build-a-blazingly-fast-key-value-store-with-rust/)
- [Redis](https://redis.io/)
- [memcached – a distributed memory object caching system](https://memcached.org/)

Rust e a fronteira com Lua:

- [Introduction to Rust Part 1](https://www.youtube.com/watch?v=WnWGO-tLtLA) (vídeo)
- [Exploring Modding Systems: A Journey With Lua and Rust](https://betterprogramming.pub/exploring-modding-systems-a-journey-with-lua-and-rust-951ad01894cf)
- [mlua – High level Lua bindings to Rust with async/await support](https://github.com/mlua-rs/mlua)
